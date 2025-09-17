mod chance_to_cost;
mod constants;
mod cost_to_chance;
mod helpers;
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
}

#[wasm_bindgen]
pub fn chance_to_cost_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();
    let normal_hone_ticks: Vec<Vec<bool>> = payload.normal_hone_ticks;
    let adv_hone_ticks: Vec<Vec<bool>> = payload.adv_hone_ticks;
    let desired_chance: f64 = payload.desired_chance;
    let adv_hone_strategy: String = payload.adv_hone_strategy;

    let out: (Vec<i64>, f64) = chance_to_cost(
        ticks_to_counts(normal_hone_ticks),
        ticks_to_counts(adv_hone_ticks),
        desired_chance,
        adv_hone_strategy,
    );

    // 3) return JS array/object
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
    let (chance, reason): (f64, String) = cost_to_chance(
        &ticks_to_counts(normal_hone_ticks),
        &budget,
        &ticks_to_counts(adv_hone_ticks),
    );
    console::log_1(&"cost_to_chance_complete".into());
    to_value(&(chance, reason)).unwrap()
}

pub fn chance_to_cost_test_wrapper(
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    desired_chance: f64,
    adv_hone_strategy: String,
) -> (Vec<i64>, f64) {
    let (mats, chance): (Vec<i64>, f64) = chance_to_cost(
        ticks_to_counts(normal_hone_ticks),
        ticks_to_counts(adv_hone_ticks),
        desired_chance,
        adv_hone_strategy,
    );
    (mats, chance)
}

pub fn cost_to_chance_test_wrapper(
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    budget: Vec<i64>,
) -> (f64, String) {
    let (chance, reason): (f64, String) = cost_to_chance(
        &ticks_to_counts(normal_hone_ticks),
        &budget,
        &ticks_to_counts(adv_hone_ticks),
    );
    (chance, reason)
}
