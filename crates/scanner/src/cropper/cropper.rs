use crate::{scanner_state::ScannerState, setup::BASE_ICONS};
use hf_core::my_dbg;

impl ScannerState {
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_some());
        assert!(BASE_ICONS.read().len() != 0);

        self.update_scale();
        // my_dbg!("starting anchor");
        self.update_anchors();
        // my_dbg!("starting page");
        self.update_page_status();
        // my_dbg!("starting slot");
        self.update_slots();
        self.update_tooltip();
        // self.downscaled_cache.reset();
    }
}
