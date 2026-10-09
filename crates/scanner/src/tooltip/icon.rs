use crate::{
    buffer::Buffer,
    setup::BASE_ICONS,
    tooltip::{detect::TitleBar, items::is_chest_icon},
};
use image::imageops::{FilterType, resize};

const ICON_LEFT: f64 = 9.33; // from the title's left edge; 7 at 1080p
const ICON_TOP: f64 = 22.67; // below the title bar; 17 at 1080p
const ICON_SIZE: f64 = 85.33; // 64 at 1080p, the size of the templates
const TEMPLATE_SIZE: usize = 64;
const KEPT: f64 = 0.75; // share of the pixels scored, so a cursor over the rest does not matter
// Difference per pixel, summed over the channels. The right icon measures 26 to 42 and the
// next best 50 and up; a tooltip of something else has nothing under 90.
const ICON_PASS: f64 = 54.0;
const ICON_MARGIN: f64 = 12.0; // over the next best

// The material the tooltip's large icon is closest to, and whether it is clearly that one: the
// two kinds of book differ by an emblem and never are. It is drawn over another background than
// a slot's, which the pixels left out absorb.
pub fn tooltip_icon(buffer: &Buffer, bar: &TitleBar, s: f64) -> Option<(String, bool)> {
    // one template pixel of room on every side, for the nine shifts
    let pad = ICON_SIZE * s / TEMPLATE_SIZE as f64;
    let left = (bar.x as f64 + ICON_LEFT * s - pad).round() as usize;
    let top = ((bar.y + bar.height) as f64 + ICON_TOP * s - pad).round() as usize;
    let size = (ICON_SIZE * s + 2.0 * pad).round() as usize;
    if left + size > buffer.width || top + size > buffer.height {
        return None;
    }
    let side = TEMPLATE_SIZE + 2;
    let seen = resize(
        &buffer.crop(left, top, left + size, top + size),
        side as u32,
        side as u32,
        FilterType::CatmullRom,
    );
    let seen = seen.as_raw();

    let kept = (KEPT * (TEMPLATE_SIZE * TEMPLATE_SIZE) as f64) as usize;
    let mut scores: Vec<(f64, String)> = BASE_ICONS
        .read()
        .iter()
        // a chest is told by what it lists, and there are too many of them to compare on every frame
        .filter(|(name, icon)| icon.tag == "Icon" && !is_chest_icon(name))
        .map(|(name, icon)| {
            let template = icon.data.as_raw();
            let mut best = f64::MAX;
            for shift in 0..9 {
                // how many pixels are each difference apart
                let mut counts = [0usize; 766];
                for y in 0..TEMPLATE_SIZE {
                    for x in 0..TEMPLATE_SIZE {
                        let a = (y * TEMPLATE_SIZE + x) * 4;
                        let b = ((y + shift / 3) * side + x + shift % 3) * 4;
                        let difference: usize =
                            (0..3).map(|c| template[a + c].abs_diff(seen[b + c]) as usize).sum();
                        counts[difference] += 1;
                    }
                }
                let (mut left, mut total) = (kept, 0);
                for (difference, count) in counts.iter().enumerate() {
                    let taken = left.min(*count);
                    total += taken * difference;
                    left -= taken;
                }
                best = best.min(total as f64 / kept as f64);
            }
            (best, name.clone())
        })
        .collect();
    scores.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (best, name) = scores.first()?;
    let second = scores.get(1).map_or(f64::MAX, |x| x.0);
    (*best <= ICON_PASS).then(|| (name.clone(), second - best >= ICON_MARGIN))
}
