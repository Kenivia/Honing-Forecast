use crate::constants::{FLOAT_TOL, float_tol};
use crate::core::cumulants::KsTuple;
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;
use crate::support::Support;
use crate::tuning::RootFinder;
use ahash::AHashMap;
use num::Integer;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};
use std::f64::{INFINITY, NAN};

pub const MIN_LATTICE_SPAN: f64 = 1.0;
/// how many upgrades deep a probability may be split on outcomes before it is clamped
const MAX_CONDITION_DEPTH: usize = 2;

/// The spacing of the lattice the sum lives on, which is how much continuity correction
/// to perform, and the smallest gap of any one upgrade. An upgrade with one outcome only
/// moves the lattice, so it is left out.
fn span_of<'a, I>(meta_support_arr: I) -> (f64, f64)
where
    I: Iterator<Item = &'a Support>,
{
    let mut span: i64 = 0;
    let mut min_delta: f64 = INFINITY;
    for support in meta_support_arr {
        if support.access_collapsed(false).len() < 2 {
            continue;
        }
        if !support.gap_size.is_finite() {
            panic!("non finite gap size")
        }
        min_delta = min_delta.min(support.gap_size);
        span = span.gcd(&(support.gap_size.round() as i64));
    }
    if min_delta < MIN_LATTICE_SPAN {
        return (MIN_LATTICE_SPAN, min_delta); // always do a little bit of cont correction ig cos why not
    }
    ((span as f64).max(MIN_LATTICE_SPAN), min_delta)
}

fn variance(pairs: &[(f64, f64)]) -> f64 {
    let mean: f64 = pairs.iter().map(|(s, p)| s * p).sum();
    pairs.iter().map(|(s, p)| (s - mean).powi(2) * p).sum()
}

/// Edgeworth series, with the cumulants at zero. Only any good near the mean.
fn edgeworth(at_zero: KsTuple, budget: f64, compute_biased: bool, normal_dist: &Normal) -> f64 {
    let std = at_zero.2.sqrt();
    let z = (budget - at_zero.1) / std;
    let gamma3 = at_zero.3 / std.powi(3);
    let cdf_correction = normal_dist.pdf(z)
        * ((gamma3 / 6.0) * (z.powi(2) - 1.0)
            + if compute_biased {
                0.0
            } else {
                let gamma4 = at_zero.4 / std.powi(4);
                (gamma4 / 24.0) * (z.powi(3) - 3.0 * z)
                    + (gamma3 * gamma3 / 72.0) * (z.powi(5) - 10.0 * z.powi(3) + 15.0 * z)
            });
    normal_dist.cdf(z) - cdf_correction
}

impl StateBundle {
    pub fn lattice_span(&self, support_index: i64, skip_count: usize) -> (f64, f64) {
        span_of(
            self.extract_all_support_with_meta(support_index)
                .skip(skip_count), // this one genuinely can skip
        )
    }

    pub fn support_size_too_big(&self, support_index: i64, skip_count: usize) -> bool {
        let mut product: usize = 1;
        for a in self.extract_collapsed_pair(support_index, skip_count) {
            product = product.saturating_mul(a.len());
        }
        product > self.tuning.max_brute_size
    }

    pub fn saddlepoint_approximation_wrapper(
        &self,
        support_index: i64,
        skip_count: usize,
        inp_budget: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        probe: &mut Probe,
    ) -> f64 {
        self.probability_within(
            support_index,
            skip_count,
            inp_budget,
            compute_biased,
            simple_mean_log,
            probe,
            0,
        )
    }

