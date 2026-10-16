use crate::scanner_state::ScannerState;

const SAMPLE_GAP: usize = 8; // one pixel in this many, each way, is compared
const SAMPLE_DIFFERS: u8 = 16; // above what a still screen does in a compressed recording
const WORTH_SCAN_THRESH: f64 = 0.005;
const QUIET_SCANS: usize = 6; // scans after the last change, for the slots' turn and a lost tooltip

impl ScannerState {
    // A frame is scanned when enough of it differs from the last scanned one, and for a few
    // scans after. It also is while something is unfinished: the hover waiting for its texts (the
    // next read is cut when they are back) or just gone missing, slots left over.
    pub fn worth_scanning(&mut self) -> bool {
        let (data, width) = (self.buffer.data(), self.buffer.width);
        let samples: Vec<[u8; 3]> = (0..self.buffer.height)
            .step_by(SAMPLE_GAP)
            .flat_map(|y| {
                (0..width).step_by(SAMPLE_GAP).map(move |x| {
                    let index = (y * width + x) * 4;
                    [data[index], data[index + 1], data[index + 2]]
                })
            })
            .collect();
        let changed = samples.len() != self.last_samples.len()
            || diff_samples(&samples, &self.last_samples) > WORTH_SCAN_THRESH;
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

pub fn diff_samples(new: &[[u8; 3]], old: &[[u8; 3]]) -> f64 {
    let count = new
        .iter()
        .zip(old)
        .filter(|(new, old)| (0..3).any(|c| new[c].abs_diff(old[c]) > SAMPLE_DIFFERS))
        .count();
    count as f64 / new.len() as f64
}
