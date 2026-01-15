use crate::constants::FLOAT_TOL;

use crate::state_bundle::StateBundle;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
pub static MAX_BRUTE_SIZE: usize = 500;

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
    let out = total_guaranteed_prob + current_states.values().sum::<f64>();
    if !out.is_finite() {
        dbg!(
            prob_dist_arr,
            support_arr,
            total_guaranteed_prob,
            current_states.values().sum::<f64>()
        );
        panic!();
    }
    out
}
/// Helper struct to hold lookahead bounds
struct LookaheadBounds {
    min_suffix: Vec<f64>,
    max_suffix: Vec<f64>,
}

pub fn brute_average_recursive(
    prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    budget: f64,
    mean: f64,
) -> f64 {
    // 1. PRE-COMPUTE BOUNDS
    // We scan backwards to calculate the min/max cost possible from each depth.
    // min_suffix[i] = sum(min(support[j])) for j in i..len
    let n = prob_dist_arr.len();
    let mut min_suffix = vec![0.0; n + 1];
    let mut max_suffix = vec![0.0; n + 1];

    for i in (0..n).rev() {
        // Assumes support_arr is sorted. If not, replace first()/last() with min()/max()
        let min_val = support_arr[i].first().cloned().unwrap_or(0.0);
        let max_val = support_arr[i].last().cloned().unwrap_or(0.0);

        min_suffix[i] = min_suffix[i + 1] + min_val;
        max_suffix[i] = max_suffix[i + 1] + max_val;
    }

    let bounds = LookaheadBounds {
        min_suffix,
        max_suffix,
    };

    // 2. DECIDE DIRECTION
    // If budget is high (>= mean), it's faster to calculate the tail (X > Budget)
    // and subtract from total (1.0).
    if budget >= mean {
        let upper_tail_val =
            recurse_upper(prob_dist_arr, support_arr, &bounds, 0.0, budget, 0, mean);
        // Total probability mass for size-biased variable is 1.0.
        // Result = 1.0 - P(SizeBiased > Budget)
        1.0 - upper_tail_val
    } else {
        recurse_lower(prob_dist_arr, support_arr, &bounds, 0.0, budget, 0, mean)
    }
}

// ---------------------------------------------------------
// DIRECTION 1: Standard (Summing paths <= Budget)
// ---------------------------------------------------------
fn recurse_lower(
    probs: &[Vec<f64>],
    supports: &[Vec<f64>],
    bounds: &LookaheadBounds,
    cost_so_far: f64,
    budget: f64,
    depth: usize,
    mean: f64,
) -> f64 {
    // PRUNING: If current cost + minimum possible future cost > budget,
    // this whole subtree is invalid. Return 0.
    if cost_so_far + bounds.min_suffix[depth] > budget {
        return 0.0;
    }

    // BASE CASE
    if depth == probs.len() {
        return cost_so_far / mean;
    }

    let prob_dist = &probs[depth];
    let cost_arr = &supports[depth];

    cost_arr
        .iter()
        .zip(prob_dist.iter())
        .map(|(cost, prob)| (cost_so_far + *cost, *prob))
        // Tighter local pruning: check if next step + min future exceeds budget
        .take_while(|(new_cost, _)| *new_cost + bounds.min_suffix[depth + 1] <= budget)
        .fold(0.0, |acc, (new_cost, prob)| {
            acc + prob * recurse_lower(probs, supports, bounds, new_cost, budget, depth + 1, mean)
        })
}

// ---------------------------------------------------------
// DIRECTION 2: Complement (Summing paths > Budget)
// ---------------------------------------------------------
fn recurse_upper(
    probs: &[Vec<f64>],
    supports: &[Vec<f64>],
    bounds: &LookaheadBounds,
    cost_so_far: f64,
    budget: f64,
    depth: usize,
    mean: f64,
) -> f64 {
    // PRUNING: If current cost + maximum possible future cost <= budget,
    // then NO path in this subtree can ever exceed the budget. Return 0.
    if cost_so_far + bounds.max_suffix[depth] <= budget {
        return 0.0;
    }

    // BASE CASE: We are summing the "bad" cases (where Cost > Budget)
    if depth == probs.len() {
        // We only return value if we exceeded budget (which is implicit due to pruning logic,
        // but strictly safe to check or just return).
        // Since we prune if <= budget, if we are here, we are > budget.
        return cost_so_far / mean;
    }

    let prob_dist = &probs[depth];
    let cost_arr = &supports[depth];

    // Note: We cannot use take_while here easily because we are looking for the "tail".
    // Low costs might fail the check (and be skipped), high costs will pass.
    cost_arr
        .iter()
        .zip(prob_dist.iter())
        .map(|(cost, prob)| (cost_so_far + *cost, *prob))
        .fold(0.0, |acc, (new_cost, prob)| {
            // Local Pruning: Skip this specific child if it can't possibly exceed budget
            if new_cost + bounds.max_suffix[depth + 1] <= budget {
                acc // Add 0.0
            } else {
                acc + prob
                    * recurse_upper(probs, supports, bounds, new_cost, budget, depth + 1, mean)
            }
        })
}

// this is actually just another wrapper

impl StateBundle {
    // it doesn't take into account leftover prices
    pub fn brute_success_prob_metric(&mut self) -> f64 {
        self.update_dist(false);

        self.update_combined();
        // brute_naive(&prob_dist_arr, &combined_costs, prep_output.eqv_gold_budget)

        brute_success_prob(
            &self.gather_prob_dist(),
            &self.gather_combined_gold_cost(),
            self.prep_output.eqv_gold_budget,
        )
    }
}
