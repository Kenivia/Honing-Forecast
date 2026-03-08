// use std::f64::NAN;

use super::constants::*;
use super::scaler::AdaptiveScaler;
use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use crate::state_bundle::StateEssence;
use ordered_float::OrderedFloat;
use priority_queue::DoublePriorityQueue;
use rand::SeedableRng;
use rand::random_bool;
use rand::random_range;
use rand::rngs::SmallRng;

pub struct SolverStateBundle {
    pub state_bundle: StateBundle,
    pub scaler: AdaptiveScaler,
    // pub temp: f64,
    pub rng: SmallRng,
    // pub max_state_len: usize,
    pub performance: Performance,
    // pub best_state_so_far: StateBundle,
    pub best_n_states: DoublePriorityQueue<StateEssence, OrderedFloat<f64>>,
    pub prev_state: StateBundle,
    pub count: i64,
    pub temps_without_improvement: i64,
    pub upgrade_impact: Vec<f64>,
    pub special_affinity: f64,
}

impl SolverStateBundle {
    pub fn initialize(
        state_bundle: &StateBundle,
        scaler: AdaptiveScaler,
        _max_state_len: usize,
        performance: Performance,
        seed: u64,
        best_n_states: &DoublePriorityQueue<StateEssence, OrderedFloat<f64>>,
        upgrade_impact: &Vec<f64>,
    ) -> Self {
        Self {
            state_bundle: state_bundle.clone(),
            scaler,

            // temp: NAN,
            special_affinity: 0.9,
            rng: SmallRng::seed_from_u64(seed),
            // max_state_len,
            performance,
            best_n_states: best_n_states.clone(),
            prev_state: state_bundle.clone(),
            count: 0,
            temps_without_improvement: 0,
            upgrade_impact: upgrade_impact.clone(),
        }
    }
    pub fn perform_crossover(&mut self) {
        if self.best_n_states.len() < 2 {
            return;
        }
        let best = self.best_n_states.peek_max().unwrap();
        self.state_bundle.clone_from_essence(best.0, best.1);
        if random_bool(self.progress()) {
            let u_len = self.state_bundle.upgrade_arr.len();
            let target_idx = random_range(0..u_len);
            let target = &self.state_bundle.upgrade_arr[target_idx];

            if !target.is_normal_honing || target.is_weapon || target.alr_failed != 0 {
                return;
            }

            let identical: Vec<usize> = self
                .state_bundle
                .upgrade_arr
                .iter()
                .enumerate()
                .filter(|(i, upgrade)| {
                    *i != target_idx
                        && upgrade.upgrade_index == target.upgrade_index
                        && upgrade.piece_type != target.piece_type
                })
                .map(|(i, _)| i)
                .collect();

            if !identical.is_empty()
                && let Some(&chosen_idx) = identical.get(random_range(0..identical.len()))
            {
                let payload = self.state_bundle.upgrade_arr[chosen_idx]
                    .state
                    .payload
                    .clone();

                self.state_bundle.upgrade_arr[target_idx]
                    .state
                    .update_payload(payload);
            }
        }
    }
    // pub fn current_resolution(&self) -> usize {
    //     if self.progress() > COOLING_PHASE_START {
    //         (self.lam_rate() / MAGIC_NUMBER).ceil() as usize * DEFAULT_RESOLUTION
    //     } else {
    //         self.state_bundle.min_resolution
    //     }
    // }
    pub fn progress(&self) -> f64 {
        self.count as f64 / MAX_ITERS as f64
    }

    pub fn lam_rate(&self) -> f64 {
        if self.progress() < WARM_UP_PHASE_END {
            MAGIC_NUMBER
                + (1.0 - MAGIC_NUMBER)
                    * (MAGIC_NUMBER * 1000.0).powf(-self.progress() / WARM_UP_PHASE_END)
        } else if self.progress() < COOLING_PHASE_START {
            MAGIC_NUMBER
        } else {
            MAGIC_NUMBER
                * (MAGIC_NUMBER * 1000.0)
                    .powf(-(self.progress() - COOLING_PHASE_START) / (1.0 - COOLING_PHASE_START)) // i mean this 1000 seems to work fine for all MAX_ITERS so whatever
        }
    }
    // pub fn one_batch(&mut self, batch_iters: i64) {

    // }
}
