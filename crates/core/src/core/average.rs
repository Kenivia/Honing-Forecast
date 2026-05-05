use crate::constants::{SPECIAL_TOL, TreatmentsType};
use crate::helpers::distribute_budgets;
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
        for (skip_count, &special_prob) in
            self.special_cache[&self.special_state].iter().enumerate()
        {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            for (support_index, thresh_price_pairs) in
                self.prep_output.optimizer_material_info.iter().enumerate()
            {
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
    pub fn ui_average_gold_metric(
        &mut self,
        inp_treatment_arr: Option<&[TreatmentsType]>,
        performance: &mut Performance,
    ) -> (Vec<f64>, Vec<f64>, Vec<Vec<f64>>) {
        self.update_prob_dist();
        self.update_cost_dist();
        self.compute_special_probs(false);
        performance.states_evaluated += 1;

        let treatment_arr: &Vec<TreatmentsType> = if inp_treatment_arr.is_none() {
            &vec![self.prep_output.optimizer_plan.clone().try_into().unwrap()]
        } else {
            &inp_treatment_arr.unwrap().to_vec()
        };
        let mut gold_breakdown: Vec<Vec<f64>> =
            vec![vec![0.0; self.prep_output.juice_info.total_num_avail]; treatment_arr.len()];
        let mut average_breakdown: Vec<f64> =
            vec![0.0; self.prep_output.juice_info.total_num_avail];
        let mut metrics_arr: Vec<f64> = vec![0.0; treatment_arr.len()];
        for (treat_index, treatment) in treatment_arr.iter().enumerate() {
            for (skip_count, &special_prob) in
                self.special_cache[&self.special_state].iter().enumerate()
            {
                if special_prob < SPECIAL_TOL {
                    continue;
                }
                for (support_index, thresh_price_pairs) in
                    distribute_budgets(&self.prep_output.raw_material_info, treatment)
                        .iter()
                        .enumerate()
                {
                    let this_avg: f64 = self.one_dimension_average_gold(
                        support_index as i64,
                        skip_count,
                        thresh_price_pairs,
                        performance,
                    );
                    let this = special_prob * this_avg;

                    gold_breakdown[treat_index][support_index] += this;
                    metrics_arr[treat_index] += this;
                    if treat_index == 0 {
                        average_breakdown[support_index] +=
                            special_prob * self.simple_avg(support_index as i64, skip_count)
                    }
                }
            }
        }
        for y in gold_breakdown.iter_mut() {
            for x in y.iter_mut() {
                *x = x.round()
            }
        }
        for x in average_breakdown.iter_mut() {
            *x = x.round() // rounding is more uh correct here (as opposed to taking ceil)
        }

        (metrics_arr, average_breakdown, gold_breakdown)
    }

    /// See Saddlepoint Approximation.pdf for more info on the math. This is a generalized version for n price breakpoints
    pub fn one_dimension_average_gold(
        &self,
        support_index: i64,
        skip_count: usize,
        thresh_price_pairs: &[(f64, f64)],
        performance: &mut Performance,
    ) -> f64 {
        let num_thresholds = thresh_price_pairs.len();

        let simple_mean: f64 = self.simple_avg(support_index, skip_count);

        if num_thresholds == 1 {
            return thresh_price_pairs[0].1 * (thresh_price_pairs[0].0 - simple_mean);
        }

        let simple_mean_log = simple_mean.ln();

        let last_thresh = thresh_price_pairs[num_thresholds - 1].0;
        let last_price = thresh_price_pairs[num_thresholds - 1].1;

        let mut out: f64 = last_price * (last_thresh - simple_mean);

        for (index, &(thresh, price)) in thresh_price_pairs.iter().enumerate().skip(1) {
            let prev_price = thresh_price_pairs[index - 1].1;

            let biased_prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                thresh,
                true,
                simple_mean_log,
                performance,
            );

            let prob = self.saddlepoint_approximation_wrapper(
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
