// A session as Rust saw it, to be run again: every input of every scan, the frames lossless.
// A file is MAGIC, then records of [tag][payload length as u32 LE][payload]. Start and Scan
// payloads are msgpack with field names.
use crate::scanner_state::{ScannerState, SlotEdit};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::hash::{DefaultHasher, Hash, Hasher};

pub const MAGIC: &[u8] = b"HFCAP001";
pub const START: u8 = b'S';
pub const SCAN: u8 = b'F';
// how the session ended on the page, corrections included, as JSON; the page writes it
pub const END: u8 = b'E';

#[derive(Debug, Serialize, Deserialize)]
pub struct Start {
    pub width: usize,
    pub height: usize,
    pub game_width: u32,
    pub game_height: u32,
    pub forced_21_9: bool,
}

// The XOR with the last scanned frame, LZ4. Alpha is apart and only there when it changed,
// which after the first frame it does not.
#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
    pub rgb: ByteBuf,
    pub alpha: Option<ByteBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scan {
    pub ocr_results: Vec<(u32, String)>,
    pub edits: Vec<SlotEdit>,
    // nothing for a frame skipped as still
    pub frame: Option<Frame>,
    // where the time budget cut the look at the slots, the one thing a scan takes from the clock
    pub slot_cut: Option<usize>,
    // of the state after the scan, see digest
    pub digest: u64,
}

pub enum Record {
    Start(Start),
    Scan(Scan),
    End(String),
}

#[derive(Debug, Default)]
pub struct Recorder {
    started: bool,
    previous: Vec<u8>,
    rgb: Vec<u8>,
    alpha: Vec<u8>,
}

impl Recorder {
    fn frame(&mut self, data: &[u8]) -> Frame {
        let pixels = data.len() / 4;
        self.previous.resize(data.len(), 0);
        // a pixel is written as four bytes and the next one overwrites its alpha
        self.rgb.resize(pixels * 3 + 1, 0);
        self.alpha.resize(pixels, 0);
        let mut alpha_changed = 0;
        for (i, (new, old)) in data.chunks_exact(4).zip(self.previous.chunks_exact(4)).enumerate() {
            let new = u32::from_le_bytes(new.try_into().unwrap());
            let old = u32::from_le_bytes(old.try_into().unwrap());
            let delta = new ^ old;
            self.rgb[i * 3..i * 3 + 4].copy_from_slice(&delta.to_le_bytes());
            self.alpha[i] = (delta >> 24) as u8;
            alpha_changed |= self.alpha[i];
        }
        self.previous.copy_from_slice(data);
        let pack = |x: &[u8]| ByteBuf::from(lz4_flex::compress_prepend_size(x));
        Frame {
            rgb: pack(&self.rgb[..pixels * 3]),
            alpha: (alpha_changed != 0).then(|| pack(&self.alpha)),
        }
    }
}

impl Frame {
    // onto the frame it is the difference from
    fn apply(&self, data: &mut [u8]) {
        let rgb = lz4_flex::decompress_size_prepended(&self.rgb).unwrap();
        for (pixel, delta) in data.chunks_exact_mut(4).zip(rgb.chunks_exact(3)) {
            for c in 0..3 {
                pixel[c] ^= delta[c];
            }
        }
        if let Some(alpha) = &self.alpha {
            let alpha = lz4_flex::decompress_size_prepended(alpha).unwrap();
            for (pixel, delta) in data.chunks_exact_mut(4).zip(alpha) {
                pixel[3] ^= delta;
            }
        }
    }
}

fn record(tag: u8, payload: &impl Serialize) -> Vec<u8> {
    let payload = rmp_serde::to_vec_named(payload).unwrap();
    let mut out = vec![tag];
    out.extend((payload.len() as u32).to_le_bytes());
    out.extend(payload);
    out
}

pub fn parse_scan(payload: &[u8]) -> Scan {
    rmp_serde::from_slice(payload).unwrap()
}

pub fn read_records(file: &[u8]) -> impl Iterator<Item = Record> + '_ {
    assert!(file.starts_with(MAGIC));
    let mut rest = &file[MAGIC.len()..];
    std::iter::from_fn(move || {
        let (&tag, after) = rest.split_first()?;
        let length = u32::from_le_bytes(after[..4].try_into().unwrap()) as usize;
        let payload = &after[4..4 + length];
        rest = &after[4 + length..];
        Some(match tag {
            START => Record::Start(rmp_serde::from_slice(payload).unwrap()),
            SCAN => Record::Scan(parse_scan(payload)),
            _ => Record::End(String::from_utf8_lossy(payload).into_owned()),
        })
    })
}

impl ScannerState {
    // What a scan left behind, to tell that a replay went the same way: every slot as the page
    // gets it, the hover, the pages, the chests and the lines sent to be read. Scores are left
    // out, they may differ in their last digits between wasm and native.
    pub fn digest(&self) -> u64 {
        let result = self.result(false);
        let mut slots: Vec<String> = result
            .slots
            .iter()
            .map(|x| {
                let icon = x.icon_name_score.as_ref().map(|x| &x.0);
                format!(
                    "{:?}",
                    (x.address, icon, x.alternatives, x.amount, x.tooltip_amount, x.tradability, x.label)
                ) + &format!("{:?}", (x.status, &x.reason, x.value, &x.variants))
            })
            .collect();
        slots.sort();
        let mut pages: Vec<String> = result.pages.iter().map(|x| format!("{x:?}")).collect();
        pages.sort();
        let hover = result.hover.map(|x| (x.last_read_title, x.title, x.amount, x.tradability));
        let jobs: Vec<u32> = result.ocr_jobs.iter().map(|x| x.id).collect();
        let mut hasher = DefaultHasher::new();
        format!("{:?}", (slots, pages, hover, &self.chests, jobs)).hash(&mut hasher);
        hasher.finish()
    }

    // The scan that just ran, as a record; the first comes with the head of the file.
    pub fn record_scan(&mut self, ocr_results: Vec<(u32, String)>, edits: Vec<SlotEdit>) -> Vec<u8> {
        let mut recorder = self.recorder.take().unwrap();
        let mut out = vec![];
        if !recorder.started {
            recorder.started = true;
            let info = &self.screen_info;
            out.extend(MAGIC);
            out.extend(record(
                START,
                &Start {
                    width: self.buffer.width,
                    height: self.buffer.height,
                    game_width: info.game_width,
                    game_height: info.game_height,
                    forced_21_9: info.forced_21_9,
                },
            ));
        }
        let scan = Scan {
            ocr_results,
            edits,
            frame: self.scanned.then(|| recorder.frame(self.buffer.data())),
            slot_cut: self.slot_cut,
            digest: self.digest(),
        };
        out.extend(record(SCAN, &scan));
        self.recorder = Some(recorder);
        out
    }

    // A recorded scan again. The buffer has to be the state's alone and start out as zeros: it
    // holds the last scanned frame between calls.
    pub fn replay_scan(&mut self, scan: Scan) {
        self.apply_edits(scan.edits);
        self.apply_ocr(scan.ocr_results);
        match &scan.frame {
            // all that a skipped frame changes
            None => self.quiet_scans += 1,
            Some(frame) => {
                frame.apply(self.buffer.data_mut());
                self.replay_cut = Some(scan.slot_cut);
                self.cropper();
            }
        }
        if self.digest() != scan.digest && self.replay_mismatch.is_none() {
            self.replay_mismatch = Some(self.replayed);
        }
        self.replayed += 1;
    }
}
