// use std::f64::NAN;

use std::f64::NAN;

// use crate::constants::FLOAT_TOL;
use crate::constants::SPECIAL_TOL;

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
    pub fn simple_average(&self, support_index: i64, skip_count: usize) -> f64 {
        let mut mean: f64 = 0.0;
        for triplet_arr in self.extract_triplet(support_index, skip_count) {
            for (s, p, _) in triplet_arr.iter() {
                mean += s * p; // technically if theta is 0.0 we can use the K' or k1 from there but like nah cbb
            }
        }
        mean
    }
    pub fn average_gold_metric(&mut self) -> f64 {
        self.update_dist(true);
        self.update_individual_support();
        self.compute_special_probs();
        // self.performance.states_evaluated += 1;

        let mut total_gold: f64 = 0.0;

        let mut dbg_sa_avg = vec![0.0; 15];
        let special_probs = self.special_cache[&self.special_state].clone();

        for (skip_count, special_prob) in special_probs.into_iter().enumerate() {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            // // dbg!(special_prob);
            // let zipped: Vec<(f64, f64, f64)> =
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
                    &mut 0.0,
                    price,
                    leftover,
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
                            self.simple_average(support_index as i64, skip_count),
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
        init_theta: &mut f64,

        price: f64,
        leftover_value: f64,
    ) -> f64 {
        let simple_mean: f64 = self.simple_average(support_index, skip_count);

        if price == leftover_value {
            return price * (effective_budget - simple_mean);
        }

        // let mut truncated_mean: f64 = NAN; // default if it's trivial
        let biased_prob: f64 = self.saddlepoint_approximation_prob_wrapper(
            support_index,
            skip_count,
            effective_budget,
            init_theta,
            true,
            simple_mean,
        );
        let prob = self.saddlepoint_approximation_prob_wrapper(
            support_index,
            skip_count,
            effective_budget,
            init_theta,
            false,
            NAN,
        );

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
