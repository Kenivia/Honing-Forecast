use crate::buffer::Buffer;
use ahash::AHashMap;
use either::Either;
use serde::{Deserialize, Serialize};

pub type ImageHash = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenInfo {
    pub total_width: i64,
    pub start_width: i64,
    pub end_width: i64,
    pub effective_width: i64,

    pub total_height: i64,
    pub start_height: i64,
    pub end_height: i64,
    pub effective_height: i64,

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
    pub page_num: i64,
    pub pos_in_inv: (i64, i64),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ScaledPosition {
    pub top_left: i64,
    pub top_right: i64,
    pub bot_left: i64,
    pub bot_right: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OneSlotInfo {
    pub hash: u64,
    pub currently_seen: bool,
    pub icon_name: String,
    pub icon_index: usize,
    pub position: ScaledPosition,
    pub amount: Either<i64, OneSlotProgress>,
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
    pub hash: ImageHash,
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
    pub screen_info: Option<ScreenInfo>,
    #[serde(default)]
    pub pending_jobs: Vec<OCRJob>,

    pub buffer: Buffer,
}
