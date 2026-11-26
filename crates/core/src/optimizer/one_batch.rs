//! One chain's working state: the state being moved, the one it came from and the best seen.

use super::scaler::AdaptiveScaler;
use crate::honing_utils::state_bundle::StateBundle;
use crate::honing_utils::state_bundle::StateEssence;
use crate::probe::Probe;
use crate::upgrade::PieceType::Armor;
use ordered_float::OrderedFloat;
use priority_queue::DoublePriorityQueue;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

pub struct SolverStateBundle {
    pub state_bundle: StateBundle,
    pub scaler: AdaptiveScaler,
    pub rng: SmallRng,
    /// the best `Tuning::max_best_size` states seen
    pub best_n_states: DoublePriorityQueue<StateEssence, OrderedFloat<f64>>,
    pub prev_state: StateBundle,
    pub count: i64,
    pub temps_without_improvement: i64,
    pub upgrade_impact: Vec<f64>,
    pub special_affinity: f64,
    /// per upgrade, its state and the rest of the state when it last took its best plan
    pub settled: Vec<(u64, u64)>,
    /// evaluations the last move counts as, on top of its own
    pub extra_cost: i64,
}

impl SolverStateBundle {
    /// a chain from this evaluated state, the only best so far
    pub fn new(
        state_bundle: &StateBundle,
        scaler: AdaptiveScaler,
        seed: u64,
        upgrade_impact: Vec<f64>,
    ) -> Self {
        let mut out = Self {
            state_bundle: state_bundle.clone(),
            scaler,
            special_affinity: 0.9,
            rng: SmallRng::seed_from_u64(seed),
            best_n_states: DoublePriorityQueue::new(),
            prev_state: state_bundle.clone(),
            count: 0,
            temps_without_improvement: 0,
            upgrade_impact,
            settled: vec![(0, 0); state_bundle.upgrade_arr.len()],
            extra_cost: 0,
        };
        out.push(state_bundle.to_essence(), state_bundle.metric);
        out
    }

    fn push(&mut self, essence: StateEssence, metric: f64) {
        let queue = &mut self.best_n_states;
        if queue.len() < self.state_bundle.tuning.max_best_size {
            queue.push(essence, OrderedFloat(metric));
        } else if OrderedFloat(metric) > *queue.peek_min().unwrap().1 {
            queue.pop_min();
            queue.push(essence, OrderedFloat(metric));
        }
    }

    pub fn best_metric(&self) -> f64 {
        f64::from(*self.best_n_states.peek_max().unwrap().1)
    }

    /// a state among the best, written to the trace as one
    pub fn keep(&mut self, essence: StateEssence, metric: f64, probe: &mut Probe) {
        self.push(essence, metric);
        probe.put("best", metric);
    }

    pub fn keep_current(&mut self, probe: &mut Probe) {
        let metric = self.state_bundle.metric;
        self.keep(self.state_bundle.to_essence(), metric, probe);
    }

    /// the state being moved becomes the best seen
    pub fn load_best(&mut self) {
        let (essence, metric) = self.best_n_states.peek_max().unwrap();
        self.state_bundle.clone_from_essence(essence, metric);
    }

    /// The end of a polish from the best state: the state is kept where it beats `start`, and
    /// the chain goes on from it, evaluated again since a turned down move leaves its supports.
    pub fn settle(&mut self, start: f64, probe: &mut Probe) {
        if self.state_bundle.metric > start {
            self.keep_current(probe);
        }
        self.state_bundle.metric = self.state_bundle.evaluate(probe);
        self.prev_state.my_clone_from(&self.state_bundle);
    }

    pub fn perform_crossover(&mut self) {
        if self.best_n_states.len() < 2 {
            return;
        }
        self.load_best();
        if self.state_bundle.tuning.fresh_streaks {
            for upgrade in self.state_bundle.upgrade_arr.iter_mut() {
                upgrade.state.streak_cache = None;
            }
        }
        if self.rng.random_bool(self.progress()) {
            let u_len = self.state_bundle.upgrade_arr.len();
            let source_index = self.rng.random_range(0..u_len);
            let source = &self.state_bundle.upgrade_arr[source_index];

            if !source.is_normal_honing || source.piece_type != Armor {
                return;
            }

            let identical: Vec<usize> = self
                .state_bundle
                .upgrade_arr
                .iter()
                .enumerate()
                .filter(|(i, target)| {
                    *i != source_index
                        && target.is_normal_honing
                        && target.upgrade_index == source.upgrade_index
                        && target.piece_type == source.piece_type
                        // one further along has fewer taps left, and a plan of another length
                        && target.starting_artisan == source.starting_artisan
                        && target.starting_num_taps == source.starting_num_taps
                })
                .map(|(i, _)| i)
                .collect();

            if !identical.is_empty()
                && let Some(&chosen_idx) = identical.get(self.rng.random_range(0..identical.len()))
            {
                let payload = self.state_bundle.upgrade_arr[chosen_idx]
                    .state
                    .payload
                    .clone();

                self.state_bundle.upgrade_arr[source_index]
                    .state
                    .update_payload(payload);
            }
        }
    }
    pub fn progress(&self) -> f64 {
        self.count as f64 / self.state_bundle.tuning.max_iters as f64
    }

    pub fn lam_rate(&self) -> f64 {
        let magic_number = self.state_bundle.tuning.magic_number;
        let warm_up_phase_end = self.state_bundle.tuning.warm_up_phase_end;
        let cooling_phase_start = self.state_bundle.tuning.cooling_phase_start;
        if self.progress() < warm_up_phase_end {
            magic_number
                + (1.0 - magic_number)
                    * (magic_number * 1000.0).powf(-self.progress() / warm_up_phase_end)
        } else if self.progress() < cooling_phase_start {
            magic_number
        } else {
            magic_number
                * (magic_number * 1000.0)
                    .powf(-(self.progress() - cooling_phase_start) / (1.0 - cooling_phase_start)) // i mean this 1000 seems to work fine for all MAX_ITERS so whatever
        }
    }
}
