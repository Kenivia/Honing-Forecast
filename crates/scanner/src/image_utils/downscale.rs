use crate::constants::TARGET_RESOLUTION;
use crate::scanner_state::{ScaledPosition, ScannerState};
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

impl ScannerState {
    pub fn downscale(&mut self, positions: &[ScaledPosition]) -> Vec<Image<'_>> {
        let src_w: u32 = self.screen_info.total_width as u32;
        let src_h: u32 = self.screen_info.total_height as u32;

        let scale_x: f64 = src_w as f64 / TARGET_RESOLUTION.0 as f64;
        let scale_y: f64 = src_h as f64 / TARGET_RESOLUTION.1 as f64;

        let src_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.buffer.pointer.expect("uninitialized buffer") as *const u8,
                self.buffer.size,
            )
        };
        let src_image: ImageRef<'_> = ImageRef::new(src_w, src_h, src_bytes, PixelType::U8x4)
            .expect("source buffer size must equal src_w * src_h * 4");

        let mut resizer: Resizer = Resizer::new();

        let mut out: Vec<Image<'_>> = Vec::with_capacity(positions.len());
        for pos in positions {
            let dst_w: u32 = pos.width as u32;
            let dst_h: u32 = pos.height as u32;

            let src_left: f64 = pos.top_left.0 as f64 * scale_x;
            let src_top: f64 = pos.top_left.1 as f64 * scale_y;
            let src_width: f64 = dst_w as f64 * scale_x;
            let src_height: f64 = dst_h as f64 * scale_y;

            let mut region_image: Image<'_> = Image::new(dst_w, dst_h, PixelType::U8x4);

            let options: ResizeOptions = ResizeOptions::new()
                .resize_alg(ResizeAlg::Convolution(FilterType::Box))
                .crop(src_left, src_top, src_width, src_height);

            resizer
                .resize(&src_image, &mut region_image, Some(&options))
                .expect("resize of a well-formed crop box should not fail");

            out.push(region_image)
        }
        out
    }
}
