use crate::constants::FLOAT_TOL;
use crate::performance::Performance;
use crate::state::StateBundle;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
pub static MAX_BRUTE_SIZE: usize = 50000;

// 1. Helper wrapper to use f64 as a HashMap key
// (Rust floats don't implement Eq/Hash by default due to NaN)
#[derive(Clone, Copy, Debug)]
struct FloatKey(f64);

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

pub fn brute_success_prob(
    prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    budget: f64,
) -> f64 {
    let n = prob_dist_arr.len();

    // --- STEP 1: Pre-calculate Look-Ahead Bounds ---
    // min_suffix[i] = sum of minimum costs from layer i to end
    // max_suffix[i] = sum of maximum costs from layer i to end
    let mut min_suffix = vec![0.0; n + 1];
    let mut max_suffix = vec![0.0; n + 1];

    for i in (0..n).rev() {
        // Find min and max cost for this specific layer
        // We use fold because f64 doesn't implement Ord
        let (local_min, local_max) = support_arr[i]
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &val| {
                (min.min(val), max.max(val))
            });

        min_suffix[i] = min_suffix[i + 1] + local_min;
        max_suffix[i] = max_suffix[i + 1] + local_max;
    }

    // --- STEP 2: Iterative DP with Pruning ---

    // Stores currently active uncertain states
    let mut current_states: HashMap<FloatKey, f64> = HashMap::with_capacity(16);
    current_states.insert(FloatKey(0.0), 1.0);

    // Accumulator for paths that are guaranteed to stay under budget
    let mut total_guaranteed_prob = 0.0;

    for i in 0..n {
        let probs = &prob_dist_arr[i];
        let costs = &support_arr[i];
        let next_min_rem = min_suffix[i + 1];
        let next_max_rem = max_suffix[i + 1];

        // Prepare next layer map
        let mut next_states: HashMap<FloatKey, f64> =
            HashMap::with_capacity(current_states.len() * costs.len());

        for (key, current_prob) in current_states.drain() {
            let current_cost = key.0;

            for (idx, &step_cost) in costs.iter().enumerate() {
                let new_cost = current_cost + step_cost;
                let step_prob = current_prob * probs[idx];

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
    }

    // The answer is the sum of paths that were "Guaranteed Early"
    // + paths that survived to the very end validly.
    total_guaranteed_prob + current_states.values().sum::<f64>()
}
// fn brute_naive_recursive(
//     prob_dist_arr: &[Vec<f64>],
//     support_arr: &[Vec<f64>],
//     cost_so_far: f64,
//     budget: f64,
//     depth: usize,
// ) -> f64 {
//     let prob_dist: &Vec<f64> = &prob_dist_arr[depth];

//     let cost_arr: &Vec<f64> = &support_arr[depth];

//     if depth == prob_dist_arr.len() - 1 {
//         return cost_arr
//             .iter()
//             .enumerate()
//             .take_while(|(_, cost)| cost_so_far + **cost <= budget)
//             .fold(0.0, |acc, (index, _)| acc + prob_dist[index]);
//     } else {
//         return cost_arr.iter().enumerate().fold(0.0, |acc, (index, cost)| {
//             let new_cost: f64 = cost_so_far + *cost;
//             acc + if budget < new_cost {
//                 0.0
//             } else {
//                 prob_dist[index]
//                     * brute_naive_recursive(
//                         prob_dist_arr,
//                         support_arr,
//                         new_cost,
//                         budget,
//                         depth + 1,
//                         // cache,
//                     )
//             }
//         });
//     }
// }

// this is actually just another wrapper

// naive as in it doesn't take into account leftover prices
pub fn brute_success_prob_metric(
    state_bundle: &mut StateBundle,
    performance: &mut Performance,
) -> f64 {
    performance.states_evaluated += 1;
    performance.brute_count += 1;
    state_bundle.update_dist();

    state_bundle.update_combined();
    // brute_naive(&prob_dist_arr, &combined_costs, prep_output.eqv_gold_budget)

    brute_success_prob(
        &state_bundle.gather_prob_dist(),
        &state_bundle.gather_combined_gold_cost(),
        state_bundle.prep_output.eqv_gold_budget,
    )
}
