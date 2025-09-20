mod chance_to_cost;
mod constants;
mod cost_to_chance;
mod helpers;
mod histogram;
mod monte_carlos;
mod parser;
mod value_estimation;
use crate::chance_to_cost::chance_to_cost;
use crate::cost_to_chance::cost_to_chance;
use crate::helpers::ticks_to_counts;

use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[derive(Deserialize)]
pub struct Payload {
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    desired_chance: f64,
    adv_hone_strategy: String,
    budget: Vec<i64>,
    express_event: bool,
    bucket_count: usize,
    user_mats_value: Option<Vec<f64>>,
}

#[wasm_bindgen]
pub fn chance_to_cost_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();
    let normal_hone_ticks: Vec<Vec<bool>> = payload.normal_hone_ticks;
    let adv_hone_ticks: Vec<Vec<bool>> = payload.adv_hone_ticks;
    let desired_chance: f64 = payload.desired_chance;
    let adv_hone_strategy: String = payload.adv_hone_strategy;

    let out = chance_to_cost(
        ticks_to_counts(normal_hone_ticks),
        ticks_to_counts(adv_hone_ticks),
        desired_chance,
        adv_hone_strategy,
        payload.express_event,
        payload.bucket_count,
    );

    // Return a JS object with fields to avoid brittle tuple indexing
    to_value(&out).unwrap()
    // input
}

#[wasm_bindgen]
pub fn cost_to_chance_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    console::log_1(&"wasm: start() called".into());
    let payload: Payload = from_value(input).unwrap();
    let normal_hone_ticks: Vec<Vec<bool>> = payload.normal_hone_ticks;
    let adv_hone_ticks: Vec<Vec<bool>> = payload.adv_hone_ticks;
    let budget: Vec<i64> = payload.budget;
    console::log_1(&"unwrap complete".into());
    let user_mats_value = payload.user_mats_value.unwrap_or(vec![0.0; 7]);
    let out = cost_to_chance(
        &ticks_to_counts(normal_hone_ticks),
        &budget,
        &ticks_to_counts(adv_hone_ticks),
        payload.express_event,
        payload.bucket_count,
        &user_mats_value,
        payload.adv_hone_strategy,
    );
    console::log_1(&"cost_to_chance_complete".into());
    to_value(&out).unwrap()
}

// Histograms are included in the default wrappers' outputs

pub fn chance_to_cost_test_wrapper(
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    desired_chance: f64,
    adv_hone_strategy: String,
    express_event: bool,
) -> (Vec<i64>, f64) {
    let out = chance_to_cost(
        ticks_to_counts(normal_hone_ticks),
        ticks_to_counts(adv_hone_ticks),
        desired_chance,
        adv_hone_strategy,
        express_event,
        1000,
    );
    (out.best_budget, out.actual_prob)
}

pub fn cost_to_chance_test_wrapper(
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    budget: Vec<i64>,
    express_event: bool,
) -> (f64, String) {
    let out = cost_to_chance(
        &ticks_to_counts(normal_hone_ticks),
        &budget,
        &ticks_to_counts(adv_hone_ticks),
        express_event,
        1000,
        &vec![0.0; 7],
        "No juice".to_owned(),
    );
    (out.chance, out.reasons.join(", "))
}
