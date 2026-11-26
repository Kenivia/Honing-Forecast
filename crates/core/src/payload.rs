//! Payload is how js and rust communicates, we also use payload as our test cases in arena
use crate::constants::BASE_JUICE_INFOS;
use crate::constants::juice_info::get_event_adjusted_juice_info;
use crate::honing_utils::chest::{ChestStates, Chests};
use crate::honing_utils::materials::{MaterialTable, OneMaterial, ValuationPlan};
use crate::honing_utils::state_bundle::StateBundle;
use crate::optimizer::maxroll::Maxroll;
use crate::parser::{AdvCache, OneUpgradeInput, PreparationOutput, UpgradeCtx, kept_adv_cache};
use crate::tuning::Tuning;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

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
    /// unread; kept because the arena's step hashes include every field
    #[serde(default = "default_one")]
    pub metric_type: i64,

    pub adv_cache: Option<AdvCache>,
    /// the plan the histogram draws the gold distribution of, none for no distribution
    #[serde(default)]
    pub dist_plan: Option<usize>,
    /// the run gone on from was of nearly this payload (a choice's option against the budgets
    /// as they are), so one fresh chain that finds nothing better is enough; never hashed
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub quick: bool,
    /// the parts of the plan the histogram takes by Maxroll's rule
    #[serde(default, skip_serializing)]
    pub maxroll: Maxroll,
}
fn default_one() -> i64 {
    1
}

impl Payload {
    /// One step of a scenario file (`arena_data/scenarios`), for tests and experiments.
    pub fn load_step(path: &str, step: usize) -> Payload {
        let file: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        serde_json::from_value(file["steps"][step].clone()).unwrap()
    }
}

impl StateBundle {
    /// `dist_plan` and `maxroll` are left for the caller.
    pub fn from_payload(payload: Payload) -> StateBundle {
        let Payload {
            material_labels,
            materials,
            plans,
            optimizer_plan,
            upgrade_info,
            special_budget,
            special_state,
            chests,
            chest_states,
            special_label,
            tier,
            express_event,
            min_resolution,
            num_threads,
            adv_cache,
            quick,
            ..
        } = payload;
        assert!(optimizer_plan < plans.len());

        let juice_info = get_event_adjusted_juice_info(&BASE_JUICE_INFOS[tier], express_event);
        let table = MaterialTable::new(material_labels, juice_info.num_juice_avail);
        let materials: Vec<OneMaterial> = table.in_row_order(&materials);

        let mut adv_cache = kept_adv_cache(adv_cache, &upgrade_info);
        let ctx = UpgradeCtx::new(express_event, tier, &juice_info);
        let upgrade_arr: Vec<Upgrade> = upgrade_info
            .into_iter()
            .map(|input| Upgrade::from_input(input, &ctx, &mut adv_cache))
            .collect();

        let u_len = upgrade_arr.len();
        let special_state = special_state
            .filter(|order| order.len() == u_len)
            .unwrap_or_else(|| (0..u_len).collect());
        // a given state that does not fit its chest falls back to the default
        let given = chest_states.unwrap_or_default();
        let chest_states: ChestStates = chests
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
            special_state,
            chest_states,
            special_invalid_index: None,
            metric: -1.0,
            prep_output: PreparationOutput {
                table,
                base_materials: materials.clone(),
                materials,
                chests,
                special_label,
                plans,
                optimizer_plan,
                bands: Vec::new(), // apply_chest_state
                special_budget,
                base_special_budget: special_budget,
                test_case: -1,
                juice_info,
            },
            special_cache: AHashMap::new(),
            latest_special_probs: None,
            min_resolution,
            num_threads,
            adv_cache,
            gold_memo: AHashMap::new(),
            coarse_memo: Default::default(),
            special_done: None,
            tuning: Tuning {
                quick,
                ..Default::default()
            },
        };
        out.apply_chest_state(false);
        out
    }
}
