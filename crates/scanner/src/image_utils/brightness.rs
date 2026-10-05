use image::{GrayImage, imageops::grayscale};

use crate::setup::OneIconConfig;

// The setting is a display gamma that grows in a straight line with it, so two settings differ
// by a power law and nothing else. Only this ratio of the line's offset to its slope can be told
// from captures; 62.6 fits those in scripts/brightness/inputs storage to 0.15 levels on average.
const GAMMA_RATIO: f64 = 62.6;
const TARGET_BRIGHTNESS: f64 = 50.0;

pub fn mean_intensity(icon: &OneIconConfig) -> f64 {
    let gray: GrayImage = grayscale(&icon.data);
    let pixels = gray.as_raw();
    let n = pixels.len() as f64;

    pixels.iter().map(|&p| p as f64).sum::<f64>() / n
}

pub fn est_ingame_brightness(best_mean_f: f64, c: &[f64; 3]) -> f64 {
    (c[0] * best_mean_f * best_mean_f + c[1] * best_mean_f + c[2]).clamp(0.0, 100.0)
}

pub fn brightness_lut(in_game_brightness: f64) -> [u8; 256] {
    let exponent = (GAMMA_RATIO + in_game_brightness.clamp(0.0, 100.0))
        / (GAMMA_RATIO + TARGET_BRIGHTNESS);

    std::array::from_fn(|v| (255.0 * (v as f64 / 255.0).powf(exponent)).round() as u8)
}

pub fn normalize_brightness(input: &mut OneIconConfig, in_game_brightness: f64) {
    if input.normalized {
        return;
    }

    let lut = brightness_lut(in_game_brightness);

    for pixel in input.data.pixels_mut() {
        for channel in pixel.0.iter_mut().take(3) {
            *channel = lut[*channel as usize];
        }
    }

    input.normalized = true;
}
