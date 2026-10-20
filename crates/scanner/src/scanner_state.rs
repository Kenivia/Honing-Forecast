use crate::{
    anchor_to_slot::anchors::AnchorInfo,
    buffer::Buffer,
    currency::Currencies,
    interface::setup::OneIconConfig,
    ocr::jobs::OcrJob,
    tooltip::{chest::Chest, hover::Hover, ocr_interface::PendingRead},
};
use ahash::{AHashMap, AHashSet};
use fast_image_resize::Resizer;
use serde::{Deserialize, Serialize};

// A setting the scanner works out by itself, unless the user gave it.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Detected<T> {
    pub value: Option<T>,
    pub manual: bool,
}

impl<T> Detected<T> {
    // what the user gave, without what was detected since
    pub fn setting(self) -> Self {
        Detected {
            value: self.value.filter(|_| self.manual),
            manual: self.manual,
        }
    }
}

const BLACK: u8 = 10; // highest channel of a letterbox bar, with compression noise
const BAR_SHARE: f64 = 0.95; // of a band's samples, leaving room for the cursor
const LIT_SHARE: f64 = 0.2; // of the picture's samples above black, for the frame to count
const SAMPLE_GAP: usize = 4;

// The capture is the whole screen and the game fills it, so the capture size is the game's.
#[derive(Debug, Default)]
pub struct ScreenInfo {
    pub forced_21_9: Detected<bool>,

    pub effective_height: u32,
    pub scale_factor: f64,
    pub ui_origin: (f64, f64),
    // top left of the area the game renders into, which the HUD and some windows hug
    pub corner: (f64, f64),
    // in pixels
    pub render_width: f64,

    pub brightness: Detected<f64>,
    // how many anchors that estimate is the average of
    pub brightness_anchors: usize,
}

impl ScannerState {
    // over a frame buffer that is already reserved; the config is set apart, in set_config
    pub fn new(
        buffer: Buffer,
        forced_21_9: Detected<bool>,
        brightness: Detected<f64>,
    ) -> ScannerState {
        let screen_info = ScreenInfo {
            forced_21_9,
            brightness,
            ..Default::default()
        };
        let mut state = ScannerState {
            buffer,
            screen_info,
            ..Default::default()
        };
        state.initialize_anchors();
        state.initialize_page_num_infos();
        state
    }

    // Forced 21:9 leaves black bars where a 16:9 picture would go on. Decided on the first frame
    // that is not black (the game fades in) and not looked at again. False while undecided.
    pub fn detect_21_9(&mut self) -> bool {
        if self.screen_info.forced_21_9.value.is_some() {
            return true;
        }
        let (game_width, game_height) = (self.buffer.width as f64, self.buffer.height as f64);
        let decided = if game_width / game_height > 2.1 {
            true
        } else {
            let edge = |ratio: f64| {
                let picture = (game_width * ratio).min(game_height);
                ((game_height - picture) / 2.0).round() as usize
            };
            let (top_16_9, top_21_9) = (edge(9.0 / 16.0), edge(27.0 / 64.0));
            let (data, width, height) = (self.buffer.data(), self.buffer.width, self.buffer.height);
            let dark_share = |rows: std::ops::Range<usize>| {
                let samples: Vec<bool> = rows
                    .step_by(SAMPLE_GAP)
                    .flat_map(|y| {
                        (0..width).step_by(SAMPLE_GAP).map(move |x| {
                            let index = (y * width + x) * 4;
                            data[index..index + 3].iter().all(|&c| c <= BLACK)
                        })
                    })
                    .collect();
                samples.iter().filter(|&&dark| dark).count() as f64 / samples.len().max(1) as f64
            };
            if 1.0 - dark_share(top_21_9..height - top_21_9) <= LIT_SHARE {
                return false;
            }
            top_21_9 > top_16_9
                && dark_share(top_16_9..top_21_9) >= BAR_SHARE
                && dark_share(height - top_21_9..height - top_16_9) >= BAR_SHARE
        };
        self.screen_info.forced_21_9.value = Some(decided);
        true
    }

