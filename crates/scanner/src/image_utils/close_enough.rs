use crate::setup::OneIconConfig;
use fast_image_resize::images::Image;
use hf_core::my_dbg;
use image_compare::Similarity;

use super::common::{config_to_rgba, image_to_rgba};

pub fn close_enough(template: &OneIconConfig, observed: Image) -> Option<f64> {
    if template.offset.width != observed.width() as usize
        || template.offset.height != observed.height() as usize
    {
        my_dbg!(
            template.offset.width,
            observed.width() as usize,
            template.offset.height,
            observed.height() as usize
        );
        return None;
    }

    let similarity: Similarity =
        image_compare::rgba_hybrid_compare(&config_to_rgba(template), &image_to_rgba(observed))
            .expect("compare failed");
    // my_dbg!(similarity.score);
    if similarity.score > 0.7 {
        return Some(similarity.score);
    } else {
        return None;
    }
}
