use crate::{
    buffer::Buffer,
    cropper::anchors::AnchorInfo,
    image_utils::common::IntegerRectangle,
    ocr_jobs::OcrJob,
    setup::{IncomingNewIcon, OneIconConfig},
    tooltip::{
        chest::Chest,
        hover::{Hover, PendingRead},
    },
};
use ahash::{AHashMap, AHashSet};
use fast_image_resize::Resizer;
use serde::{Deserialize, Serialize};

// game resolution and forced 21:9 are picked by the user, the capture size says nothing about them
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScreenInfo {
    pub game_width: u32,
    pub game_height: u32,
    #[serde(default)]
    pub forced_21_9: bool,

    #[serde(default)]
    pub effective_height: u32,
    #[serde(default)]
    pub scale_factor: f64,
    #[serde(default)]
    pub ui_origin: (f64, f64),

    #[serde(default)]
    pub brightness: Option<f64>,
    // how many anchors that estimate is the average of
    #[serde(default)]
    pub brightness_anchors: usize,
}

impl ScannerState {
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Tradability {
    Tradable,
    RosterBound,
    CharBound,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneSlotInfo {
    // pub currently_seen: bool,
    pub icon_name_score: Option<(String, f64)>,
    // it matched with the rows an item level is written over left out
    #[serde(default)]
    pub levelled: bool,
    // Other icons that passed nearly as well. Plain chests in other trims and rarities do, so a
    // chest slot is any of these.
    #[serde(default)]
    pub alternatives: Vec<String>,
    pub observed_number: OneIconConfig,
    pub processed_number: OneIconConfig,
    pub observed_icon: OneIconConfig,
    // the whole slot, number included, as last recognised, or as last seen while nothing ever was
    #[serde(skip)]
    pub display_icon: Option<OneIconConfig>,
    // a tooltip was resolved to this slot, and whether its amount failed to read
    #[serde(default)]
    pub hovered: bool,
    #[serde(default)]
    pub tooltip_failed: bool,
    // pub observed_id: Uuid,
    pub currently_seen: bool,
    pub amount: Option<String>,
    // these two come from the hover tooltip
    #[serde(default)]
    pub tooltip_amount: Option<String>,
    pub tradability: Option<Tradability>,
    // the material the tooltip said it is, for an icon several are drawn with
    #[serde(default)]
    pub label: Option<String>,
    // the OCR job whose text becomes the amount
    #[serde(skip)]
    pub amount_job: Option<u32>,
    // of the slot's raw pixels when it was last looked at
    #[serde(skip)]
    pub raw_hash: u64,
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum AnchorType {
    Storage,
    StorageRoster,
    StorageCharStorage,
    StorageInventory,
    CharInventory,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScannerState {
    #[serde(default)]
    pub debugging: bool,
    #[serde(default)]
    pub debug_info: AHashMap<String, (IntegerRectangle, f64, f64, Vec<OneIconConfig>)>,

    #[serde(default)]
    pub slot_infos: AHashMap<SlotAddress, OneSlotInfo>,
    #[serde(skip)]
    pub edits: AHashMap<SlotAddress, SlotEdit>,
    #[serde(default)]
    pub anchors: AHashMap<AnchorType, AnchorInfo>,

    #[serde(default)]
    pub page_num_infos: AHashMap<InventoryType, Vec<Option<bool>>>,
    // the page each window was last seen on, and for how many scans its tab has looked dark
    #[serde(skip)]
    pub last_pages: AHashMap<InventoryType, (usize, usize)>,

    #[serde(default)]
    pub hover: Option<Hover>,
    // chests are read off their tooltip alone, so they live apart from the slots
    #[serde(default)]
    pub chests: Vec<Chest>,

    #[serde(default)]
    pub screen_info: ScreenInfo,

    pub buffer: Buffer,

    #[serde(default)]
    pub config: Vec<OneIconConfig>, // this should be empty for cropper calls

    #[serde(default)]
    pub model: Option<Vec<u8>>,

    #[serde(default)]
    pub incoming_new_icons: Option<Vec<IncomingNewIcon>>,

    // OCR jobs not handed out yet, the id each strip got, and the texts that came back
    #[serde(skip)]
    pub ocr_queue: Vec<OcrJob>,
    #[serde(skip)]
    pub ocr_ids: AHashMap<u64, u32>,
    #[serde(skip)]
    pub ocr_texts: AHashMap<u32, String>,
    // tooltip frames waiting for their texts, and hovers that ended while some were still waiting
    #[serde(skip)]
    pub pending_reads: Vec<PendingRead>,
    #[serde(skip)]
    pub past_hovers: Vec<Hover>,
    #[serde(skip)]
    pub hover_count: u32,

    #[serde(skip)]
    pub scans: u64,
    // where the storage layout is, as one of STORAGE_SHIFTS
    #[serde(skip)]
    pub storage_shift: (f64, f64),
    // the last scanned frame, thinned out, and how many scans in a row had nothing to do
    #[serde(skip)]
    pub last_samples: Vec<[u8; 3]>,
    #[serde(skip)]
    pub quiet_scans: usize,
    #[serde(skip)]
    pub slots_left: bool,

    // written since the last result went to JS
    #[serde(skip)]
    pub changed_slots: AHashSet<SlotAddress>,
    #[serde(skip)]
    pub changed_debug: AHashSet<String>,
    #[serde(skip)]
    pub chests_changed: bool,

    #[serde(skip)]
    pub resizer: Option<Resizer>,
    // pub downscaled_cache: DownscaledCache,
}
