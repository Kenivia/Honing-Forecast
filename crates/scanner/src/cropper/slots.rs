use crate::{
    constants::{ALL_SLOT_ADDRESSS, COMBINED_NUMBER_HEIGHT, NUMBER_OFFSET, SLOT_ORDER},
    image_utils::{
        brightness::normalize_brightness,
        close_enough::{DEFAULT_CONFIDENCE, close_enough, confidence, confidence_without},
        common::{Rect, get_resizer},
    },
    ocr::jobs::{Kind, Line},
    scanner_state::{OneSlotInfo, ScannerState, SlotAddress},
    setup::{BASE_ICONS, OneIconConfig, icon_lookup},
    timing::timed,
    tooltip::items::has_level,
};
use ahash::AHashMap;
use image::RgbaImage;
use parking_lot::RwLock;
use std::sync::LazyLock;

const UNMATCHED_UNCHANGED: f64 = 0.995;
// Slots change slowly and a tooltip can be gone in a few frames, so the slots are only looked
// at on every third scan. That keeps the average scan well under a frame at 30 frames a second.
const SLOT_EVERY: u64 = 3;
// ms of matching and number clean-up per look; slots left over are picked up by the next ones,
// so opening a page does not hold up the frames that follow it
const SLOT_BUDGET: f64 = 60.0;
// Rows of the icon a chest's item level is written over, from the top of what is compared (1440p).
// Above them is the strip under the number, below them the last few rows.
const LEVEL_ROWS: (f64, f64) = (18.0, 33.0);
// Only the icons that look most like the slot from afar are compared in full: comparing all 340
// takes 13 ms a slot against 0.7, and on the recordings recognises nothing more.
const SHORTLIST: usize = 16;
// Icons this close to the best one are kept as what the slot may also be: one plain chest scored
// 0.967, 0.963 and 0.957 against three different chests.
const ALTERNATIVE: f64 = 0.02;
const CELLS: usize = 6;

// each icon as the mean colour of a few cells of its top rows, per resolution
pub static FROM_AFAR: LazyLock<RwLock<AHashMap<u32, Vec<(String, Vec<[f32; 3]>)>>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));

// the rows above the item level, which every slot shows
fn from_afar(image: &RgbaImage, rows: usize) -> Vec<[f32; 3]> {
    let (width, rows) = (image.width() as usize, rows.min(image.height() as usize));
    let mut cells = vec![[0f32; 3]; CELLS * 2];
    let mut counts = vec![0f32; CELLS * 2];
    for y in 0..rows {
        for x in 0..width {
            let cell = (y * 2 / rows) * CELLS + x * CELLS / width;
            let pixel = image.get_pixel(x as u32, y as u32).0;
            for c in 0..3 {
                cells[cell][c] += pixel[c] as f32;
            }
            counts[cell] += 1.0;
        }
    }
    for (cell, count) in cells.iter_mut().zip(counts) {
        *cell = cell.map(|x| x / count);
    }
    cells
}

impl ScannerState {
    // given the anchor found, find where this icon should be
    pub fn anchored_slot_address_position(
        &self,
        slot_address: &SlotAddress,
    ) -> Option<Rect> {
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
    ) -> (Vec<(String, f64, bool)>, OneIconConfig, OneIconConfig) {
        let number_position = NUMBER_OFFSET
            .scaled(self.screen_info.effective_height)
            .shifted(position.top_left);
        let (height, s) = (self.screen_info.effective_height, self.screen_info.scale_factor);
        let brightness = self.screen_info.brightness.unwrap();
        let observed = seen.unwrap_or_else(|| {
            let mut crop = self.crop_buffer(position);
            normalize_brightness(&mut crop, brightness);
            crop
        });
        let level_rows = ((LEVEL_ROWS.0 * s).round() as usize, (LEVEL_ROWS.1 * s).round() as usize);
        if !FROM_AFAR.read().contains_key(&height) {
            let names: Vec<String> = BASE_ICONS
                .read()
                .iter()
                .filter(|(_, one_icon)| one_icon.tag == "Icon")
                .map(|(name, _)| name.clone())
                .collect();
            let mut names = names;
            names.sort();
            let all = names
                .into_iter()
                .map(|name| {
                    let icon = icon_lookup(&name, height, get_resizer(&mut self.resizer));
                    let cells = from_afar(&icon.data, level_rows.0);
                    (name, cells)
                })
                .collect();
            FROM_AFAR.write().insert(height, all);
        }
        let seen = from_afar(&observed.data, level_rows.0);
        let mut shortlist: Vec<(f32, String)> = FROM_AFAR.read()[&height]
            .iter()
            .map(|(name, cells)| {
                let apart: f32 = cells
                    .iter()
                    .zip(&seen)
                    .map(|(a, b)| (0..3).map(|c| (a[c] - b[c]).abs()).sum::<f32>())
                    .sum();
                (apart, name.clone())
            })
            .collect();
        if shortlist.len() > SHORTLIST {
            shortlist.select_nth_unstable_by(SHORTLIST, |a, b| a.0.total_cmp(&b.0));
            shortlist.truncate(SHORTLIST);
        }

        // color variants of one icon can both pass, so the best comes first
        let mut passed: Vec<(String, f64, bool)> =
            shortlist
                .into_iter()
                .filter_map(|(_, icon_name)| {
                    let template = icon_lookup(&icon_name, height, get_resizer(&mut self.resizer));
                    if let Some(score) = close_enough(&template, &observed) {
                        return Some((icon_name, score, false));
                    }
                    // a chest that asks for an item level has it written across the slot
                    let limit = template.required_confidence.unwrap_or(DEFAULT_CONFIDENCE);
                    Some(confidence_without(&template, &observed, level_rows))
                        .filter(|score| has_level(&icon_name) && *score > limit)
                        .map(|score| (icon_name, score, true))
                })
                .collect();
        passed.sort_by(|a, b| b.1.total_cmp(&a.1));
        let best = passed.first().map_or(0.0, |x| x.1);
        passed.retain(|x| x.1 >= best - ALTERNATIVE);

        (passed, self.crop_buffer(number_position), observed)
    }

