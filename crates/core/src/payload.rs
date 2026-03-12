//! Payload is how js and rust communicates, we also use payload as our test cases in arena
use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::parser::PreparationOutput;
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone, Serialize)]
pub struct Payload {
    pub normal_hone_ticks: Vec<Vec<bool>>,
    pub adv_hone_ticks: Vec<Vec<bool>>,
    pub express_event: bool,

    pub inp_bound_mats: Vec<i64>,
    pub inp_trade_mats: Vec<i64>,

    pub inp_market_mats_price: Vec<f64>,
    pub inp_trade_mats_price: Vec<f64>,
    pub inp_left_mats_price: Vec<f64>,

    pub inp_bound_juice: Vec<(i64, i64)>,
    pub inp_trade_juice: Vec<(i64, i64)>,

    pub inp_juice_market_price: Vec<(f64, f64)>,
    pub inp_juice_trade_price: Vec<(f64, f64)>,
    pub inp_juice_left_price: Vec<(f64, f64)>,
    // i have to initialize these arrays in JS anyway (instead of leaving as null)
    // but this allows me to easily use default values ig (but I'd plug payloads in when testing anyway so ig theres not much point having Option at all)
    pub normal_progress_grid: Option<Vec<Vec<usize>>>,
    pub state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
    pub special_state: Option<Vec<usize>>,
    pub unlock_grid: Option<Vec<Vec<bool>>>,
    pub succeeded_grid: Option<Vec<Vec<bool>>>,
    pub adv_progress_grid: Option<Vec<Vec<(usize, usize, bool, bool)>>>,
    pub tier: usize,

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
        normal_hone_ticks: &[Vec<bool>],
        adv_ticks: &[Vec<bool>],
        express_event: bool,

        inp_bound_mats: &[i64],
        inp_trade_mats: &[i64],

        inp_market_mats_price: &[f64],
        inp_trade_mats_price: &[f64],
        inp_left_mats_price: &[f64],

        inp_bound_juice: &[(i64, i64)],
        inp_trade_juice: &[(i64, i64)],

        inp_juice_market_price: &[(f64, f64)],
        inp_juice_trade_price: &[(f64, f64)],
        inp_juice_left_price: &[(f64, f64)],

        normal_progress_grid: Option<Vec<Vec<usize>>>,
        state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
        unlock_grid: Option<Vec<Vec<bool>>>,
        succeeded_grid: Option<Vec<Vec<bool>>>,
        adv_progress_grid: Option<Vec<Vec<(usize, usize, bool, bool)>>>,
        tier: usize,

        special_state: Option<Vec<usize>>,
        min_resolution: usize,
        num_threads: usize,
        metric_type: i64,
    ) -> StateBundle {
        let (prep_output, upgrade_arr, adv_cache): (
            PreparationOutput,
            Vec<Upgrade>,
            AHashMap<AdvConfig, AdvDistTriplet>,
        ) = PreparationOutput::initialize(
            normal_hone_ticks,
            adv_ticks,
            express_event,
            inp_bound_mats,
            inp_trade_mats,
            inp_market_mats_price,
            inp_trade_mats_price,
            inp_left_mats_price,
            inp_bound_juice,
            inp_trade_juice,
            inp_juice_market_price,
            inp_juice_trade_price,
            inp_juice_left_price,
            normal_progress_grid,
            state_grid,
            unlock_grid,
            succeeded_grid,
            adv_progress_grid,
            tier,
        );
        let u_len = upgrade_arr.len();
        // web_sys::console::log_1(&"2".into());

        StateBundle {
            upgrade_arr,
            special_state: if special_state.is_none()
                || special_state.as_ref().unwrap().len() != u_len
            {
                (0..u_len).collect()
            } else {
                special_state.unwrap()
            },
            special_invalid_index: None,
            metric_type,
            metric: -1.0,
            prep_output,
            special_cache: AHashMap::new(),
            latest_special_probs: None,
            min_resolution,
            num_threads,
            average_breakdown: None,
            adv_cache,
        }
    }
    pub fn init_from_payload(payload: Payload) -> Self {
        StateBundle::init_from_inputs(
            &payload.normal_hone_ticks,
            &payload.adv_hone_ticks,
            payload.express_event,
            &payload.inp_bound_mats,
            &payload.inp_trade_mats,
            &payload.inp_market_mats_price,
            &payload.inp_trade_mats_price,
            &payload.inp_left_mats_price,
            &payload.inp_bound_juice,
            &payload.inp_trade_juice,
            &payload.inp_juice_market_price,
            &payload.inp_juice_trade_price,
            &payload.inp_juice_left_price,
            payload.normal_progress_grid,
            payload.state_grid,
            payload.unlock_grid,
            payload.succeeded_grid,
            payload.adv_progress_grid,
            payload.tier,
            payload.special_state,
            payload.min_resolution,
            payload.num_threads,
            payload.metric_type,
        )
    }
}

pub fn parse_to_payloads(path: &Path) -> Vec<(String, Payload)> {
    if !path.exists() {
        return Vec::new();
    }
    let mut entries: Vec<_> = fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();
    entries.sort_by_key(|entry| entry.path());

    let mut out: Vec<(String, Payload)> = Vec::new();
    for entry in entries {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }
        if file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            != Some(true)
        {
            continue;
        }
        let test_case_name = file_path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let contents = fs::read_to_string(&file_path).unwrap();
        let payload: Payload = serde_json::from_str(&contents).unwrap();
        out.push((test_case_name, payload));
    }
    out
}

pub fn parse_to_state_bundles(path: &Path) -> Vec<(String, StateBundle)> {
    let mut out: Vec<(String, StateBundle)> = Vec::new();
    for (test_case_name, payload) in parse_to_payloads(path) {
        let state_bundle = StateBundle::init_from_payload(payload);

        out.push((test_case_name, state_bundle));
    }
    out
}
