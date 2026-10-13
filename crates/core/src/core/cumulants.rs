//! The very center of the whole world
//! Computes K(s), the cumulant generating function, and its derivatives(up to the 4th)
//!
//! There was an attempt at parallelism here, but it's way too fine-grained for rayon. I'm not sure if this kstuple & fold business is any good for serial code but it cant be that bad so i cbb changing it back
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;
use crate::support::Support;
use serde::{Deserialize, Serialize};
use std::f64::NAN;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KsTuple(pub f64, pub f64, pub f64, pub f64, pub f64);
impl KsTuple {
    pub fn add(&self, new: KsTuple) -> KsTuple {
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

    /// The cumulants of the size-biased sum, for E(X'). See one_dimension_average_gold
    pub fn biased(&self, simple_mean_log: f64) -> KsTuple {
        let ksx = self;
        KsTuple(
            ksx.0 + ksx.1.ln() - simple_mean_log,
            ksx.1 + ksx.2 / ksx.1,
            ksx.2 + (ksx.3 * ksx.1 - ksx.2.powi(2)) / (ksx.1.powi(2)),
            ksx.3
                + (2.0 * ksx.2.powi(3) + ksx.4 * ksx.1.powi(2) - 3.0 * ksx.3 * ksx.1 * ksx.2)
                    / ksx.1.powi(3),
            NAN,
        )
    }

    /// The taylor series about here, a short way off
    pub fn moved(&self, d: f64) -> KsTuple {
        KsTuple(
            self.0 + d * (self.1 + d * (self.2 / 2.0 + d * (self.3 / 6.0 + d * self.4 / 24.0))),
            self.1 + d * (self.2 + d * (self.3 / 2.0 + d * self.4 / 6.0)),
            self.2 + d * (self.3 + d * self.4 / 2.0),
            self.3 + d * self.4,
            self.4,
        )
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
        probe: &mut Probe,
    ) -> KsTuple {
        let raw = self.raw_ks(theta, support_index, skip_count, probe);
        if compute_biased { raw.biased(simple_mean_log) } else { raw }
    }

    /// The cumulants of the sum itself, which the biased ones are made from
    pub fn raw_ks(&self, theta: f64, support_index: i64, skip_count: usize, probe: &mut Probe) -> KsTuple {
        probe.count(Counter::Ks);
        self.cumulant_01234(support_index, skip_count, theta)
    }

    fn cumulant_01234(&self, support_index: i64, skip_count: usize, theta: f64) -> KsTuple {
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
                prev.add(meta_support.one_cumulant(theta, index < skip_count))
            })
    }
}
impl Support {
    /// Computes K(s) and its cumulants here, with some optimizations when the support increases linearly
    ///
    /// It also shifts everything such that the biggest power we're raising to is 1, which corresponds to dividing all the support by the biggest value
    /// Otherwise if our thing costs 1 mil and theta = 1 we'll be computing e^(1 mil) and that won't end well
    pub fn one_cumulant(&self, theta: f64, skipped: bool) -> KsTuple {
        let mut sum: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        let mut third: f64 = 0.0;
        let mut fourth: f64 = 0.0;
        if skipped {
            let skipped_val = self.access_collapsed(skipped)[0].0;
            return KsTuple(skipped_val * theta, skipped_val, 0.0, 0.0, 0.0);
        }
        let biggest_shift = theta
            * if theta >= 0.0 {
                self.access_max(skipped)
            } else {
                self.access_min(skipped)
            };

        if self.arithmetic {
            // The outcomes are evenly spaced, so the moments are taken of q, the count of
            // outcomes from the light end, and scaled after. Running sums of running sums
            // give its factorial moments with one multiplication an outcome.
            let pairs = self.access_collapsed(skipped);
            let decay_factor = (-self.gap_size * theta.abs()).exp();
            let mut current_exp_val = 1.0;
            let (mut r0, mut r1, mut r2, mut r3, mut r4) = (0.0, 0.0, 0.0, 0.0, 0.0);
            let mut add = |p: f64| {
                r4 += r3;
                r3 += r2;
                r2 += r1;
                r1 += r0;
                r0 += p * current_exp_val;
                current_exp_val *= decay_factor;
            };
            if theta >= 0.0 {
                pairs.iter().rev().for_each(|pair| add(pair.1));
            } else {
                pairs.iter().for_each(|pair| add(pair.1));
            }
            let inv_sum = r0.recip();
            let (f1, f2, f3, f4) = (r1 * inv_sum, 2.0 * r2 * inv_sum, 6.0 * r3 * inv_sum, 24.0 * r4 * inv_sum);
            let (m1, m2, m3, m4) = (f1, f2 + f1, f3 + 3.0 * f2 + f1, f4 + 6.0 * f3 + 7.0 * f2 + f1);
            let mu2 = (m2 - m1 * m1).max(0.0);
            let mu3 = m3 - 3.0 * m2 * m1 + 2.0 * m1 * m1 * m1;
            let mu4 = m4 - 4.0 * m3 * m1 + 6.0 * m2 * m1 * m1 - 3.0 * m1 * m1 * m1 * m1;
            // the value is light + scale * q
            let (light, scale) =
                if theta >= 0.0 { (pairs[0].0, self.gap_size) } else { (pairs[pairs.len() - 1].0, -self.gap_size) };
            let scale2 = scale * scale;
            return KsTuple(
                biggest_shift + r0.ln(),
                light + scale * m1,
                scale2 * mu2,
                scale2 * scale * mu3,
                scale2 * scale2 * (mu4 - 3.0 * mu2 * mu2),
            );
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
