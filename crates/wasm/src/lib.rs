use hf_core::histogram::HistogramOutputs;
use hf_core::histogram::histogram;
use hf_core::optimizer::solve;
use hf_core::payload::Payload;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use hf_scanner::buffer::Buffer;
use hf_scanner::ocr::{jobs::OcrJob, recognize::load_ocr_engine};
use hf_scanner::scanner_state::{ScannerState, SlotEdit};
use hf_scanner::setup::{OneIconConfig, set_config};
use hf_scanner::timing::timed;
use rand::rngs::ThreadRng;
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[allow(unused_imports)]
use web_sys::console;

#[wasm_bindgen]
pub fn optimize_average_wrapper(input_payload: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input_payload).unwrap();
    let state_bundle: StateBundle = StateBundle::init_from_payload(payload);

    let mut rng: ThreadRng = rand::rng();
    let mut dummy_performance = Performance::new();
    let mut best_state: StateBundle = solve(&mut rng, state_bundle, &mut dummy_performance);

    best_state.optimizer_average_gold_metric(&mut dummy_performance);
    best_state.set_latest_special_probs();

    to_value(&best_state).unwrap()
}

#[wasm_bindgen]
pub fn histogram_wrapper(input_payload: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input_payload).unwrap();
    let mut state_bundle: StateBundle = StateBundle::init_from_payload(payload);
    let out: HistogramOutputs = histogram(&mut state_bundle);
    to_value(&out).unwrap()
}

// The scanner state stays here between calls; only a small result goes to JS.
// It lives and dies with the worker's wasm instance.
thread_local! {
    static SCANNER: RefCell<Option<ScannerState>> = const { RefCell::new(None) };
    static CAPTURED: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[derive(Deserialize)]
struct CropperOptions {
    full: bool,
    // texts of earlier results' ocr_jobs, as (id, text)
    ocr_results: Vec<(u32, String)>,
    // what the user changed by hand since the last scan
    #[serde(default)]
    edits: Vec<SlotEdit>,
    // keep recording, if the state was reserved to; see capture.rs
    #[serde(default)]
    record: bool,
}

#[wasm_bindgen]
pub fn cropper_wrapper(options: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let options: CropperOptions = from_value(options).unwrap();
    SCANNER.with_borrow_mut(|state| {
        let state = state.as_mut().unwrap();
        if !options.record {
            state.recorder = None;
        }
        let inputs = state.recorder.is_some().then(|| (options.ocr_results.clone(), options.edits.clone()));
        state.apply_edits(options.edits);
        timed("apply_ocr", || state.apply_ocr(options.ocr_results));
        state.cropper();
        let out = timed("to_value", || {
            to_value(&state.result(options.full)).unwrap()
        });
        if let Some((ocr_results, edits)) = inputs {
            let record = timed("capture", || state.record_scan(ocr_results, edits));
            CAPTURED.set(record);
        }
        state.clear_changed();
        out
    })
}

// the record of the last scan, empty when it was not recorded
#[wasm_bindgen]
pub fn take_capture() -> Vec<u8> {
    CAPTURED.take()
}

// One scan of a capture run again, in place of a frame: the payload of a scan record.
#[wasm_bindgen]
pub fn replay_wrapper(record: &[u8], full: bool) -> JsValue {
    console_error_panic_hook::set_once();

    SCANNER.with_borrow_mut(|state| {
        let state = state.as_mut().unwrap();
        state.replay_scan(hf_scanner::capture::parse_scan(record));
        let out = to_value(&state.result(full)).unwrap();
        state.clear_changed();
        out
    })
}

// the first replayed scan that did not end as recorded
#[wasm_bindgen]
pub fn replay_mismatch() -> Option<f64> {
    SCANNER.with_borrow(|state| state.as_ref().unwrap().replay_mismatch.map(|x| x as f64))
}

// stage timings of the calls since the last take, as [name, total, calls]
#[wasm_bindgen]
pub fn take_timings() -> JsValue {
    to_value(&hf_scanner::timing::take()).unwrap()
}

// what a new scanner state is built from
#[derive(Deserialize)]
struct ReserveInput {
    screen_info: GameResolution,
    buffer: Buffer,
    config: Vec<OneIconConfig>,
    // record the session from its first scan
    #[serde(default)]
    record: bool,
}

#[derive(Deserialize)]
struct GameResolution {
    game_width: u32,
    game_height: u32,
    forced_21_9: bool,
}

#[wasm_bindgen]
pub fn reserve_buffer_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let ReserveInput { screen_info: game, mut buffer, config, record } = from_value(input).unwrap();
    buffer.reserve();
    set_config(config);
    let mut scanner_state =
        ScannerState::new(buffer, game.game_width, game.game_height, game.forced_21_9);
    scanner_state.recorder = record.then(Default::default);
    let out = to_value(&scanner_state.result(true)).unwrap();
    if let Some(mut old) = SCANNER.replace(Some(scanner_state)) {
        old.buffer.dealloc();
    }
    out
}

// The two below run in the OCR worker, which holds the recogniser and the templates a slot's
// number is cleaned up with, and no scanner state.
#[wasm_bindgen]
pub fn ocr_init_wrapper(model: Vec<u8>, config: JsValue) {
    console_error_panic_hook::set_once();

    load_ocr_engine(model);
    set_config(from_value(config).unwrap());
}

// one of a scan result's ocr_jobs, as it came
#[wasm_bindgen]
pub fn ocr_wrapper(job: JsValue) -> String {
    from_value::<OcrJob>(job).unwrap().line.read()
}
