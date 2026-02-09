//! Serves in place of saddlepoint_approximation when the complexity isn't too bad
//! Mostly vibe coded

use crate::constants::FLOAT_TOL;
use crate::state_bundle::StateBundle;
use std::collections::HashMap;
use std::f64::{INFINITY, NEG_INFINITY};
use std::hash::{Hash, Hasher};

pub const MAX_BRUTE_SIZE: usize = 50000;

// 1. Helper wrapper to use f64 as a HashMap key
// (Rust floats don't implement Eq/Hash by default due to NaN)
#[derive(Clone, Copy, Debug)]
pub struct FloatKey(f64);

impl PartialEq for FloatKey {
    fn eq(&self, other: &Self) -> bool {
        ((self.0 * 1.0e6).round() / 1.0e6).to_bits()
            == ((other.0 * 1.0e6).round() / 1.0e6).to_bits()
    }
}
impl Eq for FloatKey {}
impl Hash for FloatKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.0 * 1.0e6).round() / 1.0e6).to_bits().hash(state);
    }
}

impl StateBundle {
    pub fn brute_success_prob(
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        mean: f64,
        biased: bool,
    ) -> f64 {
        let n = self.upgrade_arr.len();
        let inv_mean = if mean.abs() < FLOAT_TOL {
            0.0
        } else {
            mean.recip()
        };
        // --- STEP 1: Pre-calculate Look-Ahead Bounds ---
        // min_suffix[i] = sum of minimum costs from layer i to end
        // max_suffix[i] = sum of maximum costs from layer i to end
        let mut min_suffix = vec![0.0; n + 1];
        let mut max_suffix = vec![0.0; n + 1];

        let mut u_index = n - 1;

        for support in self.extract_collapsed_pair(support_index, skip_count).rev() {
            // Find min and max cost for this specific layer
            // We use fold because f64 doesn't implement Ord

            min_suffix[u_index] = min_suffix[u_index + 1]
                + support.iter().fold(INFINITY, |prev, new| prev.min(new.0));
            max_suffix[u_index] = max_suffix[u_index + 1]
                + support
                    .iter()
                    .fold(NEG_INFINITY, |prev, new| prev.max(new.0));
            u_index = u_index.saturating_sub(1);
        }
        // --- STEP 2: Iterative DP with Pruning ---

        // Stores currently active uncertain states
        let mut current_states: HashMap<FloatKey, f64> = HashMap::with_capacity(16);
        current_states.insert(FloatKey(0.0), 1.0);

        // Accumulator for paths that are guaranteed to stay under budget
        let mut total_guaranteed_prob = 0.0;

        let mut index = 0;
        for pairs in self.extract_collapsed_pair(support_index, skip_count) {
            let next_min_rem = min_suffix[index + 1];
            let next_max_rem = max_suffix[index + 1];

            // Prepare next layer map
            let mut next_states: HashMap<FloatKey, f64> =
                HashMap::with_capacity(current_states.len() * pairs.len());

            for (key, current_prob) in current_states.drain() {
                let current_cost = key.0;

                for (s, p) in pairs {
                    let new_cost = current_cost + s;
                    let step_prob = current_prob * p * if biased { s * inv_mean } else { 1.0 };

                    // PRUNE 1: Guaranteed Success
                    // If even the most expensive future choices fit in the budget,
                    // we don't need to simulate them. We just bank the probability.
                    if new_cost + next_max_rem <= budget - FLOAT_TOL {
                        total_guaranteed_prob += step_prob;
                        continue;
                    }

                    // PRUNE 2: Guaranteed Failure
                    // If even the cheapest future choices exceed the budget,
                    // drop this path immediately.
                    if new_cost + next_min_rem > budget + FLOAT_TOL {
                        continue;
                    }

                    // UNCERTAIN: Must keep calculating
                    let new_key = FloatKey(new_cost);
                    *next_states.entry(new_key).or_insert(0.0) += step_prob;
                }
            }

            current_states = next_states;

            // Optimization: If we have no uncertain states left, we are done
            if current_states.is_empty() {
                break;
            }
            index += 1;
        }

        // The answer is the sum of paths that were "Guaranteed Early"
        // + paths that survived to the very end validly.
        
        total_guaranteed_prob + current_states.values().sum::<f64>()
    }
}
