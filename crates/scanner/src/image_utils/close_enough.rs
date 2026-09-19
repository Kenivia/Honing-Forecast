use crate::{image_utils::brightness::normalize_brightness, setup::OneIconConfig};
use hf_core::my_dbg;
use image_compare::Similarity;

pub fn close_enough(
    template: &OneIconConfig,
    observed: &mut OneIconConfig,
    brightness: f64,
) -> Option<f64> {
    if template.offset.width != observed.offset.width
        || template.offset.height != observed.offset.height
    {
        // my_dbg!(
        //     template.offset.width,
        //     observed.offset.width,
        //     template.offset.height,
        //     observed.offset.height,
        // );
        return None;
    }

    normalize_brightness(observed, brightness);

    let similarity: Similarity =
        image_compare::rgba_hybrid_compare(&template.data, &observed.data).expect("compare failed");
    // my_dbg!(similarity.score);
    if similarity.score > 0.7 {
        return Some(similarity.score);
    } else {
        return None;
    }
}
