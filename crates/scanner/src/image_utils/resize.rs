use crate::image_utils::common::{Rect, Space, get_resizer};
use crate::scanner_state::ScannerState;
use crate::setup::OneIconConfig;
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::RgbaImage;

impl ScannerState {
    // a piece of the frame as captured, resampled to whole pixels
    pub fn crop_buffer(&mut self, position: Rect) -> OneIconConfig {
        assert_eq!(position.space, Space::Screen(self.screen_info.effective_height));
        let (width, height) = position.pixel_size();
        let mut dst_image: Image<'_> = Image::new(width, height, PixelType::U8x4);

        let buffer = self.buffer;
        let src_w = buffer.width as u32;
        let src_h = buffer.height as u32;

        let src_buffer: &[u8] =
            unsafe { std::slice::from_raw_parts(buffer.pointer.unwrap() as *const u8, buffer.size) };
        assert!(src_buffer.len() >= src_w as usize * src_h as usize * 4);

        let src_image = ImageRef::new(src_w, src_h, src_buffer, PixelType::U8x4)
            .expect("invalid source image buffer");

        get_resizer(&mut self.resizer)
            .resize(
                &src_image,
                &mut dst_image,
                &ResizeOptions::new()
                    .resize_alg(ResizeAlg::Interpolation(FilterType::Lanczos3))
                    .use_alpha(false) // doesn't really matter cos we should only ever be cropping observed screeen capture
                    .crop(
                        position.top_left.0,
                        position.top_left.1,
                        position.width,
                        position.height,
                    ),
            )
            .expect("crop failed");
        OneIconConfig {
            data: RgbaImage::from_raw(width, height, dst_image.into_vec()).unwrap(),
            name: "".to_string(),
            offset: position,
            tag: "".to_string(),
            required_confidence: None,
        }
    }
}

// part of a base template, given in its own pixels, resized to a pixel size
pub fn resize_one_config(
    crop_top_left: (f64, f64),
    crop_size: (f64, f64),
    pixel_size: (u32, u32),
    resizer: &mut Resizer,
    one_config: &OneIconConfig,
) -> RgbaImage {
    let (src_w, src_h) = one_config.data.dimensions();
    let mut dst_image: Image<'_> = Image::new(pixel_size.0, pixel_size.1, PixelType::U8x4);

    let src_image = ImageRef::new(src_w, src_h, &one_config.data, PixelType::U8x4)
        .expect("invalid source image buffer");

    resizer
        .resize(
            &src_image,
            &mut dst_image,
            &ResizeOptions::new()
                .resize_alg(ResizeAlg::Interpolation(FilterType::Bilinear))
                .use_alpha(false) // doesn't really matter cos we should only ever be cropping observed screeen capture
                .crop(crop_top_left.0, crop_top_left.1, crop_size.0, crop_size.1),
        )
        .expect("resize failed");
    RgbaImage::from_raw(pixel_size.0, pixel_size.1, dst_image.into_vec()).unwrap()
}
