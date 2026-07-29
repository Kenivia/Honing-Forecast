use std::slice::Iter;

use crate::{
    constants::TARGET_RESOLUTION,
    scanner_state::{ScaledPosition, ScannerState},
    setup::OneIconConfig,
};
use fast_image_resize::{
    PixelType,
    images::{Image, ImageRef},
};
use image::RgbaImage;

pub const FULL_RECT_16_9: ScaledPosition = ScaledPosition {
    top_left: (0.0, 0.0),
    width: 1280,
    height: 720,
};

pub fn config_to_rgba(template: &OneIconConfig) -> RgbaImage {
    let template_w: u32 = template.offset.width as u32;
    let template_h: u32 = template.offset.height as u32;
    RgbaImage::from_raw(template_w, template_h, template.data.clone()).unwrap()
}

pub fn image_to_rgba(observed: Image) -> RgbaImage {
    let observed_w: u32 = observed.width();
    let observed_h: u32 = observed.height();
    RgbaImage::from_raw(observed_w, observed_h, observed.into_vec()).unwrap()
}

pub fn bounding_rect(positions: Vec<ScaledPosition>) -> ScaledPosition {
    let mut iter: Iter<'_, ScaledPosition> = positions.iter();
    let first: &ScaledPosition = iter.next().unwrap();

    let mut min_x: f64 = first.top_left.0;
    let mut min_y: f64 = first.top_left.1;
    let mut max_x: f64 = first.top_left.0 + first.width as f64;
    let mut max_y: f64 = first.top_left.1 + first.height as f64;

    for pos in iter {
        min_x = min_x.min(pos.top_left.0);
        min_y = min_y.min(pos.top_left.1);
        max_x = max_x.max(pos.top_left.0 + pos.width as f64);
        max_y = max_y.max(pos.top_left.1 + pos.height as f64);
    }

    ScaledPosition {
        top_left: (min_x, min_y),
        width: (max_x - min_x).ceil() as usize,
        height: (max_y - min_y).ceil() as usize,
    }
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
