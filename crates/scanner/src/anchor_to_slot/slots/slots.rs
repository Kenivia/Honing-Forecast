use crate::{
    anchor_to_slot::{
        anchor_defs::{ALL_SLOT_ADDRESSS, COMBINED_NUMBER_HEIGHT, NUMBER_OFFSET, SLOT_ORDER},
        slots::{
            constants::{
                AFAR_ROWS, ALTERNATIVE, OVERLAY_REACH, OVERLAYS, SLOT_BUDGET, SLOT_CONFIDENCE,
                SLOT_UPDATE_FREQ, UNMATCHED_UNCHANGED,
            },
            utils::{from_afar, overlaid, overlay_areas},
        },
    }, dev_utils::timing::{now, timed}, image_utils::{
        brightness::normalize_brightness, close_enough::{DEFAULT_CLOSE_ENOUGH_CONFIDENCE, close_enough, confidence, masked_confidence}, common::{Rect, get_resizer},
    }, interface::setup::{BASE_ICONS, OneIconConfig, icon_lookup}, ocr::jobs::{Kind, OCRLine}, scanner_state::{OneSlotInfo, ScannerState, SlotAddress},
};

impl ScannerState {
    // given the anchor found, find where this icon should be
    pub fn anchored_slot_address_position(&self, slot_address: &SlotAddress) -> Option<Rect> {
        let root = self.inventory_root(slot_address.inventory_type)?;
        return Some(
            ALL_SLOT_ADDRESSS[slot_address]
                .scaled(self.screen_info.effective_height)
                .shifted(root),
        );
    }
    // The icons the slot passes as, best first, then its number as captured and its icon
    // normalised. `seen` is that icon crop where the caller has it already.
    pub fn check_through_all_icons(
        &mut self,
        position: Rect,
        seen: Option<OneIconConfig>,
    ) -> (Vec<(String, f64, u8)>, OneIconConfig, OneIconConfig) {
        let number_position = NUMBER_OFFSET
            .scaled(self.screen_info.effective_height)
            .shifted(position.top_left);
        let (height, scale) = (
            self.screen_info.effective_height,
            self.screen_info.scale_factor,
        );
        let observed = seen.unwrap_or_else(|| {
            let mut crop = self.crop_buffer(position);
            normalize_brightness(&mut crop, self.screen_info.brightness.unwrap());
            crop
        });

        let (decoys, mut passed) = self.get_all_passed_icons(&observed, height, scale);

        let mut sets: Vec<u8> = passed.iter().map(|x| x.2).chain([0]).collect();
        sets.sort();
        sets.dedup();
        let mut closest_decoy = vec![0f64; 1 << OVERLAYS.len()];
        for name in &decoys {
            let template = icon_lookup(name, height, get_resizer(&mut self.resizer));
            let whole_confidence = confidence(&template, &observed);
            for set in &sets {
                let score = match set {
                    0 => whole_confidence,
                    _ if whole_confidence < SLOT_CONFIDENCE - OVERLAY_REACH => whole_confidence,
                    _ => whole_confidence.max(masked_confidence(
                        &template,
                        &observed,
                        &overlay_areas(*set, scale),
                    )),
                };
                closest_decoy[*set as usize] = closest_decoy[*set as usize].max(score);
            }
        }
        passed.retain(|x| x.1 >= closest_decoy[x.2 as usize]);
        passed.sort_by(|a, b| b.1.total_cmp(&a.1));
        let best = passed.first().map_or(0.0, |x| x.1);
        let old_limit = |name: &String| {
            BASE_ICONS.read()[name]
                .required_confidence
                .unwrap_or(DEFAULT_CLOSE_ENOUGH_CONFIDENCE)
        };
        let mut first = true;
        passed.retain(|x: &(String, f64, u8)| {
            std::mem::take(&mut first) || (x.1 >= best - ALTERNATIVE && x.1 > old_limit(&x.0))
        });

        (passed, self.crop_buffer(number_position), observed)
    }

