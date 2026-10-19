//! The plain probability of being within a budget: the exact lattice where it is small, else the
//! Lugannani-Rice formula (split on the largest upgrade where it misbehaves), else brute force

use crate::constants::{FLOAT_TOL, float_tol};
use crate::core::probability::cumulants::KsTuple;
use crate::core::probability::shape::Shape;
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;
use crate::tuning::RootFinder;
use ahash::AHashMap;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

/// how many upgrades deep a probability may be split on outcomes before it is clamped
const MAX_CONDITION_DEPTH: usize = 2;

fn variance(pairs: &[(f64, f64)]) -> f64 {
    let mean: f64 = pairs.iter().map(|(s, p)| s * p).sum();
    pairs.iter().map(|(s, p)| (s - mean).powi(2) * p).sum()
}

/// Edgeworth series, with the cumulants at zero. Only any good near the mean.
fn edgeworth(at_zero: KsTuple, budget: f64, normal_dist: &Normal) -> f64 {
    let std = at_zero.2.sqrt();
    let z = (budget - at_zero.1) / std;
    let (gamma3, gamma4) = (at_zero.3 / std.powi(3), at_zero.4 / std.powi(4));
    let cdf_correction = normal_dist.pdf(z)
        * ((gamma3 / 6.0) * (z.powi(2) - 1.0)
            + (gamma4 / 24.0) * (z.powi(3) - 3.0 * z)
            + (gamma3 * gamma3 / 72.0) * (z.powi(5) - 10.0 * z.powi(3) + 15.0 * z));
    normal_dist.cdf(z) - cdf_correction
}

impl StateBundle {
    /// The plain probability of being within the budget
    pub fn saddlepoint_approximation_wrapper(
        &self,
        support_index: i64,
        skip_count: usize,
        inp_budget: f64,
        probe: &mut Probe,
    ) -> f64 {
        let shape = self.shape(support_index, skip_count);
        self.probability_within(support_index, skip_count, &shape, inp_budget, probe, 0)
    }

    fn probability_within(
        &self,
        support_index: i64,
        skip_count: usize,
        shape: &Shape,
        inp_budget: f64,
        probe: &mut Probe,
        depth: usize,
    ) -> f64 {
        let &Shape {
            min_value,
            max_value,
            span,
            min_delta,
            ..
        } = shape;
        if inp_budget > max_value - float_tol(max_value) {
            probe.count(Counter::Trivial);
            return 1.0;
        }

        if inp_budget < min_value - float_tol(min_value) {
            probe.count(Counter::Trivial);
            return 0.0;
        };
        if let Some(lattice) = self.lattice(support_index, skip_count) {
            probe.count(Counter::Lattice);
            return lattice.within(inp_budget);
        }
        if shape.too_big {
            // the lattice starts at the minimum, not at zero
            let budget = (min_value + ((inp_budget - min_value) / span + FLOAT_TOL).floor() * span)
                .min(max_value - span)
                .max(min_value)
                + span / 2.0;

            if max_value - min_value > min_delta // only one or two outcomes
                && min_value + min_delta + 1.0 < budget // brute should only have to handle like n = number of upgrades cases or so. In general any higher (e.g. 2* min_delta) and this explodes insanely quickly
                && budget < max_value - min_delta - 1.0
            {
                return self.saddlepoint_approximation(
                    support_index,
                    skip_count,
                    shape,
                    inp_budget,
                    budget,
                    probe,
                    depth,
                );
            }
        }
        probe.count(Counter::Brute);

        self.brute_success_prob(support_index, skip_count, inp_budget, shape.mean, false)
    }

    fn find_root(
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        guess: f64,
        min_value: f64,
        max_value: f64,
        min_delta: f64,
        seed: Option<KsTuple>,
        probe: &mut Probe,
    ) -> Option<(f64, KsTuple)> {
        match self.tuning.root_finder {
            RootFinder::Householder => self.householder(
                support_index,
                skip_count,
                budget,
                guess,
                min_value,
                max_value,
                min_delta,
                seed,
                probe,
            ),
        }
    }

