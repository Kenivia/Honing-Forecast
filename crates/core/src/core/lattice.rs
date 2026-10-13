//! The whole distribution of a sum, exactly, where the lattice it lives on is small.
//!
//! The saddlepoint formula needs many comparable upgrades, and is far off without any
//! sign of it where one upgrade outweighs the rest. That is mostly juices and books,
//! whose totals are small numbers, so few lattice points.

use crate::constants::float_tol;
use crate::state_bundle::StateBundle;
use num::Integer;
use smallvec::SmallVec;

pub struct Lattice {
    min_value: f64,
    span: f64,
    cdf: Vec<f64>,
    biased_cdf: Vec<f64>,
}

impl Lattice {
    pub fn within(&self, budget: f64, biased: bool) -> f64 {
        let index = ((budget + float_tol(budget) - self.min_value) / self.span).floor();
        if index < 0.0 {
            return 0.0;
        }
        let index = (index as usize).min(self.cdf.len() - 1);
        if biased {
            self.biased_cdf[index]
        } else {
            self.cdf[index]
        }
    }
}

impl StateBundle {
    /// None where it would take more than `max_lattice_work` multiplications
    pub fn lattice(&self, support_index: i64, skip_count: usize) -> Option<Lattice> {
        let (min_value, max_value) = self.find_min_max(support_index, skip_count);
        let (span, min_delta) = self.lattice_span(support_index, skip_count);
        if !min_delta.is_finite() || min_delta < 1.0 {
            return None;
        }
        let outcomes: usize = self
            .extract_collapsed_pair(support_index, skip_count)
            .filter(|pairs| pairs.len() > 1)
            .map(|pairs| pairs.len())
            .sum();
        if (max_value - min_value) / span * outcomes as f64 > self.tuning.max_lattice_work as f64 {
            return None;
        }

        let mut pmf: Vec<f64> = vec![1.0];
        for pairs in self.extract_collapsed_pair(support_index, skip_count) {
            if pairs.len() < 2 {
                continue;
            }
            let low = pairs[0].0;
            let steps = |value: f64| ((value - low) / span).round() as usize;
            let mut next: Vec<f64> = vec![0.0; pmf.len() + steps(pairs[pairs.len() - 1].0)];
            for &(value, p) in pairs.iter() {
                for (target, mass) in next[steps(value)..].iter_mut().zip(pmf.iter()) {
                    *target += mass * p;
                }
            }
            pmf = next;
        }

        let inv_mean = self.simple_avg(support_index, skip_count).recip();
        let (mut below, mut biased_below) = (0.0, 0.0);
        let mut cdf: Vec<f64> = Vec::with_capacity(pmf.len());
        let mut biased_cdf: Vec<f64> = Vec::with_capacity(pmf.len());
        for (index, mass) in pmf.iter().enumerate() {
            below += mass;
            biased_below += (min_value + index as f64 * span) * mass * inv_mean;
            cdf.push(below);
            biased_cdf.push(biased_below);
        }
        Some(Lattice {
            min_value,
            span,
            cdf,
            biased_cdf,
        })
    }

