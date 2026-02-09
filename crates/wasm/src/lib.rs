/// Wrappers for functions to be called via something like
/// async function EvaluateAverageWasm(payload: any) {
///     await init()
///     return (evaluate_average_wrapper as any)(payload)
/// }
/// (imported via import init, {evaluate_average_wrapper} from "@/../crates/wasm/pkg/honing_forecast.js"
mod histogram;
use crate::histogram::histogram;
use hf_arena::engine::solve;
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
