/// Wrappers for functions to be called via something like
/// async function EvaluateAverageWasm(payload: any) {
///     await init()
///     return (evaluate_average_wrapper as any)(payload)
/// }
/// (imported via import init, {evaluate_average_wrapper} from "@/../crates/wasm/pkg/honing_forecast.js"
mod histogram;
use crate::histogram::histogram;
use hf_core::optimizer::solve;
use hf_core::payload::Payload;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use rand::rngs::ThreadRng;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[allow(unused_imports)]
use web_sys::console;

#[wasm_bindgen]
#[must_use]
pub fn parser_wrapper(input_payload: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input_payload).unwrap();
    let mut state_bundle = StateBundle::init_from_payload(payload);
    let mut dummy_performance = Performance::new();
    let metric = state_bundle.average_gold_metric(true, &mut dummy_performance); // doesn't allow prob evaluation right now, even if metric_type is set 
    state_bundle.metric = metric;
    state_bundle.set_latest_special_probs();
    to_value(&state_bundle).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn evaluate_average_wrapper(input_state_bundle: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let mut state_bundle: StateBundle = from_value(input_state_bundle).unwrap();

    let mut dummy_performance = Performance::new();
    let metric = state_bundle.average_gold_metric(true, &mut dummy_performance); // doesn't allow prob evaluation right now, even if metric_type is set 
    state_bundle.metric = metric;
    state_bundle.set_latest_special_probs();
    to_value(&state_bundle).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn optimize_average_wrapper(input_state_bundle: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let json_string = js_sys::JSON::stringify(&input_state_bundle).unwrap();
    let json_str: String = json_string.into();
    let result: Result<StateBundle, _> = serde_json::from_str(&json_str);
    let state_bundle: StateBundle = result.unwrap();
    // let state_bundle: StateBundle = from_value(input_state_bundle).unwrap();

    let mut rng: ThreadRng = rand::rng();
    let mut dummy_performance = Performance::new();
    let mut best_state: StateBundle = solve(&mut rng, state_bundle, &mut dummy_performance);

    best_state.average_gold_metric(true, &mut dummy_performance);
    best_state.set_latest_special_probs();

    to_value(&best_state).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn histogram_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input).unwrap();

    let mut state_bundle = StateBundle::init_from_payload(payload);

    let out = histogram(&mut state_bundle);
    to_value(&out).unwrap()
}