    pub fn update_slots(&mut self) {
        self.scans += 1;
        if self.scans % SLOT_UPDATE_FREQ != 1 {
            return;
        }
        let active_page_nums = self.active_page_num();
        let (start, mut spent, mut matched) = (now(), 0.0, 0);
        self.slots_left = false;
        for slot_address in SLOT_ORDER.iter() {
            if active_page_nums[&slot_address.inventory_type] != Some(slot_address.page_num)
                || self.edits.contains_key(slot_address)
            {
                continue;
            }
            // this extra check is for when there's only 1 pagenum (active_page will return a result but we don't have anchor)
            let Some(position) = self.anchored_slot_address_position(slot_address) else {
                continue;
            };

            // Identical pixels are the usual case and need no comparison at all. Otherwise the slot
            // is unchanged while it is still close to what was seen.
            let (left, top) = position.top_left;
            let raw_hash = self.buffer.hash(
                left as usize,
                top as usize,
                (left + position.width).ceil() as usize,
                (top + position.height).ceil() as usize,
            );
            let same_pixels = self
                .slot_infos
                .get(slot_address)
                .map(|info| info.raw_hash == raw_hash);
            // the crop that was compared, which the matching below takes as it is
            let mut seen = None;
            let unchanged = match same_pixels {
                None => false,
                Some(true) => true,
                Some(false) => {
                    let mut crop = self.crop_buffer(position);
                    normalize_brightness(&mut crop, self.screen_info.brightness.unwrap());
                    let same =
                        close_enough(&self.slot_infos[slot_address].observed_icon, &crop).is_some();
                    seen = Some(crop);
                    if same {
                        self.slot_infos.get_mut(slot_address).unwrap().raw_hash = raw_hash;
                    }
                    same
                }
            };
            if !unchanged {
                // a replay stops where the recording did, whatever the clock says
                let over = match self.replay_cut {
                    Some(cut) => cut.is_some_and(|cut| matched >= cut),
                    None => spent > SLOT_BUDGET && !self.no_slot_budget,
                };
                if over {
                    self.slot_cut.get_or_insert(matched);
                    self.slots_left = true;
                    continue;
                }
                matched += 1;
                let (matched, observed_number, observed_icon) = timed("slots/all_icons", || {
                    self.check_through_all_icons(position, seen)
                });
                let overlays = matched.first().map_or(0, |x| x.2);
                let alternatives: Vec<String> =
                    matched.iter().skip(1).map(|x| x.0.clone()).collect();
                let icon_name_score = matched.into_iter().next().map(|x| (x.0, x.1));
                // what the page shows for this slot: the number strip and the icon below it
                let never_recognised = self
                    .slot_infos
                    .get(slot_address)
                    .is_none_or(|info| info.icon_name_score.is_none());
                let display_icon = (icon_name_score.is_some() || never_recognised).then(|| {
                    let number_height = COMBINED_NUMBER_HEIGHT * self.screen_info.scale_factor;
                    self.crop_buffer(Rect {
                        top_left: (left, top - number_height),
                        height: position.height + number_height,
                        ..position
                    })
                });
                self.changed_slots.insert(*slot_address);
                if icon_name_score.is_some() {
                    // only overwrite if it matches another
                    // the number as captured: the game's brightness setting does not touch it
                    let number = OCRLine {
                        crop: observed_number.data.clone(),
                        kind: Kind::Number {
                            icon: observed_icon.data.clone(),
                            template: icon_name_score.as_ref().unwrap().0.clone(),
                        },
                        brightness: self.screen_info.brightness.unwrap(),
                    };
                    let amount_job = self.request_ocr(number, 0, 0);

                    // what the tooltip said stays while the slot holds the same item, and so does
                    // the last number until the new one is read
                    let (
                        tooltip_amount,
                        tradability,
                        label,
                        amount,
                        amount_reads,
                        hovered,
                        tooltip_failed,
                        best,
                        passed_over,
                    ) = self
                        .slot_infos
                        .get(slot_address)
                        .filter(|old| {
                            old.icon_name_score.as_ref().map(|x| &x.0)
                                == icon_name_score.as_ref().map(|x| &x.0)
                        })
                        .map(|old| {
                            (
                                old.tooltip_amount.clone(),
                                old.tradability,
                                old.label.clone(),
                                old.amount.clone(),
                                old.amount_reads.clone(),
                                old.hovered,
                                old.tooltip_failed,
                                old.best_score,
                                old.passed_over.clone(),
                            )
                        })
                        .unwrap_or_default();
                    self.slot_infos.insert(
                        *slot_address,
                        OneSlotInfo {
                            best_score: best.max(icon_name_score.as_ref().unwrap().1),
                            icon_name_score,
                            overlays,
                            alternatives,
                            observed_number,
                            observed_icon,
                            display_icon,
                            hovered,
                            passed_over,
                            tooltip_failed,
                            amount,
                            amount_reads,
                            tooltip_amount,
                            tradability,
                            label,
                            amount_job: Some(amount_job),
                            raw_hash,
                        },
                    );
                    self.untie_rivals(slot_address);
                } else {
                    // It matches nothing: empty, unknown, or covered for now, so what was known is
                    // kept. The crop is remembered either way and the slot is matched again once it
                    // looks different. That test is strict, or an icon the cursor is slowly leaving
                    // would stay unrecognised.
                    let mut observed_icon = observed_icon;
                    observed_icon.required_confidence = Some(UNMATCHED_UNCHANGED);
                    match self.slot_infos.get_mut(slot_address) {
                        Some(this_slot) => {
                            this_slot.observed_number = observed_number;
                            this_slot.observed_icon = observed_icon;
                            if display_icon.is_some() {
                                this_slot.display_icon = display_icon;
                            }
                            this_slot.raw_hash = raw_hash;
                        }
                        None => {
                            self.slot_infos.insert(
                                *slot_address,
                                OneSlotInfo {
                                    icon_name_score: None,
                                    overlays: 0,
                                    alternatives: vec![],
                                    observed_number,
                                    observed_icon,
                                    display_icon,
                                    hovered: false,
                                    passed_over: (vec![], vec![]),
                                    tooltip_failed: false,
                                    amount: None,
                                    amount_reads: vec![],
                                    tooltip_amount: None,
                                    tradability: None,
                                    label: None,
                                    amount_job: None,
                                    best_score: 0.0,
                                    raw_hash,
                                },
                            );
                        }
                    }
                }
                spent = crate::dev_utils::timing::now() - start;
            }
        }
    }
}

