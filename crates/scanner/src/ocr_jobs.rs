use crate::{image_utils::ocr::recognize_line, scan_result::Bytes, scanner_state::ScannerState};
use image::RgbaImage;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::mem::take;

// One line of text to recognise, already the 64px strip the recogniser takes. Scanning never waits
// for the answer: it comes back through apply_ocr, from another worker in the browser.
#[derive(Debug)]
pub struct OcrJob {
    pub id: u32,
    // lower is read first
    pub priority: u8,
    pub image: RgbaImage,
}

impl Serialize for OcrJob {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("OcrJob", 5)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("priority", &self.priority)?;
        s.serialize_field("width", &self.image.width())?;
        s.serialize_field("height", &self.image.height())?;
        s.serialize_field("data", &Bytes(self.image.as_raw()))?;
        s.end()
    }
}

impl ScannerState {
    // the id its text will be stored under; a strip seen before is not read again
    pub fn request_ocr(&mut self, image: RgbaImage, priority: u8) -> u32 {
        let hash =
            ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one((image.width(), image.as_raw()));
        let next = self.ocr_ids.len() as u32;
        let id = *self.ocr_ids.entry(hash).or_insert(next);
        if id == next {
            self.ocr_queue.push(OcrJob {
                id,
                priority,
                image,
            });
        }
        id
    }

    // takes in answers, then everything that was waiting on them
    pub fn apply_ocr(&mut self, results: Vec<(u32, String)>) {
        self.ocr_texts.extend(results);
        for slot in self.slot_infos.values_mut() {
            if let Some(text) = slot.amount_job.and_then(|id| self.ocr_texts.get(&id)) {
                slot.amount = Some(text.clone());
                slot.amount_job = None;
            }
        }
        self.resolve_reads();
    }

    // for the native harnesses, which have no second worker
    pub fn run_ocr_inline(&mut self) {
        let results = take(&mut self.ocr_queue)
            .iter()
            .map(|job| (job.id, recognize_line(&job.image)))
            .collect();
        self.apply_ocr(results);
    }
}
