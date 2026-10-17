use crate::{
    anchor_to_slot::anchor_defs::{ANCHORS, AnchorSpec, Bound, Edge, anchor_spec},
    dev_utils::timing::timed,
    image_utils::{
        brightness::{est_ingame_brightness, mean_intensity, normalize_brightness},
        close_enough::close_enough,
        common::{Rect, get_resizer},
        sample::diff_samples,
        template_matching::{DEFAULT_TEMPLATE_MATCHING_CONFIDENCE, template_match},
    },
    interface::setup::icon_lookup,
    scanner_state::{AnchorType, InventoryType, ScannerState},
};
use ahash::AHashMap;

// Share of the thinned frame that has to differ from the one last searched as a whole for the lone
// inventory to be searched for again at once. Its window showing up is 13 to 15% of a 3440x1440
// frame; the cursor and a tooltip are under 3%.
const CHAR_INV_SEARCH_THRESH: f64 = 0.03;
// otherwise one of its variants is searched for on every this many scans
const CHAR_INV_FORCED_SEARCH_FREQ: u64 = 8;

pub const STORAGE_ANCHOR_SLACK: f64 = 2.0;
pub const WINDOW_ANCHOR_SLACK: f64 = 6.0;
pub const CURRENCY_ANCHOR_SLACK: f64 = 8.0;
// The storage layout is three windows and the Storage button, always in the same place: one for
// the pet menu, and shifted by this for the storage NPC.
const STORAGE_NPC_SHIFT: [(f64, f64); 2] = [(0.0, 0.0), (-6.4, -142.45)];
const STORAGE_MIN: usize = 3;

// cut on whole pixels: at a fraction of one it would be resampled
pub fn on_pixels(mut area: Rect) -> Rect {
    let (left, top) = (area.top_left.0.floor(), area.top_left.1.floor());
    area.width = (area.top_left.0 + area.width).ceil() - left;
    area.height = (area.top_left.1 + area.height).ceil() - top;
    area.top_left = (left, top);
    area
}

