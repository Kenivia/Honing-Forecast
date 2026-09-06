use fast_image_resize::Resizer;
use ocrs::ImageSource;
use rten_imageproc::{PointF, RotatedRect, Vec2};
use crate::{
    image_utils::resize::resize_one_config,
    setup::{OCR_ENGINE, OneIconConfig},
};

pub fn pre_process(input: OneIconConfig, resizer: &mut Resizer) ->  {
    let (scaled_image, _) =
        resize_one_config(None, 64.0 / input.offset.height as f64, resizer, &input);
}
pub fn get_number(scaled_image: Image, resizer: &mut Resizer) -> String {
    let read = OCR_ENGINE.read(); // need to have this line for some reason
    let engine = read.as_ref().unwrap();
    let width = scaled_image.width();
    let height = scaled_image.height();
    return engine
        .recognize_text(
            &engine
                .prepare_input(
                    ImageSource::from_bytes(&scaled_image.into_vec(), (width, height)).unwrap(),
                )
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
