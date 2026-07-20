use crate::{
    buffer::Buffer,
    setup::{IncomingNewIcon, OneIconSetup},
};
use ahash::AHashMap;
use either::Either;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScreenInfo {
    pub total_width: i64,
    #[serde(default)]
    pub start_width: i64,
    #[serde(default)]
    pub end_width: i64,
    #[serde(default)]
    pub effective_width: i64,

    pub total_height: i64,
    #[serde(default)]
    pub start_height: i64,
    #[serde(default)]
    pub end_height: i64,
    #[serde(default)]
    pub effective_height: i64,

    #[serde(default)]
    pub is_21_9: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ScaledPosition {
    pub top_left: (usize, usize),
    pub bot_right: (usize, usize),
}

impl ScaledPosition {
    pub fn width(&self) -> usize {
        self.bot_right.0 - self.top_left.0
    }
    pub fn height(&self) -> usize {
        self.bot_right.1 - self.top_left.1
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneSlotInfo {
    pub currently_seen: bool,
    pub icon_name: String,
    pub icon_id: usize,
    pub position: ScaledPosition,
    pub amount: Either<usize, OneSlotProgress>,
    pub tradability: Either<InventoryType, OneSlotProgress>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum OneSlotProgress {
    OCRing,
    NeedHover,
    HoverOCRing,
}

pub const ALL_ANCHOR_TYPES: [InventoryType; 3] = [
    InventoryType::Roster,
    InventoryType::CharStorage,
    InventoryType::CharInventory,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct OneAnchorInfo {
    pub variant: usize,
    pub position: ScaledPosition,
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OCRJob {
    pub cropped: Vec<u8>,
    pub slot_address: SlotAddress,
    // TODO there should be something here to say which part of the tooltip this is OCring
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerState {
    #[serde(default)]
    pub slot_infos: AHashMap<SlotAddress, OneSlotInfo>,
    #[serde(default)]
    pub anchors: AHashMap<InventoryType, OneAnchorInfo>,
    #[serde(default)]
    pub screen_info: ScreenInfo,
    #[serde(default)]
    pub pending_jobs: Vec<OCRJob>,

    pub buffer: Buffer,

    #[serde(default)]
    pub config: Vec<OneIconSetup>,

    #[serde(default)]
    pub incoming_new_icon: Option<IncomingNewIcon>,
}
