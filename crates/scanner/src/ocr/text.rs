use super::jobs::{Kind, OCRLine};
use crate::{
    buffer::Buffer, image_utils::brightness::brightness_lut, interface::setup::OCR_LINE_HEIGHT,
};
use image::{
    GrayImage, Luma, RgbaImage,
    imageops::{FilterType, resize, unsharpen},
};
use parking_lot::RwLock;

// How a text strip is made. The default is what the scanner does; native bins take another from
// TEXT_STRIP (see `parse`), and text_bench compares them.
#[derive(Clone, Copy, Debug)]
pub struct TextParams {
    pub filter: FilterType,
    // after the resize: ink at or below the first becomes black, paper at or above the second white
    pub levels: Option<(u8, u8)>,
    // after the resize (and levels): sigma and threshold
    pub unsharp: Option<(f32, i32)>,
}

impl TextParams {
    pub const NOW: TextParams = TextParams {
        filter: FilterType::CatmullRom,
        levels: None,
        unsharp: None,
    };

    // "filter=triangle levels=60,200 unsharp=2,10", any of them; "now" is the default
    pub fn parse(spec: &str) -> TextParams {
        let mut params = TextParams::NOW;
        for part in spec.split_whitespace().filter(|x| *x != "now") {
            let (name, value) = part.split_once('=').unwrap();
            let numbers: Vec<f64> = value.split(',').filter_map(|x| x.parse().ok()).collect();
            match name {
                "filter" => {
                    params.filter = match value {
                        "nearest" => FilterType::Nearest,
                        "triangle" => FilterType::Triangle,
                        "catmullrom" => FilterType::CatmullRom,
                        "gaussian" => FilterType::Gaussian,
                        "lanczos3" => FilterType::Lanczos3,
                        _ => panic!("no filter {value}"),
                    }
                }
                "levels" => params.levels = Some((numbers[0] as u8, numbers[1] as u8)),
                "unsharp" => params.unsharp = Some((numbers[0] as f32, numbers[1] as i32)),
                _ => panic!("no text strip setting {name}"),
            }
        }
        params
    }
}

pub static TEXT_PARAMS: RwLock<TextParams> = RwLock::new(TextParams::NOW);

// a rectangle of the frame as captured, to be read as this kind
pub fn line_of(kind: Kind, buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> OCRLine {
    OCRLine {
        crop: buffer.raw_crop(x0, y0, x1, y1),
        kind,
        brightness: buffer.brightness,
    }
}

// a line of a tooltip, whatever is bright in it
pub fn text_line(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> OCRLine {
    line_of(Kind::Text, buffer, x0, y0, x1, y1)
}

// the same, to be read by what is in the amount yellow alone
pub fn yellow_line(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> OCRLine {
    line_of(Kind::Yellow, buffer, x0, y0, x1, y1)
}

// Light text on a dark panel to dark text on white, which is what the recogniser reads best.
// `yellow` keeps only the amount yellow (255, 213, 0), so white text and the background drop out.
pub fn text_strip(crop: &RgbaImage, yellow: bool, brightness: f64) -> GrayImage {
    text_strip_with(crop, yellow, brightness, &TEXT_PARAMS.read())
}

pub fn text_strip_with(
    crop: &RgbaImage,
    yellow: bool,
    brightness: f64,
    params: &TextParams,
) -> GrayImage {
    let lut = brightness_lut(brightness);
    let ink = |x: u32, y: u32| {
        let [r, g, b] = [0, 1, 2].map(|c| lut[crop.get_pixel(x, y)[c] as usize] as i32);
        if yellow {
            (r.min(g) - b).max(0)
        } else {
            r.max(g).max(b)
        }
    };
    let (width, height) = crop.dimensions();
    let most = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| ink(x, y))
        .max()
        .unwrap()
        .max(1);
    let image = GrayImage::from_fn(width, height, |x, y| {
        Luma([255 - (ink(x, y) * 255 / most) as u8])
    });
    // the recogniser works on lines 64px tall
    let mut strip = resize(
        &image,
        width * OCR_LINE_HEIGHT / height,
        OCR_LINE_HEIGHT,
        params.filter,
    );
    if let Some((black, white)) = params.levels {
        let span = (white as i32 - black as i32).max(1);
        for pixel in strip.pixels_mut() {
            pixel[0] = ((pixel[0] as i32 - black as i32) * 255 / span).clamp(0, 255) as u8;
        }
    }
    if let Some((sigma, threshold)) = params.unsharp {
        strip = unsharpen(&strip, sigma, threshold);
    }
    strip
}
