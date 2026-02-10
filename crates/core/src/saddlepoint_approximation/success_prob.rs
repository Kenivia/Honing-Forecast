//! This was going to (and might be in the future) an alternative metric to optimize,
//! which is the proabability of "succeeding".
//! However this metric would need the additional assumption that all mats are sellable, you have no mats or you can afford pity (f is linear)
//!
//! Luckily this also doubles as the graph generator(histogram.rs)

use crate::constants::SPECIAL_TOL;
use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use itertools::Itertools;
use std::f64::NAN;

impl StateBundle {
    pub fn success_prob_metric(&mut self, performance: &mut Performance) -> f64 {
        performance.states_evaluated += 1;

        self.update_dist();
        self.update_combined();
        self.compute_special_probs();
        let budget = self.prep_output.eqv_gold_budget;
        self.one_dimension_prob(-1, budget, performance)
    }

    pub fn one_dimension_prob(
        &self,
        support_index: i64,
        budget: f64,
        performance: &mut Performance,
    ) -> f64 {
        let mut out: f64 = 0.0;
        let special_probs = self.special_cache[&self.special_state].clone();
        for (skip_count, &special_prob) in special_probs.iter().enumerate() {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            let this_prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                budget,
                false,
                NAN,
                performance,
            );

            out += special_prob * this_prob;
        }

        out
    }

    // this is also one of the things i was gonna display but this is implicitly displayed in the graph
    pub fn compute_leftover_probs(&mut self) -> Vec<f64> {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs();
        let mut prob_leftover: Vec<f64> =
            Vec::with_capacity(self.flattened_effective_budgets().try_len().unwrap());

        let items: Vec<_> = self.flattened_effective_budgets().enumerate().collect();
        let mut dummy_performance = Performance::new();
        for (support_index, effective_budget) in items {
            prob_leftover.push(self.one_dimension_prob(
                support_index as i64,
                effective_budget,
                &mut dummy_performance,
            ));
        }

        prob_leftover
    }
}