impl ScannerState {
    // A tooltip tied to another slot while this one was not known to hold what it is drawn with
    // may have been this one's: the cursor hides the slot it is on. That slot is to be hovered again.
    fn untie_rivals(&mut self, address: &SlotAddress) {
        let icon = self.slot_infos[address]
            .icon_name_score
            .as_ref()
            .unwrap()
            .0
            .clone();
        let doubted: Vec<SlotAddress> = self
            .slot_infos
            .iter()
            .filter(|(_, info)| {
                info.passed_over.1.contains(address) && info.passed_over.0.contains(&icon)
            })
            .map(|x| *x.0)
            .collect();
        for other in doubted {
            let info = self.slot_infos.get_mut(&other).unwrap();
            (info.hovered, info.tooltip_failed) = (false, false);
            (info.tooltip_amount, info.tradability, info.label) = (None, None, None);
            info.passed_over = (vec![], vec![]);
            for chest in self
                .chests
                .iter_mut()
                .filter(|chest| chest.slot == Some(other))
            {
                chest.slot = None;
                self.chests_changed = true;
            }
        }
    }

    // Debug only: the icons a slot is closest to, as (name, confidence, confidence without the
    // overlays found on it, place in the shortlist's order), whatever the pass limits.
    pub fn slot_scores(&mut self, slot_address: &SlotAddress) -> Vec<(String, f64, f64, usize)> {
        let (height, s) = (
            self.screen_info.effective_height,
            self.screen_info.scale_factor,
        );
        let brightness = self.screen_info.brightness.unwrap();
        let afar_rows = (AFAR_ROWS * s).round() as usize;
        let position = self.anchored_slot_address_position(slot_address).unwrap();
        let mut observed = self.crop_buffer(position);
        normalize_brightness(&mut observed, brightness);
        let seen = from_afar(&observed.data, afar_rows);
        let names: Vec<String> = BASE_ICONS
            .read()
            .iter()
            .filter(|(_, one_icon)| one_icon.tag == "Icon")
            .map(|(name, _)| name.clone())
            .collect();
        let mut all: Vec<(String, f64, f64, f32)> = names
            .into_iter()
            .map(|name| {
                let icon = icon_lookup(&name, height, get_resizer(&mut self.resizer));
                let apart: f32 = from_afar(&icon.data, afar_rows)
                    .iter()
                    .zip(&seen)
                    .map(|(a, b)| (0..3).map(|c| (a[c] - b[c]).abs()).sum::<f32>())
                    .sum();
                let full = confidence(&icon, &observed);
                let masked = overlaid(&icon, &observed, &name, s).0;
                (name, full, masked, apart)
            })
            .collect();
        // DUMP=<folder> writes what the slot looks like next to the icon it is closest to
        if let Ok(folder) = std::env::var("DUMP") {
            let best = all
                .iter()
                .max_by(|a, b| a.1.max(a.2).total_cmp(&b.1.max(b.2)))
                .unwrap();
            let name = format!(
                "{folder}/{:?}_{}_{}_{}",
                slot_address.inventory_type,
                slot_address.page_num,
                slot_address.pos_in_inv.0,
                slot_address.pos_in_inv.1
            );
            observed.data.save(format!("{name}_seen.png")).unwrap();
            let number = COMBINED_NUMBER_HEIGHT * s;
            let mut whole = self.crop_buffer(Rect {
                top_left: (position.top_left.0, position.top_left.1 - number),
                height: position.height + number,
                ..position
            });
            normalize_brightness(&mut whole, brightness);
            whole.data.save(format!("{name}_whole.png")).unwrap();
            let icon = icon_lookup(&best.0, height, get_resizer(&mut self.resizer));
            icon.data
                .save(format!("{name}_icon_{}.png", best.0.replace('@', "_")))
                .unwrap();
        }
        all.sort_by(|a, b| a.3.total_cmp(&b.3));
        let mut all: Vec<(String, f64, f64, usize)> = all
            .into_iter()
            .enumerate()
            .map(|(rank, x)| (x.0, x.1, x.2, rank))
            .collect();
        all.sort_by(|a, b| b.1.max(b.2).total_cmp(&a.1.max(a.2)));
        all.truncate(4);
        all
    }
}
