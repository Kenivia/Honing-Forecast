use crate::{scanner_state::ScannerState, setup::OneIconConfig};
use fast_image_resize::images::Image;
use image::Rgb;
use image_compare::{BlendInput, Similarity, rgba_hybrid_compare};

use super::common::{config_to_rgba, image_to_rgba};

pub fn close_enough(template: &OneIconConfig, observed: Image) -> Option<f64> {
    if template.offset.width != observed.width() as usize
        || template.offset.height != observed.height() as usize
    {
        return None;
    }

    let bg: Rgb<u8> = Rgb([3, 3, 3]);
    let similarity: Similarity = image_compare::rgba_blended_hybrid_compare(
        BlendInput::RGBA(&config_to_rgba(template)),
        BlendInput::RGBA(&image_to_rgba(observed)),
        bg,
    )
    .expect("template & observed dimension mismatch");
    if similarity.score > 0.9 {
        return Some(similarity.score as f64);
    } else {
        return None;
    }
}
