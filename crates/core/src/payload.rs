//! Payload is how js and rust communicates, we also use payload as our test cases in arena
use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::parser::{MaterialInput, PreparationOutput, UpgradeInput};
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone, Serialize)]
pub struct Payload {
    pub material_info: MaterialInput,
    pub upgrade_info: UpgradeInput,
    pub special_budget: i64,
    pub special_state: Option<Vec<usize>>,
    pub tier: usize,
    pub express_event: bool,

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
        material_info: MaterialInput,
        upgrade_info: UpgradeInput,
        special_budget: i64,
        express_event: bool,
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
            material_info,
            upgrade_info,
            special_budget,
            express_event,
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
            gold_breakdown: None,
            adv_cache,
            average_breakdown: None,
        }
    }
    pub fn init_from_payload(payload: Payload) -> Self {
        StateBundle::init_from_inputs(
            payload.material_info,
            payload.upgrade_info,
            payload.special_budget,
            payload.express_event,
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
