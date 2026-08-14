// use crate::buffer::Buffer;

use crate::buffer::Buffer;
use crate::scanner_state::ScaledPosition;
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use hf_core::my_dbg;

// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// pub struct DownscaledCache {
//     pub buffer: Buffer,

//     #[serde(default)]
//     pub position: Option<ScaledPosition>,
//     #[serde(default)]
//     pub occupied_size: Option<usize>,
//     #[serde(default)]
//     pub written_this_cycle: bool,
// }
// impl DownscaledCache {
//     pub fn reset(&mut self) {
//         self.position = None;
//         self.occupied_size = None;
//         self.written_this_cycle = false;
//     }
// }

// impl ScannerState {
// fn covered_by_cache(&self, pos: &ScaledPosition) -> bool {
//     let downscaled_pos: ScaledPosition = self.downscaled_cache.position.unwrap();
//     let outer_left: i64 = downscaled_pos.top_left.0 as i64;
//     let outer_top: i64 = downscaled_pos.top_left.1 as i64;
//     let outer_right: i64 = outer_left + downscaled_pos.width as i64;
//     let outer_bottom: i64 = outer_top + downscaled_pos.height as i64;

//     let inner_left: i64 = pos.top_left.0 as i64;
//     let inner_top: i64 = pos.top_left.1 as i64;
//     let inner_right: i64 = inner_left + pos.width as i64;
//     let inner_bottom: i64 = inner_top + pos.height as i64;

//     inner_left >= outer_left
//         && inner_top >= outer_top
//         && inner_right <= outer_right
//         && inner_bottom <= outer_bottom
// }
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

// fn crop_only(&pos: &ScaledPosition, dst: &mut Image<'_>, resizer: &mut Resizer) {}

// pub fn write_downscaled_cache(&mut self, position: ScaledPosition) {
//     assert!(!self.downscaled_cache.written_this_cycle);

//     let src_image: ImageRef<'_> = self.src_image();
//     let mut dst_image: Image<'_> = Image::new(
//         position.width as u32,
//         position.height as u32,
//         PixelType::U8x4,
//     );

//     self.actual_downscale(&src_image, &mut dst_image, &position);

//     unsafe {
//         let dest_slice: &mut [u8] = std::slice::from_raw_parts_mut(
//             self.downscaled_cache.buffer.pointer.unwrap() as *mut u8,
//             self.downscaled_cache.buffer.size,
//         );
//         dest_slice[..dst_image.buffer().len()].copy_from_slice(dst_image.buffer());
//     };

//     self.downscaled_cache.position = Some(position);
//     self.downscaled_cache.written_this_cycle = true;
//     self.downscaled_cache.occupied_size = Some(position.width * position.height * 4);
// }

pub fn crop_buffer(position: ScaledPosition, resizer: &mut Resizer, buffer: Buffer) -> Image<'_> {
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
