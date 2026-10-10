use crate::{
    image_utils::{
        number::{NumberParams, number_background, number_mask},
    },
    setup::{OCR_ENGINE, OCR_NUMBER_ENGINE},
};
use image::{
    GrayImage, Luma, RgbaImage,
    imageops::{FilterType, crop_imm, resize},
};
use imageproc::region_labelling::{Connectivity, connected_components};
use ocrs::ImageSource;
use rten_imageproc::{PointF, RotatedRect, Vec2};
use std::collections::HashMap;

pub const OCR_LINE_HEIGHT: u32 = 64; // what the recogniser works at

const SPECK_AREA: u32 = 32; // blobs up to the strip's height squared over this are specks
const LEFT_MARGIN: u32 = 5; // black kept left of the number, as the strip's height over this

// white blobs too small to be part of a digit,
fn remove_specks(white: &mut GrayImage) {
    let (w, h) = white.dimensions();
    let binary = GrayImage::from_fn(w, h, |x, y| Luma([(white.get_pixel(x, y)[0] > 0) as u8]));
    let labels = connected_components(&binary, Connectivity::Eight, Luma([0u8]));
    let mut sizes: HashMap<u32, u32> = HashMap::new();
    for label in labels.pixels() {
        *sizes.entry(label[0]).or_default() += 1;
    }
    let largest = h * h / SPECK_AREA;
    for (x, y, label) in labels.enumerate_pixels() {
        if label[0] != 0 && sizes[&label[0]] <= largest {
            white.put_pixel(x, y, Luma([0]));
        }
    }
}

// The strip the recogniser gets for a slot's number. `icon` is the slot's icon crop, already
// normalised, and `template` the matched icon's base template; together they say what is behind the
// number. The number crop itself is used as captured, because the game's brightness setting does not
// touch the number.
pub fn number_strip(
    number: &RgbaImage,
    icon: &RgbaImage,
    template: &RgbaImage,
    brightness: f64,
    params: &NumberParams,
) -> GrayImage {
    let (w, h) = number.dimensions();
    let background = number_background(template, icon, w, h, brightness);
    // white on black; inverting it made the recogniser worse on numbers
    let mut white = number_mask(number, &background, params);
    remove_specks(&mut white);
    // the empty part left of the number is cut off; the recogniser drops leading digits less often
    let first = (0..w).find(|x| (0..h).any(|y| white.get_pixel(*x, y)[0] > 0));
    let left = first.map_or(0, |x| x.saturating_sub(h / LEFT_MARGIN));
    let image = crop_imm(&white, left, 0, w - left, h).to_image();
    let width = image.width() * OCR_LINE_HEIGHT / h;
    resize(&image, width, OCR_LINE_HEIGHT, FilterType::CatmullRom)
}

pub fn recognize_raw(width: u32, height: u32, data: Vec<u8>, numbers: bool) -> String {
    recognize_line(&GrayImage::from_raw(width, height, data).unwrap(), numbers)
}

// the whole image is one line of text; `numbers` keeps it to what a slot's count can be
pub fn recognize_line(image: &GrayImage, numbers: bool) -> String {
    crate::timing::timed("ocr", || recognize_line_untimed(image, numbers))
}

fn recognize_line_untimed(image: &GrayImage, numbers: bool) -> String {
    let read = if numbers { OCR_NUMBER_ENGINE.read() } else { OCR_ENGINE.read() };
    let engine = read.as_ref().unwrap();
    let (width, height) = image.dimensions();
    return engine
        .recognize_text(
            &engine
                .prepare_input(ImageSource::from_bytes(image, (width, height)).unwrap())
                .unwrap(),
            &vec![vec![RotatedRect::new(
                PointF::from_yx(height as f32 / 2.0, width as f32 / 2.0),
                Vec2::from_yx(1., 0.),
                width as f32,
                height as f32,
            )]],
        )
        .iter()
        .flatten()
        .map(|x| {
            let y = x.as_ref();
            if y.is_some() {
                return y.unwrap().to_string();
            } else {
                return "".to_string();
            }
        })
        .reduce(|prev, new| prev + &new)
        .unwrap_or("".to_string());
}
