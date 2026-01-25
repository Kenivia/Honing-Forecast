/// Wrappers for functoins to be called via something like
/// async function MonteCarloWasm(payload: any) {
///     await init()
///     return (monte_carlo_wrapper as any)(payload)
/// }
/// (imported via import init, {monte_carlo_wrapper} from "@/../crates/wasm/pkg/honing_forecast.js"
mod cost_to_chance;
mod histogram;
mod success_analysis;
// use crate::cost_to_chance::{CostToChanceOut, cost_to_chance};
// use hf_core::helpers::{calc_unlock, get_count};

use hf_core::performance::Performance;
// use hf_core::state_bundle;
// use hf_core::parser::parser;
use hf_core::state_bundle::StateBundle;
// use hf_core::upgrade::Upgrade;
// use rand::rngs::ThreadRng;
use hf_arena::engine::solve;
use rand::rngs::ThreadRng;
use serde::Deserialize;

use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

pub use wasm_bindgen_rayon::init_thread_pool;
use web_sys::console;
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Payload {
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    // normal_counts: Option<Vec<Vec<i64>>>,
    // adv_counts: Option<Vec<Vec<i64>>>,
    adv_hone_strategy: String,

    express_event: bool,
    bucket_count: usize,

    data_size: usize,
    // cost_data: Option<Vec<Vec<i64>>>,
    mats_budget: Vec<i64>,
    user_price_arr: Vec<f64>,
    inp_leftover_values: Vec<f64>,
    juice_books_budget: Vec<(i64, i64)>,
    juice_prices: Vec<(f64, f64)>,
    inp_leftover_juice_values: Vec<(f64, f64)>,

    // honestly in js i have to initialize these arrays anyway (instead of leaving as null) so theres not much point doing Option but whatever
    progress_grid: Option<Vec<Vec<usize>>>,
    state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
    special_state: Option<Vec<usize>>,
    unlocked_grid: Option<Vec<Vec<bool>>>,
    succeeded_grid: Option<Vec<Vec<bool>>>,
    min_resolution: usize,
}

// #[derive(Deserialize)]
// #[allow(dead_code)]
// pub struct PayloadArr {
//     normal_hone_ticks: Option<Vec<Vec<bool>>>,
//     adv_hone_ticks: Option<Vec<Vec<bool>>>,
//     normal_counts: Option<Vec<Vec<i64>>>,
//     adv_counts: Option<Vec<Vec<i64>>>,

//     adv_hone_strategy: String,
//     budget_arr: Vec<Vec<i64>>,
//     express_event: bool,
//     user_price_arr: Option<Vec<f64>>,

//     data_size: Option<usize>,
//     // cost_data: Option<Vec<Vec<i64>>>,
// }

// #[wasm_bindgen]
// #[must_use]
// pub fn monte_carlo_wrapper(input: JsValue) -> JsValue {
//     console_error_panic_hook::set_once();
//     let payload: Payload = from_value(input).unwrap();

//     // let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
//     // let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

//     let user_price_arr: Vec<f64> = payload.user_price_arr;
//     let adv_hone_strategy: String = payload.adv_hone_strategy;
//     let data_size: usize = payload.data_size;

//     let state_bundle = StateBundle::init_from_inputs(
//         &payload.normal_hone_ticks,
//         &payload.budget,
//         &payload.adv_hone_ticks,
//         payload.express_event,
//         &payload.user_price_arr,
//         &adv_hone_strategy,
//         juice_books_budget,
//         juice_prices,
//         inp_leftover_values,
//         inp_leftover_juice_values,
//         state_bundle_js,
//     );

//     let mut rng: ThreadRng = rand::rng();
//     let cost_data: Vec<[i64; 9]> = monte_carlo_data(
//         data_size,
//         &mut upgrade_arr,
//         &prep_output.unlock_costs,
//         payload.budget[9], // Use first budget's special leap count
//         &mut rng,
//     );

//     let js_ready: Vec<Vec<i64>> = cost_data.iter().map(|arr| arr.to_vec()).collect();
//     to_value(&js_ready).unwrap()
// }

#[wasm_bindgen]
#[must_use]
pub fn evaluate_average_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input).unwrap();

    let mut state_bundle = StateBundle::init_from_inputs(
        &payload.normal_hone_ticks,
        &payload.mats_budget,
        &payload.adv_hone_ticks,
        payload.express_event,
        &payload.user_price_arr,
        &payload.adv_hone_strategy,
        &payload.juice_books_budget,
        &payload.juice_prices,
        &payload.inp_leftover_values,
        &payload.inp_leftover_juice_values,
        payload.progress_grid,
        payload.state_grid,
        payload.special_state,
        payload.unlocked_grid,
        payload.succeeded_grid,
    );

    let mut dummy_performance = Performance::new();
    let metric = state_bundle.average_gold_metric(&mut dummy_performance);
    state_bundle.metric = metric;
    state_bundle.clean_state();
    state_bundle.set_latest_special_probs();
    // let out: CostToChanceOut = cost_to_chance(&state_bundle);
    web_sys::console::log_1(&format!("{:?}", state_bundle).into());
    to_value(&state_bundle).unwrap()
}