    /// Lugannani-Rice formula, for the plain probability
    fn saddlepoint_approximation(
        &self,
        support_index: i64,
        skip_count: usize,
        shape: &Shape,
        inp_budget: f64,
        budget: f64,
        probe: &mut Probe,
        depth: usize,
    ) -> f64 {
        probe.count(Counter::Saddlepoint);
        let &Shape {
            min_value,
            max_value,
            span,
            min_delta,
            zero,
            ..
        } = shape;
        let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap();

        // the main formula is singular at the mean. the threshold is complete trial-and-error heuristic
        let (out, trusted) = if (zero.1 - budget).abs() / (zero.1.max(budget).max(1.0)) < 1e-4 {
            probe.count(Counter::Edgeworth);
            (edgeworth(zero, budget, &normal_dist), true)
        } else {
            probe.count(Counter::Lugannani);
            let guess = self
                .min_guess_max_triplet(budget, min_value, max_value, (zero.1, zero.2, zero.3))
                .1;
            match self.find_root(
                support_index,
                skip_count,
                budget,
                guess,
                min_value,
                max_value,
                min_delta,
                None,
                probe,
            ) {
                // No root, which a threshold at the very end of the range can do. The series at
                // zero stands in, not trusted, so that the sum is split and tried again.
                None => (edgeworth(zero, budget, &normal_dist), false),
                Some((theta_hat, ks_tuple)) => {
                    let w_hat =
                        theta_hat.signum() * (2.0 * (theta_hat * budget - ks_tuple.0)).sqrt();
                    let u_hat = 2.0 / span * (span * theta_hat / 2.0).sinh() * ks_tuple.2.sqrt(); // second continuity correction

                    // the approximation's slope is the density times this, so it is not a
                    // distribution function where this is negative
                    let slope = 1.0 + ks_tuple.4 / ks_tuple.2.powi(2) / 8.0
                        - 5.0 * ks_tuple.3.powi(2) / ks_tuple.2.powi(3) / 24.0;
                    (
                        normal_dist.cdf(w_hat)
                            + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat),
                        slope > 0.0,
                    )
                }
            }
        };

        if trusted && (-FLOAT_TOL..=1.0 + FLOAT_TOL).contains(&out) {
            return out;
        }
        if depth >= MAX_CONDITION_DEPTH {
            assert!(out.is_finite());
            return out.clamp(0.0, 1.0);
        }
        probe.count(Counter::Conditioned);
        self.conditioned(support_index, skip_count, inp_budget, probe, depth)
    }

    /// Where the sum is too far from normal for the formula above, which is one upgrade
    /// outweighing the rest: splits on that upgrade's outcomes, which is exact, and leaves
    /// a sum of the others for each.
    fn conditioned(
        &self,
        support_index: i64,
        skip_count: usize,
        inp_budget: f64,
        probe: &mut Probe,
        depth: usize,
    ) -> f64 {
        let (index, pairs) = self
            .extract_collapsed_pair(support_index, skip_count)
            .enumerate()
            .max_by(|a, b| variance(a.1).total_cmp(&variance(b.1)))
            .unwrap();
        let pairs = pairs.clone();
        let u_index = self.special_state[index];
        // the caches are not read on this path
        let mut pinned = StateBundle {
            upgrade_arr: self.upgrade_arr.clone(),
            special_state: self.special_state.clone(),
            chest_states: self.chest_states.clone(),
            special_invalid_index: self.special_invalid_index,
            latest_special_probs: None,
            metric_type: self.metric_type,
            metric: self.metric,
            min_resolution: self.min_resolution,
            prep_output: self.prep_output.clone(),
            num_threads: self.num_threads,
            special_cache: AHashMap::new(),
            adv_cache: AHashMap::new(),
            gold_memo: AHashMap::new(),
            special_done: None,
            tuning: self.tuning.clone(),
        };

        let mut out: f64 = 0.0;
        for (value, p) in pairs {
            pinned.upgrade_arr[u_index].cost_dist[support_index as usize].pin(value);
            let shape = pinned.shape(support_index, skip_count);
            out += p * pinned.probability_within(
                support_index,
                skip_count,
                &shape,
                inp_budget,
                probe,
                depth + 1,
            );
        }
        out
    }
}
