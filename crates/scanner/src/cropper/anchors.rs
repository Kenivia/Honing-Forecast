use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    constants::ANCHORS_LOOKUP,
    image_utils::common::{FULL_RECT_16_9, bounding_rect},
    scanner_state::{InventoryType, ScaledPosition, ScannerState},
    setup::icon_lookup,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnchorInfo {
    pub positions: Vec<Option<ScaledPosition>>,
    pub position_root: Option<ScaledPosition>,
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
                if self.images_close_enough(
                    icon_lookup(&ANCHORS_LOOKUP[inv_type][variant_index].0),
                    self.downscale(found.unwrap()),
                ) {
                    to_clear.push((*inv_type, variant_index));
                }
            }
        }

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
        if missing_anchors.len() != 0 {
            self.write_downscaled_cache(bounding_rect(
                missing_anchors
                    .iter()
                    .flat_map(|inv_type| {
                        ANCHORS_LOOKUP[inv_type]
                            .iter()
                            .map(|x| x.1.unwrap_or(FULL_RECT_16_9))
                    })
                    .collect(),
            ));
        }

        for inv_type in missing_anchors {
            for (variant_index, (variant_name, bound)) in
                ANCHORS_LOOKUP[&inv_type].iter().enumerate()
            {
                if let Some(found) = self.template_match(
                    icon_lookup(variant_name),
                    self.downscale(bound.unwrap_or(FULL_RECT_16_9)),
                ) {
                    self.anchors.get_mut(&inv_type).unwrap().positions[variant_index] = Some(found);
                    self.anchors.get_mut(&inv_type).unwrap().position_root =
                        Some(found - icon_lookup(variant_name).offset);
                }
            }
        }
    }
}
