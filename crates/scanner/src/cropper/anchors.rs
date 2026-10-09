use ahash::AHashMap;

use crate::{
    constants::{ANCHORS, AnchorSpec, Bound, STORAGE_MIN, STORAGE_SHIFTS, WINDOW_SLACK, anchor_spec},
    cropper::cropper::differing,
    image_utils::{
        brightness::{est_ingame_brightness, mean_intensity, normalize_brightness},
        close_enough::close_enough,
        common::{Rect, get_resizer},
        template_matching::{DEFAULT_TEMPLATE_MATCHING_CONFIDENCE, template_match},
    },
    scanner_state::{AnchorType, InventoryType, ScannerState},
    setup::icon_lookup,
    timing::timed,
};

// Share of the thinned frame that has to differ from the one last searched as a whole for the lone
// inventory to be searched for again at once. Its window showing up is 13 to 15% of a 3440x1440
// frame; the cursor and a tooltip are under 3%.
const SEARCH_CHANGED: f64 = 0.03;
// otherwise one of its variants is searched for on every this many scans
const SEARCH_EVERY: u64 = 8;

// cut on whole pixels: at a fraction of one it would be resampled
fn on_pixels(mut area: Rect) -> Rect {
    let (left, top) = (area.top_left.0.floor(), area.top_left.1.floor());
    area.width = (area.top_left.0 + area.width).ceil() - left;
    area.height = (area.top_left.1 + area.height).ceil() - top;
    area.top_left = (left, top);
    area
}

// one variant of an anchor, where it was matched
#[derive(Debug, Clone, Copy)]
pub struct FoundVariant {
    // absolute, in the frame
    pub position: Rect,
    pub confidence: f64,
    // the setting its patch reads as, if this variant tells
    pub brightness: Option<f64>,
}

#[derive(Debug)]
pub struct AnchorInfo {
    pub positions: Vec<Option<FoundVariant>>,
    // the point its inventories are placed from, in the frame
    pub position_root: Option<(f64, f64)>,
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
    fn compute_search_area(&self, bound: &Bound) -> Rect {
        let ui_height = self.screen_info.effective_height;
        on_pixels(match bound {
            Bound::Frame => Rect::screen(
                (0.0, 0.0),
                self.buffer.width as f64,
                self.buffer.height as f64,
                ui_height,
            ),
            Bound::Ui(b) => b.scaled(ui_height).shifted(self.screen_info.ui_origin),
            Bound::Storage(b) => b
                .shifted(self.storage_shift)
                .scaled(ui_height)
                .shifted(self.screen_info.ui_origin),
        })
    }

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
    pub fn inventory_root(&self, inventory_type: InventoryType) -> Option<(f64, f64)> {
        ANCHORS.iter().find_map(|spec| {
            let (_, origin) = spec
                .inventories
                .iter()
                .find(|(this_type, _)| *this_type == inventory_type)?;
            let root = self.anchors[&spec.anchor_type].position_root?;
            Some((
                root.0 + origin.0 * self.screen_info.scale_factor,
                root.1 + origin.1 * self.screen_info.scale_factor,
            ))
        })
    }

    pub fn check_existing_anchors(&mut self) {
        let Some(brightness) = self.screen_info.brightness else {
            return;
        };
        let mut to_clear: Vec<(AnchorType, usize)> = Vec::new();
        let mut estimates: Vec<(AnchorType, usize, f64)> = Vec::new();

        let found: Vec<(AnchorType, usize, Rect)> = self
            .anchors
            .iter()
            .flat_map(|(anchor_type, anchor_info)| {
                let positions = anchor_info.positions.iter().enumerate();
                positions.filter_map(|(index, x)| Some((*anchor_type, index, x.as_ref()?.position)))
            })
            .collect();
        for (anchor_type, variant_index, position) in found {
            let variant = &anchor_spec(anchor_type).variants[variant_index];
            let mut seen = self.crop_buffer(position);
            normalize_brightness(&mut seen, brightness);
            let template = icon_lookup(
                variant.name,
                self.screen_info.effective_height,
                get_resizer(&mut self.resizer),
            );
            if close_enough(&template, &seen).is_none() {
                to_clear.push((anchor_type, variant_index));
            } else if let Some(curve) = variant.brightness {
                // Its estimate is of this frame, not of the one it was found on, which a
                // window still fading in makes far too dark. The patch is as captured.
                let (x, y) = position.top_left;
                let patch = self.crop_buffer(position.with_top_left((x.round(), y.round())));
                let estimate = est_ingame_brightness(mean_intensity(&patch), &curve);
                estimates.push((anchor_type, variant_index, estimate));
            }
        }
        for (anchor_type, variant_index, estimate) in estimates {
            let found = &mut self.anchors.get_mut(&anchor_type).unwrap().positions[variant_index];
            found.as_mut().unwrap().brightness = Some(estimate);
        }
        // hashmap borriwng shinanigans
        for (anchor_type, variant_index) in to_clear {
            self.anchors.get_mut(&anchor_type).unwrap().positions[variant_index] = None;
        }
    }

    fn search_anchor(&mut self, spec: &AnchorSpec) {
        for (variant_index, variant) in spec.variants.iter().enumerate() {
            if self.anchors[&spec.anchor_type].positions[variant_index].is_none() {
                let search_area = self.compute_search_area(&variant.bound);
                self.search_variant(spec, variant_index, search_area);
            }
        }
    }

