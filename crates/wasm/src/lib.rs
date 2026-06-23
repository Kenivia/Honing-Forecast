use hf_core::histogram::HistogramOutputs;
use hf_core::histogram::histogram;
use hf_core::my_dbg;
use hf_core::optimizer::solve;
use hf_core::payload::Payload;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use hf_scanner::scanner_state::ScannerState;
use rand::rngs::ThreadRng;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[allow(unused_imports)]
use web_sys::console;

#[wasm_bindgen]
#[must_use]
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
#[must_use]
pub fn histogram_wrapper(input_payload: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input_payload).unwrap();
    let mut state_bundle: StateBundle = StateBundle::init_from_payload(payload);
    let out: HistogramOutputs = histogram(&mut state_bundle);
    to_value(&out).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn cropper_wrapper(inp_scanner_state: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let mut scanner_state: ScannerState = from_value(inp_scanner_state).unwrap();
    scanner_state.cropper();
    to_value(&scanner_state).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn reserve_buffer_wrapper(inp_scanner_state: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let mut scanner_state: ScannerState = from_value(inp_scanner_state).unwrap();
    scanner_state.buffer.reserve();
    to_value(&scanner_state).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn dealloc_buffer_wrapper(inp_scanner_state: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let mut scanner_state: ScannerState = from_value(inp_scanner_state).unwrap();
    scanner_state.buffer.dealloc();
    to_value(&scanner_state).unwrap()
}
