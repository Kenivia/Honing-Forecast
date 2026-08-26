use crate::{scanner_state::ScannerState, setup::BASE_ICONS};
use hf_core::my_dbg;

impl ScannerState {
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_some());
        assert!(BASE_ICONS.read().len() != 0);

        if !self.screen_info.initialized {
            self.check_21_9();
        }
        // my_dbg!("starting anchor");
        self.update_anchors();
        // my_dbg!("starting page");
        self.update_page_status();
        // my_dbg!("starting slot");
        self.update_slots();
        // TODO hover tooltip detection
        // self.downscaled_cache.reset();
    }
}
