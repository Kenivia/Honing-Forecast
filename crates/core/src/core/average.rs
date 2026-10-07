use crate::constants::{FLOAT_TOL, SPECIAL_TOL};
use crate::performance::Performance;
use crate::state_bundle::StateBundle;

use std::f64::NAN;

pub const DEBUG_AVERAGE: bool = false;
pub const DEBUG_AVG_INDEX: i64 = 7;

impl StateBundle {
    /// notably this is the average of the unbiased distribution(it's not the mean of the biased one)
    pub fn simple_avg(&self, support_index: i64, skip_count: usize) -> f64 {
        let mut mean: f64 = 0.0;
        for pair_arr in self.extract_collapsed_pair(support_index, skip_count) {
            let mut this_mean: f64 = 0.0;
            for (s, p) in pair_arr.iter() {
                this_mean += s * p;
            }
            mean += this_mean;
        }
        mean
    }

    /// Special case of ui_average_gold_metric, just separating them to keep things clean
    ///
    /// See Saddlepoint Approximation.pdf for more info on the math.
    pub fn optimizer_average_gold_metric(&mut self, performance: &mut Performance) -> f64 {
        self.update_prob_dist();
        self.update_cost_dist();
        self.compute_special_probs(false);
        performance.states_evaluated += 1;

        let mut total_gold: f64 = 0.0;
        for (skip_count, &special_prob) in self.special_probs().iter().enumerate() {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            let plan_bands = &self.prep_output.bands[self.prep_output.optimizer_plan];
            for (support_index, thresh_price_pairs) in plan_bands.iter().enumerate() {
                let this_avg: f64 = self.one_dimension_average_gold(
                    support_index as i64,
                    skip_count,
                    thresh_price_pairs,
                    performance,
                );

                total_gold += special_prob * this_avg;
            }
        }

        total_gold
    }

    /// See Saddlepoint Approximation.pdf for more info on the math.
    /// One metric and one per material breakdown for every plan in the payload, plus the
    /// average amount of each material used (which does not depend on the plan).
    pub fn ui_average_gold_metric(
        &mut self,
        performance: &mut Performance,
    ) -> (Vec<f64>, Vec<f64>, Vec<Vec<f64>>) {
        self.update_prob_dist();
        self.update_cost_dist();
        self.compute_special_probs(false);
        performance.states_evaluated += 1;

        let num_mats = self.prep_output.table.len();
        let num_plans = self.prep_output.plans.len();
        let mut gold_breakdown: Vec<Vec<f64>> = vec![vec![0.0; num_mats]; num_plans];
        let mut avg_used: Vec<f64> = vec![0.0; num_mats];
        let mut metric_per_plan: Vec<f64> = vec![0.0; num_plans];
        for plan_index in 0..num_plans {
            for (skip_count, &special_prob) in self.special_probs().iter().enumerate() {
                if special_prob < SPECIAL_TOL {
                    continue;
                }
                for support_index in 0..num_mats {
                    let this = special_prob
                        * self.one_dimension_average_gold(
                            support_index as i64,
                            skip_count,
                            &self.prep_output.bands[plan_index][support_index],
                            performance,
                        );

                    gold_breakdown[plan_index][support_index] += this;
                    metric_per_plan[plan_index] += this;
                    if plan_index == 0 {
                        avg_used[support_index] +=
                            special_prob * self.simple_avg(support_index as i64, skip_count)
                    }
                }
            }
        }

        (metric_per_plan, avg_used, gold_breakdown)
    }

    /// See Saddlepoint Approximation.pdf for more info on the math. This is a generalized version for n price breakpoints
    pub fn one_dimension_average_gold(
        &self,
        support_index: i64,
        skip_count: usize,
        thresh_price_pairs: &[(f64, f64)],
        performance: &mut Performance,
    ) -> f64 {
        let num_thresholds: usize = thresh_price_pairs.len();

        let simple_mean: f64 = self.simple_avg(support_index, skip_count);

        if num_thresholds == 1 {
            return thresh_price_pairs[0].1 * (thresh_price_pairs[0].0 - simple_mean);
        }

        let simple_mean_log: f64 = simple_mean.ln();

        let last_thresh: f64 = thresh_price_pairs[num_thresholds - 1].0;
        let last_price: f64 = thresh_price_pairs[num_thresholds - 1].1;

        let mut out: f64 = last_price * (last_thresh - simple_mean);

        for (index, &(thresh, price)) in thresh_price_pairs.iter().enumerate().skip(1) {
            let prev_price = thresh_price_pairs[index - 1].1;
            // Both probabilities below are multiplied by this gap, so a threshold the price
            // does not change at costs nothing. The top entry is the one that needs this:
            // it has to stay for the term above, because it carries a constant rather than
            // a kink, but it is free whenever the band under it is credited at the buy price.
            if (prev_price - price).abs() < FLOAT_TOL {
                continue;
            }

            let biased_prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                thresh,
                true,
                simple_mean_log,
                performance,
            );

            let prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                thresh,
                false,
                NAN,
                performance,
            );

            out += (prev_price - price) * (thresh * prob - biased_prob * simple_mean);
        }
        return out;
    }
}
