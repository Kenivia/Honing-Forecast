use crate::{scanner_state::ScannerState, setup::CONFIG};

impl ScannerState {
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_some());
        assert!(CONFIG.get().unwrap().len() != 0);

        if !self.screen_info.initialized {
            self.check_21_9();
        }

        self.update_anchors();

        self.update_page_status();

        self.update_slots();
        // TODO hover tooltip detection
        self.downscaled_cache.reset();
    }
}
