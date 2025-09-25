mod chance_to_cost;
mod constants;
mod cost_to_chance;
mod helpers;
mod histogram;
mod monte_carlo;
mod parser;
mod value_estimation;
use crate::chance_to_cost::{average_cost, chance_to_cost};
use crate::constants::EVENT_ARTISAN_MULTIPLIER;
use crate::cost_to_chance::cost_to_chance;
use crate::helpers::calc_unlock;
use crate::helpers::ticks_to_counts;
use crate::parser::{parser, parser_with_other_strategy};

use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
// use web_sys::console;

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
}

#[wasm_bindgen]
pub fn chance_to_cost_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();

    // Get counts from either direct input or converted from ticks
    let normal_counts = if let Some(counts) = payload.normal_counts {
        counts
    } else if let Some(ticks) = payload.normal_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either normal_counts or normal_hone_ticks must be provided");
    };

    let adv_counts = if let Some(counts) = payload.adv_counts {
        counts
    } else if let Some(ticks) = payload.adv_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either adv_counts or adv_hone_ticks must be provided");
    };

    let adv_hone_strategy: String = payload.adv_hone_strategy;
    let data_size: usize = payload.data_size.unwrap_or(100000).max(1000);

    let out = chance_to_cost(
        normal_counts,
        adv_counts,
        adv_hone_strategy,
        payload.express_event,
        payload.bucket_count,
        data_size,
    );

    // Return a JS object with fields to avoid brittle tuple indexing
    to_value(&out).unwrap()
    // input
}

#[wasm_bindgen]
pub fn cost_to_chance_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let payload: Payload = from_value(input).unwrap();

    // Get counts from either direct input or converted from ticks
    let normal_counts = if let Some(counts) = payload.normal_counts {
        counts
    } else if let Some(ticks) = payload.normal_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either normal_counts or normal_hone_ticks must be provided");
    };

    let adv_counts = if let Some(counts) = payload.adv_counts {
        counts
    } else if let Some(ticks) = payload.adv_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either adv_counts or adv_hone_ticks must be provided");
    };

    let budget: Vec<i64> = payload.budget;
    // console::log_1(&"unwrap complete".into());
    let user_mats_value = payload.user_mats_value.unwrap_or(vec![0.0; 7]);
    // console::log_1(user_mats_value[0].into());
    let data_size: usize = payload.data_size.unwrap_or(100000).max(1000);
    let out = cost_to_chance(
        &normal_counts,
        &budget,
        &adv_counts,
        payload.express_event,
        payload.bucket_count,
        &user_mats_value,
        payload.adv_hone_strategy,
        data_size,
    );
    // console::log_1(&"cost_to_chance_complete".into());
    to_value(&out).unwrap()
}

#[wasm_bindgen]
pub fn parser_wrapper_unified(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();

    // Get counts from either direct input or converted from ticks
    let normal_counts = if let Some(counts) = payload.normal_counts {
        counts
    } else if let Some(ticks) = payload.normal_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either normal_counts or normal_hone_ticks must be provided");
    };

    let adv_counts = if let Some(counts) = payload.adv_counts {
        counts
    } else if let Some(ticks) = payload.adv_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either adv_counts or adv_hone_ticks must be provided");
    };

    let artisan_rate_arr: Vec<f64>;
    if payload.express_event {
        artisan_rate_arr = EVENT_ARTISAN_MULTIPLIER.to_vec();
    } else {
        artisan_rate_arr = vec![1.0; 25];
    }

    // Default extra values (same as other functions)
    let extra_arr = vec![0.0; 25];
    let extra_num_arr = vec![0; 25];

    let (upgrades, other_strategy_prob_dists) = parser_with_other_strategy(
        &normal_counts,
        &adv_counts,
        &payload.adv_hone_strategy,
        &artisan_rate_arr,
        &extra_arr,
        &extra_num_arr,
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
pub fn average_cost_wrapper(input: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    let payload: Payload = from_value(input).unwrap();

    // Get counts from either direct input or converted from ticks
    let normal_counts = if let Some(counts) = payload.normal_counts {
        counts
    } else if let Some(ticks) = payload.normal_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either normal_counts or normal_hone_ticks must be provided");
    };

    let adv_counts = if let Some(counts) = payload.adv_counts {
        counts
    } else if let Some(ticks) = payload.adv_hone_ticks {
        ticks_to_counts(ticks)
    } else {
        panic!("Either adv_counts or adv_hone_ticks must be provided");
    };

    let artisan_rate_arr: Vec<f64>;
    if payload.express_event {
        artisan_rate_arr = EVENT_ARTISAN_MULTIPLIER.to_vec();
    } else {
        artisan_rate_arr = vec![1.0; 25];
    }

    // Default extra values (same as other functions)
    let extra_arr = vec![0.0; 25];
    let extra_num_arr = vec![0; 25];

    let upgrades = parser(
        &normal_counts,
        &adv_counts,
        &payload.adv_hone_strategy,
        &artisan_rate_arr,
        &extra_arr,
        &extra_num_arr,
        payload.express_event,
    );

    let avg_costs = average_cost(&upgrades);
    to_value(&avg_costs).unwrap()
}

// Histograms are included in the default wrappers' outputs

pub fn chance_to_cost_test_wrapper(
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    adv_hone_strategy: String,
    express_event: bool,
) -> (Vec<Vec<i64>>, Vec<f64>) {
    let out = chance_to_cost(
        ticks_to_counts(normal_hone_ticks),
        ticks_to_counts(adv_hone_ticks),
        adv_hone_strategy,
        express_event,
        1000,
        100000,
    );
    (out.hundred_budgets, out.hundred_chances)
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
        100000,
    );
    (out.chance, out.reasons.join(", "))
}
