mod bitset;
mod chance_to_cost;
mod constants;
mod cost_to_chance;
mod helpers;
mod histogram;
pub mod juice_generation;
mod monte_carlo;
mod parser;

#[cfg(test)]
#[macro_use]
mod test_utils;

mod value_estimation;

use crate::chance_to_cost::{ChanceToCostOut, chance_to_cost};

use crate::cost_to_chance::{
    CostToChanceArrOut, CostToChanceOut, cost_to_chance, cost_to_chance_arr,
};
use crate::helpers::{average_cost, calc_unlock, get_count};
use crate::monte_carlo::monte_carlo_data;
use crate::parser::{PreparationOutputs, Upgrade, parser, parser_with_other_strategy, preparation};

use rand::prelude::*;
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
// use web_sys::console;
// use std::cell::RefCell;
#[derive(Deserialize)]
pub struct Payload {
    normal_hone_ticks: Option<Vec<Vec<bool>>>,
    adv_hone_ticks: Option<Vec<Vec<bool>>>,
    normal_counts: Option<Vec<Vec<i64>>>,
    adv_counts: Option<Vec<Vec<i64>>>,

    adv_hone_strategy: String,
    budget: Vec<i64>,
    express_event: bool,
    bucket_count: usize,
    user_mats_value: Option<Vec<f64>>,
    data_size: Option<usize>,
    cost_data: Option<Vec<Vec<i64>>>,
}

#[derive(Deserialize)]
pub struct PayloadArr {
    normal_hone_ticks: Option<Vec<Vec<bool>>>,
    adv_hone_ticks: Option<Vec<Vec<bool>>>,
    normal_counts: Option<Vec<Vec<i64>>>,
    adv_counts: Option<Vec<Vec<i64>>>,

    adv_hone_strategy: String,
    budget_arr: Vec<Vec<i64>>,
    express_event: bool,
    user_mats_value: Option<Vec<f64>>,

    #[allow(dead_code)]
    data_size: Option<usize>,
    cost_data: Option<Vec<Vec<i64>>>,
}

#[wasm_bindgen]
#[must_use]
pub fn monte_carlo_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();

    let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
    let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

    let user_mats_value: Vec<f64> = payload.user_mats_value.unwrap_or(vec![0.0; 7]);
    let adv_hone_strategy: String = payload.adv_hone_strategy;
    let data_size: usize = payload.data_size.unwrap_or(100000).max(1000);

    let mut prep_outputs: PreparationOutputs = preparation(
        &normal_counts,
        &payload.budget,
        &adv_counts,
        payload.express_event,
        &user_mats_value,
        &adv_hone_strategy,
    );
    let mut rng: ThreadRng = rand::rng();
    let cost_data: Vec<[i64; 9]> = monte_carlo_data(
        data_size,
        &mut prep_outputs.upgrade_arr,
        &prep_outputs.unlock_costs,
        payload.budget[9], // Use first budget's special leap count
        &mut rng,
    );

    let js_ready: Vec<Vec<i64>> = cost_data.iter().map(|arr| arr.to_vec()).collect();
    to_value(&js_ready).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn cost_to_chance_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input).unwrap();

    let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
    let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

    let budget: Vec<i64> = payload.budget;
    let user_mats_value: Vec<f64> = payload.user_mats_value.unwrap_or(vec![0.0; 7]);
    // let data_size: usize = payload.data_size.unwrap_or(100000).max(1000);

    // let mut rng: ThreadRng = rand::rng();
    let cost_vec: Vec<Vec<i64>> = payload.cost_data.unwrap();
    let cost_data: Vec<[i64; 9]> = cost_vec
        .into_iter()
        .map(|row| {
            let mut a = [0i64; 9];
            for (i, v) in row.into_iter().enumerate().take(9) {
                a[i] = v;
            }
            a
        })
        .collect();
    let out: CostToChanceOut = cost_to_chance(
        &normal_counts,
        &budget,
        &adv_counts,
        payload.express_event,
        payload.bucket_count,
        &user_mats_value,
        payload.adv_hone_strategy,
        &cost_data,
    );

    // console::log_1(&"cost_to_chance_complete".into());
    to_value(&out).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn chance_to_cost_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();

    let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
    let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

    let adv_hone_strategy: String = payload.adv_hone_strategy;

    let cost_vec: Vec<Vec<i64>> = payload.cost_data.unwrap();
    let cost_data: Vec<[i64; 9]> = cost_vec
        .into_iter()
        .map(|row| {
            let mut a = [0i64; 9];
            for (i, v) in row.into_iter().enumerate().take(9) {
                a[i] = v;
            }
            a
        })
        .collect();
    let out: ChanceToCostOut = chance_to_cost(
        &normal_counts,
        &adv_counts,
        &adv_hone_strategy,
        payload.express_event,
        payload.bucket_count,
        &cost_data,
    );

    // Return a JS object with fields to avoid brittle tuple indexing
    to_value(&out).unwrap()
    // input
}
#[wasm_bindgen]
#[must_use]
pub fn parser_wrapper_unified(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();

    let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
    let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

    let (upgrades, other_strategy_prob_dists): (Vec<Upgrade>, Vec<Vec<f64>>) =
        parser_with_other_strategy(
            &normal_counts,
            &adv_counts,
            &payload.adv_hone_strategy,
            payload.express_event,
        );

    to_value(&(
        upgrades,
        calc_unlock(&normal_counts, &adv_counts, payload.express_event),
        other_strategy_prob_dists,
    ))
    .unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn average_cost_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();

    let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
    let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

    let upgrades: Vec<Upgrade> = parser(
        &normal_counts,
        &adv_counts,
        &payload.adv_hone_strategy,
        payload.express_event,
    );

    let avg_costs: Vec<f64> = average_cost(&upgrades);
    to_value(&avg_costs).unwrap()
}

