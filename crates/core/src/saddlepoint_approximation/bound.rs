use num::traits::Inv;

use crate::state_bundle::StateBundle;

impl StateBundle {
    pub fn min_guess_max_triplet(
        &self,
        budget: f64,
        min_value: f64,
        max_value: f64,
        // support_index: i64,
        // skip_count: usize,
        mean_var_skew: (f64, f64, f64),
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
            inverse_shifted_sigmoid(min_value, max_value, mean_var_skew, min_value + 1.0),
            // 1.0 * self.theta_guess_min_tail(
            //     support_index,
            //     skip_count,
            //     // EDGE_PERCENTAGES * (max_value - min_value) + min_value,
            //     // min_value,
            // ),
            inverse_shifted_sigmoid(min_value, max_value, mean_var_skew, budget),
            inverse_shifted_sigmoid(min_value, max_value, mean_var_skew, max_value - 1.0),
            // 1.0 * self.theta_guess_max_tail(
            //     support_index,
            //     skip_count,
            //     // (1.0 - EDGE_PERCENTAGES) * (max_value - min_value) + min_value,
            //     max_value,
            // ),
        )
    }
    // pub fn theta_guess_sigmoid(
    //     &self,
    //     min_value: f64,
    //     max_value: f64,
    //     budget: f64,
    //     mean_var_skew: (f64, f64, f64),
    // ) -> f64 {
    //     inverse_sigmoid(min_value, max_value, mean_var_skew, budget)
    // }

    // pub fn theta_guess_max_tail(
    //     &self,
    //     support_index: i64,
    //     skip_count: usize,
    //     // budget: f64,
    //     // max_value: f64,
    // ) -> f64 {
    //     let mut min_delta: f64 = INFINITY;
    //     let mut sum_c: f64 = 0.0;
    //     for support in self.extract_support_with_meta(support_index, skip_count) {
    //         if support.ignore {
    //             continue;
    //         }

    //         let mut last_two: [(f64, f64); 2] = [(NAN, NAN); 2];
    //         for (index, pair) in support.access_collapsed().iter().rev().take(2).enumerate() {
    //             last_two[index] = *pair;
    //             // if index > 1 {
    //             //     break;
    //             // }
    //         }
    //         //               s_next           s_max
    //         let delta: f64 = last_two[1].0 - last_two[0].0;
    //         min_delta = min_delta.min(delta); // these should be negative
    //         sum_c += last_two[1].0 * last_two[0].1 / last_two[1].1;
    //     }
    //     assert!(min_delta < 0.0);
    //     let max = ((min_delta.abs()) / sum_c).ln() / min_delta;
    //     // dbg!(max, sum_c, min_delta, budget, max_value);
    //     max
    //     // 999.9
    // }

    // pub fn theta_guess_min_tail(
    //     &self,
    //     support_index: i64,
    //     skip_count: usize,
    //     // budget: f64,
    //     // min_value: f64,
    // ) -> f64 {
    //     let mut max_delta: f64 = NEG_INFINITY;
    //     let mut min_delta: f64 = INFINITY;
    //     let mut sum_c: f64 = 0.0;
    //     for support in self.extract_support_with_meta(support_index, skip_count) {
    //         if support.ignore {
    //             continue;
    //         }

    //         let mut first_two: [(f64, f64); 2] = [(NAN, NAN); 2];
    //         let mut index: usize = 0;
    //         for pair in support.access_collapsed().iter() {
    //             if pair.1 < FLOAT_TOL {
    //                 continue;
    //             }

    //             first_two[index] = *pair;
    //             index += 1;
    //             if index > 1 {
    //                 break;
    //             }
    //         }
    //         let delta: f64 = first_two[1].0 - first_two[0].0;
    //         max_delta = max_delta.max(delta);
    //         min_delta = min_delta.min(delta);
    //         sum_c += first_two[1].0 * first_two[1].1 / first_two[0].1;
    //     }

    //     let min = ((min_delta.abs()) / sum_c).ln() / min_delta;
    //     // dbg!(min_delta);
    //     min
    //     // -999.9
    // }
}
// pub fn scaled_sigmoid(
//     min_value: f64,
//     max_value: f64,
//     (mean, var, skew): (f64, f64, f64),
//     theta: f64,
// ) -> f64 {
//     let a = min_value;
//     let k: f64 = max_value;

//     v = LOG2_E / ((k - a) / (budget - a)).ln();
//     b = 2.0.powf(1.0 + 1.0 / v) / (k - a) * var;
//     // let denom: f64 = var.powi(2) - mean * skew;

//     // let v: f64 = if denom < FLOAT_TOL {
//     //     1.0
//     // } else {
//     //     var.powi(2) / denom
//     // };

//     // let a: f64 = mean * 2.0_f64.powf(v);
//     // let b: f64 = 2.0 * var / (mean * v);

//     // a / (1.0 + (-b * theta).exp()).powf(v)
// }
pub fn inverse_shifted_sigmoid(
    min_value: f64,
    max_value: f64,
    (mean, var, skew): (f64, f64, f64),
    input_y: f64,
) -> f64 {
    let z = (mean - min_value) / (max_value - min_value);
    let v1 = var / (max_value - min_value);
    let v2 = skew / (max_value - min_value);
    let t0 = (v1 * (1.0 - 2.0 * z) / (z * (1.0 - z)) - v2 / v1).inv();

    let beta = t0 * v1 / (z * (1.0 - z));
    let alpha = t0 * ((1.0 - z) / z).powf(beta.inv());

    let out = alpha * ((input_y - min_value) / (max_value - input_y)).powf(1.0 / beta) - t0;

    if !out.is_finite() {
        panic!(
            "min {:?} max {:?} mean {:?} var {:?} skew {:?} input_y {:?} z {:?} v1 {:?} v2 {:?} t0 {:?} beta {:?} alpha  {:?} ",
            min_value, max_value, mean, var, skew, input_y, z, v1, v2, t0, beta, alpha
        );
    }
    out

    // // if (var.powi(2) - mean * skew) < FLOAT_TOL {}
    // let denom: f64 = var.powi(2) - mean * skew;
    // let v: f64 = if denom < 1.0 {
    //     1.0
    // } else {
    //     var.powi(2) / denom
    // };

    // let a: f64 = mean * 2.0_f64.powf(v);
    // let b: f64 = 2.0 * var / (mean * v);
    // dbg!(a, b, v, input_y);
    // ((a / input_y).powf(1.0 / v) - 1.0).ln() / -b
}
