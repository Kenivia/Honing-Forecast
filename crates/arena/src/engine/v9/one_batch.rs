use super::core::ITERS_PER_TEMP;
use super::core::new_temp;
use super::simulated_annealing::my_push;

use super::core::neighbour;
use super::scaler::AdaptiveScaler;
use hf_core::performance::Performance;

#[cfg(target_arch = "wasm32")]
use hf_core::send_progress::send_progress;
use hf_core::state_bundle::StateBundle;
use hf_core::state_bundle::StateEssence;

use ordered_float::OrderedFloat;

use super::core::{INIT_TEMP, RESOLUTION_CUTOFF_TEMP};
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
}

impl SolverStateBundle {
    pub fn initialize(
        state_bundle: &StateBundle,
        scaler: AdaptiveScaler,
        max_state_len: usize,
        performance: Performance,
        seed: u64,

        best_n_states: &DoublePriorityQueue<StateEssence, OrderedFloat<f64>>,
    ) -> Self {
        Self {
            state_bundle: state_bundle.clone(),
            scaler,

            temp: INIT_TEMP,

            rng: SmallRng::seed_from_u64(seed),
            max_state_len,
            performance,
            best_n_states: best_n_states.clone(),
            prev_state: state_bundle.clone(),
            count: 0,
            temps_without_improvement: 0,
        }
    }

    pub fn one_batch(&mut self, batch_iters: i64) {
        for _ in 0..batch_iters {
            let current_resolution = if self.temp > RESOLUTION_CUTOFF_TEMP {
                // Logarithmic interpolation from Max Len -> Min Res
                let log_curr = self.temp.ln();
                let log_start = INIT_TEMP.ln();
                let log_end = RESOLUTION_CUTOFF_TEMP.ln();

                // 0.0 at cutoff, 1.0 at start
                let ratio = ((log_curr - log_end) / (log_start - log_end)).clamp(0.0, 1.0);

                ((self.state_bundle.min_resolution as f64
                    + (self.max_state_len as f64 - self.state_bundle.min_resolution as f64)
                        * ratio)
                    / self.state_bundle.min_resolution as f64)
                    .floor() as usize
                    * self.state_bundle.min_resolution
            } else {
                self.state_bundle.min_resolution
            };

            neighbour(
                &mut self.state_bundle,
                self.temp,
                current_resolution,
                &mut self.rng,
            );
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
                .update_stats(is_uphill, accepted, (self.temp / INIT_TEMP) * 0.69);
            self.count += 1;

            if self.count > ITERS_PER_TEMP {
                self.count = 0;
                if self.temps_without_improvement as f64 > (1.0 * self.temp).max(3.0) {
                    let chosen_index: usize = self.rng.random_range(0..self.best_n_states.len());
                    for (index, (chosen, metric)) in self.best_n_states.iter().enumerate() {
                        if index < chosen_index {
                            continue;
                        }
                        self.state_bundle.clone_from_essence(chosen, metric);
                        self.temps_without_improvement = 0;
                        break;
                    }
                }
                self.temps_without_improvement += 1;
                self.temp = new_temp(self.temp);
            }
        }
    }
}
