use crate::{
    constants::{ALL_SLOT_ADDRESSS, COMBINED_NUMBER_HEIGHT, NUMBER_OFFSET},
    image_utils::{
        brightness::{mean_intensity, normalize_brightness},
        close_enough::{DEFAULT_CONFIDENCE, close_enough, confidence_without},
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        ocr::pre_process_icon_number,
        resize::crop_buffer,
    },
    scanner_state::{OneSlotInfo, ScannerState, SlotAddress},
    setup::{BASE_ICONS, OneIconConfig, icon_lookup},
    timing::timed,
    tooltip::items::has_level,
};
use ahash::AHashMap;
use image::RgbaImage;
use parking_lot::RwLock;
use std::sync::LazyLock;
// use hf_core::my_dbg;
// use uuid::Uuid;

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
    ) -> Option<FloatRectangle> {
        let root = self.inventory_root(slot_address.inventory_type)?;
        return Some(
            ALL_SLOT_ADDRESSS[slot_address]
                .scaled(self.screen_info.scale_factor)
                .use_root(&root),
        );
    }
    pub fn check_through_all_icons(
        &mut self,
        position: FloatRectangle,
        position_root: IntegerRectangle,
    ) -> (Vec<(String, f64, bool)>, OneIconConfig, OneIconConfig) {
        let number_position = NUMBER_OFFSET
            .scaled(self.screen_info.scale_factor)
            .use_root(&position);
        let mut observed: OneIconConfig = crop_buffer(
            position,
            get_resizer(&mut self.resizer),
            self.buffer,
            Some(position_root),
        );

        let (height, s) = (self.screen_info.effective_height, self.screen_info.scale_factor);
        let brightness = self.screen_info.brightness.unwrap();
        let level_rows = ((LEVEL_ROWS.0 * s).round() as usize, (LEVEL_ROWS.1 * s).round() as usize);
        if !FROM_AFAR.read().contains_key(&height) {
            let names: Vec<String> = BASE_ICONS
                .read()
                .iter()
                .filter(|(_, one_icon)| one_icon.tag == "Icon")
                .map(|(name, _)| name.clone())
                .collect();
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
        normalize_brightness(&mut observed, brightness);
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
                    if let Some(score) = close_enough(&template, &mut observed, brightness) {
                        return Some((icon_name, score, false));
                    }
                    // a chest that asks for an item level has it written across the slot
                    let limit = template.required_confidence.unwrap_or(DEFAULT_CONFIDENCE);
                    confidence_without(&template, &mut observed, brightness, level_rows)
                        .filter(|score| has_level(&icon_name) && *score > limit)
                        .map(|score| (icon_name, score, true))
                })
                .collect();
        passed.sort_by(|a, b| b.1.total_cmp(&a.1));
        let best = passed.first().map_or(0.0, |x| x.1);
        passed.retain(|x| x.1 >= best - ALTERNATIVE);

        (
            passed,
            crop_buffer(
                number_position,
                get_resizer(&mut self.resizer),
                self.buffer,
                Some(position_root),
            ),
            observed,
        )
    }

    pub fn update_slots(&mut self) {
        self.scans += 1;
        if self.scans % SLOT_EVERY != 1 {
            return;
        }
        let active_page_nums = self.active_page_num();
        let (start, mut spent) = (crate::timing::now(), 0.0);
        self.slots_left = false;
        // my_dbg!(active_page_nums, self.anchors);
        for slot_address in ALL_SLOT_ADDRESSS.keys() {
            if active_page_nums[&slot_address.inventory_type] != Some(slot_address.page_num)
                || self.edits.contains_key(slot_address)
            {
                continue;
            }
            // this extra check is for when there's only 1 pagenum (active_page will return a result but we don't have anchor)
            let Some(root) = self.inventory_root(slot_address.inventory_type) else {
                continue;
            };

            // my_dbg!(active_page_nums[&slot_address.inventory_type]);
            let position: FloatRectangle =
                self.anchored_slot_address_position(slot_address).unwrap();

            // Identical pixels are the usual case and need no comparison at all. Otherwise the slot
            // is unchanged while it is still close to what was seen.
            let (left, top) = position.top_left;
            let raw_hash = self.buffer.hash(
                left as usize,
                top as usize,
                (left + position.width).ceil() as usize,
                (top + position.height).ceil() as usize,
            );
            let unchanged = self.slot_infos.get(slot_address).is_some_and(|info| {
                info.raw_hash == raw_hash
                    || close_enough(
                        &info.observed_icon,
                        &mut crop_buffer(
                            position,
                            get_resizer(&mut self.resizer),
                            self.buffer,
                            None,
                        ),
                        self.screen_info.brightness.unwrap(),
                    )
                    .is_some()
            });
            if unchanged {
                self.slot_infos.get_mut(slot_address).unwrap().raw_hash = raw_hash;
            } else {
                if spent > SLOT_BUDGET {
                    self.slots_left = true;
                    continue;
                }
                let (matched, observed_number, observed_icon) =
                    timed("slots/all_icons", || {
                        self.check_through_all_icons(position, root)
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
                    crop_buffer(
                        FloatRectangle {
                            top_left: (left, top - number_height),
                            width: position.width,
                            height: position.height + number_height,
                        },
                        get_resizer(&mut self.resizer),
                        self.buffer,
                        None,
                    )
                });
                let debug_start = crate::timing::now();
                if self.debugging {
                    let raw_icon =
                        crop_buffer(position, get_resizer(&mut self.resizer), self.buffer, None);
                    // let number = crop_buffer(
                    //     NUMBER_OFFSET
                    //         .scaled(self.screen_info.scale_factor)
                    //         .use_root(&position),
                    //     get_resizer(&mut self.resizer),
                    //     self.buffer,
                    //     None,
                    // );
                    let mut out = vec![raw_icon.clone(), observed_icon.clone()];
                    if let Some((name, _)) = &icon_name_score {
                        out.push(
                            icon_lookup(
                                name,
                                self.screen_info.effective_height,
                                get_resizer(&mut self.resizer),
                            )
                            .clone(),
                        );
                    }

                    let name = format!(
                        "{:?} slot{:?}",
                        slot_address.inventory_type, slot_address.pos_in_inv
                    );
                    self.changed_debug.insert(name.clone());
                    self.debug_info.insert(
                        name,
                        (
                            position.to_rounded(),
                            mean_intensity(&raw_icon),
                            icon_name_score.clone().unwrap_or_default().1,
                            out,
                        ),
                    );
                };
                crate::timing::record("slots/debug_crops", crate::timing::now() - debug_start);
                // my_dbg!("New", slot_address, "icon:", icon_name_score);
                self.changed_slots.insert(*slot_address);
                if icon_name_score.is_some() {
                    // only overwrite if it matches another
                    let pre_processed = timed("slots/number_preprocess", || {
                        pre_process_icon_number(
                            observed_number.clone(),
                            &observed_icon.data,
                            &BASE_ICONS.read()[&icon_name_score.as_ref().unwrap().0].data,
                            self.screen_info.brightness.unwrap(),
                        )
                    });
                    let amount_job = self.request_ocr(pre_processed.data.clone(), 0);

                    // what the tooltip said stays while the slot holds the same item, and so does
                    // the last number until the new one is read
                    let (tooltip_amount, tradability, label, amount, hovered, tooltip_failed) = self
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
                            processed_number: pre_processed,
                            amount,
                            tooltip_amount,
                            tradability,
                            label,
                            currently_seen: true,
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
                            this_slot.currently_seen = false;
                            this_slot.raw_hash = raw_hash;
                        }
                        None => {
                            self.slot_infos.insert(
                                *slot_address,
                                OneSlotInfo {
                                    icon_name_score: None,
                                    levelled: false,
                                    alternatives: vec![],
                                    processed_number: observed_number.clone(),
                                    observed_number,
                                    observed_icon,
                                    display_icon,
                                    hovered: false,
                                    tooltip_failed: false,
                                    currently_seen: false,
                                    amount: None,
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
        let mut observed = crop_buffer(position, get_resizer(&mut self.resizer), self.buffer, None);
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
                let full = crate::image_utils::close_enough::confidence(&icon, &mut observed, brightness).unwrap();
                let masked = confidence_without(&icon, &mut observed, brightness, level_rows).unwrap();
                (name, full, masked, apart)
            })
            .collect();
        // DUMP=<folder> writes what the slot looks like next to the icon it is closest to
        if let Ok(folder) = std::env::var("DUMP") {
            let best = all.iter().max_by(|a, b| a.1.max(a.2).total_cmp(&b.1.max(b.2))).unwrap();
            let name = format!("{folder}/{:?}_{}_{}_{}", slot_address.inventory_type, slot_address.page_num, slot_address.pos_in_inv.0, slot_address.pos_in_inv.1);
            observed.data.save(format!("{name}_seen.png")).unwrap();
            let number = COMBINED_NUMBER_HEIGHT * s;
            let mut whole = crop_buffer(
                FloatRectangle {
                    top_left: (position.top_left.0, position.top_left.1 - number),
                    width: position.width,
                    height: position.height + number,
                },
                get_resizer(&mut self.resizer),
                self.buffer,
                None,
            );
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
