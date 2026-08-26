use crate::scanner_state::ScannerState;

impl ScannerState {
    pub fn check_21_9(&mut self) {
        // TODO (lowkey just need to check 21:9, won't worry about having to find non-full screen i think)
        // check for black bars either on top or on the bottom
        self.screen_info.start_height = 0;
        self.screen_info.start_width = 0;
        self.screen_info.end_height = 1080;
        self.screen_info.end_width = 1920;

        self.screen_info.effective_width =
            self.screen_info.end_width - self.screen_info.start_width;
        self.screen_info.effective_height =
            self.screen_info.end_height - self.screen_info.start_height;
        self.screen_info.scale_factor = 1080.0 / 1440.0;
    }
}
