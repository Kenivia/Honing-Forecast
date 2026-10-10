use super::{
    jobs::{Kind, Line},
    recognize::OCR_LINE_HEIGHT,
};
use crate::{buffer::Buffer, image_utils::brightness::brightness_lut};
use image::{
    GrayImage, Luma, RgbaImage,
    imageops::{FilterType, resize},
};

// a line of a tooltip, whatever is bright in it
pub fn text_line(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> Line {
    Line { crop: buffer.raw_crop(x0, y0, x1, y1), kind: Kind::Text, brightness: buffer.brightness }
}

// the same, to be read by what is in the amount yellow alone
pub fn yellow_line(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> Line {
    Line { crop: buffer.raw_crop(x0, y0, x1, y1), kind: Kind::Yellow, brightness: buffer.brightness }
}

// Light text on a dark panel to dark text on white, which is what the recogniser reads best.
// `yellow` keeps only the amount yellow (255, 213, 0), so white text and the background drop out.
pub fn text_strip(crop: &RgbaImage, yellow: bool, brightness: f64) -> GrayImage {
    let lut = brightness_lut(brightness);
    let ink = |x: u32, y: u32| {
        let [r, g, b] = [0, 1, 2].map(|c| lut[crop.get_pixel(x, y)[c] as usize] as i32);
        if yellow { (r.min(g) - b).max(0) } else { r.max(g).max(b) }
    };
    let (width, height) = crop.dimensions();
    let most = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| ink(x, y))
        .max()
        .unwrap()
        .max(1);
    let image = GrayImage::from_fn(width, height, |x, y| Luma([255 - (ink(x, y) * 255 / most) as u8]));
    // the recogniser works on lines 64px tall
    resize(&image, width * OCR_LINE_HEIGHT / height, OCR_LINE_HEIGHT, FilterType::CatmullRom)
}
