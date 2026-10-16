//! The very center of the whole world
//! Computes K(s), the cumulant generating function, and its derivatives(up to the 4th)
//!
//! There was an attempt at parallelism here, but it's way too fine-grained for rayon. I'm not sure if this kstuple & fold business is any good for serial code but it cant be that bad so i cbb changing it back
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;
use crate::support::Support;
use serde::{Deserialize, Serialize};

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
    /// The cumulants of the sum
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
            // The outcomes are evenly spaced, so the moments are taken of q, a count of outcomes
            // from one end, and scaled after. Running sums of running sums give its factorial
            // moments with one multiplication an outcome, q counting from the outcome added last.
            let pairs = self.access_collapsed(skipped);
            let n = pairs.len();
            let (top, bottom, gap) = (pairs[n - 1].0, pairs[0].0, self.gap_size);
            let rate = gap * theta.abs();
            // the probability of the outcome this far from the one the tilt puts the weight on
            let heavy = |far: usize| pairs[if theta >= 0.0 { n - 1 - far } else { far }].1;
            // that one first, so that one factor makes every weight
            let decay = (-rate).exp();
            let mut sums = [0.0; 5];
            let mut weight = 1.0;
            for far in 0..n {
                push(&mut sums, heavy(far) * weight);
                weight *= decay;
            }
            let (lost, out) = from_sums(sums, biggest_shift, if theta >= 0.0 { (bottom, gap) } else { (top, -gap) });
            if !lost {
                return out;
            }
            // Where the weight sits far from the end q counts from, the moments about the mean
            // are small differences of large numbers and come out as noise: a negative mean,
            // once, and no root for the probability. Counted from the heavy end instead, each
            // weight its own exponential.
            let mut sums = [0.0; 5];
            for far in (0..n).rev() {
                push(&mut sums, heavy(far) * (-rate * far as f64).exp());
            }
            return from_sums(sums, biggest_shift, if theta >= 0.0 { (top, -gap) } else { (bottom, gap) }).1;
        }        for (s, p) in self.access_collapsed(skipped).iter() {
            let u: f64 = p * (s * theta - biggest_shift).exp();
            mean += s * u;
            let x2: f64 = s * s;
            second += x2 * u;
            let x3: f64 = x2 * s;
            third += x3 * u;
            fourth += x3 * s * u;
            sum += u;
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

/// one outcome of weight w into the running sums of running sums
fn push(sums: &mut [f64; 5], w: f64) {
    sums[4] += sums[3];
    sums[3] += sums[2];
    sums[2] += sums[1];
    sums[1] += sums[0];
    sums[0] += w;
}

/// The cumulants from those sums, the value being origin + step * q, and whether the variance
/// came out of a difference that kept fewer than 8 of its digits.
fn from_sums(sums: [f64; 5], shift: f64, (origin, step): (f64, f64)) -> (bool, KsTuple) {
    let inv_sum = sums[0].recip();
    let (f1, f2, f3, f4) = (sums[1] * inv_sum, 2.0 * sums[2] * inv_sum, 6.0 * sums[3] * inv_sum, 24.0 * sums[4] * inv_sum);
    let (m1, m2, m3, m4) = (f1, f2 + f1, f3 + 3.0 * f2 + f1, f4 + 6.0 * f3 + 7.0 * f2 + f1);
    let mu2 = (m2 - m1 * m1).max(0.0);
    let mu3 = m3 - 3.0 * m2 * m1 + 2.0 * m1 * m1 * m1;
    let mu4 = m4 - 4.0 * m3 * m1 + 6.0 * m2 * m1 * m1 - 3.0 * m1 * m1 * m1 * m1;
    let step2 = step * step;
    (m2 - m1 * m1 < 1e-8 * m2, KsTuple(shift + sums[0].ln(), origin + step * m1, step2 * mu2, step2 * step * mu3, step2 * step2 * (mu4 - 3.0 * mu2 * mu2)))
}