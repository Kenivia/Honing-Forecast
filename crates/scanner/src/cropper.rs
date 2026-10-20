use crate::{dev_utils::timing::timed, interface::setup::BASE_ICONS, scanner_state::ScannerState};

impl ScannerState {
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_some());
        assert!(BASE_ICONS.read().len() != 0);

        // nothing can be placed until the aspect is known
        self.scanned = self.detect_21_9() && timed("changed", || self.worth_scanning());
        if !self.scanned {
            return;
        }
        self.update_scale();
        timed("anchors", || self.update_anchors());
        timed("pages", || self.update_page_status());
        timed("slots", || self.update_slots());
        timed("tooltip", || self.update_tooltip());
        timed("currency", || self.update_currency());
    }
}
