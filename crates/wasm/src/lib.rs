use hf_core::histogram::HistogramOutputs;
use hf_core::histogram::histogram;
use hf_core::optimizer::solve;
use hf_core::payload::Payload;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use hf_scanner::buffer::Buffer;
use hf_scanner::image_utils::ocr::recognize_raw;
use hf_scanner::scanner_state::{ScannerState, SlotEdit};
use hf_scanner::setup::{OneIconConfig, load_ocr_engine, set_config};
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
}

#[derive(Deserialize)]
struct CropperOptions {
    full: bool,
    // texts of earlier results' ocr_jobs, as (id, text)
    ocr_results: Vec<(u32, String)>,
    // what the user changed by hand since the last scan
    #[serde(default)]
    edits: Vec<SlotEdit>,
}

#[wasm_bindgen]
pub fn cropper_wrapper(options: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let options: CropperOptions = from_value(options).unwrap();
    SCANNER.with_borrow_mut(|state| {
        let state = state.as_mut().unwrap();
        state.apply_edits(options.edits);
        timed("apply_ocr", || state.apply_ocr(options.ocr_results));
        state.cropper();
        let out = timed("to_value", || {
            to_value(&state.result(options.full)).unwrap()
        });
        state.clear_changed();
        out
    })
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

    let ReserveInput { screen_info: game, mut buffer, config } = from_value(input).unwrap();
    buffer.reserve();
    set_config(config);
    let scanner_state =
        ScannerState::new(buffer, game.game_width, game.game_height, game.forced_21_9);
    let out = to_value(&scanner_state.result(true)).unwrap();
    if let Some(mut old) = SCANNER.replace(Some(scanner_state)) {
        old.buffer.dealloc();
    }
    out
}

// The two below run in the OCR worker, which holds the recogniser and nothing else.
#[wasm_bindgen]
pub fn ocr_init_wrapper(model: Vec<u8>) {
    console_error_panic_hook::set_once();

    load_ocr_engine(model);
}

#[wasm_bindgen]
pub fn ocr_wrapper(width: u32, height: u32, data: Vec<u8>, numbers: bool) -> String {
    recognize_raw(width, height, data, numbers)
}
