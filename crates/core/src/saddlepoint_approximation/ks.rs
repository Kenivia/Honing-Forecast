use std::f64::NAN;

use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use crate::upgrade::Support;
use rayon::prelude::*;
#[derive(Debug, Clone, Copy)]
pub struct KsTuple(pub f64, pub f64, pub f64, pub f64, pub f64);
impl KsTuple {
    fn add(&self, new: KsTuple) -> KsTuple {
        KsTuple(
            self.0 + new.0,
            self.1 + new.1,
            self.2 + new.2,
            self.3 + new.3,
            self.4 + new.4,
        )
    }
    pub fn access(&self) -> (f64, f64, f64, f64, f64) {
        (self.0, self.1, self.2, self.3, self.4)
    }
}
impl Default for KsTuple {
    fn default() -> Self {
        KsTuple(0.0, 0.0, 0.0, 0.0, 0.0)
    }
}
impl StateBundle {
    pub fn ks(
        &self,
        theta: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        support_index: i64,
        skip_count: usize,
        performance: &mut Performance,
    ) -> KsTuple {
        performance.ks_count += 1;
        if compute_biased {
            let ksx = self.ks_01234(support_index, skip_count, theta);
            KsTuple(
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
    ) -> KsTuple {
        // self.special_state
        //     .par_iter()
        //     .skip(skip_count)
        //     .map(|&u_index| self.support_from_index(u_index, support_index))
        //     .filter(|support| !support.ignore)
        //     .fold(
        //         || KsTuple::default(),
        //         |prev, meta_support| prev.add(meta_support.one_ks(theta)),
        //     )
        //     .reduce(|| KsTuple::default(), |a, b| a.add(b))

        self.special_state
            .iter()
            .skip(skip_count)
            .map(|&u_index| self.support_from_index(u_index, support_index))
            .filter(|support| !support.ignore)
            .fold(KsTuple::default(), |prev, meta_support| {
                prev.add(meta_support.one_ks(theta))
            })
    }
}
impl Support {
    fn one_ks(&self, theta: f64) -> KsTuple {
        let mut sum: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        let mut third: f64 = 0.0;
        let mut fourth: f64 = 0.0;

        let biggest_shift = theta
            * if theta >= 0.0 {
                self.max_value
            } else {
                self.access_collapsed()
                    .iter()
                    // .skip(meta_support.first_non_zero_prob_index) // i mean this should just always be 0 now
                    .next()
                    .unwrap()
                    .0
            };

        if self.linear {
            let step = self.gap_size;
            if theta >= 0.0 {
                let decay_factor = (-step * theta).exp();
                let mut current_exp_val = 1.0;

                for (s, p) in self.access_collapsed().iter().rev()
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

                for (s, p) in self.access_collapsed().iter()
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
            for (s, p) in self.access_collapsed().iter()
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

        let mu2 = (second - mean * mean).max(0.0);
        // total_k2 += mu2;

        let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
        // total_k3 += mu3;

        let mu4 = fourth - 4.0 * third * mean + 6.0 * second * mean * mean
            - 3.0 * mean * mean * mean * mean;
        KsTuple(
            biggest_shift + sum.ln(),
            mean,
            mu2,
            mu3,
            mu4 - 3.0 * mu2 * mu2,
        )
    }
}
