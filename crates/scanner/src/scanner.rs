use crate::{
    constants::{ALL_ANCHOR_TEMPLATES, ALL_ICONS, ALL_SLOT_ADDRESSS, ANCHOR_BOUNDS},
    scanner_state::{
        ALL_ANCHOR_TYPES, InventoryType, OCRJob, OneAnchorInfo, OneSlotInfo,
        OneSlotProgress::OCRing, ScaledPosition, ScannerState, SlotAddress,
    },
};
use either::Either::Right;

impl ScannerState {
    fn find_active_game_area(&mut self) {
        // TODO
        self.screen_info.start_height = 0;
        self.screen_info.start_width = 0;
        self.screen_info.end_height = 1280;
        self.screen_info.end_width = 1280;

        self.screen_info.effective_width =
            self.screen_info.end_width - self.screen_info.start_width;
        self.screen_info.effective_height =
            self.screen_info.end_height - self.screen_info.start_height;
    }

    fn active_page_num(&self, inv_type: InventoryType) -> Option<usize> {
        if self.anchors.contains_key(&inv_type) {
            Some(self.anchors[&inv_type].variant)
        } else {
            None
        }
    }

    // given the anchor found, find where this icon should be
    fn anchored_position(&self, slot_address: SlotAddress) -> ScaledPosition {
        assert!(self.anchors.contains_key(&slot_address.inventory_type));
        ScaledPosition {
            top_left: (0, 0),
            bot_right: (0, 0),
        }
        //
    }
    pub fn cropper(&mut self) {
        // assert!(self.buffer.pointer.is_some());

        // self.find_active_game_area();

        // // TODO some kind of detection & recovery when the screne size changes?

        // for inv_type in ALL_ANCHOR_TYPES {
        //     if self.anchors.contains_key(&inv_type)
        //         && !self.images_close_enough(
        //             self.access_pixels(self.anchors[&inv_type].position),
        //             self.access_icon_id(self.anchors[&inv_type].id),
        //         )
        //     {
        //         self.anchors.remove(&inv_type);
        //     }

        //     // TODO how reliable is using inv_type as an index? maybe just use usize
        //     if !self.anchors.contains_key(&inv_type) {
        //         if let Some((variant, (position, icon_id))) = ALL_ANCHOR_TEMPLATES
        //             [inv_type as usize]
        //             .iter()
        //             .enumerate()
        //             .find_map(|(variant, &icon_id)| {
        //                 self.template_match(
        //                     self.access_icon_id(icon_id),
        //                     Some(ANCHOR_BOUNDS[inv_type as usize]),
        //                 )
        //                 .map(|position| (variant, (position, icon_id)))
        //             })
        //         {
        //             self.anchors.insert(
        //                 inv_type,
        //                 OneAnchorInfo {
        //                     variant,
        //                     position,
        //                     id: icon_id,
        //                 },
        //             );
        //         }
        //     }
        // }

        // // TODO hover tooltip detection, also filter out slot_address that's being covered

        // for slot_address in ALL_SLOT_ADDRESSS {
        //     let position: ScaledPosition = self.anchored_position(slot_address);
        //     if self.slot_infos.contains_key(&slot_address)
        //         && !self.images_close_enough(
        //             self.access_pixels(position),
        //             self.access_icon_id(self.slot_infos[&slot_address].icon_id),
        //         )
        //     {
        //         self.slot_infos
        //             .get_mut(&slot_address)
        //             .unwrap()
        //             .currently_seen = false; // do it this way because we dont' want to delete previous info just because it got obscured for a bit
        //     }

        //     if !self.slot_infos.contains_key(&slot_address)
        //         || self.slot_infos[&slot_address].currently_seen == false
        //     {
        //         if let Some((icon_id, icon_name)) =
        //             ALL_ICONS
        //                 .iter()
        //                 .enumerate()
        //                 .find_map(|(icon_id, icon_name)| {
        //                     if self
        //                         .slot_infos
        //                         .get(&slot_address)
        //                         .is_some_and(|s| s.icon_id == icon_id)
        //                     {
        //                         return None;
        //                     }
        //                     self.images_close_enough(
        //                         self.access_icon_id(icon_id),
        //                         self.access_pixels(position),
        //                     )
        //                     .then(|| (icon_id, icon_name.clone()))
        //                 })
        //         {
        //             self.slot_infos.insert(
        //                 slot_address,
        //                 OneSlotInfo {
        //                     currently_seen: true,
        //                     icon_id,
        //                     icon_name,
        //                     position,
        //                     amount: Right(OCRing),
        //                     tradability: Right(OCRing),
        //                 },
        //             );
        //             self.pending_jobs.push(OCRJob {
        //                 cropped: self.access_pixels(position).collect(),
        //                 slot_address,
        //             })
        //         }
        //     }
        // }
    }
}
