//! The very center of the whole world
//! Computes K(s), the cumulant generating function, and its derivatives(up to the 4th)
//!
//! There was an attempt at parallelism here, but it's way too fine-grained for rayon. I'm not sure if this kstuple & fold business is any good for serial code but it cant be that bad so i cbb changing it back
use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use crate::upgrade::Support;
use std::f64::NAN;

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
            // This is for E(X'). See one_dimension_average_gold
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

    fn ks_01234(&self, support_index: i64, skip_count: usize, theta: f64) -> KsTuple {
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
            .enumerate()
            .map(|(index, &u_index)| (index, self.support_from_index(u_index, support_index)))
            .filter(|(_, support)| !support.ignore)
            .fold(KsTuple::default(), |prev, (index, meta_support)| {
                prev.add(meta_support.one_ks(theta, index < skip_count))
            })
    }
}
impl Support {
    /// Computes K(s) and its cumulants here, with some optimizations when the support increases linearly
    ///
    /// It also shifts everything such that the biggest power we're raising to is 1, which corresponds to dividing all the support by the biggest value
    /// Otherwise if our thing costs 1 mil we'll be computing e^(1 mil) and that won't end well
    fn one_ks(&self, theta: f64, skipped: bool) -> KsTuple {
        let mut sum: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        let mut third: f64 = 0.0;
        let mut fourth: f64 = 0.0;
        if skipped {
            let s = self.access_collapsed(skipped).iter().next().unwrap().0;
            return KsTuple(s * theta, s, 0.0, 0.0, 0.0);
        }
        let biggest_shift = theta
            * if theta >= 0.0 {
                self.access_max(skipped)
            } else {
                self.access_min(skipped)
            };

        if self.linear {
            let step = self.gap_size;
            if theta >= 0.0 {
                let decay_factor = (-step * theta).exp();
                let mut current_exp_val = 1.0;

                for (s, p) in self.access_collapsed(skipped).iter().rev() {
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

                for (s, p) in self.access_collapsed(skipped).iter() {
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
            }
        } else {
            for (s, p) in self.access_collapsed(skipped).iter() {
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
        let inv_sum = sum.recip();

        mean *= inv_sum;
        second *= inv_sum;
        third *= inv_sum;
        fourth *= inv_sum;

        let mu2 = (second - mean * mean).max(0.0);
        let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
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