// #[derive(Serialize, Debug)]
// pub struct OptimizeAverageOut {
//     best: f64,
//     state_bundle_js: StateBundleJs,
// }
#[wasm_bindgen]
#[must_use]
pub fn optimize_average_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input).unwrap();

    let state_bundle = StateBundle::init_from_inputs(
        &payload.normal_hone_ticks,
        &payload.mats_budget,
        &payload.adv_hone_ticks,
        payload.express_event,
        &payload.user_price_arr,
        &payload.adv_hone_strategy,
        &payload.juice_books_budget,
        &payload.juice_prices,
        &payload.inp_leftover_values,
        &payload.inp_leftover_juice_values,
        payload.progress_grid,
        payload.state_grid,
        payload.special_state,
        payload.unlocked_grid,
        payload.succeeded_grid,
    );

    let mut rng: ThreadRng = rand::rng();
    let mut dummy_performance = Performance::new();
    let mut best_state: StateBundle = solve(
        &mut rng,
        1,
        payload.min_resolution,
        state_bundle.clone(),
        &mut dummy_performance,
    );
    web_sys::console::log_1(&format!("{:?}", best_state).into());
    best_state.update_dist();
    best_state.update_individual_support();
    best_state.update_combined();
    best_state.clean_state();
    best_state.compute_special_probs();
    best_state.set_latest_special_probs();
    // let out: CostToChanceOut = cost_to_chance(&state_bundle);
    web_sys::console::log_1(&format!("{:?}", best_state).into());
    to_value(&best_state).unwrap()
}

// #[wasm_bindgen]
// #[must_use]
// pub fn parser_wrapper(input: JsValue) -> JsValue {
//     console_error_panic_hook::set_once();
//     let payload: Payload = from_value(input).unwrap();

//     let (upgrades, other_strategy_prob_dists): (Vec<Upgrade>, Vec<Vec<f64>>) =
//         parser_with_other_strategy(
//             &normal_counts,
//             &adv_counts,
//             &payload.adv_hone_strategy,
//             payload.express_event,
//         );

//     to_value(&(
//         upgrades,
//         calc_unlock(&normal_counts, &adv_counts, payload.express_event),
//         other_strategy_prob_dists,
//     ))
//     .unwrap()
// }

// #[wasm_bindgen]
// #[must_use]
// pub fn average_cost_wrapper(input: JsValue) -> JsValue {
//     console_error_panic_hook::set_once();
//     let payload: Payload = from_value(input).unwrap();

//     let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
//     let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

//     let upgrades: Vec<Upgrade> = parser(
//         &normal_counts,
//         &adv_counts,
//         &payload.adv_hone_strategy,
//         payload.express_event,
//     );

//     let avg_costs: Vec<f64> = average_cost(&upgrades);
//     to_value(&avg_costs).unwrap()
// }

// #[wasm_bindgen]
// #[must_use]
// pub fn cost_to_chance_arr_wrapper(input: JsValue) -> JsValue {
//     console_error_panic_hook::set_once();
//     let payload: PayloadArr = from_value(input).unwrap();

//     let normal_counts: Vec<Vec<i64>> = get_count(payload.normal_counts, payload.normal_hone_ticks);
//     let adv_counts: Vec<Vec<i64>> = get_count(payload.adv_counts, payload.adv_hone_ticks);

//     let budget_arr: Vec<Vec<i64>> = payload.budget_arr;
//     let user_price_arr: Vec<f64> = payload.user_price_arr.unwrap_or(vec![0.0; 7]);
//     let cost_vec: Vec<Vec<i64>> = payload.cost_data.unwrap();
//     let cost_data: Vec<[i64; 9]> = cost_vec
//         .into_iter()
//         .map(|row| {
//             let mut a = [0i64; 9];
//             for (i, v) in row.into_iter().enumerate().take(9) {
//                 a[i] = v;
//             }
//             a
//         })
//         .collect();
//     let result: CostToChanceArrOut = cost_to_chance_arr(
//         &normal_counts,
//         &budget_arr,
//         &adv_counts,
//         payload.express_event,
//         &user_price_arr,
//         payload.adv_hone_strategy,
//         &cost_data,
//     );

//     to_value(&result).unwrap()
// }
