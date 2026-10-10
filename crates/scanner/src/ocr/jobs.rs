use super::{
    number::{NumberParams, number_strip},
    recognize::recognize_line,
    text::text_strip,
};
use crate::{scanner_state::ScannerState, setup::BASE_ICONS, timing::timed};
use image::{GrayImage, RgbaImage};
use serde::{Deserialize, Serialize};
use std::mem::take;

// an image as it crosses to JS and back: its size and its pixels as a Uint8Array
mod pixels {
    use image::RgbaImage;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize)]
    struct Out<'a> {
        width: u32,
        height: u32,
        #[serde(with = "serde_bytes")]
        data: &'a [u8],
    }

    #[derive(Deserialize)]
    struct In {
        width: u32,
        height: u32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    }

    pub fn serialize<S: Serializer>(image: &RgbaImage, serializer: S) -> Result<S::Ok, S::Error> {
        Out { width: image.width(), height: image.height(), data: image.as_raw() }.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<RgbaImage, D::Error> {
        let image = In::deserialize(deserializer)?;
        Ok(RgbaImage::from_raw(image.width, image.height, image.data).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Kind {
    Text,
    // a tooltip's amount, by its yellow alone
    Yellow,
    // A slot's count: the crop is the number's. `icon` is the slot's icon crop, normalised, and
    // `template` the name of the icon it matched; together they say what is behind the number.
    Number {
        #[serde(with = "pixels")]
        icon: RgbaImage,
        template: String,
    },
}

// One line of text cut out of a frame, as captured. Scanning only cuts it; whoever reads it
// makes it into the strip the recogniser takes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    #[serde(with = "pixels")]
    pub crop: RgbaImage,
    pub kind: Kind,
    // the setting the frame was at
    pub brightness: f64,
}

impl Line {
    // the 64px strip the recogniser takes
    pub fn strip(&self) -> GrayImage {
        match &self.kind {
            Kind::Text => text_strip(&self.crop, false, self.brightness),
            Kind::Yellow => text_strip(&self.crop, true, self.brightness),
            Kind::Number { icon, template } => number_strip(
                &self.crop,
                icon,
                &BASE_ICONS.read()[template].data,
                self.brightness,
                &NumberParams::default(),
            ),
        }
    }

    // a slot's count is read with digits only
    pub fn read(&self) -> String {
        let strip = timed("ocr/preprocess", || self.strip());
        recognize_line(&strip, matches!(self.kind, Kind::Number { .. }))
    }
}

// A line to read. Scanning never waits for the answer: it comes back through apply_ocr, from
// another worker in the browser.
#[derive(Debug, Serialize, Deserialize)]
pub struct OcrJob {
    pub id: u32,
    // lower is read first
    pub priority: u8,
    pub line: Line,
}

impl ScannerState {
    // The id its text will be stored under; a line seen before is not read again. The brightness
    // estimate is not part of what is compared: it moves a little on every scan.
    pub fn request_ocr(&mut self, line: Line, priority: u8) -> u32 {
        let (icon, template) = match &line.kind {
            Kind::Number { icon, template } => (icon.as_raw().as_slice(), template.as_str()),
            _ => (&[][..], ""),
        };
        let hash = ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one((
            line.crop.width(),
            line.crop.as_raw(),
            matches!(line.kind, Kind::Yellow),
            icon,
            template,
        ));
        let next = self.ocr_ids.len() as u32;
        let id = *self.ocr_ids.entry(hash).or_insert(next);
        if id == next {
            self.ocr_queue.push(OcrJob { id, priority, line });
        }
        id
    }

    // takes in answers, then everything that was waiting on them
    pub fn apply_ocr(&mut self, results: Vec<(u32, String)>) {
        self.ocr_texts.extend(results);
        for slot in self.slot_infos.values_mut() {
            if let Some(text) = slot.amount_job.and_then(|id| self.ocr_texts.get(&id)) {
                slot.vote_amount(text.clone());
                slot.amount_job = None;
            }
        }
        self.resolve_reads();
    }

    // for the native harnesses, which have no second worker
    pub fn run_ocr_inline(&mut self) {
        let results = take(&mut self.ocr_queue).iter().map(|job| (job.id, job.line.read())).collect();
        self.apply_ocr(results);
    }
}
