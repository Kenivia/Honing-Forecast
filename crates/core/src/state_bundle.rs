use crate::parser::PreparationOutput;

use crate::performance::Performance;
use crate::upgrade::Upgrade;
use std::collections::HashMap;

use std::f64::NAN;

#[derive(Clone, Debug)]
pub struct StateBundle {
    pub upgrade_arr: Vec<Upgrade>,
    pub special_state: Vec<usize>, // arbitrary length

    pub metric_type: i64,
    pub metric: f64,
    pub state_index: Vec<Vec<Vec<i64>>>, // i pre-added this for caching but havnt implemented anything
    pub prep_output: PreparationOutput,
    pub special_cache: HashMap<Vec<usize>, Vec<f64>>,
    // pub performance: Performance,
}

impl StateBundle {
    pub fn metric_router(&mut self, metric_type: i64, performance: &mut Performance) -> f64 {
        match metric_type {
            0 => self.success_prob_metric(performance),
            1 => self.average_gold_metric(performance),
            _ => NAN,
        }
    }

    pub fn update_state_hash(&mut self) {
        for upgrade in self.upgrade_arr.iter_mut() {
            upgrade.state.update_hash();
        }
    }

    pub fn new(prep_output: PreparationOutput, upgrade_arr: Vec<Upgrade>) -> StateBundle {
        let mut starting_special: Vec<usize> = Vec::with_capacity(upgrade_arr.len() * 2);
        for (index, _upgrade) in upgrade_arr.iter().enumerate() {
            starting_special.push(index); //, (1.0 / upgrade.base_chance).round() as usize));
        }

        let state_bundle: StateBundle = StateBundle {
            state_index: vec![],
            metric: -1.0,
            special_state: starting_special,
            prep_output,
            special_cache: HashMap::new(),
            upgrade_arr,
            metric_type: -1,
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
