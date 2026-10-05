use ahash::AHashMap;
use hf_core::my_dbg;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{ANCHORS, AnchorSpec, Bound, STORAGE_MIN, STORAGE_SHIFTS, anchor_spec},
    image_utils::{
        brightness::est_ingame_brightness,
        close_enough::{DEFAULT_CONFIDENCE, close_enough},
        common::{FloatRectangle, IntegerRectangle, Rectangle, get_resizer},
        resize::crop_buffer,
        template_matching::{DEFAULT_TEMPLATE_MATCHING_CONFIDENCE, template_match},
    },
    scanner_state::{AnchorType, InventoryType, ScannerState},
    setup::icon_lookup,
    timing::timed,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnchorInfo {
    // per variant: absolute position, confidence, and the brightness it was found at if it tells
    pub positions: Vec<Option<(IntegerRectangle, f64, Option<f64>)>>,
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

    // Window origin of this inventory, from the first anchor with a root that locates it. A
    // storage window keeps its root while both of its anchors are covered.
    pub fn inventory_root(&self, inventory_type: InventoryType) -> Option<IntegerRectangle> {
        ANCHORS.iter().find_map(|spec| {
            let (_, origin) = spec
                .inventories
                .iter()
                .find(|(this_type, _)| *this_type == inventory_type)?;
            Some(self.anchors[&spec.anchor_type].position_root?.shifted((
                origin.0 * self.screen_info.scale_factor,
                origin.1 * self.screen_info.scale_factor,
            )))
        })
    }

    pub fn check_existing_anchors(&mut self) {
        let Some(brightness) = self.screen_info.brightness else {
            return;
        };
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
                    brightness,
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
            if self.anchors[&spec.anchor_type].positions[variant_index].is_some() {
                continue;
            }
            let template = icon_lookup(
                variant.name,
                self.screen_info.effective_height,
                get_resizer(&mut self.resizer),
            );
            let template_offset = template.offset;
            let required_confidence = template
                .required_confidence
                .unwrap_or(DEFAULT_TEMPLATE_MATCHING_CONFIDENCE);
            drop(template);
            let search_area = match variant.bound {
                Bound::Frame => FloatRectangle {
                    top_left: (0.0, 0.0),
                    width: self.buffer.width as f64,
                    height: self.buffer.height as f64,
                },
                Bound::Ui(bound) => bound.scaled(scale).shifted(self.screen_info.ui_origin),
                Bound::Storage(bound) => bound
                    .shifted(self.storage_shift)
                    .scaled(scale)
                    .shifted(self.screen_info.ui_origin),
            };
            // on whole pixels, or the crop is resampled and the brightness comes out a setting or two off
            let (left, top) = (search_area.top_left.0.floor(), search_area.top_left.1.floor());
            let search_area = FloatRectangle {
                top_left: (left, top),
                width: (search_area.top_left.0 + search_area.width).ceil() - left,
                height: (search_area.top_left.1 + search_area.height).ceil() - top,
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
            let brightness = variant.brightness.map(|curve| est_ingame_brightness(best_mean_f, &curve));

            if self.debugging {
                let name = format!("{:?} {}", spec.anchor_type, variant.name);
                self.changed_debug.insert(name.clone());
                self.debug_info.insert(
                    name,
                    (
                        found_position,
                        confidence,
                        brightness.unwrap_or_default(),
                        vec![crop_buffer(
                            found_position,
                            get_resizer(&mut self.resizer),
                            self.buffer,
                            None,
                        )],
                    ),
                );
            }
            if confidence > required_confidence {
                let anchor_info = self.anchors.get_mut(&spec.anchor_type).unwrap();
                anchor_info.positions[variant_index] = Some((found_position, confidence, brightness));
                // variants place the root a pixel apart, so the first one found keeps it
                if !anchor_info.positions[..variant_index].iter().any(|x| x.is_some()) {
                    anchor_info.position_root = Some(found_position.get_offset(&template_offset));
                }
                // only to start with: until an anchor has held up, there is nothing better
                if self.screen_info.brightness_anchors == 0 && brightness.is_some() {
                    self.screen_info.brightness = brightness;
                }
            }
        }
    }

    // The in-game setting does not change during a session, but each anchor's estimate of it is
    // a few settings off, and one that matched the wrong thing is far off. So the estimate is the
    // average over the most anchors ever found together, and stays once some of them are lost.
    // Anchors only count after their re-check, which a wrong match does not survive.
    fn update_brightness(&mut self) {
        let found: Vec<f64> = self
            .anchors
            .values()
            .flat_map(|anchor| anchor.positions.iter().flatten().filter_map(|x| x.2))
            .collect();
        if found.len() > self.screen_info.brightness_anchors {
            self.screen_info.brightness = Some(found.iter().sum::<f64>() / found.len() as f64);
            self.screen_info.brightness_anchors = found.len();
        }
    }

    fn storage_found(&self) -> usize {
        self.anchors
            .iter()
            .filter(|(anchor_type, _)| **anchor_type != AnchorType::CharInventory)
            .map(|(_, anchor)| anchor.positions.iter().flatten().count())
            .sum()
    }

    fn clear_anchor(&mut self, anchor_type: AnchorType) {
        let anchor_info = self.anchors.get_mut(&anchor_type).unwrap();
        anchor_info.positions.fill(None);
        anchor_info.position_root = None;
    }

    fn search_storage(&mut self) {
        for spec in ANCHORS.iter().filter(|spec| spec.anchor_type != AnchorType::CharInventory) {
            timed("anchors/storage", || self.search_anchor(spec));
        }
    }

    pub fn update_anchors(&mut self) {
        self.check_existing_anchors();
        self.update_brightness();

        if self.storage_found() >= STORAGE_MIN {
            self.search_storage();
        } else {
            // closed, or only just opened: look where the pet and then the NPC would put it
            for shift in STORAGE_SHIFTS {
                for spec in ANCHORS.iter() {
                    if spec.anchor_type != AnchorType::CharInventory {
                        self.clear_anchor(spec.anchor_type);
                    }
                }
                self.storage_shift = shift;
                self.search_storage();
                if self.storage_found() >= STORAGE_MIN {
                    break;
                }
            }
        }

        // the lone inventory's templates are in every storage window, and finding it takes a
        // search of the whole frame
        let lone = anchor_spec(AnchorType::CharInventory);
        if self.storage_found() >= STORAGE_MIN {
            self.clear_anchor(lone.anchor_type);
        } else if !self.anchors[&lone.anchor_type].is_found() {
            self.clear_anchor(lone.anchor_type);
            timed("anchors/search", || self.search_anchor(lone));
        }
        if !self.debugging {
            self.debug_info = AHashMap::new();
        }
    }
}
