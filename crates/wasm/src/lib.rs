use hf_core::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use hf_core::histogram::HistogramOutputs;
use hf_core::histogram::histogram;
use hf_core::optimizer::solve;
use hf_core::payload::Payload;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_scanner::buffer::Buffer;
use hf_scanner::dev_utils::capture::{InputRecorder, parse};
use hf_scanner::dev_utils::timing::{take, timed};
use hf_scanner::interface::setup::{OneIconConfig, load_ocr_engine, set_config};
use hf_scanner::ocr::jobs::OcrJob;
use hf_scanner::scanner_state::{Detected, ScannerState, SlotEdit};
use rand::rngs::ThreadRng;
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[allow(unused_imports)]
use web_sys::console;

// every strategy of an advanced upgrade not yet started, from build.rs
static ADV_FRESH: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/adv_fresh.bin"));

fn add_fresh_adv(payload: &mut Payload) {
    let fresh = Some((0, 0, false, false));
    if payload
        .upgrade_info
        .iter()
        .any(|upgrade| !upgrade.is_normal_honing && upgrade.adv_progress == fresh)
    {
        let table: Vec<(AdvConfig, AdvDistTriplet)> = postcard::from_bytes(ADV_FRESH).unwrap();
        payload.adv_cache.get_or_insert_default().extend(table);
    }
}

#[wasm_bindgen]
pub fn optimize_average_wrapper(input_payload: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let mut payload: Payload = from_value(input_payload).unwrap();
    add_fresh_adv(&mut payload);
    let state_bundle: StateBundle = StateBundle::init_from_payload(payload);

    let mut rng: ThreadRng = rand::rng();
    let mut dummy_probe = Probe::default();
    let mut best_state: StateBundle = solve(&mut rng, state_bundle, None, &mut dummy_probe).best;

    best_state.optimizer_average_gold_metric(&mut dummy_probe);
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
    // write the scan's trace, see capture.rs
    #[serde(default)]
    record: bool,
    // read every slot at once, so that a re-run repeats
    #[serde(default)]
    no_budget: bool,
}

#[wasm_bindgen]
pub fn cropper_wrapper(options: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let options: CropperOptions = from_value(options).unwrap();
    SCANNER.with_borrow_mut(|state| {
        let state = state.as_mut().unwrap();
        let ocr_results = options.record.then(|| options.ocr_results.clone());
        state.apply_edits(options.edits);
        timed("apply_ocr", || state.apply_ocr(options.ocr_results));
        state.cropper();
        let out = timed("to_value", || {
            to_value(&state.result(options.full)).unwrap()
        });
        if let Some(ocr_results) = ocr_results {
            CAPTURED.set(timed("capture", || state.record_scan(ocr_results)));
        }
        state.clear_changed();
        out
    })
}

thread_local! {
    // a recorder worker's
    static RECORDER: RefCell<InputRecorder> = RefCell::default();
}

// where the recorder worker copies the next frame to
#[wasm_bindgen]
pub fn recorder_buffer(size: usize) -> usize {
    RECORDER.with_borrow_mut(|x| x.buffer(size).as_mut_ptr() as usize)
}

// the frame copied in, with its timestamp and the edits its scan took in
#[wasm_bindgen]
pub fn record_frame(time: f64, edits: JsValue) -> Vec<u8> {
    let edits: Vec<SlotEdit> = from_value(edits).unwrap();
    RECORDER.with_borrow_mut(|x| timed("record", || x.record(time, edits)))
}

// the record of the last scan, empty when it was not recorded
#[wasm_bindgen]
pub fn take_capture() -> Vec<u8> {
    CAPTURED.take()
}

// A recorded frame run again in place of a live one: an input record's payload, and its trace's
// to run as recorded. Without the trace the texts are this run's own.
#[wasm_bindgen]
pub fn replay_wrapper(input: &[u8], trace: Option<Vec<u8>>, options: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let input = parse(input);
    let trace = trace.map(|x| parse(&x));
    let CropperOptions {
        full,
        ocr_results,
        no_budget,
        ..
    } = from_value(options).unwrap();
    SCANNER.with_borrow_mut(|state| {
        let state = state.as_mut().unwrap();
        state.no_slot_budget = no_budget;
        state.replay(input, trace, ocr_results);
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
    to_value(&take()).unwrap()
}

// what a new scanner state is built from
#[derive(Deserialize)]
struct ReserveInput {
    settings: ScanSettings,
    buffer: Buffer,
    config: Vec<OneIconConfig>,
}

#[derive(Deserialize)]
struct ScanSettings {
    forced_21_9: Detected<bool>,
    brightness: Detected<f64>,
}

#[wasm_bindgen]
pub fn reserve_buffer_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let ReserveInput {
        settings,
        mut buffer,
        config,
    } = from_value(input).unwrap();
    buffer.reserve();
    set_config(config);
    let scanner_state = ScannerState::new(buffer, settings.forced_21_9, settings.brightness);
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
