use crate::scanner_state::{ALL_ANCHOR_TYPES, ScannerState};

impl ScannerState {
    pub fn find_active_game_area(&mut self) {
        self.screen_info = None; // TODO
    }
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_none());

        if self.screen_info.is_none() {
            self.find_active_game_area();
        }

        for anchor_type in ALL_ANCHOR_TYPES{
            if (self.anchors.contains_key(&anchor_type)){
                
            }
        }
    }
}