    // the UI scales with the height of the 16:9 (or 21:9) area the game renders into
    pub fn update_scale(&mut self) {
        let info = &mut self.screen_info;
        let (width, height) = (self.buffer.width as f64, self.buffer.height as f64);
        let forced_21_9 = info.forced_21_9.value.unwrap();
        // the game's 21:9 is really 64:27 (2560x1080)
        let ratio = if forced_21_9 { 27.0 / 64.0 } else { 9.0 / 16.0 };
        info.effective_height = (width * ratio).min(height).round() as u32;
        info.scale_factor = info.effective_height as f64 / 1440.0;
        // the UI is laid out in a 16:9 block, centred in the capture
        info.ui_origin = (
            (width - info.effective_height as f64 * 16.0 / 9.0) / 2.0,
            (height - info.effective_height as f64) / 2.0,
        );
        // full width: a native ultrawide (3440x1440, wider than 64:27) is drawn edge to edge
        info.render_width = width;
        info.corner = (0.0, info.ui_origin.1);
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum InventoryType {
    Roster,
    CharStorage,
    CharInventory,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SlotAddress {
    pub inventory_type: InventoryType,
    pub page_num: usize,
    pub pos_in_inv: (usize, usize),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Tradability {
    Tradable,
    RosterBound,
    CharBound,
}

#[derive(Debug)]
pub struct OneSlotInfo {
    pub icon_name_score: Option<(String, f64)>,
    // what is drawn over it, as bits of slots::OVERLAYS
    pub overlays: u8,
    // Other icons that passed nearly as well. Plain chests in other trims and rarities do, so a
    // chest slot is any of these.
    pub alternatives: Vec<String>,
    pub observed_number: OneIconConfig,
    pub observed_icon: OneIconConfig,
    // the whole slot, number included, as last recognised, or as last seen while nothing ever was
    pub display_icon: Option<OneIconConfig>,
    // a tooltip was resolved to this slot, and whether its amount failed to read
    pub hovered: bool,
    // the icons that tooltip's item is drawn with, and the slots it could have sat against as
    // well, which showed none of them then
    pub passed_over: (Vec<String>, Vec<SlotAddress>),
    pub tooltip_failed: bool,
    pub amount: Option<String>,
    // the last few reads of the number; the amount is the one most of them agree on
    pub amount_reads: Vec<String>,
    // these two come from the hover tooltip
    pub tooltip_amount: Option<String>,
    pub tradability: Option<Tradability>,
    // the material the tooltip said it is, for an icon several are drawn with
    pub label: Option<String>,
    // the OCR job whose text becomes the amount
    pub amount_job: Option<u32>,
    // the best it ever scored as its icon
    pub best_score: f64,
    // of the slot's raw pixels when it was last looked at
    pub raw_hash: u64,
}

const AMOUNT_READS: usize = 5;

impl OneSlotInfo {
    // One read under the cursor or a highlight does not undo several that agreed. The newest wins
    // a tie.
    pub fn vote_amount(&mut self, text: String) {
        let digits = |text: &String| {
            text.chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
        };
        self.amount_reads.push(text);
        if self.amount_reads.len() > AMOUNT_READS {
            self.amount_reads.remove(0);
        }
        let reads = &self.amount_reads;
        let votes = |read: &String| reads.iter().filter(|x| digits(x) == digits(read)).count();
        let most = reads.iter().map(votes).max().unwrap();
        self.amount = reads.iter().rev().find(|x| votes(x) == most).cloned();
    }
}

// What the user typed in for a slot, which the scanner then leaves alone. A retry instead forgets
// the slot, edit and all, so it is read again.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotEdit {
    pub address: SlotAddress,
    #[serde(default)]
    pub retry: bool,
    pub item: Option<String>,
    pub amount: Option<u32>,
    pub tradability: Option<Tradability>,
}

impl ScannerState {
    pub fn apply_edits(&mut self, edits: Vec<SlotEdit>) {
        for edit in edits {
            if edit.retry {
                self.edits.remove(&edit.address);
                self.slot_infos.remove(&edit.address);
                // a still screen is not scanned otherwise
                self.slots_left = true;
            } else {
                self.edits.insert(edit.address, edit);
            }
        }
    }
}

// a piece of fixed UI that locates inventories, or other anchors
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum AnchorType {
    Storage,
    StorageRoster,
    StorageCharStorage,
    StorageInventory,
    CharInventory,
    // the currency breakdown window, in its default place
    Currency,
}

impl AnchorType {
    pub fn in_storage(self) -> bool {
        !matches!(self, AnchorType::CharInventory | AnchorType::Currency)
    }
}

// Everything the scanner remembers between frames. It never leaves Rust: see scan_result.rs.
#[derive(Debug, Default)]
pub struct ScannerState {
    pub slot_infos: AHashMap<SlotAddress, OneSlotInfo>,
    pub edits: AHashMap<SlotAddress, SlotEdit>,
    pub anchors: AHashMap<AnchorType, AnchorInfo>,

    pub page_num_infos: AHashMap<InventoryType, Vec<Option<bool>>>,
    // the page each window was last seen on, and for how many scans its tab has looked dark
    pub last_pages: AHashMap<InventoryType, (usize, usize)>,

    pub hover: Option<Hover>,
    // chests are read off their tooltip alone, so they live apart from the slots
    pub chests: Vec<Chest>,
    // silver, gold and shards, apart from the slots: see currency.rs
    pub currency: Currencies,

    pub screen_info: ScreenInfo,

    pub buffer: Buffer,

    // OCR jobs not handed out yet, the id each strip got, and the texts that came back
    pub ocr_queue: Vec<OcrJob>,
    pub ocr_ids: AHashMap<u64, u32>,
    pub ocr_priority: AHashMap<u32, u8>,
    pub read_count: u32,
    pub ocr_texts: AHashMap<u32, String>,
    // tooltip frames waiting for their texts, and hovers that ended while some were still waiting
    pub pending_reads: Vec<PendingRead>,
    pub past_hovers: Vec<Hover>,
    pub hover_count: u32,

    pub scans: u64,
    // where the storage layout is, as one of STORAGE_SHIFTS
    pub storage_shift: (f64, f64),
    // the last scanned frame, thinned out, and how many scans in a row had nothing to do
    pub last_samples: Vec<[u8; 3]>,
    pub quiet_scans: usize,
    // the frame the lone inventory was last searched for in as a whole, thinned out, and the
    // scans since that had no such search
    pub search_samples: Vec<[u8; 3]>,
    pub search_ticks: u64,
    pub slots_left: bool,
    // the native harness reads every slot at once, so that its runs repeat
    pub no_slot_budget: bool,

    // written since the last result went to JS
    pub changed_slots: AHashSet<SlotAddress>,
    pub chests_changed: bool,

    pub resizer: Option<Resizer>,

    // whether the last frame was scanned
    pub scanned: bool,
    // debug only: every hover as it ended, once its last texts were in
    pub hover_log: Option<Vec<Hover>>,
}
