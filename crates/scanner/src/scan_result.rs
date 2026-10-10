use crate::{
    buffer::Buffer,
    ocr::jobs::OcrJob,
    scanner_state::{InventoryType, OneSlotInfo, ScannerState, SlotAddress, Tradability},
    setup::{OneIconConfig, WireRect},
    tooltip::{
        chest::Chest,
        items::{Variant, bound_alike, is_chest_icon, open_alike, shares_icon, variant, variants_of},
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
        s.serialize_field("offset", &WireRect::from(&self.0.offset))?;
        s.serialize_field("tag", &self.0.tag)?;
        s.end()
    }
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
    // other icons it may be drawn with
    pub alternatives: &'a Vec<String>,
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
    // for a chest: the chests of templates/chests.json its icon can be, by id
    pub variants: Vec<u32>,
    pub image: Option<IconBytes<'a>>,
}

// the number drawn on an icon: stray characters are dropped, and a single item shows none
fn icon_number(text: &str) -> u32 {
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    digits.parse().unwrap_or(1)
}

// The chests a slot's icon can be: those that ask for an item level when one is written across
// the slot, and those that do not when none is.
fn slot_variants(info: &OneSlotInfo) -> Vec<&'static Variant> {
    let all: Vec<&Variant> = info.icons().flat_map(|icon| variants_of(icon).iter().copied()).collect();
    let by_level: Vec<&Variant> =
        all.iter().copied().filter(|x| x.level.is_some() == info.levelled).collect();
    if by_level.is_empty() { all } else { by_level }
}

impl OneSlotInfo {
    // every icon the slot may be drawn with, the closest first
    pub fn icons(&self) -> impl Iterator<Item = &String> {
        self.icon_name_score.iter().map(|x| &x.0).chain(&self.alternatives)
    }

    pub fn shows(&self, icons: &[String]) -> bool {
        self.icons().any(|icon| icons.contains(icon))
    }
}

impl Chest {
    // read in this very slot, or with the amount the slot shows
    fn stands_for(&self, address: &SlotAddress, number: Option<u32>) -> bool {
        self.slot == Some(*address)
            || self.amount.as_ref().is_some_and(|x| Some(icon_number(x)) == number)
    }
}

impl ScannerState {
    // status, reason, and the slot's tradability: its own tooltip's, else its chest's, else the
    // one way every chest it can be is bound
    fn slot_status(
        &self,
        address: &SlotAddress,
        info: &OneSlotInfo,
        variants: &[&Variant],
    ) -> (SlotStatus, String, Option<Tradability>) {
        let (status, reason, tradability) = self.read_status(address, info, variants);
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
        variants: &[&Variant],
    ) -> (SlotStatus, String, Option<Tradability>) {
        let (status, reason) = self.amount_status(address, info, variants);
        let chest_tradability = || {
            info.icon_name_score.as_ref()?;
            let number = info.amount.as_ref().map(|text| icon_number(text));
            let column = (address.inventory_type, address.page_num, address.pos_in_inv.1);
            let mut chests: Vec<&Chest> = self
                .chests
                .iter()
                .filter(|chest| info.shows(&chest.icons) && chest.column == Some(column))
                .collect();
            // the ones that stand for this slot, if any does
            if chests.iter().any(|chest| chest.stands_for(address, number)) {
                chests.retain(|chest| chest.stands_for(address, number));
            }
            let inventory = address.inventory_type;
            let read = chests.iter().find_map(|chest| {
                let bound = || bound_alike(chest.variants.iter().map(|id| variant(*id)), inventory);
                chest.tradability.or_else(bound)
            });
            read.or_else(|| bound_alike(variants.iter().copied(), inventory))
        };
        (status, reason, info.tradability.or_else(chest_tradability))
    }

