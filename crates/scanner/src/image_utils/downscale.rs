use crate::buffer::Buffer;

use crate::scanner_state::{ScaledPosition, ScannerState};
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownscaledCache {
    pub buffer: Buffer,
    pub position: ScaledPosition,
    pub occupied_size: Option<usize>,
    pub written_this_cycle: bool,
}

impl ScannerState {
    fn contains(&self, pos: &ScaledPosition) -> bool {
        let outer_left: i64 = self.downscaled_cache.position.top_left.0 as i64;
        let outer_top: i64 = self.downscaled_cache.position.top_left.1 as i64;
        let outer_right: i64 = outer_left + self.downscaled_cache.position.width as i64;
        let outer_bottom: i64 = outer_top + self.downscaled_cache.position.height as i64;

        let inner_left: i64 = pos.top_left.0 as i64;
        let inner_top: i64 = pos.top_left.1 as i64;
        let inner_right: i64 = inner_left + pos.width as i64;
        let inner_bottom: i64 = inner_top + pos.height as i64;

        inner_left >= outer_left
            && inner_top >= outer_top
            && inner_right <= outer_right
            && inner_bottom <= outer_bottom
    }
    fn actual_downscale(
        &self,
        src_image: &ImageRef<'_>,
        dst_image: &mut Image<'_>,
        pos: &ScaledPosition,
    ) {
        let (scale_x, scale_y) = self.scale_factors();
        let dst_w: u32 = pos.width as u32;
        let dst_h: u32 = pos.height as u32;

        let src_left: f64 = pos.top_left.0 as f64 * scale_x;
        let src_top: f64 = pos.top_left.1 as f64 * scale_y;
        let src_width: f64 = dst_w as f64 * scale_x;
        let src_height: f64 = dst_h as f64 * scale_y;

        let mut resizer: Resizer = Resizer::new();

        let options: ResizeOptions = ResizeOptions::new()
            .resize_alg(ResizeAlg::Convolution(FilterType::Box))
            .crop(src_left, src_top, src_width, src_height);

        resizer
            .resize(src_image, dst_image, Some(&options))
            .unwrap();
    }

    fn crop_only(&self, pos: &ScaledPosition, dst: &mut Image<'_>) {
        let left: u32 =
            (pos.top_left.0 as i64 - self.downscaled_cache.position.top_left.0 as i64) as u32;
        let top: u32 =
            (pos.top_left.1 as i64 - self.downscaled_cache.position.top_left.1 as i64) as u32;

        let dst_w: usize = dst.width() as usize;
        let dst_h: usize = dst.height() as usize;
        let src_stride: usize = self.downscaled_cache.position.width as usize * 4;
        let dst_stride: usize = dst_w * 4;

        let src_buffer: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(
                self.downscaled_cache.buffer.pointer.unwrap() as *mut u8,
                self.downscaled_cache.buffer.size,
            )
        };
        let dst_buffer = dst.buffer_mut();

        for row in 0..dst_h {
            let src_row_start: usize = (top as usize + row) * src_stride + left as usize * 4;
            let dst_row_start: usize = row * dst_stride;

            dst_buffer[dst_row_start..dst_row_start + dst_stride]
                .copy_from_slice(&src_buffer[src_row_start..src_row_start + dst_stride]);
            if row == dst_h - 1 {
                assert!(src_row_start + dst_stride < self.downscaled_cache.occupied_size.unwrap())
            }
        }
    }

    pub fn write_downscaled_cache(&mut self, position: ScaledPosition) {
        assert!(!self.downscaled_cache.written_this_cycle);

        let src_image: ImageRef<'_> = self.src_image();
        let mut dst_image: Image<'_> = Image::new(
            position.width as u32,
            position.height as u32,
            PixelType::U8x4,
        );

        self.actual_downscale(&src_image, &mut dst_image, &position);

        unsafe {
            let dest_slice: &mut [u8] = std::slice::from_raw_parts_mut(
                self.downscaled_cache.buffer.pointer.unwrap() as *mut u8,
                self.downscaled_cache.buffer.size,
            );
            dest_slice[..dst_image.buffer().len()].copy_from_slice(dst_image.buffer());
        };
        self.downscaled_cache.position = position;
        self.downscaled_cache.written_this_cycle = true;
        self.downscaled_cache.occupied_size = Some(position.width * position.height * 4);
    }

    pub fn downscale(&self, positions: &[ScaledPosition]) -> Vec<Image<'_>> {
        let src_image: ImageRef<'_> = self.src_image(); // pre sure initiailizing this is cheap enough so i won't bother skipping it potentially
        let mut out: Vec<Image<'_>> = Vec::with_capacity(positions.len());

        for pos in positions {
            let mut dst_image: Image<'_> =
                Image::new(pos.width as u32, pos.height as u32, PixelType::U8x4);

            if self.downscaled_cache.written_this_cycle && self.contains(pos) {
                self.crop_only(pos, &mut dst_image);
                out.push(dst_image);
                continue;
            }

            self.actual_downscale(&src_image, &mut dst_image, pos);
            out.push(dst_image);
        }

        out
    }
}
