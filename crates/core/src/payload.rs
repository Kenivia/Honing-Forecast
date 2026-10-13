//! Payload is how js and rust communicates, we also use payload as our test cases in arena
use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::chest::{ChestStates, Chests};
use crate::materials::{OneMaterial, ValuationPlan};
use crate::parser::{OneUpgradeInput, PreparationOutput};
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone, Serialize)]
pub struct Payload {
    /// row order, length NUM_BASE_MATS + number of juices for the tier
    pub material_labels: Vec<String>,
    pub materials: AHashMap<String, OneMaterial>,
    /// every plan to evaluate; the histogram reports one metric per plan
    pub plans: Vec<ValuationPlan>,
    pub optimizer_plan: usize,

    pub upgrade_info: Vec<OneUpgradeInput>,
    pub special_budget: i64,

    pub special_state: Option<Vec<usize>>,
    /// every chest stack, one-option ones included, already of this tier
    #[serde(default)]
    pub chests: Chests,
    #[serde(default)]
    pub chest_states: Option<ChestStates>,
    #[serde(default)]
    pub special_label: String,
    pub tier: usize,
    pub express_event: bool,

    pub min_resolution: usize,
    #[serde(default)]
    pub num_threads: usize,
    #[serde(default = "default_one")]
    pub metric_type: i64,

    pub adv_cache: Option<AHashMap<AdvConfig, AdvDistTriplet>>,
}
fn default_one() -> i64 {
    1
}
impl StateBundle {
    pub fn init_from_inputs(
        material_labels: Vec<String>,
        materials: AHashMap<String, OneMaterial>,
        plans: Vec<ValuationPlan>,
        optimizer_plan: usize,
        upgrade_info: Vec<OneUpgradeInput>,
        special_budget: i64,
        express_event: bool,
        tier: usize,
        special_state: Option<Vec<usize>>,
        chests: Chests,
        chest_states: Option<ChestStates>,
        special_label: String,
        min_resolution: usize,
        num_threads: usize,
        metric_type: i64,
        adv_cache: Option<AHashMap<AdvConfig, AdvDistTriplet>>,
    ) -> StateBundle {
        let (prep_output, upgrade_arr, adv_cache): (
            PreparationOutput,
            Vec<Upgrade>,
            AHashMap<AdvConfig, AdvDistTriplet>,
        ) = PreparationOutput::initialize(
            material_labels,
            materials,
            plans,
            optimizer_plan,
            upgrade_info,
            special_budget,
            chests,
            special_label,
            express_event,
            tier,
            adv_cache,
        );
        let u_len = upgrade_arr.len();
        // web_sys::console::log_1(&"2".into());
        let given = chest_states.unwrap_or_default();
        let chest_states: ChestStates = prep_output
            .chests
            .iter()
            .map(|(key, chest)| {
                let state = given
                    .get(key)
                    .filter(|state| chest.is_valid(state))
                    .cloned()
                    .unwrap_or_else(|| chest.default_state());
                (key.clone(), state)
            })
            .collect();

        let mut out = StateBundle {
            upgrade_arr,
            special_state: if special_state.is_none()
                || special_state.as_ref().unwrap().len() != u_len
            {
                (0..u_len).collect()
            } else {
                special_state.unwrap()
            },
            chest_states,
            special_invalid_index: None,
            metric_type,
            metric: -1.0,
            prep_output,
            special_cache: AHashMap::new(),
            latest_special_probs: None,
            min_resolution,
            num_threads,

            adv_cache,
            gold_memo: AHashMap::new(),
            special_done: None,
            tuning: Default::default(),
        };
        out.apply_chest_state(false);
        out
    }
    pub fn init_from_payload(payload: Payload) -> Self {
        StateBundle::init_from_inputs(
            payload.material_labels,
            payload.materials,
            payload.plans,
            payload.optimizer_plan,
            payload.upgrade_info,
            payload.special_budget,
            payload.express_event,
            payload.tier,
            payload.special_state,
            payload.chests,
            payload.chest_states,
            payload.special_label,
            payload.min_resolution,
            payload.num_threads,
            payload.metric_type,
            payload.adv_cache,
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
