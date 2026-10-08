use crate::{image_utils::brightness::normalize_brightness, setup::OneIconConfig};
use image::RgbaImage;

pub const DEFAULT_CONFIDENCE: f64 = 0.95;

// mean absolute RGB difference, best of the 9 one-pixel shifts so a slightly misplaced crop still matches
fn shifted_distance(template: &RgbaImage, observed: &RgbaImage) -> f64 {
    let (w, h) = (template.width() as usize, template.height() as usize);
    let (a, b) = (template.as_raw(), observed.as_raw());
    let mut best = u32::MAX;
    for sy in 0..3 {
        for sx in 0..3 {
            let mut total = 0u32;
            for y in 1..h - 1 {
                let row_a = &a[(y * w + 1) * 4..(y * w + w - 1) * 4];
                let start_b = ((y + sy - 1) * w + sx) * 4;
                let row_b = &b[start_b..start_b + (w - 2) * 4];
                for (p, q) in row_a.chunks_exact(4).zip(row_b.chunks_exact(4)) {
                    total += p[0].abs_diff(q[0]) as u32
                        + p[1].abs_diff(q[1]) as u32
                        + p[2].abs_diff(q[2]) as u32;
                }
            }
            best = best.min(total);
        }
    }
    best as f64 / (3 * (w - 2) * (h - 2)) as f64
}

// how alike the two are, whatever the pass limit
pub fn confidence(template: &OneIconConfig, observed: &mut OneIconConfig, brightness: f64) -> Option<f64> {
    if template.offset.width != observed.offset.width
        || template.offset.height != observed.offset.height
    {
        return None;
    }

    normalize_brightness(observed, brightness);

    let distance = shifted_distance(&template.data, &observed.data);
    Some(1.0 - distance / 255.0)
}

pub fn close_enough(
    template: &OneIconConfig,
    observed: &mut OneIconConfig,
    brightness: f64,
) -> Option<f64> {
    confidence(template, observed, brightness)
        .filter(|x| *x > template.required_confidence.unwrap_or(DEFAULT_CONFIDENCE))
}
