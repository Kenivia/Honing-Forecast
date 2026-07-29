use ahash::AHashMap;
use either::Either::Right;

use crate::{
    constants::{ALL_ICONS, ALL_SLOT_ADDRESSS},
    scanner_state::{
        OCRJob, OneSlotInfo, OneSlotProgress::OCRing, ScaledPosition, ScannerState, SlotAddress,
    },
    setup::icon_lookup,
};

impl ScannerState {
    pub fn initialize_slots(&mut self) {
        self.slot_infos = AHashMap::new();
        for &slot_address in ALL_SLOT_ADDRESSS.keys() {
            self.slot_infos.insert(
                slot_address,
                OneSlotInfo {
                    currently_seen: None,
                    icon_name: None,
                    position: None,
                    amount: None,
                    tradability: None,
                },
            );
        }
    }

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
        //
    }

    pub fn update_slots(&mut self) {
        let active_page_nums = self.active_page_num();
        for slot_address in ALL_SLOT_ADDRESSS.keys() {
            if active_page_nums[&slot_address.inventory_type] != Some(slot_address.page_num) {
                continue;
            }

            let position: Option<ScaledPosition> =
                self.anchored_slot_address_position(slot_address);
            if position.is_none() {
                continue;
            }
            let this_slot: &OneSlotInfo = self.slot_infos.get(slot_address).unwrap();

            if this_slot.currently_seen == Some(true)
                && !self.images_close_enough(
                    icon_lookup(
                        &self.slot_infos
                            .get(slot_address)
                            .unwrap()
                            .icon_name
                            .unwrap(),
                    ),
                    self.downscale(position.unwrap()),
                )
            {
                this_slot.currently_seen = Some(false); // doing it this way because we dont' want to delete previous info just because it got obscured for a bit
            }

            if self.slot_infos[slot_address]
                .currently_seen
                .is_none_or(|x| x == false)
            {}

            //     // TODO filter all icons here to limit to actual icons
            //     if let Some((icon_id, icon_name)) =
            //         ALL_ICONS
            //             .iter()
            //             .enumerate()
            //             .find_map(|(icon_id, icon_name)| {
            //                 if self
            //                     .slot_infos
            //                     .get(&slot_address)
            //                     .unwrap()
            //                     s.icon_name.unwrap() == *icon_name)
            //                 {
            //                     return None;
            //                 }
            //                 self.images_close_enough(
            //                     icon_lookup(icon_name),
            //                     self.downscale(position.unwrap()),
            //                 )
            //                 .then(|| (icon_id, icon_name.clone()))
            //             })
            //     {
            //         this_slot(
            //             *slot_address,
            //             OneSlotInfo {
            //                 currently_seen: true,
            //                 icon_id,
            //                 icon_name,
            //                 position: position.unwrap(),
            //                 amount: Right(OCRing),
            //                 tradability: Right(OCRing),
            //             },
            //         );
            //         self.pending_jobs.push(OCRJob {
            //             cropped: self.downscale(position.unwrap()).into_vec(),
            //             slot_address: *slot_address,
            //         })
            //     }
            // }
        }
    }
}
