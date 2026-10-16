use image::RgbaImage;

use crate::interface::setup::OneIconConfig;

pub const DEFAULT_CLOSE_ENOUGH_CONFIDENCE: f64 = 0.95;

// part of a template, in its pixels: x0, y0, x1, y1
pub type Area = (usize, usize, usize, usize);

// Mean absolute RGB difference, best of the 9 one-pixel shifts so a slightly misplaced crop still
// matches. The parts of the template in `skip` are left out.
fn shifted_distance(template: &RgbaImage, observed: &RgbaImage, skip: &[Area]) -> f64 {
    let (w, h) = (template.width() as usize, template.height() as usize);
    let (a, b) = (template.as_raw(), observed.as_raw());
    // the stretches of each row that are compared
    let mut runs: Vec<(usize, usize, usize)> = vec![];
    for y in 1..h - 1 {
        let mut from = None;
        for x in 1..w {
            let keep = x < w - 1
                && !skip
                    .iter()
                    .any(|s| s.0 <= x && x < s.2 && s.1 <= y && y < s.3);
            match (keep, from) {
                (true, None) => from = Some(x),
                (false, Some(start)) => {
                    runs.push((y, start, x));
                    from = None;
                }
                _ => {}
            }
        }
    }
    let mut best = u32::MAX;
    for sy in 0..3 {
        for sx in 0..3 {
            let mut total = 0u32;
            for (y, x0, x1) in &runs {
                let row_a = &a[(y * w + x0) * 4..(y * w + x1) * 4];
                let start_b = ((y + sy - 1) * w + x0 + sx - 1) * 4;
                let row_b = &b[start_b..start_b + (x1 - x0) * 4];
                for (p, q) in row_a.chunks_exact(4).zip(row_b.chunks_exact(4)) {
                    total += p[0].abs_diff(q[0]) as u32
                        + p[1].abs_diff(q[1]) as u32
                        + p[2].abs_diff(q[2]) as u32;
                }
            }
            best = best.min(total);
        }
    }
    let compared: usize = runs.iter().map(|x| x.2 - x.1).sum();
    best as f64 / (3 * compared) as f64
}

// How alike the two are, whatever the pass limit. Both are at the reference brightness already.
pub fn confidence(template: &OneIconConfig, observed: &OneIconConfig) -> f64 {
    masked_confidence(template, observed, &[])
}

// the same with parts of the template left out, for a slot with something drawn over them
pub fn masked_confidence(template: &OneIconConfig, observed: &OneIconConfig, skip: &[Area]) -> f64 {
    assert_eq!(template.offset.space, observed.offset.space);
    assert_eq!(template.offset.pixel_size(), observed.offset.pixel_size());

    let distance = shifted_distance(&template.data, &observed.data, skip);
    1.0 - distance / 255.0
}

pub fn close_enough(template: &OneIconConfig, observed: &OneIconConfig) -> Option<f64> {
    Some(confidence(template, observed)).filter(|x| {
        *x > template
            .required_confidence
            .unwrap_or(DEFAULT_CLOSE_ENOUGH_CONFIDENCE)
    })
}
