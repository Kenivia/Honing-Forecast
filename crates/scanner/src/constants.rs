pub const SAMPLE_GAP: usize = 8; // one pixel in this many, each way, is compared
pub const SAMPLE_DIFFERS: u8 = 16; // above what a still screen does in a compressed recording
pub const WORTH_SCAN_THRESH: f64 = 0.005;
pub const QUIET_SCANS: usize = 6; // scans after the last change, for the slots' turn and a lost tooltip

// Share of the thinned frame that has to differ from the one last searched as a whole for the lone
// inventory to be searched for again at once. Its window showing up is 13 to 15% of a 3440x1440
// frame; the cursor and a tooltip are under 3%.
pub const CHAR_INV_SEARCH_THRESH: f64 = 0.03;
// otherwise one of its variants is searched for on every this many scans
pub const CHAR_INV_FORCED_SEARCH_FREQ: u64 = 8;

pub const STORAGE_ANCHOR_SLACK: f64 = 2.0;
pub const WINDOW_ANCHOR_SLACK: f64 = 6.0;
// The storage layout is three windows and the Storage button, always in the same place: one for
// the pet menu, and shifted by this for the storage NPC.
pub const STORAGE_NPC_SHIFT: [(f64, f64); 2] = [(0.0, 0.0), (-6.4, -142.45)];
pub const STORAGE_MIN: usize = 3;

pub const PAGE_NUM_LEAD: f64 = 0.1; // an active tab leads by 0.2, one under the cursor by 0.05
pub const DARK_SCANS: usize = 3; // scans a tab has to look dark for before its page is given up