// with this much more on every side
pub fn grown(area: Rect, by: f64) -> Rect {
    Rect {
        top_left: (area.top_left.0 - by, area.top_left.1 - by),
        width: area.width + by * 2.0,
        height: area.height + by * 2.0,
        ..area
    }
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
            Bound::Corner(b) => b.scaled(ui_height).shifted(self.screen_info.corner),
            Bound::Render {
                left,
                right,
                top,
                bottom,
            } => {
                let (s, corner) = (self.screen_info.scale_factor, self.screen_info.corner);
                let x = |edge: &Edge| match edge {
                    Edge::Left(x) => corner.0 + x * s,
                    Edge::Right(x) => corner.0 + self.screen_info.render_width + x * s,
                };
                Rect::screen(
                    (x(left), corner.1 + top * s),
                    x(right) - x(left),
                    (bottom - top) * s,
                    ui_height,
                )
            }
        })
    }

    // the part of an area inside the frame, on whole pixels
    pub fn clipped(&self, area: Rect) -> Rect {
        let area = on_pixels(area);
        let (left, top) = (area.top_left.0.max(0.0), area.top_left.1.max(0.0));
        let right = (area.top_left.0 + area.width).min(self.buffer.width as f64);
        let bottom = (area.top_left.1 + area.height).min(self.buffer.height as f64);
        Rect {
            top_left: (left, top),
            width: (right - left).max(0.0),
            height: (bottom - top).max(0.0),
            ..area
        }
    }

    // Where a template matches best in an area of the frame, if it passes: its place in the
    // frame, the score, and the matched patch's mean. Matched as captured, so the score does not
    // go by brightness.
    pub fn find_template(&mut self, name: &str, area: Rect) -> Option<(Rect, f64, f64)> {
        let area = self.clipped(area);
        let ui_height = self.screen_info.effective_height;
        let template = icon_lookup(name, ui_height, get_resizer(&mut self.resizer));
        let (width, height) = template.offset.pixel_size();
        if area.width < width as f64 || area.height < height as f64 {
            return None;
        }
        drop(template);
        let searched = self.crop_buffer(area);
        let template = icon_lookup(name, ui_height, get_resizer(&mut self.resizer));
        let (found, confidence, mean_intensity) = template_match(&template, &searched)?;
        let limit = template
            .required_confidence
            .unwrap_or(DEFAULT_TEMPLATE_MATCHING_CONFIDENCE);
        (confidence > limit).then(|| (found.shifted(area.top_left), confidence, mean_intensity))
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

    // Each found anchor is compared where it was. Until something tells the brightness, which
    // the comparison needs, it is matched again just around that place instead.
    pub fn check_existing_anchors(&mut self) {
        let brightness = self.screen_info.brightness;
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
            let held = match brightness {
                Some(brightness) => {
                    let mut seen = self.crop_buffer(position);
                    normalize_brightness(&mut seen, brightness);
                    let template = icon_lookup(
                        variant.name,
                        self.screen_info.effective_height,
                        get_resizer(&mut self.resizer),
                    );
                    close_enough(&template, &seen).is_some()
                }
                None => self
                    .find_template(variant.name, grown(position, 2.0))
                    .is_some(),
            };
            if !held {
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
        let Some((found_position, confidence, mean_intensity)) =
            self.find_template(variant.name, search_area)
        else {
            return;
        };
        let template_offset = icon_lookup(
            variant.name,
            self.screen_info.effective_height,
            get_resizer(&mut self.resizer),
        )
        .offset;
        let brightness = variant
            .brightness
            .map(|curve| est_ingame_brightness(mean_intensity, &curve));

        {
            let anchor_info = self.anchors.get_mut(&spec.anchor_type).unwrap();
            anchor_info.positions[variant_index] = Some(FoundVariant {
                position: found_position,
                confidence,
                brightness,
            });
            // variants place the root SLIGHTLY (like < 0.1 pixel) apart, we let the first one
            // keep it to make comparisons more stable
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
            .flat_map(|spec| {
                self.anchors[&spec.anchor_type]
                    .positions
                    .iter()
                    .flatten()
                    .filter_map(|x| x.brightness)
            })
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
            .filter(|(anchor_type, _)| anchor_type.in_storage())
            .map(|(_, anchor)| anchor.positions.iter().flatten().count())
            .sum()
    }

    fn clear_anchor(&mut self, anchor_type: AnchorType) {
        let anchor_info = self.anchors.get_mut(&anchor_type).unwrap();
        anchor_info.positions.fill(None);
        anchor_info.position_root = None;
    }

    fn search_storage(&mut self) {
        for spec in ANCHORS.iter().filter(|spec| spec.anchor_type.in_storage()) {
            timed("anchors/storage", || self.search_anchor(spec));
        }
    }

    // Searched where it can be dragged to once it is not where the game first puts it. That is a
    // large search, so it runs when the frame differs enough from the one last searched, measured
    // against that frame and not the previous one so that a window fading in adds up
    fn search_lone(&mut self, lost: bool) {
        let spec = anchor_spec(AnchorType::CharInventory);
        if !self.anchors[&spec.anchor_type].is_found() {
            self.clear_anchor(spec.anchor_type);
            let changed = self.search_samples.len() != self.last_samples.len()
                || diff_samples(&self.last_samples, &self.search_samples) > CHAR_INV_SEARCH_THRESH;
            let variants_to_search: Vec<usize> = if lost || changed {
                self.search_samples = self.last_samples.clone();
                self.search_ticks = 0;
                (0..spec.variants.len()).collect()
            } else {
                // search one at a time
                self.search_ticks += 1;
                let turn = self.search_ticks / CHAR_INV_FORCED_SEARCH_FREQ;
                if self.search_ticks % CHAR_INV_FORCED_SEARCH_FREQ != 0 {
                    return;
                }
                vec![turn as usize % spec.variants.len()]
            };
            for variant_index in variants_to_search {
                if !self.anchors[&spec.anchor_type].is_found() {
                    let area =
                        self.compute_search_area(&spec.variants[variant_index].roam.unwrap());
                    timed("anchors/search", || {
                        self.search_variant(spec, variant_index, area)
                    });
                }
            }
        }
        // once one variant gives the root, the others are only looked for where it puts them
        let Some(root) = self.anchors[&spec.anchor_type].position_root else {
            return;
        };
        let slack: f64 = WINDOW_ANCHOR_SLACK * self.screen_info.scale_factor;
        for (variant_index, variant) in spec.variants.iter().enumerate() {
            if self.anchors[&spec.anchor_type].positions[variant_index].is_none() {
                let at = icon_lookup(
                    variant.name,
                    self.screen_info.effective_height,
                    get_resizer(&mut self.resizer),
                )
                .offset
                .shifted(root);
                let near = on_pixels(grown(at, slack));
                timed("anchors/near", || {
                    self.search_variant(spec, variant_index, near)
                });
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
            for shift in STORAGE_NPC_SHIFT {
                for spec in ANCHORS.iter() {
                    if spec.anchor_type.in_storage() {
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

        // the lone inventory and the currency window are only looked for outside storage
        if self.storage_open() {
            self.clear_anchor(lone.anchor_type);
            self.clear_anchor(AnchorType::Currency);
        } else {
            let lost = lone_was_found && !self.anchors[&lone.anchor_type].is_found();
            // both at the spot the game first puts them, which is cheap
            timed("anchors/default", || self.search_anchor(lone));
            let currency = anchor_spec(AnchorType::Currency);
            timed("anchors/currency", || self.search_anchor(currency));
            self.search_lone(lost);
        }
    }

    pub fn storage_open(&self) -> bool {
        self.storage_found() >= STORAGE_MIN
    }
}
