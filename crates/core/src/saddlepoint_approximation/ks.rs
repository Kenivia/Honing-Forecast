use std::f64::NAN;

use crate::constants::FLOAT_TOL;
use crate::performance::Performance;
use crate::state_bundle::StateBundle;

impl StateBundle {
    pub fn ks(
        &self,
        theta: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        support_index: i64,
        skip_count: usize,
        performance: &mut Performance,
    ) -> (f64, f64, f64, f64, f64) {
        performance.ks_count += 1;
        if compute_biased {
            let ksx = self.ks_01234(support_index, skip_count, theta);
            (
                ksx.0 + ksx.1.ln() - simple_mean_log,
                ksx.1 + ksx.2 / ksx.1,
                ksx.2 + (ksx.3 * ksx.1 - ksx.2.powi(2)) / (ksx.1.powi(2)),
                ksx.3
                    + (2.0 * ksx.2.powi(3) + ksx.4 * ksx.1.powi(2) - 3.0 * ksx.3 * ksx.1 * ksx.2)
                        / ksx.1.powi(3),
                NAN,
            )
        } else {
            self.ks_01234(support_index, skip_count, theta)
        }
    }
    fn ks_01234(
        &self,
        support_index: i64,
        skip_count: usize,
        theta: f64,
        // toggle: &(bool, bool, bool, bool, bool),
    ) -> (f64, f64, f64, f64, f64) {
        let mut total_k = 0.0;
        let mut total_k1 = 0.0;
        let mut total_k2 = 0.0;
        let mut total_k3 = 0.0;
        let mut total_k4 = 0.0;

        for meta_support in self
            .extract_support_with_meta(support_index, skip_count)
            .filter(|x| !x.ignore)
        {
            let mut sum: f64 = 0.0;
            let mut mean: f64 = 0.0;
            let mut second: f64 = 0.0;
            let mut third: f64 = 0.0;
            let mut fourth: f64 = 0.0;

            let biggest_shift = theta
                * if theta >= 0.0 {
                    meta_support.max_value
                } else {
                    meta_support
                        .access_collapsed()
                        .iter()
                        .skip(meta_support.first_non_zero_prob_index)
                        .next()
                        .unwrap()
                        .0
                };

            if meta_support.linear {
                let step = meta_support.gap_size;
                if theta >= 0.0 {
                    let decay_factor = (-step * theta).exp();
                    let mut current_exp_val = 1.0;

                    for (s, p) in meta_support.access_collapsed().iter().rev()
                    // .filter(|(_, p)| *p > FLOAT_TOL)
                    // shouldn't matter but whatever  {
                    {
                        let u = p * current_exp_val;

                        mean += s * u;
                        let x2: f64 = s * s;
                        second += x2 * u;
                        let x3: f64 = x2 * s;
                        third += x3 * u;
                        fourth += x3 * s * u;
                        sum += u;

                        current_exp_val *= decay_factor;
                    }
                } else {
                    let decay_factor = (step * theta).exp();
                    let mut current_exp_val = 1.0;

                    for (s, p) in meta_support.access_collapsed().iter()
                    // .filter(|(_, p)| *p > FLOAT_TOL)
                    // dont multiply by decay factor if p is 0
                    {
                        let u = p * current_exp_val;
                        mean += s * u;
                        let x2: f64 = s * s;
                        second += x2 * u;
                        let x3: f64 = x2 * s;
                        third += x3 * u;
                        fourth += x3 * s * u;
                        sum += u;

                        // if index > 0 {
                        current_exp_val *= decay_factor;
                        // }
                    }
                }
            } else {
                for (s, p) in meta_support.access_collapsed().iter()
                // .filter(|(_, p)| *p > FLOAT_TOL)
                // this avoids 0.0 * inf  which is NAN
                {
                    let u: f64 = p * (s * theta - biggest_shift).exp();
                    mean += s * u;
                    let x2: f64 = s * s;
                    second += x2 * u;
                    let x3: f64 = x2 * s;
                    third += x3 * u;
                    fourth += x3 * s * u;
                    sum += u;
                }
            }
            // if sum == 0.0 || !sum.is_finite() {
            //     dbg!(
            //         // &u_arr,
            //         theta,
            //         biggest_shift,
            //         meta_support.linear,
            //         meta_support.access_collapsed()
            //     );
            //     // panic!();
            //     return (NAN, NAN, NAN, NAN, NAN);
            // }

            mean /= sum;
            second /= sum;
            third /= sum;
            fourth /= sum;

            total_k += biggest_shift + sum.ln();

            total_k1 += mean;

            let mu2 = (second - mean * mean).max(0.0);
            total_k2 += mu2;

            let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
            total_k3 += mu3;

            let mu4 = fourth - 4.0 * third * mean + 6.0 * second * mean * mean
                - 3.0 * mean * mean * mean * mean;
            total_k4 += mu4 - 3.0 * mu2 * mu2;
        }
        (total_k, total_k1, total_k2, total_k3, total_k4)
    }
}
