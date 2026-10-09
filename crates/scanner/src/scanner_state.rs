use crate::{
    buffer::Buffer,
    cropper::anchors::AnchorInfo,
    ocr_jobs::OcrJob,
    setup::OneIconConfig,
    tooltip::{
        chest::Chest,
        hover::{Hover, PendingRead},
    },
};
use ahash::{AHashMap, AHashSet};
use fast_image_resize::Resizer;
use serde::{Deserialize, Serialize};

// game resolution and forced 21:9 are picked by the user, the capture size says nothing about them
#[derive(Debug, Default)]
pub struct ScreenInfo {
    pub game_width: u32,
    pub game_height: u32,
    pub forced_21_9: bool,

    pub effective_height: u32,
    pub scale_factor: f64,
    pub ui_origin: (f64, f64),

    pub brightness: Option<f64>,
    // how many anchors that estimate is the average of
    pub brightness_anchors: usize,
}

impl ScannerState {
    // over a frame buffer that is already reserved; the config is set apart, in set_config
    pub fn new(buffer: Buffer, game_width: u32, game_height: u32, forced_21_9: bool) -> ScannerState {
        let screen_info = ScreenInfo { game_width, game_height, forced_21_9, ..Default::default() };
        let mut state = ScannerState { buffer, screen_info, ..Default::default() };
        state.initialize_anchors();
        state.initialize_page_num_infos();
        state
    }

    // the UI scales with the height of the 16:9 (or 21:9) area the game renders into
    pub fn update_scale(&mut self) {
        let info = &mut self.screen_info;
        // the game's 21:9 is really 64:27 (2560x1080)
        let ratio = if info.forced_21_9 {
            27.0 / 64.0
        } else {
            9.0 / 16.0
        };
        info.effective_height = (info.game_width as f64 * ratio)
            .min(info.game_height as f64)
            .round() as u32;
        info.scale_factor = info.effective_height as f64 / 1440.0;
        // the UI is laid out in a 16:9 block, assumed centered in the capture
        info.ui_origin = (
            (self.buffer.width as f64 - info.effective_height as f64 * 16.0 / 9.0) / 2.0,
            (self.buffer.height as f64 - info.effective_height as f64) / 2.0,
        );
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
    // it matched with the rows an item level is written over left out
    pub levelled: bool,
    // Other icons that passed nearly as well. Plain chests in other trims and rarities do, so a
    // chest slot is any of these.
    pub alternatives: Vec<String>,
    pub observed_number: OneIconConfig,
    pub observed_icon: OneIconConfig,
    // the whole slot, number included, as last recognised, or as last seen while nothing ever was
    pub display_icon: Option<OneIconConfig>,
    // a tooltip was resolved to this slot, and whether its amount failed to read
    pub hovered: bool,
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
    // of the slot's raw pixels when it was last looked at
    pub raw_hash: u64,
}

const AMOUNT_READS: usize = 5;

impl OneSlotInfo {
    // One read under the cursor or a highlight does not undo several that agreed. The newest wins
    // a tie.
    pub fn vote_amount(&mut self, text: String) {
        let digits = |text: &String| text.chars().filter(char::is_ascii_digit).collect::<String>();
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
#[derive(Debug, Deserialize)]
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

    pub screen_info: ScreenInfo,

    pub buffer: Buffer,

    // OCR jobs not handed out yet, the id each strip got, and the texts that came back
    pub ocr_queue: Vec<OcrJob>,
    pub ocr_ids: AHashMap<u64, u32>,
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
}
