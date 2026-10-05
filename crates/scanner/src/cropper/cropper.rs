use crate::{scanner_state::ScannerState, setup::BASE_ICONS, timing::timed};
use hf_core::my_dbg;

impl ScannerState {
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_some());
        assert!(BASE_ICONS.read().len() != 0);

        self.update_scale();
        // my_dbg!("starting anchor");
        timed("anchors", || self.update_anchors());
        // my_dbg!("starting page");
        timed("pages", || self.update_page_status());
        // my_dbg!("starting slot");
        timed("slots", || self.update_slots());
        timed("tooltip", || self.update_tooltip());
        // self.downscaled_cache.reset();
    }
}
