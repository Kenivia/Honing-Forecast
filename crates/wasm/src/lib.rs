/// Wrappers for functions to be called via something like
/// async function EvaluateAverageWasm(payload: any) {
///     await init()
///     return (evaluate_average_wrapper as any)(payload)
/// }
/// (imported via import init, {evaluate_average_wrapper} from "@/../crates/wasm/pkg/honing_forecast.js"
mod histogram;
use crate::histogram::histogram;
use hf_core::engine::solve;
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
pub fn evaluate_average_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input).unwrap();
    let mut state_bundle = StateBundle::init_from_payload(payload);
    let mut dummy_performance = Performance::new();
    let metric = state_bundle.average_gold_metric_with_breakdown(&mut dummy_performance); // doesn't allow prob evaluation right now, even if metric_type is set 
    state_bundle.metric = metric;
    state_bundle.set_latest_special_probs();
    to_value(&state_bundle).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn optimize_average_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input).unwrap();

    let state_bundle = StateBundle::init_from_payload(payload);

    let mut rng: ThreadRng = rand::rng();
    let mut dummy_performance = Performance::new();
    let mut best_state: StateBundle = solve(&mut rng, state_bundle.clone(), &mut dummy_performance);

    best_state.average_gold_metric_with_breakdown(&mut dummy_performance);
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