    fn search_variant(&mut self, spec: &AnchorSpec, variant_index: usize, search_area: Rect) {
        let variant = &spec.variants[variant_index];
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
        // matched as captured: the score does not go by brightness
        let searched = self.crop_buffer(search_area);
        let Some((found_in_area, confidence, best_mean_f)) = template_match(
            &icon_lookup(
                variant.name,
                self.screen_info.effective_height,
                get_resizer(&mut self.resizer),
            ),
            &searched,
        ) else {
            return;
        };
        let found_position = found_in_area.shifted(search_area.top_left);
        let brightness = variant
            .brightness
            .map(|curve| est_ingame_brightness(best_mean_f, &curve));

        if confidence > required_confidence {
            let anchor_info = self.anchors.get_mut(&spec.anchor_type).unwrap();
            anchor_info.positions[variant_index] =
                Some(FoundVariant { position: found_position, confidence, brightness });
            // variants place the root a pixel apart, so the first one found keeps it
            if !anchor_info.positions[..variant_index]
                .iter()
                .any(|x| x.is_some())
            {
                anchor_info.position_root = Some(found_position.get_offset(&template_offset));
            }
            // only to start with: until an anchor has held up, there is nothing better
            if self.screen_info.brightness_anchors == 0 && brightness.is_some() {
                self.screen_info.brightness = brightness;
            }
        }
    }

    // The in-game setting does not change during a session, but each anchor's estimate of it is
    // a few settings off, and one that matched the wrong thing is far off. So the estimate is the
    // average over the most anchors ever found together, and stays once some of them are lost.
    // Anchors only count after their re-check, which a wrong match does not survive.
    fn update_brightness(&mut self) {
        // in list order, so the sum does not depend on map order
        let found: Vec<f64> = ANCHORS
            .iter()
            .flat_map(|spec| self.anchors[&spec.anchor_type].positions.iter().flatten().filter_map(|x| x.brightness))
            .collect();
        // nothing held up at this estimate, so it is not one to keep: the next anchor found replaces it
        if found.is_empty() {
            self.screen_info.brightness_anchors = 0;
        } else if found.len() >= self.screen_info.brightness_anchors {
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
        for spec in ANCHORS
            .iter()
            .filter(|spec| spec.anchor_type != AnchorType::CharInventory)
        {
            timed("anchors/storage", || self.search_anchor(spec));
        }
    }

    // The lone inventory can be anywhere, and a search of the whole frame takes 180 to 590 ms. So
    // it is searched for when the frame differs enough from the one last searched, measured
    // against that frame and not the previous one so that a window fading in adds up; when it
    // was lost at its re-check on this scan; and otherwise one variant on every few scans. No
    // rule waits for a later frame: a still screen sends none.
    fn search_lone(&mut self, lost: bool) {
        let spec = anchor_spec(AnchorType::CharInventory);
        if !self.anchors[&spec.anchor_type].is_found() {
            self.clear_anchor(spec.anchor_type);
            let changed = self.search_samples.len() != self.last_samples.len()
                || differing(&self.last_samples, &self.search_samples) > SEARCH_CHANGED;
            let whole: Vec<usize> = if lost || changed {
                self.search_samples = self.last_samples.clone();
                self.search_ticks = 0;
                (0..spec.variants.len()).collect()
            } else {
                self.search_ticks += 1;
                let turn = self.search_ticks / SEARCH_EVERY;
                if self.search_ticks % SEARCH_EVERY != 0 {
                    return;
                }
                vec![turn as usize % spec.variants.len()]
            };
            let frame = self.compute_search_area(&Bound::Frame);
            for variant_index in whole {
                if !self.anchors[&spec.anchor_type].is_found() {
                    timed("anchors/search", || self.search_variant(spec, variant_index, frame));
                }
            }
        }
        // once one variant gives the root, the others are only looked for where it puts them
        let Some(root) = self.anchors[&spec.anchor_type].position_root else {
            return;
        };
        let slack = WINDOW_SLACK * self.screen_info.scale_factor;
        for (variant_index, variant) in spec.variants.iter().enumerate() {
            if self.anchors[&spec.anchor_type].positions[variant_index].is_none() {
                let at = icon_lookup(
                    variant.name,
                    self.screen_info.effective_height,
                    get_resizer(&mut self.resizer),
                )
                .offset
                .shifted(root);
                let near = on_pixels(Rect {
                    top_left: (at.top_left.0 - slack, at.top_left.1 - slack),
                    width: at.width + slack * 2.0,
                    height: at.height + slack * 2.0,
                    ..at
                });
                timed("anchors/near", || self.search_variant(spec, variant_index, near));
            }
        }
    }

    pub fn update_anchors(&mut self) {
        let lone = anchor_spec(AnchorType::CharInventory);
        let lone_was_found = self.anchors[&lone.anchor_type].is_found();
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

        // only template match for the char inventory anchor if we're not in storage
        if self.storage_found() >= STORAGE_MIN {
            self.clear_anchor(lone.anchor_type);
        } else {
            let lost = lone_was_found && !self.anchors[&lone.anchor_type].is_found();
            self.search_lone(lost);
        }
    }
}
