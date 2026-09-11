use crate::{
    buffer::Buffer,
    cropper::anchors::AnchorInfo,
    image_utils::common::IntegerRectangle,
    setup::{IncomingNewIcon, OneIconConfig},
};
use ahash::AHashMap;
use fast_image_resize::Resizer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScreenInfo {
    pub total_width: u32,
    #[serde(default)]
    pub start_width: u32,
    #[serde(default)]
    pub end_width: u32,
    #[serde(default)]
    pub effective_width: u32,

    pub total_height: u32,
    #[serde(default)]
    pub start_height: u32,
    #[serde(default)]
    pub end_height: u32,
    #[serde(default)]
    pub effective_height: u32,

    #[serde(default)]
    pub is_21_9: bool,

    #[serde(default)]
    pub initialized: bool,

    #[serde(default)]
    pub brightness: Option<f64>,
    #[serde(default)]
    pub scale_factor: f64,
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

#[derive(Debug, Serialize, Deserialize)]
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
    pub tradability: Option<Tradability>,
}

pub const ALL_ANCHOR_TYPES: [InventoryType; 3] = [
    InventoryType::Roster,
    InventoryType::CharStorage,
    InventoryType::CharInventory,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerState {
    #[serde(default)]
    pub debugging: bool,
    #[serde(default)]
    pub debug_info: AHashMap<String, (IntegerRectangle, f64, f64, Vec<OneIconConfig>)>,

    #[serde(default)]
    pub slot_infos: AHashMap<SlotAddress, OneSlotInfo>,
    #[serde(default)]
    pub anchors: AHashMap<InventoryType, AnchorInfo>,

    #[serde(default)]
    pub page_num_infos: AHashMap<InventoryType, Vec<Option<bool>>>,

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
