// use crate::buffer::Buffer;

use crate::buffer::Buffer;
use crate::image_utils::common::Rectangle;
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

pub fn crop_buffer<R: Rectangle>(position: R, resizer: &mut Resizer, buffer: Buffer) -> Image<'_> {
    // let src_image: ImageRef<'_> = self.src_image(); // pre sure initiailizing this is cheap enough so i won't bother skipping it potentially

    let mut dst_image: Image<'_> = Image::new(
        position.width_usize() as u32,
        position.width_usize() as u32,
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

    resizer
        .resize(
            &src_image,
            &mut dst_image,
            &ResizeOptions::new()
                .resize_alg(ResizeAlg::Interpolation(FilterType::Bilinear))
                .use_alpha(false) // doesn't really matter cos we should only ever be cropping observed screeen capture
                .crop(
                    position.top_left().0,
                    position.top_left().1,
                    position.width_f64(),
                    position.height_f64(),
                ),
        )
        .expect("crop failed");
    // }

    // self.actual_downscale(&src_image, &mut dst_image, &position);
    dst_image
}
// }
