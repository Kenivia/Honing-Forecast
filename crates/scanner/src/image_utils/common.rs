use crate::{constants::TARGET_RESOLUTION, scanner_state::ScannerState, setup::OneIconConfig};
use fast_image_resize::{
    PixelType,
    images::{Image, ImageRef},
};
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
impl ScannerState {
    pub fn src_image(&self) -> ImageRef<'_> {
        let src_w: u32 = self.screen_info.total_width as u32;
        let src_h: u32 = self.screen_info.total_height as u32;

        let src_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.buffer.pointer.expect("uninitialized buffer") as *const u8,
                self.buffer.size,
            )
        };

        ImageRef::new(src_w, src_h, src_bytes, PixelType::U8x4)
            .expect("source buffer size must equal src_w * src_h * 4")
    }

    pub fn scale_factors(&self) -> (f64, f64) {
        let src_w: u32 = self.screen_info.total_width as u32;
        let src_h: u32 = self.screen_info.total_height as u32;

        (
            src_w as f64 / TARGET_RESOLUTION.0 as f64,
            src_h as f64 / TARGET_RESOLUTION.1 as f64,
        )
    }
}