#[wasm_bindgen]
#[must_use]
pub fn cost_to_chance_arr_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: PayloadArr = from_value(input).unwrap();

    let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
    let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

    let budget_arr: Vec<Vec<i64>> = payload.budget_arr;
    let user_mats_value: Vec<f64> = payload.user_mats_value.unwrap_or(vec![0.0; 7]);
    let cost_vec: Vec<Vec<i64>> = payload.cost_data.unwrap();
    let cost_data: Vec<[i64; 9]> = cost_vec
        .into_iter()
        .map(|row| {
            let mut a = [0i64; 9];
            for (i, v) in row.into_iter().enumerate().take(9) {
                a[i] = v;
            }
            a
        })
        .collect();
    let result: CostToChanceArrOut = cost_to_chance_arr(
        &normal_counts,
        &budget_arr,
        &adv_counts,
        payload.express_event,
        &user_mats_value,
        payload.adv_hone_strategy,
        &cost_data,
    );

    to_value(&result).unwrap()
}

// Histograms are included in the default wrappers' outputs

// #[must_use]
// pub fn chance_to_cost_test_wrapper(
//     normal_hone_ticks: Vec<Vec<bool>>,
//     adv_hone_ticks: Vec<Vec<bool>>,
//     adv_hone_strategy: String,
//     express_event: bool,
// ) -> (Vec<Vec<i64>>, Vec<f64>) {
//     let mut rng: ThreadRng = rand::rng();
//     let out: ChanceToCostOut = chance_to_cost(
//         &ticks_to_counts(normal_hone_ticks),
//         &ticks_to_counts(adv_hone_ticks),
//         &adv_hone_strategy,
//         express_event,
//         1000,
//         100000,
//         &mut rng,
//     );
//     (out.hundred_budgets, out.hundred_chances)
// }

// #[must_use]
// pub fn cost_to_chance_test_wrapper(
//     normal_hone_ticks: Vec<Vec<bool>>,
//     adv_hone_ticks: Vec<Vec<bool>>,
//     budget: Vec<i64>,
//     express_event: bool,
// ) -> (f64, String) {
//     let mut rng: ThreadRng = rand::rng();
//     let out: CostToChanceOut = cost_to_chance(
//         &ticks_to_counts(normal_hone_ticks),
//         &budget,
//         &ticks_to_counts(adv_hone_ticks),
//         express_event,
//         1000,
//         &[0.0; 7],
//         "No juice".to_owned(),
//         100000,
//         &mut rng,
//     );
//     (
//         out.chance,
//         out.reasons
//             .iter()
//             .map(|r| format!("{:.2}%", r * 100.0))
//             .collect::<Vec<String>>()
//             .join(", "),
//     )
// }
