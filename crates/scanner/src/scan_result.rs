use crate::{
    buffer::Buffer,
    ocr_jobs::OcrJob,
    image_utils::common::IntegerRectangle,
    scanner_state::{ScannerState, SlotAddress, Tradability},
    setup::OneIconConfig,
    tooltip::chest::Chest,
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

#[derive(Serialize)]
pub struct SlotResult<'a> {
    pub address: SlotAddress,
    pub icon_name_score: &'a (String, f64),
    pub amount: &'a Option<String>,
    pub tooltip_amount: &'a Option<String>,
    pub tradability: Option<Tradability>,
    // icon, number as seen, number as processed
    pub images: Option<[IconBytes<'a>; 3]>,
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
            .filter_map(|(address, info)| {
                Some(SlotResult {
                    address: *address,
                    icon_name_score: info.icon_name_score.as_ref()?,
                    amount: &info.amount,
                    tooltip_amount: &info.tooltip_amount,
                    tradability: info.tradability,
                    images: (full || self.changed_slots.contains(address)).then(|| {
                        [&info.observed_icon, &info.observed_number, &info.processed_number]
                            .map(IconBytes)
                    }),
                })
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
