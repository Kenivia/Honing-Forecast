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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InventoryType {
    Roster,
    CharStorage,
    CharInventory,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SlotPosition {
    pub inventory_type: InventoryType,
    pub page_num: i64,
    pub pos_in_inv: (i64, i64),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixelPosition {
    pub top_left: i64,
    pub top_right: i64,
    pub bot_left: i64,
    pub bot_right: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OneSlotInfo {
    pub last_seen_hash: u64,
    pub icon_name: String,
    pub pixel_position: PixelPosition,
    pub amount: Either<i64, OneSlotProgress>,
    pub tradability: Either<InventoryType, OneSlotProgress>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum OneSlotProgress {
    OCRing,
    NeedHover,
    HoverOCRing,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnchorType {
    Roster,
    CharStorage,
    CharInventory,
}
pub const ALL_ANCHOR_TYPES: [AnchorType; 3] = [
    AnchorType::Roster,
    AnchorType::CharStorage,
    AnchorType::CharInventory,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct OCRJob {
    pub cropped: Vec<u8>,
    pub slot_position: SlotPosition,
    // TODO there should be something here to say which part of the tooltip this is OCring
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerState {
    #[serde(default)]
    pub slot_infos: AHashMap<SlotPosition, OneSlotInfo>,
    #[serde(default)]
    pub anchors: AHashMap<AnchorType, (i64, PixelPosition, ImageHash)>,
    #[serde(default)]
    pub screen_info: Option<ScreenInfo>,
    #[serde(default)]
    pub pending_jobs: Vec<OCRJob>,

    pub buffer: Buffer,
}
