use crate::constants::FLOAT_TOL;

use crate::state_bundle::StateBundle;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
pub static MAX_BRUTE_SIZE: usize = 50000;

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

/// Helper struct to hold lookahead bounds
struct LookaheadBounds {
    min_suffix: Vec<f64>,
    max_suffix: Vec<f64>,
}

// this is actually just another wrapper

impl StateBundle {
    // // it doesn't take into account leftover prices
    // pub fn brute_success_prob_metric(&mut self) -> f64 {
    //     self.update_dist();

    //     self.update_combined();
    //     // brute_naive(&prob_dist_arr, &combined_costs, prep_output.eqv_gold_budget)

    //     self.brute_success_prob(
    //         // &self.gather_prob_dist(),
    //         // &self.gather_combined_gold_cost(),
    //         sup
    //         self.prep_output.eqv_gold_budget,
    //     )
    // }

    pub fn brute_success_prob(
        // prob_dist_arr: &[Vec<f64>],
        // support_arr: &[Vec<f64>],
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
    ) -> f64 {
        let n = self.upgrade_arr.len() - skip_count;

        // --- STEP 1: Pre-calculate Look-Ahead Bounds ---
        // min_suffix[i] = sum of minimum costs from layer i to end
        // max_suffix[i] = sum of maximum costs from layer i to end
        let mut min_suffix = vec![0.0; n + 1];
        let mut max_suffix = vec![0.0; n + 1];

        let mut u_index = n - 1; // cant use enumerate for some reason

        for pairs in self.extract_collapsed_pair(support_index, skip_count).rev() {
            // Find min and max cost for this specific layer
            // We use fold because f64 doesn't implement Ord
            let (local_min, local_max) =
                pairs
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &(s, p)| {
                        (
                            if p.abs() > FLOAT_TOL { min.min(s) } else { min },
                            if p.abs() > FLOAT_TOL { max.max(s) } else { max },
                        )
                    });

            min_suffix[u_index] = min_suffix[u_index + 1] + local_min;
            max_suffix[u_index] = max_suffix[u_index + 1] + local_max;
            u_index -= 1;
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
                    let step_prob = current_prob * p;

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
        let out = total_guaranteed_prob + current_states.values().sum::<f64>();
        // if !out.is_finite() {
        //     dbg!(
        //         prob_dist_arr,
        //         support_arr,
        //         total_guaranteed_prob,
        //         current_states.values().sum::<f64>()
        //     );
        //     panic!();
        // }
        out
    }

    pub fn brute_biased_recursive(
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        middle: f64,
    ) -> f64 {
        // 1. PRE-COMPUTE BOUNDS
        // We scan backwards to calculate the min/max cost possible from each depth.
        // min_suffix[i] = sum(min(support[j])) for j in i..len
        let n = self.upgrade_arr.len() - skip_count;
        let mut min_suffix = vec![0.0; n + 1];
        let mut max_suffix = vec![0.0; n + 1];

        let mut index = n - 1;
        for pairs in self.extract_collapsed_pair(support_index, skip_count).rev() {
            // Assumes support_arr is sorted. If not, replace first()/last() with min()/max()
            let min_val = pairs
                .iter()
                .find(|(_s, p)| *p > FLOAT_TOL)
                .unwrap_or(&(0.0, 0.0))
                .0;
            let max_val = pairs.last().cloned().unwrap_or((0.0, 0.0)).0;

            min_suffix[index] = min_suffix[index + 1] + min_val;
            max_suffix[index] = max_suffix[index + 1] + max_val;
            index -= 1;
        }

        let bounds = LookaheadBounds {
            min_suffix,
            max_suffix,
        };

        // 2. DECIDE DIRECTION
        // If budget is high (>= mean), it's faster to calculate the tail (X > Budget)
        // and subtract from total (1.0).
        if budget >= middle {
            let upper_tail_val =
                self.recurse_upper(support_index, skip_count, &bounds, 0.0, budget, 0, middle);
            // Total probability mass for size-biased variable is 1.0.
            // Result = 1.0 - P(SizeBiased > Budget)
            1.0 - upper_tail_val
        } else {
            self.recurse_lower(support_index, skip_count, &bounds, 0.0, budget, 0, middle)
        }
    }

    // ---------------------------------------------------------
    // DIRECTION 1: Standard (Summing paths <= Budget)
    // ---------------------------------------------------------
    fn recurse_lower(
        &self,
        support_index: i64,
        skip_count: usize,
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
        if depth == self.upgrade_arr.len() - skip_count {
            return cost_so_far / mean;
        }

        self.extract_collapsed_pair(support_index, skip_count)
            .skip(depth)
            .next()
            .unwrap()
            .iter()
            .map(|(cost, prob)| (cost_so_far + *cost, *prob))
            // Tighter local pruning: check if next step + min future exceeds budget
            .take_while(|(new_cost, _)| *new_cost + bounds.min_suffix[depth + 1] <= budget)
            .fold(0.0, |acc, (new_cost, prob)| {
                acc + if prob.abs() > FLOAT_TOL {
                    0.0
                } else {
                    prob * self.recurse_lower(
                        support_index,
                        skip_count,
                        bounds,
                        new_cost,
                        budget,
                        depth + 1,
                        mean,
                    )
                }
            })
    }

    // ---------------------------------------------------------
    // DIRECTION 2: Complement (Summing paths > Budget)
    // ---------------------------------------------------------
    fn recurse_upper(
        &self,
        support_index: i64,
        skip_count: usize,
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
        if depth == self.upgrade_arr.len() - skip_count {
            // We only return value if we exceeded budget (which is implicit due to pruning logic,
            // but strictly safe to check or just return).
            // Since we prune if <= budget, if we are here, we are > budget.
            return cost_so_far / mean;
        }

        // Note: We cannot use take_while here easily because we are looking for the "tail".
        // Low costs might fail the check (and be skipped), high costs will pass.
        self.extract_collapsed_pair(support_index, skip_count)
            .skip(depth)
            .next()
            .unwrap()
            .iter()
            .map(|(cost, prob)| (cost_so_far + *cost, *prob))
            .fold(0.0, |acc, (new_cost, prob)| {
                // Local Pruning: Skip this specific child if it can't possibly exceed budget
                if new_cost + bounds.max_suffix[depth + 1] <= budget {
                    acc // Add 0.0
                } else {
                    acc + if prob.abs() > FLOAT_TOL {
                        0.0
                    } else {
                        prob * self.recurse_upper(
                            support_index,
                            skip_count,
                            bounds,
                            new_cost,
                            budget,
                            depth + 1,
                            mean,
                        )
                    }
                }
            })
    }
}
