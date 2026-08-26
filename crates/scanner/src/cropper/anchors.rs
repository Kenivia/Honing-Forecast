use ahash::AHashMap;
use hf_core::my_dbg;
use serde::{Deserialize, Serialize};

use crate::{
    constants::ANCHORS_LOOKUP,
    image_utils::{
        close_enough::close_enough,
        common::{FULL_RECT_16_9, IntegerRectangle, Rectangle, get_resizer},
        downscale::crop_buffer,
        template_matching::template_match,
    },
    scanner_state::{InventoryType, ScannerState},
    setup::icon_lookup,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnchorInfo {
    pub positions: Vec<Option<(IntegerRectangle, f64)>>, // absolute positions here
    pub position_root: Option<IntegerRectangle>,
}

impl AnchorInfo {
    pub fn is_found(&self) -> bool {
        let out = self.positions.iter().any(|x| x.is_some());
        if out {
            assert!(self.position_root.is_some())
        }
        return out;
    }
}

impl ScannerState {
    pub fn initialize_anchors(&mut self) {
        self.anchors = AHashMap::new();
        for inv_type in ANCHORS_LOOKUP.keys() {
            self.anchors.insert(
                *inv_type,
                AnchorInfo {
                    positions: ANCHORS_LOOKUP[inv_type].iter().map(|_| None).collect(),
                    position_root: None,
                },
            );
        }
    }
    pub fn check_existing_anchors(&mut self) {
        let mut to_clear: Vec<(InventoryType, usize)> = Vec::new();

        for (inv_type, anchor_info) in self.anchors.iter() {
            for (variant_index, found) in anchor_info
                .positions
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
            {
                if close_enough(
                    &icon_lookup(
                        &ANCHORS_LOOKUP[inv_type][variant_index].0,
                        self.screen_info.effective_height,
                        get_resizer(&mut self.resizer),
                    ),
                    crop_buffer(
                        found.unwrap().0.to_float(),
                        get_resizer(&mut self.resizer),
                        self.buffer,
                    ),
                )
                .is_none()
                {
                    to_clear.push((*inv_type, variant_index));
                }
            }
        }
        // my_dbg!(
        //     "Cleared",
        //     to_clear.len(),
        //     "Total",
        //     self.anchors.keys().len()
        // );
        // hashmap borriwng shinanigans
        for (inv_type, variant_index) in to_clear {
            self.anchors.get_mut(&inv_type).unwrap().positions[variant_index] = None;
        }
    }

    pub fn update_anchors(&mut self) {
        self.check_existing_anchors();

        let missing_anchors: Vec<InventoryType> = ANCHORS_LOOKUP
            .keys()
            .into_iter()
            .filter(|k| !self.anchors[k].is_found())
            .copied()
            .collect();

        for inv_type in missing_anchors {
            for (variant_index, (variant_name, bound)) in
                ANCHORS_LOOKUP[&inv_type].iter().enumerate()
            {
                if let Some((found_position, confidence, brightness)) = template_match(
                    &icon_lookup(
                        variant_name,
                        self.screen_info.effective_height,
                        get_resizer(&mut self.resizer),
                    ),
                    crop_buffer(
                        bound
                            .unwrap_or(FULL_RECT_16_9)
                            .scaled(self.screen_info.scale_factor),
                        get_resizer(&mut self.resizer),
                        self.buffer,
                    ),
                ) {
                    if self.debugging {
                        self.debug_info.insert(
                            variant_name.clone(),
                            (found_position, confidence, brightness),
                        );
                    }
                    if confidence > 0.9 {
                        self.anchors.get_mut(&inv_type).unwrap().positions[variant_index] =
                            Some((found_position, confidence));
                        my_dbg!(
                            found_position,
                            &icon_lookup(
                                variant_name,
                                self.screen_info.effective_height,
                                get_resizer(&mut self.resizer),
                            )
                            .offset,
                            found_position.get_offset(
                                &icon_lookup(
                                    variant_name,
                                    self.screen_info.effective_height,
                                    get_resizer(&mut self.resizer),
                                )
                                .offset
                            ),
                        );
                        self.anchors.get_mut(&inv_type).unwrap().position_root = Some(
                            found_position.get_offset(
                                &icon_lookup(
                                    variant_name,
                                    self.screen_info.effective_height,
                                    get_resizer(&mut self.resizer),
                                )
                                .offset,
                            ),
                        );
                        self.screen_info.brightness = Some(brightness);
                    }
                }
                if !self.debugging {
                    self.debug_info = AHashMap::new();
                }
            }
        }
    }
}
