use crate::parser::PreparationOutput;
// use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Clone, Serialize)]
pub struct Payload {
    pub normal_hone_ticks: Vec<Vec<bool>>,
    pub adv_hone_ticks: Vec<Vec<bool>>,
    // normal_counts: Option<Vec<Vec<i64>>>,
    // adv_counts: Option<Vec<Vec<i64>>>,
    pub adv_hone_strategy: String,

    pub express_event: bool,

    // cost_data: Option<Vec<Vec<i64>>>,
    pub mats_budget: Vec<i64>,
    pub user_price_arr: Vec<f64>,
    pub inp_leftover_values: Vec<f64>,
    pub juice_books_budget: Vec<(i64, i64)>,
    pub juice_prices: Vec<(f64, f64)>,
    pub inp_leftover_juice_values: Vec<(f64, f64)>,

    // honestly in js i have to initialize these arrays anyway (instead of leaving as null) so theres not much point doing Option but whatever
    pub progress_grid: Option<Vec<Vec<usize>>>,
    pub state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
    pub special_state: Option<Vec<usize>>,
    pub unlocked_grid: Option<Vec<Vec<bool>>>,
    pub succeeded_grid: Option<Vec<Vec<bool>>>,
    pub min_resolution: usize,
    #[serde(default)]
    pub num_threads: usize,
    #[serde(default = "default_one")]
    pub metric_type: i64,
}
fn default_one() -> i64 {
    1
}
impl StateBundle {
    pub fn init_from_inputs(
        hone_ticks: &[Vec<bool>],
        input_budgets: &[i64],
        adv_ticks: &[Vec<bool>],
        express_event: bool,
        inp_price_arr: &[f64],
        adv_hone_strategy: &str,
        juice_books_budget: &[(i64, i64)],
        juice_prices: &[(f64, f64)],
        inp_leftover_values: &[f64],
        inp_leftover_juice_values: &[(f64, f64)],
        progress_grid: Option<Vec<Vec<usize>>>,
        state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
        special_state: Option<Vec<usize>>,
        unlock_grid: Option<Vec<Vec<bool>>>,
        succeeded_grid: Option<Vec<Vec<bool>>>,
        min_resolution: usize,
        num_threads: usize,
        metric_type: i64,
    ) -> StateBundle {
        // web_sys::console::log_1(&"1".into());
        let (prep_output, upgrade_arr): (PreparationOutput, Vec<Upgrade>) =
            PreparationOutput::initialize(
                hone_ticks,
                input_budgets,
                adv_ticks,
                express_event,
                inp_price_arr,
                adv_hone_strategy,
                juice_books_budget,
                juice_prices,
                inp_leftover_values,
                inp_leftover_juice_values,
                progress_grid,
                state_grid,
                unlock_grid,
                succeeded_grid,
            );
        let u_len = upgrade_arr.len();
        // web_sys::console::log_1(&"2".into());
        let out = StateBundle {
            upgrade_arr,
            special_state: if special_state.is_none()
                || special_state.as_ref().unwrap().len() != u_len
            {
                (0..u_len).collect()
            } else {
                special_state.unwrap()
            },
            special_invalid_index: None,
            metric_type: -1,
            metric: -1.0,
            prep_output,
            special_cache: HashMap::new(),
            latest_special_probs: None,
            min_resolution,
            num_threads,
        };
        // web_sys::console::log_1(&"3".into());

        out
    }
    pub fn init_from_payload(payload: Payload) -> Self {
        StateBundle::init_from_inputs(
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
            payload.min_resolution,
            payload.num_threads,
            payload.metric_type,
        )
    }
}
