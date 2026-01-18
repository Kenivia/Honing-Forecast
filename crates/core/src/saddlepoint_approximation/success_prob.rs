use crate::constants::SPECIAL_TOL;

use crate::state_bundle::StateBundle;

use itertools::Itertools;

impl StateBundle {
    pub fn success_prob_metric(&mut self) -> f64 {
        // self.performance.states_evaluated += 1;

        self.update_dist();
        self.update_combined();
        self.compute_special_probs();
        let budget = self.prep_output.eqv_gold_budget;
        let out: f64 = self.honing_sa_wrapper(-1, budget);

        out
    }
    fn honing_sa_wrapper(&self, support_index: i64, budget: f64) -> f64 {
        let mut out: f64 = 0.0;

        let special_probs = self.special_cache[&self.special_state].clone();

        for (skip_count, &special_prob) in special_probs.iter().enumerate() {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            // dbg!(&support_arr[index..]);
            let this_prob: f64 = self.saddlepoint_approximation_prob_wrapper(
                support_index,
                skip_count,
                budget,
                &mut 0.0,
                false,
                self.simple_avg_var(support_index, skip_count),
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

        for (support_index, effective_budget) in self.flattened_effective_budgets().enumerate() {
            prob_leftover.push(self.honing_sa_wrapper(support_index as i64, effective_budget));
        }

        prob_leftover
    }
}
