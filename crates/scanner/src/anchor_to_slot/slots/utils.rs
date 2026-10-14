use image::RgbaImage;

use crate::{
    anchor_to_slot::slots::constants::{
        AFAR_ROWS, CELLS, FROM_AFAR, OVERLAY_REACH, OVERLAYS, SHORTLIST, SLOT_CONFIDENCE,
    },
    image_utils::{
        close_enough::{Area, confidence, masked_confidence},
        common::get_resizer,
    },
    interface::setup::{BASE_ICONS, OneIconConfig, icon_lookup},
    scanner_state::ScannerState,
};

pub struct Overlay {
    // the part of the compared icon it covers (1440p: x0, y0, x1, y1)
    pub area: (f64, f64, f64, f64),
    // how much more alike the slot and the icon have to be with that part left out for it to be there
    pub gain: f64,
    // the icons that can have it
    pub on: fn(&str) -> bool,
}

// the parts of an icon these overlays cover, in screen pixels
pub fn overlay_areas(overlays: u8, s: f64) -> Vec<Area> {
    let px = |x: f64| (x * s).round() as usize;
    (0..OVERLAYS.len())
        .filter(|i| overlays >> i & 1 == 1)
        .map(|i| OVERLAYS[i].area)
        .map(|(x0, y0, x1, y1)| (px(x0), px(y0), px(x1), px(y1)))
        .collect()
}

// How alike a slot and an icon are with the overlays on the slot left out, and which those are, as
// bits. Of the overlays the icon can have, it is the set that gains the most over what each has to.
pub fn overlaid(
    template: &OneIconConfig,
    observed: &OneIconConfig,
    name: &str,
    s: f64,
) -> (f64, u8) {
    let whole = confidence(template, observed);
    if whole < SLOT_CONFIDENCE - OVERLAY_REACH {
        return (whole, 0);
    }
    let on = |set: u8, i: usize| set >> i & 1 == 1;
    let can = (0..OVERLAYS.len())
        .filter(|i| (OVERLAYS[*i].on)(name))
        .fold(0u8, |set, i| set | 1 << i);
    let (mut best, mut lead) = ((whole, 0), whole);
    for set in (1..1u8 << OVERLAYS.len()).filter(|set| set & !can == 0) {
        let score = masked_confidence(template, observed, &overlay_areas(set, s));
        let cost: f64 = (0..OVERLAYS.len())
            .filter(|i| on(set, *i))
            .map(|i| OVERLAYS[i].gain)
            .sum();
        if score - cost > lead {
            (best, lead) = ((score, set), score - cost);
        }
    }
    best
}
// the rows above the item level, which every slot shows
pub fn from_afar(image: &RgbaImage, rows: usize) -> Vec<[f32; 3]> {
    let (width, rows) = (image.width() as usize, rows.min(image.height() as usize));
    let mut cells = vec![[0f32; 3]; CELLS * 2];
    let mut counts = vec![0f32; CELLS * 2];
    for y in 0..rows {
        for x in 0..width {
            let cell = (y * 2 / rows) * CELLS + x * CELLS / width;
            let pixel = image.get_pixel(x as u32, y as u32).0;
            for c in 0..3 {
                cells[cell][c] += pixel[c] as f32;
            }
            counts[cell] += 1.0;
        }
    }
    for (cell, count) in cells.iter_mut().zip(counts) {
        *cell = cell.map(|x| x / count);
    }
    cells
}

// The icons and the decoys are shortlisted apart, so that the decoys, which are many more,
// do not crowd an icon out.

impl ScannerState {
    fn populate_afar_cache(&mut self, height: u32, scale: f64) {
        if !FROM_AFAR.read().contains_key(&height) {
            let mut names: Vec<(String, bool)> = BASE_ICONS
                .read()
                .iter()
                .filter(|(_, one_icon)| one_icon.is_slot_icon())
                .map(|(name, one_icon)| (name.clone(), one_icon.is_decoy()))
                .collect();
            names.sort();
            let all = names
                .into_iter()
                .map(|(name, decoy)| {
                    let icon = icon_lookup(&name, height, get_resizer(&mut self.resizer));
                    let cells = from_afar(&icon.data, (AFAR_ROWS * scale).round() as usize);
                    (name, decoy, cells)
                })
                .collect();
            FROM_AFAR.write().insert(height, all);
        }
    }
    pub fn get_all_passed_icons(
        &mut self,
        observed: &OneIconConfig,
        height: u32,
        scale: f64,
    ) -> (Vec<String>, Vec<(String, f64, u8)>) {
        self.populate_afar_cache(height, scale);
        let observed_afar = from_afar(&observed.data, (AFAR_ROWS * scale).round() as usize);

        let mut decoys: Vec<String> = vec![];
        let mut shortlist: Vec<String> = vec![];
        for is_decoy in [false, true] {
            let mut nearest: Vec<(f32, String)> = FROM_AFAR.read()[&height]
                .iter()
                .filter(|(_, decoy, _)| *decoy == is_decoy)
                .map(|(name, _, cells)| {
                    let apart: f32 = cells
                        .iter()
                        .zip(&observed_afar)
                        .map(|(a, b)| (0..3).map(|c| (a[c] - b[c]).abs()).sum::<f32>())
                        .sum();
                    (apart, name.clone())
                })
                .collect();
            if nearest.len() > SHORTLIST {
                nearest.select_nth_unstable_by(SHORTLIST, |a, b| a.0.total_cmp(&b.0));
                nearest.truncate(SHORTLIST);
            }
            shortlist.extend(nearest.into_iter().map(|x| x.1));
        }
        let passed: Vec<(String, f64, u8)> = shortlist
            .into_iter()
            .filter_map(|icon_name| {
                let template = icon_lookup(&icon_name, height, get_resizer(&mut self.resizer));
                if template.is_decoy() {
                    decoys.push(icon_name);
                    return None;
                }

                let (score, overlays) = overlaid(&template, &observed, &icon_name, scale);
                (score > template.required_confidence.unwrap_or(SLOT_CONFIDENCE))
                    .then_some((icon_name, score, overlays))
            })
            .collect();
        (decoys, passed)
    }
}
