use crate::{
    buffer::Buffer,
    ocr_jobs::OcrJob,
    image_utils::common::IntegerRectangle,
    scanner_state::{InventoryType, OneSlotInfo, ScannerState, SlotAddress, Tradability},
    setup::OneIconConfig,
    tooltip::{
        chest::Chest,
        items::{is_chest_icon, shares_icon},
    },
};
use serde::{Serialize, Serializer, ser::SerializeStruct};

pub struct Bytes<'a>(pub &'a [u8]);

impl Serialize for Bytes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

// an icon whose pixels reach JS as a Uint8Array instead of an array of numbers
pub struct IconBytes<'a>(pub &'a OneIconConfig);

impl Serialize for IconBytes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("OneIconConfig", 4)?;
        s.serialize_field("data", &Bytes(self.0.data.as_raw()))?;
        s.serialize_field("name", &self.0.name)?;
        s.serialize_field("offset", &self.0.offset)?;
        s.serialize_field("tag", &self.0.tag)?;
        s.end()
    }
}

pub fn optional_icon_bytes<S: Serializer>(
    icon: &Option<OneIconConfig>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    icon.as_ref().map(IconBytes).serialize(serializer)
}

// what the page colours a slot by
#[derive(Debug, Serialize, Clone, Copy)]
pub enum SlotStatus {
    Pending,
    Good,
    NeedHover,
    // in order but for its tradability, which the page may assume for a whole window
    NeedTradability,
    Error,
    Irrelevant,
}

#[derive(Serialize)]
pub struct SlotResult<'a> {
    pub address: SlotAddress,
    // nothing when the slot matches no icon
    pub icon_name_score: &'a Option<(String, f64)>,
    pub amount: &'a Option<String>,
    pub tooltip_amount: &'a Option<String>,
    pub tradability: Option<Tradability>,
    // the material the tooltip said it is, where the icon alone does not say
    pub label: &'a Option<String>,
    pub status: SlotStatus,
    // why it needs a hover or is an error, for the user
    pub reason: String,
    // the tooltip's amount, else the number on the icon
    pub value: Option<u32>,
    pub image: Option<IconBytes<'a>>,
}

// the number drawn on an icon: stray characters are dropped, and a single item shows none
fn icon_number(text: &str) -> u32 {
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    digits.parse().unwrap_or(1)
}

impl Chest {
    // read in this very slot, or with the amount the slot shows
    fn stands_for(&self, address: &SlotAddress, number: Option<u32>) -> bool {
        self.slot == Some(*address)
            || self.amount.as_ref().is_some_and(|x| Some(icon_number(x)) == number)
    }
}

impl ScannerState {
    // status, reason, and the slot's tradability: its own tooltip's, else its chest's
    fn slot_status(
        &self,
        address: &SlotAddress,
        info: &OneSlotInfo,
    ) -> (SlotStatus, String, Option<Tradability>) {
        let (status, reason, tradability) = self.read_status(address, info);
        let shared = info.icon_name_score.as_ref().is_some_and(|x| shares_icon(&x.0));
        if matches!(status, SlotStatus::Good) && shared && info.label.is_none() {
            return (
                SlotStatus::NeedHover,
                "Several items share this icon, and only the tooltip says which this is. Hover it.".into(),
                tradability,
            );
        }
        if matches!(status, SlotStatus::Good) && tradability.is_none() {
            return (
                SlotStatus::NeedTradability,
                "Whether this stack can be traded is only on its tooltip. Hover it.".into(),
                None,
            );
        }
        (status, reason, tradability)
    }

    fn read_status(
        &self,
        address: &SlotAddress,
        info: &OneSlotInfo,
    ) -> (SlotStatus, String, Option<Tradability>) {
        let (status, reason) = self.amount_status(address, info);
        let chest_tradability = || {
            let (icon, _) = info.icon_name_score.as_ref()?;
            let number = info.amount.as_ref().map(|text| icon_number(text));
            let column = (address.inventory_type, address.page_num, address.pos_in_inv.1);
            let mut chests: Vec<&Chest> = self
                .chests
                .iter()
                .filter(|chest| chest.icon.as_ref() == Some(icon) && chest.column == Some(column))
                .collect();
            // the ones that stand for this slot, if any does
            if chests.iter().any(|chest| chest.stands_for(address, number)) {
                chests.retain(|chest| chest.stands_for(address, number));
            }
            chests.iter().find_map(|chest| chest.tradability)
        };
        (status, reason, info.tradability.or_else(chest_tradability))
    }

