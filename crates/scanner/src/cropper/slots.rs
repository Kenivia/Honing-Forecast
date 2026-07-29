use crate::{
    constants::{ALL_ICONS, ALL_SLOT_ADDRESSS, NUMBER_OFFSET},
    scanner_state::{OneSlotInfo, OneSlotProgress, ScaledPosition, ScannerState, SlotAddress},
    setup::{OneIconConfig, icon_lookup},
};

impl ScannerState {
    // given the anchor found, find where this icon should be
    pub fn anchored_slot_address_position(
        &self,
        slot_address: &SlotAddress,
    ) -> Option<ScaledPosition> {
        if !self.anchors[&slot_address.inventory_type].is_found() {
            return None;
        }

        return Some(
            ALL_SLOT_ADDRESSS[slot_address]
                + self.anchors[&slot_address.inventory_type]
                    .position_root
                    .unwrap(),
        );

    }
    pub fn check_through_all_icons(
        &self,
        position: ScaledPosition,
        position_root: ScaledPosition,
    ) -> (Option<String>, OneIconConfig, OneIconConfig) {
        let observed_number = OneIconConfig {
            data: self.downscale(NUMBER_OFFSET + position).into_vec(),
            name: "".to_string(),
            offset: (NUMBER_OFFSET + position) - position_root,
            tag: "".to_string(),
        };
        let observed_icon = OneIconConfig {
            data: self.downscale(position).into_vec(),
            name: "".to_string(),
            offset: position - position_root,
            tag: "".to_string(),
        };

        (
            ALL_ICONS.iter().find_map(|icon_name| {
                self.images_close_enough(
                    icon_lookup(icon_name),
                    self.downscale(position), // cloning doesn't seem to exist for Image
                )
                .then(|| icon_name.clone())
            }),
            observed_number,
            observed_icon,
        )
    }

    pub fn update_slots(&mut self) {
        let active_page_nums = self.active_page_num();
        for slot_address in ALL_SLOT_ADDRESSS.keys() {
            if active_page_nums[&slot_address.inventory_type] != Some(slot_address.page_num) {
                continue;
            }
            // active_page_nums being present means anchor is  ready
            let position: ScaledPosition =
                self.anchored_slot_address_position(slot_address).unwrap();

            if !self.slot_infos.contains_key(slot_address) {
                let (icon_name, observed_number, observed_icon) = self.check_through_all_icons(
                    position,
                    self.anchors[&slot_address.inventory_type]
                        .position_root
                        .unwrap(),
                );

                self.slot_infos.insert(
                    *slot_address,
                    OneSlotInfo {
                        // currently_seen: true,
                        icon_name: icon_name.clone(),
                        observed_number,
                        observed_icon,
                        progress: if icon_name.is_some() {
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
                    && !self.images_close_enough(
                        &self.slot_infos[slot_address].observed_icon,
                        self.downscale(position), // cloning doesn't seem to exist for Image
                    )
                {
                    // only run the check if it changed
                    let (icon_name, observed_number, observed_icon) = self.check_through_all_icons(
                        position,
                        self.anchors[&slot_address.inventory_type]
                            .position_root
                            .unwrap(),
                    );
                    if icon_name.is_some() {
                        // only overwrite if it matches another
                        *self.slot_infos.get_mut(slot_address).unwrap() = OneSlotInfo {
                            // currently_seen: true,
                            icon_name: icon_name,
                            observed_number,
                            observed_icon,
                            progress: OneSlotProgress::OCRing,
                            amount: None,
                            tradability: None,
                        }
                    } else {
                        self.slot_infos
                            .get_mut(slot_address)
                            .unwrap()
                            .observed_number = observed_number;
                        self.slot_infos.get_mut(slot_address).unwrap().observed_icon =
                            observed_icon;
                        self.slot_infos.get_mut(slot_address).unwrap().progress =
                            OneSlotProgress::NA;
                    }
                }
            }

            // if position.is_none() {
            //     continue;
            // }
            // let this_slot: &OneSlotInfo = self.slot_infos.get(slot_address).unwrap();

            // if this_slot.currently_seen == Some(true)
            //     && !self.images_close_enough(
            //         icon_lookup(
            //             &self.slot_infos
            //                 .get(slot_address)
            //                 .unwrap()
            //                 .icon_name
            //                 .unwrap(),
            //         ),
            //         self.downscale(position.unwrap()),
            //     )
            // {
            //     this_slot.currently_seen = Some(false); // doing it this way because we dont' want to delete previous info just because it got obscured for a bit
            // }

            // if self.slot_infos[slot_address]
            //     .currently_seen
            //     .is_none_or(|x| x == false)
            // {}

            //     // TODO filter all icons here to limit to actual icons

            // }
        }
    }
}
