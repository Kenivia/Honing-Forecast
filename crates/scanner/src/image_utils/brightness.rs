use image::{GrayImage, imageops::grayscale};

use crate::{constants::ANCHORS_LOOKUP, scanner_state::InventoryType, setup::OneIconConfig};

const GAMMA_RATE: f64 = 0.01502885;
const GAIN_LINEAR: f64 = 0.045408;
const GAIN_QUADRATIC: f64 = -0.047688;
const TARGET_BRIGHTNESS: f64 = 50.0;

pub fn mean_intensity(icon: &OneIconConfig) -> f64 {
    let gray: GrayImage = grayscale(&icon.data);
    let pixels = gray.as_raw();
    let n = pixels.len() as f64;

    pixels.iter().map(|&p| p as f64).sum::<f64>() / n
}

pub fn est_ingame_brightness(
    best_mean_f: f64,
    inventory_type: InventoryType,
    variant_name: &str,
) -> f64 {
    let c = ANCHORS_LOOKUP
        .get(&inventory_type)
        .expect("no anchors registered for inventory type")
        .iter()
        .find(|(name, _, _)| name == variant_name)
        .map(|(_, _, c)| *c)
        .expect("no matching anchor for variant_name");

    (c[0] * best_mean_f * best_mean_f + c[1] * best_mean_f + c[2]).clamp(0.0, 100.0)
}

pub fn normalize_brightness(input: &mut OneIconConfig, in_game_brightness: f64) {
    if input.normalized {
        return;
    }

    let exponent = (1.0 + GAMMA_RATE * in_game_brightness.clamp(0.0, 100.0))
        / (1.0 + GAMMA_RATE * TARGET_BRIGHTNESS);
    let gain = (GAIN_LINEAR * (exponent - 1.0)
        + GAIN_QUADRATIC * (exponent - 1.0) * (exponent - 1.0))
        .exp();

    let lut: [u8; 256] = std::array::from_fn(|v| {
        (255.0 * gain * (v as f64 / 255.0).powf(exponent))
            .round()
            .clamp(0.0, 255.0) as u8
    });

    for pixel in input.data.pixels_mut() {
        for channel in pixel.0.iter_mut().take(3) {
            *channel = lut[*channel as usize];
        }
    }

    input.normalized = true;
}
