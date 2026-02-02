use std::f64::NAN;

use super::constants::*;
use super::simulated_annealing::my_push;

use super::scaler::AdaptiveScaler;
use hf_core::performance::Performance;

#[cfg(target_arch = "wasm32")]
use hf_core::send_progress::send_progress;
use hf_core::state_bundle::StateBundle;
use hf_core::state_bundle::StateEssence;

use ordered_float::Float;
use ordered_float::OrderedFloat;
use rand::seq::IteratorRandom;

use priority_queue::DoublePriorityQueue;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

pub struct SolverStateBundle {
    pub state_bundle: StateBundle,
    pub scaler: AdaptiveScaler,

    pub temp: f64,

    pub rng: SmallRng,
    pub max_state_len: usize,

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
        max_state_len: usize,
        performance: Performance,
        seed: u64,
        best_n_states: &DoublePriorityQueue<StateEssence, OrderedFloat<f64>>,
        upgrade_impact: &Vec<f64>,
    ) -> Self {
        Self {
            state_bundle: state_bundle.clone(),
            scaler,

            temp: NAN,
            special_affinity: 0.9,
            rng: SmallRng::seed_from_u64(seed),
            max_state_len,

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

        // let random_idx = self.rng.random_range(1..self.best_n_states.len());
        // let partner_essence = self.best_n_states.iter().nth(random_idx).unwrap().0;

        // if self.rng.random_bool(1.0 - self.progress()) {
        //     self.state_bundle
        //         .special_state
        //         .clone_from(&partner_essence.special_state);
        // }
        // let progress = self.progress();
        // let random_indices = (0..self.state_bundle.upgrade_arr.len()).choose_multiple(
        //     &mut self.rng,
        //     ((1.0 - progress) * (self.state_bundle.upgrade_arr.len() - 1) as f64).ceil() as usize,
        // );

        // for index in random_indices {
        //     let this = &mut self.state_bundle.upgrade_arr[index].state;
        //     this.payload.clone_from(&partner_essence.state_arr[index]);
        //     this.update_hash();
        // }
    }
    pub fn current_resolution(&self) -> usize {
        if self.progress() > COOLING_PHASE_START {
            (self.lam_rate() / MAGIC_NUMBER).ceil() as usize * DEFAULT_RESOLUTION
        } else {
            self.state_bundle.min_resolution
        }
    }
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
