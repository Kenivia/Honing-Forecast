use std::f64::NAN;

use crate::constants::SPECIAL_TOL;

use crate::performance::Performance;
use crate::state_bundle::StateBundle;

use itertools::Itertools;

impl StateBundle {
    pub fn success_prob_metric(&mut self, performance: &mut Performance) -> f64 {
        performance.states_evaluated += 1;

        self.update_dist();
        self.update_combined();
        self.compute_special_probs();
        let budget = self.prep_output.eqv_gold_budget;
        let out: f64 = self.honing_sa_wrapper(-1, budget, performance);

        out
    }

    pub fn success_prob_for_analysis(&self, budget: f64) -> f64 {
        let mut dummy_performance = Performance::new();
        let out: f64 = self.honing_sa_wrapper(-1, budget, &mut dummy_performance);

        out
    }
    pub fn honing_sa_wrapper(
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
            // dbg!(&support_arr[index..]);
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
    pub fn compute_leftover_probs(&mut self) -> Vec<f64> {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs();
        let mut prob_leftover: Vec<f64> =
            Vec::with_capacity(self.flattened_effective_budgets().try_len().unwrap());

        let items: Vec<_> = self.flattened_effective_budgets().enumerate().collect();
        let mut dummy_performance = Performance::new();
        for (support_index, effective_budget) in items {
            prob_leftover.push(self.honing_sa_wrapper(
                support_index as i64,
                effective_budget,
                &mut dummy_performance,
            ));
        }

        prob_leftover
    }

    pub fn compute_leftover_probs_for_analysis(&self) -> Vec<f64> {
        // assume initialized properly
        let mut prob_leftover: Vec<f64> =
            Vec::with_capacity(self.flattened_effective_budgets().try_len().unwrap());

        let items: Vec<_> = self.flattened_effective_budgets().enumerate().collect();
        let mut dummy_performance = Performance::new();
        for (support_index, effective_budget) in items {
            prob_leftover.push(self.honing_sa_wrapper(
                support_index as i64,
                effective_budget,
                &mut dummy_performance,
            ));
        }

        prob_leftover
    }
}