    fn amount_status(&self, address: &SlotAddress, info: &OneSlotInfo) -> (SlotStatus, String) {
        let Some((icon, _)) = &info.icon_name_score else {
            return (SlotStatus::Irrelevant, String::new());
        };
        if info.hovered && info.tooltip_failed {
            return (
                SlotStatus::Error,
                "A tooltip was seen for this slot, but its amount could not be read. Hover it again, or set the amount by hand.".into(),
            );
        }
        if is_chest_icon(icon) {
            // A pushed-up tooltip only tells the column. A chest read there with this icon stands
            // for every slot showing its amount, whose tooltips would be the same; failing that
            // (a number misread), the slots are done once as many chests were read as there are slots.
            let column = (address.inventory_type, address.page_num, address.pos_in_inv.1);
            let chests: Vec<&Chest> = self
                .chests
                .iter()
                .filter(|chest| chest.icon.as_ref() == Some(icon) && chest.column == Some(column))
                .collect();
            let number = info.amount.as_ref().map(|text| icon_number(text));
            let alike = self
                .slot_infos
                .iter()
                .filter(|(other, other_info)| {
                    (other.inventory_type, other.page_num, other.pos_in_inv.1) == column
                        && other_info.icon_name_score.as_ref().map(|x| &x.0) == Some(icon)
                        && !self.edits.contains_key(*other)
                })
                .count();
            return if info.hovered
                || chests.iter().any(|chest| chest.stands_for(address, number))
                || chests.len() >= alike
            {
                (SlotStatus::Good, String::new())
            } else {
                (
                    SlotStatus::NeedHover,
                    format!(
                        "A chest's contents are only on its tooltip, so hover it. None read in this column has this stack's amount; chests with this icon read there so far: {} of {alike}.",
                        chests.len()
                    ),
                )
            };
        }
        if info.hovered {
            return (SlotStatus::Good, String::new());
        }
        match &info.amount {
            None => (SlotStatus::Pending, "The number on the icon is still being read.".into()),
            Some(text) if icon_number(text) >= 9999 => (
                SlotStatus::NeedHover,
                "The icon only shows up to 9999, so the real amount is on the tooltip. Hover this stack.".into(),
            ),
            Some(_) => (SlotStatus::Good, String::new()),
        }
    }
}

#[derive(Serialize)]
pub struct HoverResult<'a> {
    pub last_read_title: &'a String,
    pub title: &'a Option<String>,
    pub amount: &'a Option<String>,
    pub tradability: Option<Tradability>,
}

// What JS gets after a scan; the state itself never leaves Rust. Images, debug entries and chests
// are only included when written since the last result, unless everything is asked for.
#[derive(Serialize)]
pub struct ScanResult<'a> {
    pub full: bool,
    pub buffer: Buffer,
    pub slots: Vec<SlotResult<'a>>,
    pub hover: Option<HoverResult<'a>>,
    // the page each located window shows in game
    pub pages: Vec<(InventoryType, usize)>,
    pub chests: Option<&'a Vec<Chest>>,
    pub debug: Vec<(&'a String, IntegerRectangle, f64, f64, Vec<IconBytes<'a>>)>,
    // lines to recognise, whose text is to come back with a later scan
    pub ocr_jobs: &'a [OcrJob],
}

impl ScannerState {
    pub fn result(&self, full: bool) -> ScanResult<'_> {
        let slots = self
            .slot_infos
            .iter()
            // an edited slot is the page's to show
            .filter(|(address, _)| !self.edits.contains_key(*address))
            .map(|(address, info)| {
                let (status, reason, tradability) = self.slot_status(address, info);
                let read = info.tooltip_amount.as_ref().or(info.amount.as_ref());
                SlotResult {
                    address: *address,
                    icon_name_score: &info.icon_name_score,
                    amount: &info.amount,
                    tooltip_amount: &info.tooltip_amount,
                    tradability,
                    label: &info.label,
                    status,
                    reason,
                    value: read
                        .filter(|_| info.icon_name_score.is_some())
                        .map(|text| icon_number(text)),
                    image: info
                        .display_icon
                        .as_ref()
                        .filter(|_| full || self.changed_slots.contains(address))
                        .map(IconBytes),
                }
            })
            .collect();
        let debug = self
            .debug_info
            .iter()
            .filter(|(name, _)| full || self.changed_debug.contains(*name))
            .map(|(name, (position, confidence, brightness, icons))| {
                (name, *position, *confidence, *brightness, icons.iter().map(IconBytes).collect())
            })
            .collect();
        ScanResult {
            full,
            buffer: self.buffer,
            slots,
            pages: self
                .active_page_num()
                .into_iter()
                .filter(|(inventory, _)| self.inventory_root(*inventory).is_some())
                .filter_map(|(inventory, page)| Some((inventory, page?)))
                .collect(),
            hover: self.hover.as_ref().map(|hover| HoverResult {
                last_read_title: &hover.last_read_title,
                title: &hover.title,
                amount: &hover.amount,
                tradability: hover.tradability,
            }),
            chests: (full || self.chests_changed).then_some(&self.chests),
            debug,
            ocr_jobs: &self.ocr_queue,
        }
    }

    // what the last result carried is not sent again
    pub fn clear_changed(&mut self) {
        self.ocr_queue.clear();
        self.changed_slots.clear();
        self.changed_debug.clear();
        self.chests_changed = false;
    }
}
