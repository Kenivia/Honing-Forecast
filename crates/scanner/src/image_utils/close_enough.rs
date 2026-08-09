use crate::{scanner_state::ScannerState, setup::OneIconConfig};
use fast_image_resize::images::Image;
use image_compare::{Similarity, rgba_hybrid_compare};

use super::common::{config_to_rgba, image_to_rgba};

impl ScannerState {
    pub fn images_close_enough(&self, template: &OneIconConfig, observed: Image) -> Option<f64> {
        if template.offset.width != observed.width() as usize
            || template.offset.height != observed.height() as usize
        {
            return None;
        }
        let template_img = config_to_rgba(template);
        let observed_img = image_to_rgba(observed);

        let similarity: Similarity = rgba_hybrid_compare(&template_img, &observed_img)
            .expect("template & observed dimension mismatch");
        if similarity.score > 0.9 {
            return Some(similarity.score as f64);
        } else {
            return None;
        }
    }
}
