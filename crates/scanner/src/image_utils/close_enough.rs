use crate::setup::OneIconConfig;
use image::RgbaImage;

pub const DEFAULT_CONFIDENCE: f64 = 0.95;

// Mean absolute RGB difference, best of the 9 one-pixel shifts so a slightly misplaced crop still
// matches. Template rows in `skip` are left out.
fn shifted_distance(template: &RgbaImage, observed: &RgbaImage, skip: (usize, usize)) -> f64 {
    let (w, h) = (template.width() as usize, template.height() as usize);
    let (a, b) = (template.as_raw(), observed.as_raw());
    let mut best = u32::MAX;
    for sy in 0..3 {
        for sx in 0..3 {
            let mut total = 0u32;
            for y in (1..h - 1).filter(|y| *y < skip.0 || *y >= skip.1) {
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
    let rows = (1..h - 1).filter(|y| *y < skip.0 || *y >= skip.1).count();
    best as f64 / (3 * (w - 2) * rows) as f64
}

// How alike the two are, whatever the pass limit. Both are at the reference brightness already.
pub fn confidence(template: &OneIconConfig, observed: &OneIconConfig) -> f64 {
    confidence_without(template, observed, (0, 0))
}

// the same with some rows of the template left out, for a slot with text over them
pub fn confidence_without(
    template: &OneIconConfig,
    observed: &OneIconConfig,
    skip: (usize, usize),
) -> f64 {
    assert_eq!(template.offset.space, observed.offset.space);
    assert_eq!(template.offset.pixel_size(), observed.offset.pixel_size());

    let distance = shifted_distance(&template.data, &observed.data, skip);
    1.0 - distance / 255.0
}

pub fn close_enough(template: &OneIconConfig, observed: &OneIconConfig) -> Option<f64> {
    Some(confidence(template, observed))
        .filter(|x| *x > template.required_confidence.unwrap_or(DEFAULT_CONFIDENCE))
}
