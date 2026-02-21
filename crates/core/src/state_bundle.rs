use crate::parser::PreparationOutput;
use crate::performance::Performance;
use crate::upgrade::{State, Upgrade};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::NAN;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateBundle {
    pub upgrade_arr: Vec<Upgrade>,
    pub special_state: Vec<usize>,
    pub special_invalid_index: Option<usize>,
    pub latest_special_probs: Option<Vec<f64>>,
    pub metric_type: i64,
    pub metric: f64,
    pub min_resolution: usize,
    pub prep_output: PreparationOutput,
    #[serde(skip)]
    pub special_cache: HashMap<Vec<usize>, Vec<f64>>,
    pub num_threads: usize,
    pub average_breakdown: Option<Vec<f64>>,
}

pub fn default_state_arr(upgrade_arr: &Vec<Upgrade>) -> Vec<Vec<(bool, usize)>> {
    let mut out: Vec<Vec<(bool, usize)>> = Vec::with_capacity(upgrade_arr.len());
    for upgrade in upgrade_arr {
        out.push(State::new(upgrade.prob_dist.len()).payload.clone());
    }
    out
}
impl StateBundle {
    pub fn set_latest_special_probs(&mut self) {
        let mut out = Vec::with_capacity(self.upgrade_arr.len());
        self.clean_special_state();
        let dist = self.compute_special_probs(true).unwrap();
        for (index, _) in dist.iter().enumerate() {
            out.push(if index < self.special_invalid_index.unwrap() {
                dist.iter().skip(index + 1).sum::<f64>()
            } else {
                0.0
            });
        }
        self.latest_special_probs = Some(out);
    }

    pub fn metric_router(&mut self, performance: &mut Performance) -> f64 {
        match self.metric_type {
            0 => self.success_prob_metric(performance),
            1 => self.average_gold_metric(performance),
            _ => NAN,
        }
    }

    pub fn new(prep_output: PreparationOutput, upgrade_arr: Vec<Upgrade>) -> StateBundle {
        let state_bundle: StateBundle = StateBundle {
            special_invalid_index: None,
            metric: -1.0,
            special_state: (0..upgrade_arr.len()).collect(),
            prep_output,
            special_cache: HashMap::new(),
            upgrade_arr,
            metric_type: -1,
            latest_special_probs: None,
            min_resolution: 1,
            num_threads: 0,
            average_breakdown: None,
        };

        state_bundle
    }

    pub fn my_clone_from(&mut self, source: &StateBundle) {
        for (s, upgrade) in source.upgrade_arr.iter().zip(self.upgrade_arr.iter_mut()) {
            upgrade.state.clone_from(&s.state);
        }
        self.special_state.clone_from(&source.special_state);
        self.metric = source.metric;
    }

    pub fn to_essence(&self) -> StateEssence {
        StateEssence {
            state_arr: self
                .upgrade_arr
                .iter()
                .map(|x| x.state.payload.clone())
                .collect(),
            special_state: self.special_state.clone(),
        }
    }

    pub fn clone_from_essence(&mut self, source: &StateEssence, input_metric: &OrderedFloat<f64>) {
        for (s, upgrade) in source.state_arr.iter().zip(self.upgrade_arr.iter_mut()) {
            upgrade.state.payload.clone_from(s);
            upgrade.state.update_hash();
        }
        self.special_state.clone_from(&source.special_state);
        self.metric = f64::from(*input_metric);
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct StateEssence {
    pub state_arr: Vec<Vec<(bool, usize)>>,
    pub special_state: Vec<usize>,
}
