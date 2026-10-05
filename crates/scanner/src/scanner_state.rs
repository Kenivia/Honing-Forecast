use crate::{
    buffer::Buffer,
    cropper::anchors::AnchorInfo,
    image_utils::common::IntegerRectangle,
    setup::{IncomingNewIcon, OneIconConfig},
    tooltip::{chest::Chest, hover::Hover},
};
use ahash::AHashMap;
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
    pub observed_number: OneIconConfig,
    pub processed_number: OneIconConfig,
    pub observed_icon: OneIconConfig,
    // pub observed_id: Uuid,
    pub currently_seen: bool,
    pub amount: Option<String>,
    // these two come from the hover tooltip
    #[serde(default)]
    pub tooltip_amount: Option<String>,
    pub tradability: Option<Tradability>,
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
    #[serde(default)]
    pub anchors: AHashMap<AnchorType, AnchorInfo>,

    #[serde(default)]
    pub page_num_infos: AHashMap<InventoryType, Vec<Option<bool>>>,

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

    #[serde(skip)]
    pub resizer: Option<Resizer>,
    // pub downscaled_cache: DownscaledCache,
}
