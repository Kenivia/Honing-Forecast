// use std::f64::NAN;

// use crate::constants::FLOAT_TOL;
use crate::{
    constants::{FLOAT_TOL, SPECIAL_TOL},
    performance::Performance,
};

use itertools::izip;

use crate::state_bundle::StateBundle;

pub static DEBUG_AVERAGE: bool = false;
pub static DEBUG_AVG_INDEX: i64 = 0;
// use statrs::distribution::{Continuous, Normal};

// fn simple_variance<'a, I>(prob_dist_arr: I, support_arr: I) -> f64
// where
//     I: F64_2d<'a>,
// {
//     let mut total_var = 0.0;
//     for (support, prob_dist) in support_arr.into_iter().zip(prob_dist_arr) {
//         let mut mean = 0.0;
//         let mut ex2 = 0.0;
//         for (s, p) in support.iter().zip(prob_dist) {
//             mean += s * p;
//             ex2 += s * s * p;
//         }
//         total_var += ex2 - mean * mean;
//     }

//     total_var
// }
impl StateBundle {
    pub fn simple_avg_var(&self, support_index: i64, skip_count: usize) -> (f64, f64) {
        let mut mean: f64 = 0.0;
        let mut var: f64 = 0.0;
        for pair_arr in self.extract_collapsed_pair(support_index, skip_count) {
            let mut this_mean: f64 = 0.0;
            let mut x2 = 0.0;
            for (s, p) in pair_arr.iter() {
                this_mean += s * p;
                x2 += s * s * p
            }
            mean += this_mean;
            var += x2 - this_mean * this_mean;
        }
        (mean, var)
    }

    pub fn average_gold_metric(&mut self, performance: &mut Performance) -> f64 {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs();
        performance.states_evaluated += 1;

        let mut total_gold: f64 = 0.0;

        let mut dbg_sa_avg = vec![0.0; 15];

        for (skip_count, &special_prob) in
            self.special_cache[&self.special_state].iter().enumerate()
        {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            // // dbg!(special_prob);
            // let zipped: Vec<(f64, f64)> =
            // .collect();

            for (support_index, (effective_budget, price, leftover)) in izip!(
                self.flattened_effective_budgets(),
                self.flattened_price(),
                self.flattened_leftover()
            )
            .enumerate()
            {
                let this_avg: f64 = self.saddlepoint_approximation_average_wrapper(
                    support_index as i64,
                    skip_count,
                    effective_budget,
                    price,
                    leftover,
                    performance,
                );
                // dbg!(this_avg);

                if DEBUG_AVERAGE {
                    if support_index == DEBUG_AVG_INDEX as usize {
                        dbg!(
                            skip_count,
                            support_index,
                            effective_budget,
                            price,
                            leftover,
                            this_avg,
                            special_prob,
                            self.simple_avg_var(support_index as i64, skip_count),
                            special_prob * this_avg,
                            "================================"
                        );
                    }
                }
                total_gold += special_prob * this_avg;
                dbg_sa_avg[support_index] += special_prob * this_avg;
            }
        }

        if DEBUG_AVERAGE {
            dbg!(dbg_sa_avg);
        }

        total_gold
    }

    pub fn saddlepoint_approximation_average_wrapper(
        &self,
        support_index: i64,
        skip_count: usize,
        effective_budget: f64,

        price: f64,
        leftover_value: f64,
        performance: &mut Performance,
    ) -> f64 {
        let mean_var: (f64, f64) = self.simple_avg_var(support_index, skip_count);

        if (price - leftover_value).abs() < FLOAT_TOL {
            return price * (effective_budget - mean_var.0);
        }

        let biased_prob: f64 = self.saddlepoint_approximation_wrapper(
            support_index,
            skip_count,
            effective_budget,
            true,
            mean_var,
            performance,
        );
        let prob = self.saddlepoint_approximation_wrapper(
            support_index,
            skip_count,
            effective_budget,
            false,
            mean_var,
            performance,
        );
        let simple_mean = mean_var.0;
        let out: f64 = price * (effective_budget - simple_mean)
            + (leftover_value - price) * (effective_budget * prob - biased_prob * simple_mean);

        let left = effective_budget - simple_mean;
        let right = effective_budget * prob - biased_prob * (simple_mean);
        let truncated_mean = biased_prob * (simple_mean);
        if !out.is_finite() || DEBUG_AVERAGE {
            if support_index == DEBUG_AVG_INDEX {
                dbg!(
                    simple_mean,
                    effective_budget,
                    biased_prob,
                    prob,
                    truncated_mean,
                    left,
                    right,
                    price,
                    leftover_value,
                    out,
                );
            }
        }

        // }
        return out;
    }
}
