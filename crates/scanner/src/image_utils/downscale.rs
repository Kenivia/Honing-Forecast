// use crate::buffer::Buffer;

use crate::buffer::Buffer;
use crate::image_utils::common::{FloatRectangle, IntegerRectangle, Rectangle};
use crate::setup::OneIconConfig;
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

pub fn crop_buffer<R: Rectangle>(position: R, resizer: &mut Resizer, buffer: Buffer) -> Image<'_> {
    // let src_image: ImageRef<'_> = self.src_image(); // pre sure initiailizing this is cheap enough so i won't bother skipping it potentially

    let mut dst_image: Image<'_> = Image::new(
        position.width_usize() as u32,
        position.height_usize() as u32,
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
                .resize_alg(ResizeAlg::Interpolation(FilterType::Lanczos3))
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

pub fn resize_one_config<'a>(
    crop_within_icon: Option<FloatRectangle>,
    scale_factor: f64,
    resizer: &mut Resizer,
    one_config: &'a OneIconConfig,
    original_offset: IntegerRectangle,
) -> (Image<'a>, IntegerRectangle) {
    // let src_image: ImageRef<'_> = self.src_image(); // pre sure initiailizing this is cheap enough so i won't bother skipping it potentially
    let src_w = one_config.offset.width as u32;
    let src_h = one_config.offset.height as u32;
    let default = FloatRectangle {
        top_left: (0.0, 0.0),
        width: src_w as f64,
        height: src_h as f64,
    };
    let src_position = crop_within_icon.unwrap_or(default);
    let dst_position = src_position.scaled(scale_factor); // the top_left of dst_position has no effect 

    let mut dst_image: Image<'_> = Image::new(
        dst_position.width_usize() as u32,
        dst_position.height_usize() as u32,
        PixelType::U8x4,
    );

    // if self.downscaled_cache.written_this_cycle && self.covered_by_cache(&position) {

    let src_image = ImageRef::new(src_w, src_h, &one_config.data, PixelType::U8x4)
        .expect("invalid source image buffer");

    resizer
        .resize(
            &src_image,
            &mut dst_image,
            &ResizeOptions::new()
                .resize_alg(ResizeAlg::Interpolation(FilterType::Lanczos3))
                .use_alpha(false) // doesn't really matter cos we should only ever be cropping observed screeen capture
                .crop(
                    src_position.top_left().0,
                    src_position.top_left().1,
                    src_position.width_f64(),
                    src_position.height_f64(),
                ),
        )
        .expect("resize failed");
    // }

    // self.actual_downscale(&src_image, &mut dst_image, &position);
    (
        dst_image,
        dst_position
            .to_rounded()
            .with_top_left(original_offset.to_float().scaled(scale_factor).top_left),
    )
}
// }
