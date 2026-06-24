// use crate::{
//     constants::{ALL_ANCHOR_TEMPLATES, ALL_ICONS, ALL_SLOT_ADDRESSS, ANCHOR_BOUNDS},
//     image_utils::hash_close_enough,
//     scanner_state::{
//         ALL_ANCHOR_TYPES, InventoryType, OCRJob, OneAnchorInfo, OneSlotInfo,
//         OneSlotProgress::OCRing, ScaledPosition, ScannerState, SlotAddress,
//     },
// };
use either::Either::Right;

use crate::scanner_state::ScannerState;

impl ScannerState {
    // fn find_active_game_area(&mut self) {
    //     self.screen_info = None; // TODO, probably should scaled everythign down to 1280x720
    // }

    // fn active_page_num(&self, inv_type: InventoryType) -> Option<usize> {
    //     if self.anchors.contains_key(&inv_type) {
    //         Some(self.anchors[&inv_type].variant)
    //     } else {
    //         None
    //     }
    // }

    // fn anchored_position(&self, slot_address: SlotAddress) -> ScaledPosition {
    //     assert!(self.anchors.contains_key(&slot_address.inventory_type));

    //     // this will need some kind of hard-coded constant
    // }
    pub fn cropper(&mut self) {
        // assert!(self.buffer.pointer.is_some());

        // if self.screen_info.is_none() {
        //     self.find_active_game_area();
        // }
        // // TODO some kind of detection & recovery when the screne size changes?

        // for inv_type in ALL_ANCHOR_TYPES {
        //     if self.anchors.contains_key(&inv_type)
        //         && !hash_close_enough(
        //             self.image_hash(self.anchors[&inv_type].position),
        //             self.anchors[&inv_type].hash,
        //         )
        //     {
        //         self.anchors.remove(&inv_type);
        //     }

        //     // TODO how reliable is using inv_type as an index? maybe just use usize
        //     if !self.anchors.contains_key(&inv_type) {
        //         if let Some((variant, (position, hash))) = ALL_ANCHOR_TEMPLATES[inv_type as usize]
        //             .iter()
        //             .enumerate()
        //             .find_map(|(variant, variant_template)| {
        //                 self.template_match(
        //                     variant_template,
        //                     Some(ANCHOR_BOUNDS[inv_type as usize]),
        //                 )
        //                 .map(|(position, hash)| (variant, (position, hash)))
        //             })
        //         {
        //             self.anchors.insert(
        //                 inv_type,
        //                 OneAnchorInfo {
        //                     variant,
        //                     position,
        //                     hash,
        //                 },
        //             );
        //         }
        //     }
        // }

        // // TODO hover tooltip detection, also filter out slot_address that's being covered

        // for slot_address in ALL_SLOT_ADDRESSS {
        //     let position: ScaledPosition = self.anchored_position(slot_address);
        //     let new_hash: u64 = self.image_hash(position);
        //     if self.slot_infos.contains_key(&slot_address)
        //         && !hash_close_enough(new_hash, self.slot_infos[&slot_address].hash)
        //     {
        //         self.slot_infos
        //             .get_mut(&slot_address)
        //             .unwrap()
        //             .currently_seen = false; // do it this way because we dont' want to delete previous info just because it got obscured for a bit
        //     }

        //     if !self.slot_infos.contains_key(&slot_address)
        //         || self.slot_infos[&slot_address].currently_seen == false
        //     {
        //         if let Some((icon_index, icon_name, hash)) =
        //             ALL_ICONS
        //                 .iter()
        //                 .enumerate()
        //                 .find_map(|(icon_index, (icon_name, hash))| {
        //                     if self
        //                         .slot_infos
        //                         .get(&slot_address)
        //                         .is_some_and(|s| s.icon_index == icon_index)
        //                     {
        //                         return None;
        //                     }
        //                     hash_close_enough(*hash, new_hash)
        //                         .then(|| (icon_index, icon_name.clone(), *hash))
        //                 })
        //         {
        //             self.slot_infos.insert(
        //                 slot_address,
        //                 OneSlotInfo {
        //                     hash,
        //                     currently_seen: true,
        //                     icon_index,
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
