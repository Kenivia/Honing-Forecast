use crate::parser::PreparationOutput;
use crate::performance::Performance;
use crate::upgrade::{State, Upgrade};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::f64::NAN;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateBundle {
    pub upgrade_arr: Vec<Upgrade>,
    pub special_state: Vec<usize>, // arbitrary length

    pub special_invalid_index: Option<usize>,
    pub latest_special_probs: Option<Vec<f64>>,
    pub metric_type: i64,
    pub metric: f64,
    pub min_resolution: usize,
    // pub state_index: Vec<Vec<Vec<i64>>>, // i pre-added this for caching but havnt implemented anything
    pub prep_output: PreparationOutput,
    #[serde(skip)]
    pub special_cache: HashMap<Vec<usize>, Vec<f64>>,
    pub num_threads: usize, // #[serde(skip)]
                            // pub scaler: Adaptive, // pub performance: Performance,
}

pub fn default_special(length: usize) -> Vec<usize> {
    let mut starting_special: Vec<usize> = Vec::with_capacity(length);
    for index in 0..length {
        starting_special.push(index); //, (1.0 / upgrade.base_chance).round() as usize));
    }
    starting_special
}
pub fn default_state_arr(upgrade_arr: &Vec<Upgrade>) -> Vec<Vec<(bool, usize)>> {
    let mut out: Vec<Vec<(bool, usize)>> = Vec::with_capacity(upgrade_arr.len());
    for upgrade in upgrade_arr {
        out.push(State::new(upgrade.prob_dist.len()).payload.clone()); //, (1.0 / upgrade.base_chance).round() as usize));
    }
    out
}
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct StateBundleJs {
//     pub state_arr: Vec<Vec<(bool, usize)>>,
//     pub special_state: Vec<usize>,
// }
// impl StateBundleJs {
//     fn my_default(upgrade_arr: &Vec<Upgrade>) -> Self {
//         StateBundleJs {
//             state_arr: default_state_arr(upgrade_arr),
//             special_state: default_special(upgrade_arr.len()),
//         }
//     }
// }
impl StateBundle {
    // pub fn to_js(&self) -> StateBundleJs {
    //     let mut state_arr: Vec<Vec<(bool, usize)>> = Vec::with_capacity(self.upgrade_arr.len());
    //     for upgrade in self.upgrade_arr.iter() {
    //         state_arr.push(upgrade.state.to_vec());
    //     }
    //     StateBundleJs {
    //         state_arr,
    //         special_state: self.special_state.clone(),
    //     }
    // }
    pub fn set_latest_special_probs(&mut self) {
        let mut out = Vec::with_capacity(self.upgrade_arr.len());
        self.clean_special_state();
        for (index, _) in self.special_probs().iter().enumerate() {
            out.push(if index < self.special_invalid_index.unwrap() {
                self.special_probs().iter().skip(index + 1).sum::<f64>()
            } else {
                0.0
            });
        }
        self.latest_special_probs = Some(out);
    }
    // this should be built into neighbour
    // well actually maybe not
    // pub fn clean_state(&mut self) {
    //     for upgrade in self.upgrade_arr.iter_mut() {
    //         let p_len = upgrade.prob_dist.len();
    //         for (index, s) in upgrade.state.iter_mut().enumerate() {
    //             if index >= p_len - 2 {
    //                 // 1 for pity(you cant juice the pity tap), 1 for 0th tap
    //                 *s = (false, 0);
    //             }
    //         }
    //     }
    // }
    pub fn metric_router(&mut self, metric_type: i64, performance: &mut Performance) -> f64 {
        match metric_type {
            0 => self.success_prob_metric(performance),
            1 => self.average_gold_metric(performance),
            _ => NAN,
        }
    }

    // pub fn update_state_hash(&mut self) {
    //     for upgrade in self.upgrade_arr.iter_mut() {
    //         upgrade.state.update_hash();
    //     }
    // }

    pub fn new(prep_output: PreparationOutput, upgrade_arr: Vec<Upgrade>) -> StateBundle {
        let state_bundle: StateBundle = StateBundle {
            // state_index: vec![],
            special_invalid_index: None,
            metric: -1.0,
            special_state: default_special(upgrade_arr.len()),
            prep_output,
            special_cache: HashMap::new(),
            upgrade_arr,
            metric_type: -1,
            latest_special_probs: None,
            min_resolution: 1,
            num_threads: 0,
        };

        return state_bundle;
    }

    pub fn my_clone_from(&mut self, source: &StateBundle) {
        // update_special_cache: bool
        for (source, upgrade) in source.upgrade_arr.iter().zip(self.upgrade_arr.iter_mut()) {
            upgrade.state.clone_from(&source.state);
        }
        self.special_state.clone_from(&source.special_state);
        self.metric = source.metric;
        // if update_special_cache {
        //     for (k, v) in source.special_cache.iter() {
        //         self.special_cache.entry(k.clone()).or_insert(v.clone());
        //     }
        // }

        // metric type should be the same
    }
}