    pub fn update_slots(&mut self) {
        self.scans += 1;
        if self.scans % SLOT_EVERY != 1 {
            return;
        }
        let active_page_nums = self.active_page_num();
        let (start, mut spent, mut matched) = (crate::timing::now(), 0.0, 0);
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
            let same_pixels = self.slot_infos.get(slot_address).map(|info| info.raw_hash == raw_hash);
            // the crop that was compared, which the matching below takes as it is
            let mut seen = None;
            let unchanged = match same_pixels {
                None => false,
                Some(true) => true,
                Some(false) => {
                    let mut crop = self.crop_buffer(position);
                    normalize_brightness(&mut crop, self.screen_info.brightness.unwrap());
                    let same = close_enough(&self.slot_infos[slot_address].observed_icon, &crop).is_some();
                    seen = Some(crop);
                    same
                }
            };
            if unchanged {
                self.slot_infos.get_mut(slot_address).unwrap().raw_hash = raw_hash;
            } else {
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
                let (matched, observed_number, observed_icon) =
                    timed("slots/all_icons", || {
                        self.check_through_all_icons(position, seen)
                    });
                let levelled = matched.first().is_some_and(|x| x.2);
                let alternatives: Vec<String> = matched.iter().skip(1).map(|x| x.0.clone()).collect();
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
                    let number = Line {
                        crop: observed_number.data.clone(),
                        kind: Kind::Number {
                            icon: observed_icon.data.clone(),
                            template: icon_name_score.as_ref().unwrap().0.clone(),
                        },
                        brightness: self.screen_info.brightness.unwrap(),
                    };
                    let amount_job = self.request_ocr(number, 0);

                    // what the tooltip said stays while the slot holds the same item, and so does
                    // the last number until the new one is read
                    let (tooltip_amount, tradability, label, amount, amount_reads, hovered, tooltip_failed) = self
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
                            )
                        })
                        .unwrap_or_default();
                    self.slot_infos.insert(
                        *slot_address,
                        OneSlotInfo {
                            icon_name_score,
                            levelled,
                            alternatives,
                            observed_number,
                            observed_icon,
                            display_icon,
                            hovered,
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
                                    levelled: false,
                                    alternatives: vec![],
                                    observed_number,
                                    observed_icon,
                                    display_icon,
                                    hovered: false,
                                    tooltip_failed: false,
                                    amount: None,
                                    amount_reads: vec![],
                                    tooltip_amount: None,
                                    tradability: None,
                                    label: None,
                                    amount_job: None,
                                    raw_hash,
                                },
                            );
                        }
                    }
                }
                spent = crate::timing::now() - start;
            }
        }
    }
}

impl ScannerState {
    // Debug only: the icons a slot is closest to, as (name, confidence, confidence without the
    // item level's rows, place in the shortlist's order), whatever the pass limits.
    pub fn slot_scores(&mut self, slot_address: &SlotAddress) -> Vec<(String, f64, f64, usize)> {
        let (height, s) = (self.screen_info.effective_height, self.screen_info.scale_factor);
        let brightness = self.screen_info.brightness.unwrap();
        let level_rows = ((LEVEL_ROWS.0 * s).round() as usize, (LEVEL_ROWS.1 * s).round() as usize);
        let position = self.anchored_slot_address_position(slot_address).unwrap();
        let mut observed = self.crop_buffer(position);
        normalize_brightness(&mut observed, brightness);
        let seen = from_afar(&observed.data, level_rows.0);
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
                let apart: f32 = from_afar(&icon.data, level_rows.0)
                    .iter()
                    .zip(&seen)
                    .map(|(a, b)| (0..3).map(|c| (a[c] - b[c]).abs()).sum::<f32>())
                    .sum();
                let full = confidence(&icon, &observed);
                let masked = confidence_without(&icon, &observed, level_rows);
                (name, full, masked, apart)
            })
            .collect();
        // DUMP=<folder> writes what the slot looks like next to the icon it is closest to
        if let Ok(folder) = std::env::var("DUMP") {
            let best = all.iter().max_by(|a, b| a.1.max(a.2).total_cmp(&b.1.max(b.2))).unwrap();
            let name = format!("{folder}/{:?}_{}_{}_{}", slot_address.inventory_type, slot_address.page_num, slot_address.pos_in_inv.0, slot_address.pos_in_inv.1);
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
            icon.data.save(format!("{name}_icon_{}.png", best.0.replace('@', "_"))).unwrap();
        }
        all.sort_by(|a, b| a.3.total_cmp(&b.3));
        let mut all: Vec<(String, f64, f64, usize)> =
            all.into_iter().enumerate().map(|(rank, x)| (x.0, x.1, x.2, rank)).collect();
        all.sort_by(|a, b| b.1.max(b.2).total_cmp(&a.1.max(a.2)));
        all.truncate(4);
        all
    }
}
