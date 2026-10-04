use ahash::AHashMap;
use hf_core::my_dbg;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{ANCHORS, AnchorSpec, Bound, anchor_spec},
    image_utils::{
        brightness::est_ingame_brightness,
        close_enough::close_enough,
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        resize::crop_buffer,
        template_matching::template_match,
    },
    scanner_state::{AnchorType, InventoryType, ScannerState},
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
        for spec in ANCHORS.iter() {
            self.anchors.insert(
                spec.anchor_type,
                AnchorInfo {
                    positions: vec![None; spec.variants.len()],
                    position_root: None,
                },
            );
        }
    }

    // window origin of this inventory, from the first found anchor that locates it
    pub fn inventory_root(&self, inventory_type: InventoryType) -> Option<IntegerRectangle> {
        ANCHORS
            .iter()
            .filter(|spec| self.anchors[&spec.anchor_type].is_found())
            .find_map(|spec| {
                let (_, origin) = spec
                    .inventories
                    .iter()
                    .find(|(this_type, _)| *this_type == inventory_type)?;
                Some(
                    self.anchors[&spec.anchor_type]
                        .position_root
                        .unwrap()
                        .shifted((
                            origin.0 * self.screen_info.scale_factor,
                            origin.1 * self.screen_info.scale_factor,
                        )),
                )
            })
    }

    pub fn check_existing_anchors(&mut self) {
        let mut to_clear: Vec<(AnchorType, usize)> = Vec::new();

        for (anchor_type, anchor_info) in self.anchors.iter() {
            for (variant_index, found) in anchor_info
                .positions
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
            {
                assert!(anchor_info.position_root.is_some());
                if close_enough(
                    &icon_lookup(
                        anchor_spec(*anchor_type).variants[variant_index].name,
                        self.screen_info.effective_height,
                        get_resizer(&mut self.resizer),
                    ),
                    &mut crop_buffer(
                        found.unwrap().0.to_float(),
                        get_resizer(&mut self.resizer),
                        self.buffer,
                        anchor_info.position_root,
                    ),
                    self.screen_info.brightness.unwrap(),
                )
                .is_none()
                {
                    to_clear.push((*anchor_type, variant_index));
                }
            }
        }
        // hashmap borriwng shinanigans
        for (anchor_type, variant_index) in to_clear {
            self.anchors.get_mut(&anchor_type).unwrap().positions[variant_index] = None;
        }
    }

    fn search_anchor(&mut self, spec: &AnchorSpec) {
        let scale = self.screen_info.scale_factor;
        for (variant_index, variant) in spec.variants.iter().enumerate() {
            let template_offset = icon_lookup(
                variant.name,
                self.screen_info.effective_height,
                get_resizer(&mut self.resizer),
            )
            .offset;
            let search_area = match variant.bound {
                Bound::Frame => FloatRectangle {
                    top_left: (0.0, 0.0),
                    width: self.buffer.width as f64,
                    height: self.buffer.height as f64,
                },
                Bound::Ui(bound) => bound.scaled(scale).shifted(self.screen_info.ui_origin),
                Bound::Relative(parent, bound) => bound
                    .scaled(scale)
                    .use_root(&self.anchors[&parent].position_root.unwrap()),
            };
            let Some((found_in_area, confidence, best_mean_f)) = template_match(
                &icon_lookup(
                    variant.name,
                    self.screen_info.effective_height,
                    get_resizer(&mut self.resizer),
                ),
                &crop_buffer(
                    search_area,
                    get_resizer(&mut self.resizer),
                    self.buffer,
                    None,
                ),
            ) else {
                continue;
            };
            let found_position = found_in_area.shifted(search_area.top_left);
            let brightness = est_ingame_brightness(best_mean_f, &variant.brightness);

            if self.debugging {
                self.debug_info.insert(
                    format!("{:?} {}", spec.anchor_type, variant.name),
                    (
                        found_position,
                        confidence,
                        brightness,
                        vec![crop_buffer(
                            found_position,
                            get_resizer(&mut self.resizer),
                            self.buffer,
                            None,
                        )],
                    ),
                );
            }
            if confidence > 0.9 {
                let anchor_info = self.anchors.get_mut(&spec.anchor_type).unwrap();
                anchor_info.positions[variant_index] = Some((found_position, confidence));
                anchor_info.position_root = Some(
                    found_position
                        .get_offset(&template_offset)
                        .shifted((variant.root_shift.0 * scale, variant.root_shift.1 * scale)),
                );
                self.screen_info.brightness = Some(brightness);
            }
        }
    }

    pub fn update_anchors(&mut self) {
        self.check_existing_anchors();

        let mut forbidden: Vec<AnchorType> = Vec::new();
        for spec in ANCHORS.iter() {
            let parent_missing = spec.variants.iter().any(|variant| {
                matches!(variant.bound, Bound::Relative(parent, _) if !self.anchors[&parent].is_found())
            });
            if forbidden.contains(&spec.anchor_type) || parent_missing {
                let anchor_info = self.anchors.get_mut(&spec.anchor_type).unwrap();
                anchor_info.positions.fill(None);
                anchor_info.position_root = None;
                continue;
            }
            if !self.anchors[&spec.anchor_type].is_found() {
                self.search_anchor(spec);
            }
            if self.anchors[&spec.anchor_type].is_found() {
                forbidden.extend(&spec.forbids);
            }
        }
        if !self.debugging {
            self.debug_info = AHashMap::new();
        }
    }
}