    /// The lattice's answers at every threshold, plain and biased, for each of these free
    /// tap outcomes at once. Skipping k upgrades leaves the sum of the rest, so the sums are
    /// built from the last upgrade backwards and each one is the next with one more upgrade.
    /// Nothing past the largest threshold is kept. None where `lattice` would be.
    pub fn lattice_table(
        &self,
        support_index: i64,
        skips: &[usize],
        thresholds: &[f64],
    ) -> Vec<Option<Vec<(f64, f64)>>> {
        let n = self.special_state.len();
        let support = |index: usize| self.support_from_index(self.special_state[index], support_index);
        let top = thresholds.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let mut out: Vec<Option<Vec<(f64, f64)>>> = vec![None; skips.len()];

        // Which outcomes the lattice is small enough for, and its span from each upgrade on.
        // The work only grows towards the first upgrade, so this stops where it is too much,
        // which for a base material is after a few upgrades.
        let mut spans: SmallVec<[i64; 32]> = SmallVec::new();
        let mut wanted: SmallVec<[(usize, usize); 8]> = SmallVec::new();
        let (mut span, mut min_delta, mut outcomes, mut range) = (0i64, f64::INFINITY, 0usize, 0.0);
        let mut slot = skips.len();
        for index in (0..=n).rev() {
            if index < n {
                let this = support(index);
                let pairs = this.access_collapsed(false);
                if pairs.len() > 1 {
                    min_delta = min_delta.min(this.gap_size);
                    span = span.gcd(&(this.gap_size.round() as i64));
                    outcomes += pairs.len();
                    range += this.access_max(false) - this.access_min(false);
                }
                if min_delta < 1.0 || range / (span.max(1) as f64) * outcomes as f64 > self.tuning.max_lattice_work as f64 {
                    break;
                }
            }
            spans.push(span);
            while slot > 0 && skips[slot - 1] >= index {
                slot -= 1;
                if skips[slot] == index && min_delta.is_finite() {
                    wanted.push((index, slot));
                }
            }
        }
        let Some(&(first, _)) = wanted.iter().min() else {
            return out;
        };
        // by index, as far down as the pass went
        let span = |index: usize| spans[n - index];
        let minimums: Vec<f64> = wanted.iter().map(|&(skip, _)| self.find_min_max(support_index, skip).0).collect();

        let mut pmf: Vec<f64> = vec![1.0];
        let mut this_span: i64 = 0;
        // how far above its minimum a sum is still needed, by those built on it
        let mut reach = f64::NEG_INFINITY;
        let mut reaches = vec![0.0; n + 1];
        for index in first..=n {
            if let Some(at) = wanted.iter().position(|&(skip, _)| skip == index) {
                reach = reach.max(top + float_tol(top) - minimums[at]);
            }
            reaches[index] = reach;
        }
        for index in (first..n).rev() {
            let pairs = support(index).access_collapsed(false);
            if pairs.len() > 1 {
                let fine = span(index).max(1);
                if this_span != 0 && this_span != fine {
                    let stride = (this_span / fine) as usize;
                    let mut spread = vec![0.0; (pmf.len() - 1) * stride + 1];
                    for (at, mass) in pmf.iter().enumerate() {
                        spread[at * stride] = *mass;
                    }
                    pmf = spread;
                }
                this_span = fine;
                let unit = fine as f64;
                let low = pairs[0].0;
                let steps = |value: f64| ((value - low) / unit).round() as usize;
                let full = pmf.len() + steps(pairs[pairs.len() - 1].0);
                let kept = if reaches[index] < 0.0 { 1 } else { full.min((reaches[index] / unit) as usize + 2) };
                let mut next: Vec<f64> = vec![0.0; kept];
                for &(value, p) in pairs.iter() {
                    let from = steps(value);
                    if from >= kept {
                        break;
                    }
                    for (target, mass) in next[from..].iter_mut().zip(pmf.iter()) {
                        *target += mass * p;
                    }
                }
                pmf = next;
            }
            let Some(at) = wanted.iter().position(|&(skip, _)| skip == index) else {
                continue;
            };
            let unit = this_span.max(1) as f64;
            let min_value = minimums[at];
            let inv_mean = self.simple_avg(support_index, index).recip();
            // the sum in full has this many points, of which the far ones may not be kept
            let last = ((self.find_min_max(support_index, index).1 - min_value) / unit).round() as usize;
            let mut answers = vec![(0.0, 0.0); thresholds.len()];
            let mut order: Vec<(usize, usize)> = Vec::with_capacity(thresholds.len());
            for (which, &budget) in thresholds.iter().enumerate() {
                let point = ((budget + float_tol(budget) - min_value) / unit).floor();
                if point >= 0.0 {
                    order.push(((point as usize).min(last), which));
                }
            }
            order.sort();
            let (mut below, mut biased_below) = (0.0, 0.0);
            let mut next_point = 0;
            for &(point, which) in &order {
                while next_point <= point && next_point < pmf.len() {
                    below += pmf[next_point];
                    biased_below += (min_value + next_point as f64 * unit) * pmf[next_point] * inv_mean;
                    next_point += 1;
                }
                answers[which] = (below, biased_below);
            }
            out[wanted[at].1] = Some(answers);
        }
        out
    }
}
