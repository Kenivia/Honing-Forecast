use image::GrayImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use parking_lot::RwLock;
use rten::Model;
use rten_imageproc::{PointF, RotatedRect, Vec2};
use std::sync::LazyLock;

pub const OCR_LINE_HEIGHT: u32 = 64; // what the recogniser works at

pub static OCR_ENGINE: LazyLock<RwLock<Option<OcrEngine>>> = LazyLock::new(|| RwLock::new(None));
// the same model, kept to what a slot's count can be
pub static OCR_NUMBER_ENGINE: LazyLock<RwLock<Option<OcrEngine>>> =
    LazyLock::new(|| RwLock::new(None));
const NUMBER_CHARS: &str = "0123456789.+-";

pub fn load_ocr_engine(model: Vec<u8>) {
    // the alphabet is fixed per engine, so the model is loaded once for each
    let engine = |allowed_chars: Option<String>| {
        OcrEngine::new(OcrEngineParams {
            recognition_model: Some(Model::load(model.clone()).expect("model load failed")),
            allowed_chars,
            ..Default::default()
        })
        .expect("model load failed")
    };
    *OCR_ENGINE.write() = Some(engine(None));
    *OCR_NUMBER_ENGINE.write() = Some(engine(Some(NUMBER_CHARS.to_string())));
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
