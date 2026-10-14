use crate::{dev_utils::timing::timed, interface::setup::BASE_ICONS, scanner_state::ScannerState};

impl ScannerState {
    pub fn cropper(&mut self) {
        assert!(self.buffer.pointer.is_some());
        assert!(BASE_ICONS.read().len() != 0);

        self.slot_cut = None;
        self.scanned = timed("changed", || self.worth_scanning());
        if !self.scanned {
            return;
        }
        self.update_scale();
        timed("anchors", || self.update_anchors());
        timed("pages", || self.update_page_status());
        timed("slots", || self.update_slots());
        timed("tooltip", || self.update_tooltip());
    }
}
