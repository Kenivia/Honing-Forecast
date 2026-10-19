// A session recorded to be run again. A file is MAGIC, then records of [tag][payload length as
// u32 LE][payload]: the start, then per frame the scanner was sent an input and a trace, then the
// end. Start, input and trace payloads are msgpack with field names.
// The input is what any build can be run on: the frame, lossless, its time and the edits. The trace
// is what this build made of it, for running the same build again scan for scan.
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::scanner_state::{Detected, ScannerState, SlotEdit};

pub const MAGIC: &[u8] = b"HFCAP004";
pub const START: u8 = b'S';
pub const INPUT: u8 = b'I';
pub const TRACE: u8 = b'F';
// how the session ended on the page, corrections included, as JSON
pub const END: u8 = b'E';

// written by the page
#[derive(Debug, Serialize, Deserialize)]
pub struct Start {
    pub width: usize,
    pub height: usize,
    // as the user gave them: what is detected is detected again on replay
    pub forced_21_9: Detected<bool>,
    pub brightness: Detected<f64>,
    pub build: Build,
}

// which build recorded it: a trace is only good for the same wasm
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Build {
    pub commit: String,
    pub wasm: String,
}

// A frame the scanner was sent: the XOR with the one before it, RGBA, LZ4, nothing when the two
// are the same. Its timestamp in microseconds, and the edits the scan of it took in.
#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub delta: Option<ByteBuf>,
    pub time: f64,
    pub edits: Vec<SlotEdit>,
}

// what the scan of a frame took from elsewhere, and the state it left
#[derive(Debug, Serialize, Deserialize)]
pub struct Trace {
    pub ocr_results: Vec<(u32, String)>,
    // where the time budget cut the look at the slots, the one thing a scan takes from the clock
    pub slot_cut: Option<usize>,
    // of the state after the scan, see digest
    pub digest: u64,
}

pub enum Record {
    Start(Start),
    Input(Input),
    Trace(Trace),
    End(String),
}

// Records every frame on a worker of its own. Frames are copied into the two buffers in turn,
// so the one before is still there.
#[derive(Debug, Default)]
pub struct InputRecorder {
    frames: [Vec<u8>; 2],
    current: usize,
    delta: Vec<u8>,
}

impl InputRecorder {
    // where the next frame is to be copied
    pub fn buffer(&mut self, size: usize) -> &mut [u8] {
        for x in self.frames.iter_mut().chain([&mut self.delta]) {
            x.resize(size, 0);
        }
        &mut self.frames[self.current]
    }

    // the frame just copied in, as a record
    pub fn record(&mut self, time: f64, edits: Vec<SlotEdit>) -> Vec<u8> {
        let [a, b] = &self.frames;
        let (new, old) = if self.current == 0 { (a, b) } else { (b, a) };
        let mut changed = 0;
        for ((delta, new), old) in self.delta.iter_mut().zip(new).zip(old) {
            *delta = new ^ old;
            changed |= *delta;
        }
        self.current ^= 1;
        let delta =
            (changed != 0).then(|| ByteBuf::from(lz4_flex::compress_prepend_size(&self.delta)));
        record(INPUT, &Input { delta, time, edits })
    }
}

impl Input {
    // onto the frame before
    fn apply(&self, data: &mut [u8]) {
        let Some(delta) = &self.delta else { return };
        let delta = lz4_flex::decompress_size_prepended(delta).unwrap();
        for (pixel, delta) in data.iter_mut().zip(delta) {
            *pixel ^= delta;
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

pub fn parse<T: serde::de::DeserializeOwned>(payload: &[u8]) -> T {
    rmp_serde::from_slice(payload).unwrap()
}

pub fn read_records(file: &[u8]) -> impl Iterator<Item = Record> + '_ {
    assert!(file.starts_with(MAGIC), "not an {MAGIC:?} capture");
    let mut rest = &file[MAGIC.len()..];
    std::iter::from_fn(move || {
        let (&tag, after) = rest.split_first()?;
        let length = u32::from_le_bytes(after[..4].try_into().unwrap()) as usize;
        let payload = &after[4..4 + length];
        rest = &after[4 + length..];
        Some(match tag {
            START => Record::Start(parse(payload)),
            INPUT => Record::Input(parse(payload)),
            TRACE => Record::Trace(parse(payload)),
            _ => Record::End(String::from_utf8_lossy(payload).into_owned()),
        })
    })
}

// a file's frames, each with its trace
pub fn read_frames(file: &[u8]) -> (Start, Vec<(Input, Trace)>, Option<String>) {
    let mut records = read_records(file);
    let Some(Record::Start(start)) = records.next() else {
        panic!("no start record")
    };
    let (mut frames, mut input, mut end) = (vec![], None, None);
    for record in records {
        match record {
            Record::Input(x) => input = Some(x),
            Record::Trace(x) => frames.push((input.take().unwrap(), x)),
            Record::End(x) => end = Some(x),
            Record::Start(_) => panic!("a second start record"),
        }
    }
    (start, frames, end)
}

impl ScannerState {
    // What a scan left behind, to tell that a replay went the same way: every slot as the page
    // gets it, the hover, the pages, the chests, the currencies and the lines sent to be read. Scores are left
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
                    (
                        x.address,
                        icon,
                        x.alternatives,
                        x.amount,
                        x.tooltip_amount,
                        x.tradability,
                        x.label
                    )
                ) + &format!("{:?}", (x.status, &x.reason, x.value, &x.variants))
            })
            .collect();
        slots.sort();
        let mut pages: Vec<String> = result.pages.iter().map(|x| format!("{x:?}")).collect();
        pages.sort();
        let hover = result
            .hover
            .map(|x| (x.last_read_title, x.title, x.amount, x.tradability));
        let jobs: Vec<u32> = result.ocr_jobs.iter().map(|x| x.id).collect();
        let mut hasher = DefaultHasher::new();
        let currencies = format!("{:?}", result.currencies);
        format!(
            "{:?}",
            (slots, pages, hover, &self.chests, jobs, currencies)
        )
        .hash(&mut hasher);
        hasher.finish()
    }

    // the scan that just ran, as a record
    pub fn record_scan(&self, ocr_results: Vec<(u32, String)>) -> Vec<u8> {
        let trace = Trace {
            ocr_results,
            slot_cut: self.slot_cut,
            digest: self.digest(),
        };
        record(TRACE, &trace)
    }

    // A recorded frame again. With its trace the scan goes as it did, and is checked against it;
    // without, the texts are the caller's and the time budget is the clock's. The buffer has to
    // be the state's alone and start out as zeros: it holds the frame before between calls.
    pub fn replay(&mut self, input: Input, trace: Option<Trace>, ocr_results: Vec<(u32, String)>) {
        self.apply_edits(input.edits.clone());
        input.apply(self.buffer.data_mut());
        match trace {
            Some(trace) => {
                self.apply_ocr(trace.ocr_results);
                self.replay_cut = Some(trace.slot_cut);
                self.cropper();
                if self.digest() != trace.digest && self.replay_mismatch.is_none() {
                    self.replay_mismatch = Some(self.replayed);
                }
            }
            None => {
                self.apply_ocr(ocr_results);
                self.replay_cut = None;
                self.cropper();
            }
        }
        self.replayed += 1;
    }
}
