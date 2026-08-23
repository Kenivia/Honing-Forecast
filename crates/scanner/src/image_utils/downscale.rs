// use crate::buffer::Buffer;

use crate::buffer::Buffer;
use crate::scanner_state::Rectangle;
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use hf_core::my_dbg;

// impl ScannerState {

// fn actual_downscale(
//     &self,
//     src_image: &ImageRef<'_>,
//     dst_image: &mut Image<'_>,
//     pos: &ScaledPosition,
// ) {
//     let (scale_x, scale_y) = self.scale_factors();
//     let dst_w: u32 = pos.width as u32;
//     let dst_h: u32 = pos.height as u32;

//     let src_left: f64 = pos.top_left.0 as f64 * scale_x;
//     let src_top: f64 = pos.top_left.1 as f64 * scale_y;
//     let src_width: f64 = dst_w as f64 * scale_x;
//     let src_height: f64 = dst_h as f64 * scale_y;

//     let mut resizer: Resizer = Resizer::new();

//     let options: ResizeOptions = ResizeOptions::new()
//         .resize_alg(ResizeAlg::Convolution(FilterType::Box))
//         .crop(src_left, src_top, src_width, src_height);

//     self.resizer
//         .as_mut()
//         .unwrap_or(&mut Resizer::new())
//         .resize(src_image, dst_image, Some(&options))
//         .unwrap();
// }

pub fn crop_buffer(position: Rectangle, resizer: &mut Resizer, buffer: Buffer) -> Image<'_> {
    // let src_image: ImageRef<'_> = self.src_image(); // pre sure initiailizing this is cheap enough so i won't bother skipping it potentially

    let mut dst_image: Image<'_> = Image::new(
        position.width as u32,
        position.height as u32,
        PixelType::U8x4,
    );

    // if self.downscaled_cache.written_this_cycle && self.covered_by_cache(&position) {
    let src_w = buffer.width as u32;
    let src_h = buffer.height as u32;

    let src_buffer: &[u8] =
        unsafe { std::slice::from_raw_parts(buffer.pointer.unwrap() as *const u8, buffer.size) };
    assert!(src_buffer.len() >= src_w as usize * src_h as usize * 4);

    let src_image = ImageRef::new(src_w, src_h, src_buffer, PixelType::U8x4)
        .expect("invalid source image buffer");

    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Interpolation(FilterType::Bilinear))
        .use_alpha(false) // doesn't really matter cos we should only ever be cropping observed screeen capture
        .crop(
            position.top_left.0,
            position.top_left.1,
            dst_image.width() as f64,
            dst_image.height() as f64,
        );

    resizer
        .resize(&src_image, &mut dst_image, &options)
        .expect("crop failed");
    // }

    // self.actual_downscale(&src_image, &mut dst_image, &position);
    dst_image
}
// }
