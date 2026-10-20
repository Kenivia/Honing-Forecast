// A session recorded to be run again on any build. A file is MAGIC, then records of [tag][payload
// length as u32 LE][payload]: the start, one input per frame the scanner was sent, then the end.
// Start and input payloads are msgpack with field names.
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::scanner_state::{Detected, ScannerState};

pub const MAGIC: &[u8] = b"HFCAP005";
pub const START: u8 = b'S';
pub const INPUT: u8 = b'I';
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
    // the git commit that recorded it
    pub commit: String,
}

// A frame the scanner was sent: the XOR with the one before it, RGBA, LZ4, nothing when the two
// are the same.
#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub delta: Option<ByteBuf>,
}

pub enum Record {
    Start(Start),
    Input(Input),
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
    pub fn record(&mut self) -> Vec<u8> {
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
        record(INPUT, &Input { delta })
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
            _ => Record::End(String::from_utf8_lossy(payload).into_owned()),
        })
    })
}

// a file's start, frames and end
pub fn read_frames(file: &[u8]) -> (Start, Vec<Input>, Option<String>) {
    let mut records = read_records(file);
    let Some(Record::Start(start)) = records.next() else {
        panic!("no start record")
    };
    let (mut frames, mut end) = (vec![], None);
    for record in records {
        match record {
            Record::Input(x) => frames.push(x),
            Record::End(x) => end = Some(x),
            Record::Start(_) => panic!("a second start record"),
        }
    }
    (start, frames, end)
}

impl ScannerState {
    // A recorded frame again, with the texts back since the last one. Every slot is read at
    // once, so that a run repeats. The buffer has to be the state's alone and start out as
    // zeros: it holds the frame before between calls.
    pub fn replay(&mut self, input: Input, ocr_results: Vec<(u32, String)>) {
        input.apply(self.buffer.data_mut());
        self.no_slot_budget = true;
        self.apply_ocr(ocr_results);
        self.cropper();
    }
}
