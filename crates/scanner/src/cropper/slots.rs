use crate::{
    constants::{ALL_SLOT_ADDRESSS, NUMBER_OFFSET},
    image_utils::{
        close_enough::close_enough,
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        downscale::crop_buffer,
    },
    scanner_state::{OneSlotInfo, OneSlotProgress, ScannerState, SlotAddress},
    setup::{BASE_ICONS, COMPUTED_ICONS, OneIconConfig, icon_lookup},
};
use hf_core::my_dbg;
use uuid::Uuid;

impl ScannerState {
    // given the anchor found, find where this icon should be
    pub fn anchored_slot_address_position(
        &self,
        slot_address: &SlotAddress,
    ) -> Option<FloatRectangle> {
        if !self.anchors[&slot_address.inventory_type].is_found() {
            return None;
        }
        return Some(
            ALL_SLOT_ADDRESSS[slot_address]
                .scaled(self.screen_info.scale_factor)
                .use_root(
                    &self.anchors[&slot_address.inventory_type]
                        .position_root
                        .unwrap(),
                ),
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

        (
            BASE_ICONS
                .read()
                .iter()
                .filter(|(_, one_icon)| one_icon.tag == "Icon")
                .find_map(|(icon_name, _)| {
                    let x = close_enough(
                        &icon_lookup(
                            icon_name,
                            self.screen_info.effective_height,
                            get_resizer(&mut self.resizer),
                        ),
                        crop_buffer(position, get_resizer(&mut self.resizer), self.buffer), // cloning doesn't seem to exist for Image
                    );
                    // my_dbg!(icon_name, x);
                    if x.is_some() {
                        return Some((icon_name.clone(), x.unwrap()));
                    }
                    None
                }),
            OneIconConfig {
                data: crop_buffer(number_position, get_resizer(&mut self.resizer), self.buffer)
                    .into_vec(),
                name: "".to_string(),
                offset: (number_position.get_offset(&position_root)).to_rounded(),
                tag: "".to_string(),
            },
            OneIconConfig {
                data: crop_buffer(position, get_resizer(&mut self.resizer), self.buffer).into_vec(),
                name: "".to_string(),
                offset: (position.get_offset(&position_root)).to_rounded(),
                tag: "".to_string(),
            },
        )
    }

    pub fn update_slots(&mut self) {
        let active_page_nums = self.active_page_num();
        my_dbg!(active_page_nums, self.anchors);
        for slot_address in ALL_SLOT_ADDRESSS.keys() {
            if active_page_nums[&slot_address.inventory_type] != Some(slot_address.page_num)
                || !self.anchors[&slot_address.inventory_type].is_found()
            // this extra check is for when there's only 1 pagenum (active_page will return a result but we don't have anchor)
            {
                continue;
            }

            // my_dbg!(active_page_nums[&slot_address.inventory_type]);
            let position: FloatRectangle =
                self.anchored_slot_address_position(slot_address).unwrap();

            if !self.slot_infos.contains_key(slot_address) {
                let (icon_name_score, observed_number, observed_icon) = self
                    .check_through_all_icons(
                        position,
                        self.anchors[&slot_address.inventory_type]
                            .position_root
                            .unwrap(),
                    );
                if self.debugging {
                    self.debug_info.insert(
                        "slot".to_string() + &format!("{:?}", slot_address.pos_in_inv),
                        (position.to_rounded(), -6.9, 6.9),
                    );
                };
                // my_dbg!("New", slot_address, "icon:", icon_name_score);

                self.slot_infos.insert(
                    *slot_address,
                    OneSlotInfo {
                        // currently_seen: true,
                        icon_name_score: icon_name_score.clone(),
                        observed_number,
                        observed_icon,
                        observed_id: Uuid::new_v4(),
                        progress: if icon_name_score.is_some() {
                            OneSlotProgress::OCRing
                        } else {
                            OneSlotProgress::NA
                        },
                        amount: None,
                        tradability: None,
                    },
                );
            } else {
                if self.slot_infos[slot_address].progress == OneSlotProgress::NA
                    && close_enough(
                        &self.slot_infos[slot_address].observed_icon,
                        crop_buffer(position, get_resizer(&mut self.resizer), self.buffer), // cloning doesn't seem to exist for Image
                    )
                    .is_none()
                {
                    // only run the check if it changed
                    let (icon_name_score, observed_number, observed_icon) = self
                        .check_through_all_icons(
                            position,
                            self.anchors[&slot_address.inventory_type]
                                .position_root
                                .unwrap(),
                        );
                    // my_dbg!("Old", slot_address, "icon:", icon_name_score.clone());
                    if icon_name_score.is_some() {
                        // only overwrite if it matches another
                        *self.slot_infos.get_mut(slot_address).unwrap() = OneSlotInfo {
                            // currently_seen: true,
                            icon_name_score,
                            observed_number,
                            observed_icon,
                            observed_id: Uuid::new_v4(),
                            progress: OneSlotProgress::OCRing,
                            amount: None,
                            tradability: None,
                        }
                    } else {
                        let this_slot = self.slot_infos.get_mut(slot_address).unwrap();
                        this_slot.observed_number = observed_number;
                        this_slot.observed_icon = observed_icon;
                        this_slot.progress = OneSlotProgress::NA;
                        this_slot.observed_id = Uuid::new_v4();
                    }
                }
            }
        }
    }
}
