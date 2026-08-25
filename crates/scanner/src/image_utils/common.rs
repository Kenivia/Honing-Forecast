use crate::{scanner_state::ScannerState, setup::OneIconConfig};
use fast_image_resize::{
    PixelType, Resizer,
    images::{Image, ImageRef},
};
use image::RgbaImage;

use serde::{Deserialize, Serialize};

pub trait Rectangle: Sized + Copy {
    type Unit: Copy + ToF64 + ToUsize;

    fn top_left(&self) -> (f64, f64);
    fn width(&self) -> Self::Unit;
    fn height(&self) -> Self::Unit;
    fn width_f64(&self) -> f64 {
        self.width().to_f64()
    }
    fn height_f64(&self) -> f64 {
        self.height().to_f64()
    }
    fn width_usize(&self) -> usize {
        self.width().to_usize()
    }
    fn height_usize(&self) -> usize {
        self.height().to_usize()
    }
    fn unit_to_f64(unit: Self::Unit) -> f64;
    fn unit_to_usize(unit: Self::Unit) -> usize;

    fn with_top_left(&self, top_left: (f64, f64)) -> Self;
    fn offset_from<R: Rectangle>(&self, other: &R) -> Self {
        let (ox, oy) = other.top_left();
        let (sx, sy) = self.top_left();
        self.with_top_left((sx - ox, sy - oy))
    }
    fn use_root<R: Rectangle>(&self, root: &R) -> Self {
        let (rx, ry) = root.top_left();
        let (sx, sy) = self.top_left();
        self.with_top_left((sx + rx, sy + ry))
    }

    fn to_rounded(&self) -> IntegerRectangle {
        IntegerRectangle {
            top_left: self.top_left(),
            width: Self::unit_to_usize(self.width()),
            height: Self::unit_to_usize(self.height()),
        }
    }

    fn to_float(&self) -> FloatRectangle {
        FloatRectangle {
            top_left: self.top_left(),
            width: Self::unit_to_f64(self.width()),
            height: Self::unit_to_f64(self.height()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct FloatRectangle {
    pub top_left: (f64, f64),
    pub width: f64,
    pub height: f64,
}

impl Rectangle for FloatRectangle {
    type Unit = f64;

    fn top_left(&self) -> (f64, f64) {
        self.top_left
    }
    fn width(&self) -> f64 {
        self.width
    }
    fn height(&self) -> f64 {
        self.height
    }
    fn with_top_left(&self, top_left: (f64, f64)) -> Self {
        Self { top_left, ..*self }
    }
    fn unit_to_f64(unit: f64) -> f64 {
        unit
    }
    fn unit_to_usize(unit: f64) -> usize {
        unit.round() as usize
    }
}

impl FloatRectangle {
    pub fn scaled(&self, scale: f64) -> FloatRectangle {
        Self {
            top_left: (self.top_left.0 * scale, self.top_left.1 * scale),
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct IntegerRectangle {
    pub top_left: (f64, f64),
    pub width: usize,
    pub height: usize,
}

impl Rectangle for IntegerRectangle {
    type Unit = usize;

    fn top_left(&self) -> (f64, f64) {
        self.top_left
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn with_top_left(&self, top_left: (f64, f64)) -> Self {
        Self { top_left, ..*self }
    }
    fn unit_to_f64(unit: usize) -> f64 {
        unit as f64
    }
    fn unit_to_usize(unit: usize) -> usize {
        unit
    }
}

pub trait ToF64: Copy {
    fn to_f64(self) -> f64;
}

impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

impl ToF64 for usize {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

pub trait ToUsize: Copy {
    fn to_usize(self) -> usize;
}

impl ToUsize for f64 {
    fn to_usize(self) -> usize {
        self.round() as usize
    }
}

impl ToUsize for usize {
    fn to_usize(self) -> usize {
        self
    }
}

pub const FULL_RECT_16_9: FloatRectangle = FloatRectangle {
    top_left: (0.0, 0.0),
    width: 1920.0,
    height: 1080.0,
};

pub fn config_to_rgba(template: &OneIconConfig) -> RgbaImage {
    let (template_w, template_h) = (template.offset.width as u32, template.offset.height as u32);
    RgbaImage::from_raw(template_w, template_h, template.data.clone()).unwrap()
}

pub fn image_to_rgba(observed: Image) -> RgbaImage {
    let (observed_w, observed_h) = (observed.width(), observed.height());
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
}

pub fn get_resizer(resizer: &mut Option<Resizer>) -> &mut Resizer {
    resizer.get_or_insert(Resizer::new())
}
