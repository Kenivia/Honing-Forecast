use fast_image_resize::Resizer;

// what a rectangle's numbers are in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    // 1440p UI units
    Ui,
    // screen pixels at this UI height
    Screen(u32),
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub top_left: (f64, f64),
    pub width: f64,
    pub height: f64,
    pub space: Space,
}

impl Rect {
    pub const fn ui(top_left: (f64, f64), width: f64, height: f64) -> Rect {
        Rect { top_left, width, height, space: Space::Ui }
    }

    pub const fn screen(top_left: (f64, f64), width: f64, height: f64, ui_height: u32) -> Rect {
        Rect { top_left, width, height, space: Space::Screen(ui_height) }
    }

    pub fn scaled(&self, ui_height: u32) -> Rect {
        assert_eq!(self.space, Space::Ui);
        let scale = ui_height as f64 / 1440.0;
        Rect {
            top_left: (self.top_left.0 * scale, self.top_left.1 * scale),
            width: self.width * scale,
            height: self.height * scale,
            space: Space::Screen(ui_height),
        }
    }

    pub fn with_top_left(&self, top_left: (f64, f64)) -> Rect {
        Rect { top_left, ..*self }
    }

    // by a point of the same space, which a point cannot be asked
    pub fn shifted(&self, by: (f64, f64)) -> Rect {
        self.with_top_left((self.top_left.0 + by.0, self.top_left.1 + by.1))
    }

    pub fn get_offset(&self, other: &Rect) -> (f64, f64) {
        assert_eq!(self.space, other.space);
        (self.top_left.0 - other.top_left.0, self.top_left.1 - other.top_left.1)
    }

    // the only place a size is rounded
    pub fn pixel_size(&self) -> (u32, u32) {
        (self.width.round() as u32, self.height.round() as u32)
    }

    // the same place, at exactly its pixel size
    pub fn whole_pixels(&self) -> Rect {
        let (width, height) = self.pixel_size();
        Rect { width: width as f64, height: height as f64, ..*self }
    }
}

pub fn get_resizer(resizer: &mut Option<Resizer>) -> &mut Resizer {
    resizer.get_or_insert(Resizer::new())
}
