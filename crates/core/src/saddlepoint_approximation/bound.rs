use std::f64::{INFINITY, NAN, NEG_INFINITY};

use crate::{constants::FLOAT_TOL, state_bundle::StateBundle};

static EDGE_PERCENTAGES: f64 = 0.05;
impl StateBundle {
    pub fn min_guess_max_triplet(
        &self,
        // budget: f64,
        // max_value: f64,
        // min_value: f64,
        support_index: i64,
        skip_count: usize,
        // mean_var: (f64, f64),
        // compute_biased: bool,
    ) -> (f64, f64, f64) {
        // let x = (budget - min_value) / (max_value - min_value);
        // // let guess = self.theta_guess_mean(budget, mean_var);
        // let guess = 0.0;

        // let biggest_s: f64 = self
        //     .extract_support_with_meta(support_index, skip_count)
        //     .filter(|x| !x.ignore)
        //     .map(|pair_arr| {
        //         pair_arr
        //             .access_collapsed()
        //             .iter()
        //             .find(|x| x.0 > FLOAT_TOL)
        //             .unwrap()
        //             .0
        //     })
        //     .fold(NEG_INFINITY, |a, b| a.max(b));
        // let limit: f64 = 10.0_f64 / biggest_s;

        // if compute_biased {
        //     return (-limit, guess, limit);
        // }
        // if x < EDGE_PERCENTAGES {
        //     return (
        //         1.0 * self.theta_guess_min_tail(support_index, skip_count, budget, min_value),
        //         guess,
        //         0.0,
        //     );
        // } else if x > 1.0 - EDGE_PERCENTAGES {
        //     return (
        //         0.0,
        //         guess,
        //         1.0 * self.theta_guess_max_tail(support_index, skip_count, budget, max_value),
        //     );
        // }
        (
            2.0 * self.theta_guess_min_tail(
                support_index,
                skip_count,
                // EDGE_PERCENTAGES * (max_value - min_value) + min_value,
                // min_value,
            ),
            0.0,
            2.0 * self.theta_guess_max_tail(
                support_index,
                skip_count,
                // (1.0 - EDGE_PERCENTAGES) * (max_value - min_value) + min_value,
                // max_value,
            ),
        )
    }
    pub fn theta_guess_mean(&self, budget: f64, mean_var: (f64, f64)) -> f64 {
        (budget - mean_var.0) / mean_var.1
    }

    pub fn theta_guess_max_tail(
        &self,
        support_index: i64,
        skip_count: usize,
        // budget: f64,
        // max_value: f64,
    ) -> f64 {
        let mut min_delta: f64 = INFINITY;
        let mut sum_c: f64 = 0.0;
        for support in self.extract_support_with_meta(support_index, skip_count) {
            if support.ignore {
                continue;
            }

            let mut last_two: [(f64, f64); 2] = [(NAN, NAN); 2];
            for (index, pair) in support.access_collapsed().iter().rev().take(2).enumerate() {
                last_two[index] = *pair;
                if index > 1 {
                    break;
                }
            }
            //               s_next           s_max
            let delta: f64 = last_two[1].0 - last_two[0].0;
            min_delta = min_delta.min(delta); // these should be negative
            sum_c += last_two[1].0 * last_two[0].1 / last_two[1].1;
        }
        assert!(min_delta < 0.0);
        let max = ((min_delta.abs() * 0.5) / sum_c).ln() / min_delta;
        // dbg!(max, sum_c, min_delta, budget, max_value);
        max
    }

    pub fn theta_guess_min_tail(
        &self,
        support_index: i64,
        skip_count: usize,
        // budget: f64,
        // min_value: f64,
    ) -> f64 {
        let mut min_delta: f64 = INFINITY;
        let mut sum_c: f64 = 0.0;
        for support in self.extract_support_with_meta(support_index, skip_count) {
            if support.ignore {
                continue;
            }

            let mut first_two: [(f64, f64); 2] = [(NAN, NAN); 2];
            let mut index: usize = 0;
            for pair in support.access_collapsed().iter() {
                if pair.1 < FLOAT_TOL {
                    continue;
                }

                first_two[index] = *pair;
                index += 1;
                if index > 1 {
                    break;
                }
            }
            let delta: f64 = first_two[1].0 - first_two[0].0;
            min_delta = min_delta.min(delta);
            sum_c += first_two[1].0 * first_two[1].1 / first_two[0].1;
        }

        let min = (((min_delta.abs() * 0.5) / sum_c).ln() / min_delta);

        min
    }
}
