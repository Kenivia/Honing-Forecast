use image::RgbaImage;

use crate::interface::setup::OneIconConfig;

pub const DEFAULT_CLOSE_ENOUGH_CONFIDENCE: f64 = 0.95;

// part of a template, in its pixels: x0, y0, x1, y1
pub type Area = (usize, usize, usize, usize);

// Mean absolute RGB difference, without the outermost pixels and the parts of the template in `skip`
fn distance(template: &RgbaImage, observed: &RgbaImage, skip: &[Area]) -> f64 {
    let (w, h) = (template.width() as usize, template.height() as usize);
    let (a, b) = (template.as_raw(), observed.as_raw());
    let (mut total, mut compared) = (0u32, 0u32);
    for y in 1..h - 1 {
        let row = (y * w + 1) * 4..(y * w + w - 1) * 4;
        let pixels = a[row.clone()].chunks_exact(4).zip(b[row].chunks_exact(4));
        for (x, (p, q)) in (1..).zip(pixels) {
            if skip
                .iter()
                .any(|s| s.0 <= x && x < s.2 && s.1 <= y && y < s.3)
            {
                continue;
            }
            total += p[0].abs_diff(q[0]) as u32
                + p[1].abs_diff(q[1]) as u32
                + p[2].abs_diff(q[2]) as u32;
            compared += 1;
        }
    }
    total as f64 / (3 * compared) as f64
}

// How alike the two are, whatever the pass limit. Both are at the reference brightness already.
pub fn confidence(template: &OneIconConfig, observed: &OneIconConfig) -> f64 {
    masked_confidence(template, observed, &[])
}

// the same with parts of the template left out, for a slot with something drawn over them
pub fn masked_confidence(template: &OneIconConfig, observed: &OneIconConfig, skip: &[Area]) -> f64 {
    assert_eq!(template.offset.space, observed.offset.space);
    assert_eq!(template.offset.pixel_size(), observed.offset.pixel_size());

    let distance = distance(&template.data, &observed.data, skip);
    1.0 - distance / 255.0
}

pub fn close_enough(template: &OneIconConfig, observed: &OneIconConfig) -> Option<f64> {
    Some(confidence(template, observed)).filter(|x| {
        *x > template
            .required_confidence
            .unwrap_or(DEFAULT_CLOSE_ENOUGH_CONFIDENCE)
    })
}
