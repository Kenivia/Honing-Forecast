// Screenshots of the game's client area.
use crate::game::Game;
use image::RgbaImage;
use xcap::Monitor;

impl Game {
    /// The whole client area, or a crop of it in client pixels.
    pub fn shot(&mut self, crop: Option<(u32, u32, u32, u32)>) -> RgbaImage {
        self.focus();
        let mon = Monitor::from_point(self.x + self.w / 2, self.y + self.h / 2).unwrap();
        let (ox, oy) = (
            (self.x - mon.x().unwrap()) as u32,
            (self.y - mon.y().unwrap()) as u32,
        );
        let (x, y, w, h) = crop.unwrap_or((0, 0, self.w as u32, self.h as u32));
        mon.capture_region(ox + x, oy + y, w, h).unwrap()
    }
}
