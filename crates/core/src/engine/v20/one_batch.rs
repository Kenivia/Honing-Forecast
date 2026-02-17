use std::f64::NAN;

use super::constants::*;
use super::simulated_annealing::my_push;

use super::scaler::AdaptiveScaler;
use crate::performance::Performance;

#[cfg(target_arch = "wasm32")]
use crate::send_progress::send_progress;
use crate::state_bundle::StateBundle;
use crate::state_bundle::StateEssence;

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
        let progress = self.progress();

        let indices = (0..self.best_n_states.len()).choose_multiple(
            &mut self.rng,
            ((self.best_n_states.len() as f64 * (1.0 - progress) * 0.5)
                .round()
                .max(1.0)) as usize,
        );
        for random_idx in indices {
            let partner_essence = self.best_n_states.iter().nth(random_idx).unwrap().0;

            if self.rng.random_bool(1.0 - self.progress()) {
                self.state_bundle
                    .special_state
                    .clone_from(&partner_essence.special_state);
            }
            let progress = self.progress();
            let random_indices = (0..self.state_bundle.upgrade_arr.len()).choose_multiple(
                &mut self.rng,
                ((1.0 - progress) * (self.state_bundle.upgrade_arr.len() - 1) as f64).ceil()
                    as usize,
            );

            for index in random_indices {
                let this = &mut self.state_bundle.upgrade_arr[index].state;
                this.payload.clone_from(&partner_essence.state_arr[index]);
                this.update_hash();
            }
        }
    }
    pub fn current_resolution(&self) -> usize {
        if self.progress() > COOLING_PHASE_START {
            (self.lam_rate() / MAGIC_NUMBER).round() as usize * DEFAULT_RESOLUTION
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
    pub fn one_batch(&mut self, batch_iters: i64) {
        self.prev_state.my_clone_from(&self.state_bundle);
        for i in 0..batch_iters {
            if i >= 0 {
                self.neighbour();
            }

            self.state_bundle.metric = self.state_bundle.metric_router(&mut self.performance);

            // highest_seen = highest_seen.max(state_bundle.metric);
            // lowest_seen = lowest_seen.min(state_bundle.metric);

            if OrderedFloat(self.state_bundle.metric) > *self.best_n_states.peek_max().unwrap().1 {
                my_push(
                    &mut self.best_n_states,
                    self.state_bundle.to_essence(),
                    OrderedFloat(self.state_bundle.metric),
                );
                self.temps_without_improvement = 0;
            }

            let delta =
                (self.prev_state.metric - self.state_bundle.metric) / self.scaler.current_scale;
            let is_uphill = delta < 0.0; // Assuming maximization? Adjust if minimization.
            let accepted = if !is_uphill {
                true
            } else {
                let prob = (-delta.abs()).exp();
                self.rng.random_bool(prob)
            };
            if accepted {
                self.prev_state.my_clone_from(&self.state_bundle);
            } else {
                self.state_bundle.my_clone_from(&self.prev_state);
            }
            self.scaler
                .update_stats(is_uphill, accepted, self.lam_rate());
            self.count += 1;

            if self.count > ITERS_PER_TEMP {
                // self.count = 0;
                if self.temps_without_improvement as f64 > (1.0 * self.temp).max(3.0) {
                    self.perform_crossover();
                    self.temps_without_improvement = 0;
                }
                self.temps_without_improvement += 1;
                // self.temp = new_temp(self.temp);
            }
        }
    }
}
