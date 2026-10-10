use crate::{scanner_state::ScannerState, setup::BASE_ICONS, timing::timed};

const SAMPLE_EVERY: usize = 8; // one pixel in this many, each way, is compared
const SAMPLE_DIFFERS: u8 = 16; // above what a still screen does in a compressed recording
// share of the samples that has to differ from the last scanned frame: a tooltip showing up or
// moving on, not the cursor alone
const CHANGED: f64 = 0.005;
const QUIET_SCANS: usize = 6; // scans after the last change, for the slots' turn and a lost tooltip

// share of the thinned frame's samples that differ between two frames
pub fn differing(new: &[[u8; 3]], old: &[[u8; 3]]) -> f64 {
    let count = new
        .iter()
        .zip(old)
        .filter(|(new, old)| (0..3).any(|c| new[c].abs_diff(old[c]) > SAMPLE_DIFFERS))
        .count();
    count as f64 / new.len() as f64
}

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

    // A frame is scanned when enough of it differs from the last scanned one, and for a few
    // scans after. It also is while something is unfinished: the hover waiting for its texts (the
    // next read is cut when they are back) or just gone missing, slots left over.
    pub fn worth_scanning(&mut self) -> bool {
        let (data, width) = (self.buffer.data(), self.buffer.width);
        let samples: Vec<[u8; 3]> = (0..self.buffer.height)
            .step_by(SAMPLE_EVERY)
            .flat_map(|y| {
                (0..width).step_by(SAMPLE_EVERY).map(move |x| {
                    let index = (y * width + x) * 4;
                    [data[index], data[index + 1], data[index + 2]]
                })
            })
            .collect();
        let changed = samples.len() != self.last_samples.len()
            || differing(&samples, &self.last_samples) > CHANGED;
        let unfinished = self.slots_left
            || self
                .hover
                .as_ref()
                .is_some_and(|hover| hover.missed > 0 || hover.waiting != [0; 4]);
        self.quiet_scans = if changed || unfinished {
            0
        } else {
            self.quiet_scans + 1
        };
        if self.quiet_scans > QUIET_SCANS {
            return false;
        }
        self.last_samples = samples;
        true
    }
}
