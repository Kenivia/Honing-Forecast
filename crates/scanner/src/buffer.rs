use std::mem::forget;

use image::RgbaImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct Buffer {
    #[serde(default)]
    pub pointer: Option<usize>,
    pub width: usize,
    pub height: usize,
    pub size: usize,
    // raw channel value to its brightness-normalised value, rebuilt from the brightness estimate each scan
    #[serde(skip)]
    pub lut: Option<[u8; 256]>,
    // the estimate that table is of
    #[serde(skip)]
    pub brightness: f64,
}
impl Buffer {
    pub fn reserve(&mut self) {
        assert!(self.pointer.is_none());
        let mut buf: Vec<u8> = Vec::with_capacity(self.size);
        let ptr: *mut u8 = buf.as_mut_ptr();
        forget(buf);
        self.pointer = Some(ptr as usize);
    }

    pub fn dealloc(&mut self) {
        assert!(self.pointer.is_some());
        unsafe {
            let _ = Vec::from_raw_parts(self.pointer.unwrap() as *mut u8, self.size, self.size);
            // dropped here i think
            self.pointer = None;
            self.size = 0;
        }
    }

    pub fn data(&self) -> &'static [u8] {
        unsafe { std::slice::from_raw_parts(self.pointer.unwrap() as *const u8, self.size) }
    }

    // brightness-normalised colour of one pixel
    #[inline(always)]
    pub fn rgb(&self, x: usize, y: usize) -> [i32; 3] {
        let (data, lut) = (self.data(), self.lut.as_ref().unwrap());
        let index = (y * self.width + x) * 4;
        std::array::from_fn(|c| lut[data[index + c] as usize] as i32)
    }

    // colour of one pixel as captured
    #[inline(always)]
    pub fn raw(&self, x: usize, y: usize) -> [i32; 3] {
        let index = (y * self.width + x) * 4;
        std::array::from_fn(|c| self.data()[index + c] as i32)
    }

    // of the raw pixels of a rectangle, to tell cheaply that nothing in it changed
    pub fn hash(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> u64 {
        let data = self.data();
        let (x0, x1) = (x0.min(self.width), x1.min(self.width));
        let rows: Vec<&[u8]> = (y0..y1.min(self.height))
            .map(|y| &data[(y * self.width + x0) * 4..(y * self.width + x1) * 4])
            .collect();
        ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one(rows)
    }

    // copy of a rectangle as captured
    pub fn raw_crop(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> RgbaImage {
        let data = self.data();
        let rows = (y0..y1).flat_map(|y| &data[(y * self.width + x0) * 4..(y * self.width + x1) * 4]);
        RgbaImage::from_raw((x1 - x0) as u32, (y1 - y0) as u32, rows.copied().collect()).unwrap()
    }

    // brightness-normalised copy of a rectangle
    pub fn crop(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> RgbaImage {
        RgbaImage::from_fn((x1 - x0) as u32, (y1 - y0) as u32, |x, y| {
            let [r, g, b] = self.rgb(x0 + x as usize, y0 + y as usize);
            image::Rgba([r as u8, g as u8, b as u8, 255])
        })
    }
}
