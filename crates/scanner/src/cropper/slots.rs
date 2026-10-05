use crate::{
    constants::{ALL_SLOT_ADDRESSS, NUMBER_OFFSET},
    image_utils::{
        brightness::mean_intensity,
        close_enough::close_enough,
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        ocr::pre_process_icon_number,
        resize::crop_buffer,
    },
    scanner_state::{OneSlotInfo, ScannerState, SlotAddress},
    setup::{BASE_ICONS, OneIconConfig, icon_lookup},
    timing::timed,
};
use hf_core::my_dbg;
// use uuid::Uuid;

const UNMATCHED_UNCHANGED: f64 = 0.995;
// Slots change slowly and a tooltip can be gone in a few frames, so the slots are only looked
// at on every third scan. That keeps the average scan well under a frame at 30 frames a second.
const SLOT_EVERY: u64 = 3;
// ms of matching and number clean-up per look; slots left over are picked up by the next ones,
// so opening a page does not hold up the frames that follow it
const SLOT_BUDGET: f64 = 60.0;

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
    ) -> (Option<(String, f64)>, OneIconConfig, OneIconConfig) {
        let number_position = NUMBER_OFFSET
            .scaled(self.screen_info.scale_factor)
            .use_root(&position);
        let mut observed: OneIconConfig = crop_buffer(
            position,
            get_resizer(&mut self.resizer),
            self.buffer,
            Some(position_root),
        );

        (
            BASE_ICONS
                .read()
                .iter()
                .filter(|(_, one_icon)| one_icon.tag == "Icon")
                // color variants of one icon can both pass, so take the best
                .filter_map(|(icon_name, _)| {
                    close_enough(
                        &icon_lookup(
                            icon_name,
                            self.screen_info.effective_height,
                            get_resizer(&mut self.resizer),
                        ),
                        &mut observed,
                        self.screen_info.brightness.unwrap(),
                    )
                    .map(|score| (icon_name.clone(), score))
                })
                .max_by(|a, b| a.1.total_cmp(&b.1)),
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
            if active_page_nums[&slot_address.inventory_type] != Some(slot_address.page_num) {
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
                let (icon_name_score, observed_number, observed_icon) =
                    timed("slots/all_icons", || self.check_through_all_icons(position, root));
                let debug_start = crate::timing::now();
                if self.debugging {
                    let raw_icon =
                        crop_buffer(position, get_resizer(&mut self.resizer), self.buffer, None);
                    let number = crop_buffer(
                        NUMBER_OFFSET
                            .scaled(self.screen_info.scale_factor)
                            .use_root(&position),
                        get_resizer(&mut self.resizer),
                        self.buffer,
                        None,
                    );
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
                    let (tooltip_amount, tradability, amount) = self
                        .slot_infos
                        .get(slot_address)
                        .filter(|old| {
                            old.icon_name_score.as_ref().map(|x| &x.0)
                                == icon_name_score.as_ref().map(|x| &x.0)
                        })
                        .map(|old| (old.tooltip_amount.clone(), old.tradability, old.amount.clone()))
                        .unwrap_or_default();
                    self.slot_infos.insert(
                        *slot_address,
                        OneSlotInfo {
                            icon_name_score,
                            observed_number,
                            observed_icon,
                            processed_number: pre_processed,
                            amount,
                            tooltip_amount,
                            tradability,
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
                            this_slot.currently_seen = false;
                            this_slot.raw_hash = raw_hash;
                        }
                        None => {
                            self.slot_infos.insert(
                                *slot_address,
                                OneSlotInfo {
                                    icon_name_score: None,
                                    processed_number: observed_number.clone(),
                                    observed_number,
                                    observed_icon,
                                    currently_seen: false,
                                    amount: None,
                                    tooltip_amount: None,
                                    tradability: None,
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