    fn probability_within(
        &self,
        support_index: i64,
        skip_count: usize,
        inp_budget: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        probe: &mut Probe,
        depth: usize,
    ) -> f64 {
        let (min_value, max_value) = self.find_min_max(support_index, skip_count);
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
            return lattice.within(inp_budget, compute_biased);
        }
        let (span, min_delta) = self.lattice_span(support_index, skip_count);
        if self.support_size_too_big(support_index, skip_count) {
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
                    min_value,
                    max_value,
                    span,
                    inp_budget,
                    budget,
                    compute_biased,
                    simple_mean_log,
                    probe,
                    min_delta,
                    depth,
                );
            }
        }
        probe.count(Counter::Brute);

        self.brute_success_prob(
            support_index,
            skip_count,
            inp_budget,
            self.simple_avg(support_index, skip_count),
            compute_biased,
        )
    }

    /// Lugganani-Rice formula
    pub fn saddlepoint_approximation(
        &self,
        support_index: i64,
        skip_count: usize,
        min_value: f64,
        max_value: f64,
        span: f64,
        inp_budget: f64,
        budget: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        probe: &mut Probe,
        min_delta: f64,
        depth: usize,
    ) -> f64 {
        probe.count(Counter::Saddlepoint);
        let at_zero: KsTuple = self.ks(
            0.0,
            compute_biased,
            simple_mean_log,
            support_index,
            skip_count,
            probe,
        );
        let k1_zero = at_zero.1;
        let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap();

        // the main formula is singular at the mean. the threshold is complete trial-and-error heuristic
        let (out, trusted) = if (k1_zero - budget).abs() / (k1_zero.max(budget).max(1.0)) < 1e-4 {
            probe.count(Counter::Edgeworth);
            (edgeworth(at_zero, budget, compute_biased, &normal_dist), true)
        } else {
            probe.count(Counter::Lugannani);
            let (_, guess, _) = self.min_guess_max_triplet(
                budget,
                min_value,
                max_value,
                (at_zero.1, at_zero.2, at_zero.3),
            );
            let result_opt = match self.tuning.root_finder {
                RootFinder::Householder => self.householder(
                    compute_biased,
                    simple_mean_log,
                    support_index,
                    skip_count,
                    budget,
                    guess,
                    min_value,
                    max_value,
                    min_delta,
                    probe,
                ),
            };

            let result: (f64, f64, usize, KsTuple) = result_opt.unwrap();
            let theta_hat = result.0;
            let ks_tuple: KsTuple = result.3;

            let w_hat = theta_hat.signum() * (2.0 * (theta_hat * budget - ks_tuple.0)).sqrt();
            let u_hat = 2.0 / span * (span * theta_hat / 2.0).sinh() * ks_tuple.2.sqrt(); // second continuity correction

            // the approximation's slope is the density times this, so it is not a
            // distribution function where this is negative. the biased one has no 4th cumulant
            let slope = 1.0 + ks_tuple.4 / ks_tuple.2.powi(2) / 8.0
                - 5.0 * ks_tuple.3.powi(2) / ks_tuple.2.powi(3) / 24.0;
            (
                normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat),
                compute_biased || slope > 0.0,
            )
        };

        if trusted && (-FLOAT_TOL..=1.0 + FLOAT_TOL).contains(&out) {
            return out;
        }
        if depth >= MAX_CONDITION_DEPTH {
            assert!(out.is_finite());
            return out.clamp(0.0, 1.0);
        }
        probe.count(Counter::Conditioned);
        self.conditioned(
            support_index,
            skip_count,
            inp_budget,
            compute_biased,
            probe,
            depth,
        )
    }

    /// Where the sum is too far from normal for the formula above, which is one upgrade
    /// outweighing the rest: splits on that upgrade's outcomes, which is exact, and leaves
    /// a sum of the others for each.
    fn conditioned(
        &self,
        support_index: i64,
        skip_count: usize,
        inp_budget: f64,
        compute_biased: bool,
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
        let mean = self.simple_avg(support_index, skip_count);

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
            tuning: self.tuning.clone(),
        };

        let mut out: f64 = 0.0;
        for (value, p) in pairs {
            pinned.upgrade_arr[u_index].cost_dist[support_index as usize].pin(value);
            let this_mean = pinned.simple_avg(support_index, skip_count);
            if compute_biased && this_mean < FLOAT_TOL {
                continue;
            }
            let weight = if compute_biased {
                p * this_mean / mean
            } else {
                p
            };
            out += weight
                * pinned.probability_within(
                    support_index,
                    skip_count,
                    inp_budget,
                    compute_biased,
                    if compute_biased { this_mean.ln() } else { NAN },
                    probe,
                    depth + 1,
                );
        }
        out
    }
}