    fn amount_status(
        &self,
        address: &SlotAddress,
        info: &OneSlotInfo,
        variants: &[&Variant],
    ) -> (SlotStatus, String) {
        let Some((icon, _)) = &info.icon_name_score else {
            return (SlotStatus::Irrelevant, String::new());
        };
        let failed = || {
            (
                SlotStatus::Error,
                "A tooltip was seen for this slot, but its amount could not be read. Hover it again, or set the amount by hand.".to_string(),
            )
        };
        if is_chest_icon(icon) {
            let good = |irrelevant: bool| match irrelevant {
                true => (SlotStatus::Irrelevant, "It opens to nothing that is counted.".to_string()),
                false => (SlotStatus::Good, String::new()),
            };
            // Nothing to find out when every chest it can be opens to the same things. One that
            // can only be chests that count for nothing is still hovered: which of several
            // look-alike icons a slot shows is not sure enough to drop it unseen.
            let sure = open_alike(variants) && !variants[0].irrelevant;
            // A pushed-up tooltip only tells the column. A chest read there with this icon stands
            // for every slot showing its amount, whose tooltips would be the same; failing that
            // (a number misread), the slots are done once as many chests were read as there are slots.
            let column = (address.inventory_type, address.page_num, address.pos_in_inv.1);
            let chests: Vec<&Chest> = self
                .chests
                .iter()
                .filter(|chest| info.shows(&chest.icons) && chest.column == Some(column))
                .collect();
            let number = info.amount.as_ref().map(|text| icon_number(text));
            let standing: Vec<&&Chest> =
                chests.iter().filter(|chest| chest.stands_for(address, number)).collect();
            // the amount of a chest that counts for nothing does not matter
            if !sure && !standing.is_empty() && standing.iter().all(|chest| chest.irrelevant) {
                return good(true);
            }
            if info.hovered && info.tooltip_failed {
                return failed();
            }
            if sure || !standing.is_empty() || info.hovered {
                return good(false);
            }
            let alike = self
                .slot_infos
                .iter()
                .filter(|(other, other_info)| {
                    (other.inventory_type, other.page_num, other.pos_in_inv.1) == column
                        && other_info.icons().any(|other| info.icons().any(|own| own == other))
                        && !self.edits.contains_key(*other)
                })
                .count();
            return if chests.len() >= alike {
                good(chests.iter().all(|chest| chest.irrelevant))
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
        if info.hovered && info.tooltip_failed {
            return failed();
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

// What JS gets after a scan; the state itself never leaves Rust. Images and chests are only
// included when written since the last result, unless everything is asked for.
#[derive(Serialize)]
pub struct ScanResult<'a> {
    pub full: bool,
    pub buffer: Buffer,
    pub slots: Vec<SlotResult<'a>>,
    pub hover: Option<HoverResult<'a>>,
    // the page each located window shows in game
    pub pages: Vec<(InventoryType, usize)>,
    pub chests: Option<&'a Vec<Chest>>,
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
                let variants = slot_variants(info);
                let (status, reason, tradability) = self.slot_status(address, info, &variants);
                let read = info.tooltip_amount.as_ref().or(info.amount.as_ref());
                SlotResult {
                    address: *address,
                    icon_name_score: &info.icon_name_score,
                    alternatives: &info.alternatives,
                    amount: &info.amount,
                    tooltip_amount: &info.tooltip_amount,
                    tradability,
                    label: &info.label,
                    status,
                    reason,
                    value: read
                        .filter(|_| info.icon_name_score.is_some())
                        .map(|text| icon_number(text)),
                    variants: variants.iter().map(|x| x.id).collect(),
                    image: info
                        .display_icon
                        .as_ref()
                        .filter(|_| full || self.changed_slots.contains(address))
                        .map(IconBytes),
                }
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
            ocr_jobs: &self.ocr_queue,
        }
    }

    // what the last result carried is not sent again
    pub fn clear_changed(&mut self) {
        self.ocr_queue.clear();
        self.changed_slots.clear();
        self.chests_changed = false;
    }
}
