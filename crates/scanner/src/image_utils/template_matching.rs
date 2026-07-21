use crate::{
    scanner_state::{ScaledPosition, ScannerState},
    setup::OneIconConfig,
};
use fast_image_resize::images::Image;
use image::{GrayImage, ImageBuffer, Luma, imageops::grayscale};
use imageproc::template_matching::{Extremes, MatchTemplateMethod, find_extremes, match_template};

use super::common::{config_to_rgba, image_to_rgba};

impl ScannerState {
    pub fn template_match(
        &self,
        template: &OneIconConfig,
        observed: Image,
    ) -> Option<ScaledPosition> {
        let template_img = config_to_rgba(template);
        let observed_img = image_to_rgba(observed);

        // imageproc's template matching only works on grayscale images.
        let template_gray: GrayImage = grayscale(&template_img);
        let observed_gray: GrayImage = grayscale(&observed_img);

        let result: ImageBuffer<Luma<f32>, Vec<f32>> = match_template(
            &observed_gray,
            &template_gray,
            MatchTemplateMethod::CrossCorrelationNormalized,
        );

        let extremes: Extremes<f32> = find_extremes(&result);

        if extremes.max_value < 0.9 {
            return None;
        }

        let (x, y) = extremes.max_value_location;

        Some(ScaledPosition {
            top_left: (x as f64, y as f64),
            width: template.position.width,
            height: template.position.height,
        })
    }
}
