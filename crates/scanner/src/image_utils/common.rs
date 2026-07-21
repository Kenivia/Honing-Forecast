use crate::setup::OneIconConfig;
use fast_image_resize::images::Image;
use image::RgbaImage;

pub fn config_to_rgba(template: &OneIconConfig) -> RgbaImage {
    let template_w: u32 = template.position.width as u32;
    let template_h: u32 = template.position.height as u32;
    RgbaImage::from_raw(template_w, template_h, template.data.clone()).unwrap()
}

pub fn image_to_rgba(observed: Image) -> RgbaImage {
    let observed_w: u32 = observed.width();
    let observed_h: u32 = observed.height();
    RgbaImage::from_raw(observed_w, observed_h, observed.into_vec()).unwrap()
}
