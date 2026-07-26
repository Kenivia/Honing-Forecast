use crate::{
    constants::{ALL_ANCHOR_TEMPLATES, ALL_ICONS, ALL_SLOT_ADDRESSS, ANCHOR_BOUNDS},
    scanner_state::{
        ALL_ANCHOR_TYPES, OCRJob, OneAnchorInfo, OneSlotInfo, OneSlotProgress::OCRing,
        ScaledPosition, ScannerState,
    },
    setup::CONFIG,
};
use either::Either::Right;

impl ScannerState {
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_some());
        assert!(CONFIG.get().unwrap().len() != 0);

        if !self.screen_info.initialized {
            self.check_21_9();
        }

        for inv_type in ALL_ANCHOR_TYPES {
            self.validate_anchors();
        }
        // TODO hover tooltip detection, also filter out slot_address that's being covered

        for slot_address in ALL_SLOT_ADDRESSS {
            let position: ScaledPosition = self.anchored_position(slot_address);
            if self.slot_infos.contains_key(&slot_address)
                && !self.images_close_enough(
                    self.access_pixels(position),
                    self.access_icon_id(self.slot_infos[&slot_address].icon_id),
                )
            {
                self.slot_infos
                    .get_mut(&slot_address)
                    .unwrap()
                    .currently_seen = false; // do it this way because we dont' want to delete previous info just because it got obscured for a bit
            }

            if !self.slot_infos.contains_key(&slot_address)
                || self.slot_infos[&slot_address].currently_seen == false
            {
                if let Some((icon_id, icon_name)) =
                    ALL_ICONS
                        .iter()
                        .enumerate()
                        .find_map(|(icon_id, icon_name)| {
                            if self
                                .slot_infos
                                .get(&slot_address)
                                .is_some_and(|s| s.icon_id == icon_id)
                            {
                                return None;
                            }
                            self.images_close_enough(
                                self.access_icon_id(icon_id),
                                self.access_pixels(position),
                            )
                            .then(|| (icon_id, icon_name.clone()))
                        })
                {
                    self.slot_infos.insert(
                        slot_address,
                        OneSlotInfo {
                            currently_seen: true,
                            icon_id,
                            icon_name,
                            position,
                            amount: Right(OCRing),
                            tradability: Right(OCRing),
                        },
                    );
                    self.pending_jobs.push(OCRJob {
                        cropped: self.access_pixels(position).collect(),
                        slot_address,
                    })
                }
            }
        }

        self.downscaled_cache.written_this_cycle = true;
    }
}
