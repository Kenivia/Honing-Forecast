use crate::{
    buffer::Buffer,
    cropper::anchors::AnchorInfo,
    image_utils::downscale::DownscaledCache,
    setup::{IncomingNewIcon, OneIconConfig},
};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

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

    #[serde(default)]
    pub initialized: bool,
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
    pub top_left: (f64, f64),
    pub width: usize,
    pub height: usize,
}

impl Sub for ScaledPosition {
    type Output = Self;
    fn sub(self, other: Self) -> ScaledPosition {
        Self {
            top_left: (
                self.top_left.0 - other.top_left.0,
                self.top_left.1 - other.top_left.1,
            ),
            width: self.width,
            height: self.height,
        }
    }
}
impl Add for ScaledPosition {
    type Output = Self;
    fn add(self, other: Self) -> ScaledPosition {
        Self {
            top_left: (
                self.top_left.0 + other.top_left.0,
                self.top_left.1 + other.top_left.1,
            ),
            width: self.width,
            height: self.height,
        }
    }
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
    pub icon_name: Option<String>,
    pub observed_number: OneIconConfig,
    pub observed_icon: OneIconConfig,
    pub progress: OneSlotProgress,
    pub amount: Option<usize>,
    pub tradability: Option<Tradability>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum OneSlotProgress {
    OCRing,
    NeedHover,
    HoverOCRing,
    NA,
}

pub const ALL_ANCHOR_TYPES: [InventoryType; 3] = [
    InventoryType::Roster,
    InventoryType::CharStorage,
    InventoryType::CharInventory,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerState {
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
    pub incoming_new_icon: Option<IncomingNewIcon>,

    pub downscaled_cache: DownscaledCache,
}
