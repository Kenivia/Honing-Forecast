//! The whole distribution of a sum, exactly, where the lattice it lives on is small.
//!
//! The saddlepoint formula needs many comparable upgrades, and is far off without any
//! sign of it where one upgrade outweighs the rest. That is mostly juices and books,
//! whose totals are small numbers, so few lattice points.

use crate::constants::float_tol;
use crate::state_bundle::StateBundle;

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
}
