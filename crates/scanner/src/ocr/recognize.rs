use crate::{
    dev_utils::timing::timed,
    interface::setup::{OCR_ENGINE, OCR_NUMBER_ENGINE},
};
use image::GrayImage;
use ocrs::ImageSource;
use rten_imageproc::{PointF, RotatedRect, Vec2};

// the whole image is one line of text; `numbers` keeps it to what a slot's count can be
pub fn recognize_line(image: &GrayImage, numbers: bool) -> String {
    timed("ocr", || recognize_line_untimed(image, numbers))
}

fn recognize_line_untimed(image: &GrayImage, numbers: bool) -> String {
    let read = if numbers {
        OCR_NUMBER_ENGINE.read()
    } else {
        OCR_ENGINE.read()
    };
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
