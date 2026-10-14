use crate::constants::{FLOAT_TOL, float_tol};
use crate::core::cumulants::KsTuple;
use crate::core::lattice::Lattice;
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

/// E[(x - S)+] at a lattice point x from the Edgeworth series, the integral of the one above. The
/// last term is the lattice's: 1/s^2 becomes 1/s^2 - span^2/12 + ... on a lattice.
fn edgeworth_leftover(at_zero: KsTuple, x: f64, span: f64, normal_dist: &Normal) -> f64 {
    let sd = at_zero.2.sqrt();
    let z = (x - at_zero.1) / sd;
    let (gamma3, gamma4) = (at_zero.3 / sd.powi(3), at_zero.4 / sd.powi(4));
    let series = 1.0
        + gamma3 / 6.0 * z
        + gamma4 / 24.0 * (z * z - 1.0)
        + gamma3 * gamma3 / 72.0 * (z.powi(4) - 6.0 * z * z + 3.0);
    sd * (z * normal_dist.cdf(z) + normal_dist.pdf(z) * series) - span * span / 12.0 * normal_dist.pdf(z) / sd
}

/// Closer to the mean than this many standard deviations, the one saddlepoint formula for the
/// leftover is a difference of terms in 1/w^3 and the series stands in.
const LEFTOVER_EDGEWORTH_Z: f64 = 0.01;

/// What every threshold of one material shares, under one free tap outcome
pub struct Shape {
    pub min_value: f64,
    pub max_value: f64,
    pub span: f64,
    pub min_delta: f64,
    pub too_big: bool,
    pub mean: f64,
    /// the cumulants at zero
    pub zero: KsTuple,
}

impl StateBundle {
    pub fn lattice_span(&self, support_index: i64, skip_count: usize) -> (f64, f64) {
        span_of(
            self.extract_all_support_with_meta(support_index)
                .skip(skip_count), // this one genuinely can skip
        )
    }

    pub fn shape(&self, support_index: i64, skip_count: usize) -> Shape {
        let (mut min_value, mut max_value, mut mean) = (0.0, 0.0, 0.0);
        let mut product: usize = 1;
        let mut span: i64 = 0;
        let mut min_delta: f64 = INFINITY;
        let mut zero = KsTuple::default();
        for (index, &u_index) in self.special_state.iter().enumerate() {
            let support = self.support_from_index(u_index, support_index);
            let skipped = index < skip_count;
            min_value += support.access_min(skipped);
            max_value += support.access_max(skipped);
            if skipped {
                mean += support.access_min(true);
                if !support.ignore {
                    zero.1 += support.access_min(true);
                }
                continue;
            }
            mean += support.mean;
            if !support.ignore {
                zero = zero.add(support.zero);
            }
            let outcomes = support.access_collapsed(false).len();
            product = product.saturating_mul(outcomes);
            if outcomes >= 2 {
                if !support.gap_size.is_finite() {
                    panic!("non finite gap size")
                }
                min_delta = min_delta.min(support.gap_size);
                span = span.gcd(&(support.gap_size.round() as i64));
            }
        }
        Shape {
            min_value,
            max_value,
            // always do a little bit of cont correction ig cos why not
            span: if min_delta < MIN_LATTICE_SPAN { MIN_LATTICE_SPAN } else { (span as f64).max(MIN_LATTICE_SPAN) },
            min_delta,
            too_big: product > self.tuning.max_brute_size,
            mean,
            zero,
        }
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
        let shape = self.shape(support_index, skip_count);
        self.probability_within(
            support_index,
            skip_count,
            &shape,
            inp_budget,
            compute_biased,
            simple_mean_log,
            probe,
            0,
            &mut None,
            true,
        )
    }

    /// The plain probability and the biased one, of a sum the lattice is too big for. The
    /// biased root starts from the plain one.
    pub fn probability_pair(
        &self,
        support_index: i64,
        skip_count: usize,
        shape: &Shape,
        inp_budget: f64,
        simple_mean_log: f64,
        probe: &mut Probe,
    ) -> (f64, f64) {
        let mut root: Option<(f64, KsTuple)> = None;
        let plain =
            self.probability_within(support_index, skip_count, shape, inp_budget, false, NAN, probe, 0, &mut root, false);
        if !self.tuning.seed_biased {
            root = None;
        }
        let biased = self.probability_within(
            support_index,
            skip_count,
            shape,
            inp_budget,
            true,
            simple_mean_log,
            probe,
            0,
            &mut root,
            false,
        );
        (plain, biased)
    }

    fn probability_within(
        &self,
        support_index: i64,
        skip_count: usize,
        shape: &Shape,
        inp_budget: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        probe: &mut Probe,
        depth: usize,
        root: &mut Option<(f64, KsTuple)>,
        try_lattice: bool,
    ) -> f64 {
        let &Shape { min_value, max_value, span, min_delta, .. } = shape;
        if inp_budget > max_value - float_tol(max_value) {
            probe.count(Counter::Trivial);
            return 1.0;
        }

        if inp_budget < min_value - float_tol(min_value) {
            probe.count(Counter::Trivial);
            return 0.0;
        };
        if try_lattice && let Some(lattice) = self.lattice(support_index, skip_count) {
            probe.count(if matches!(lattice, Lattice::Coarse(_)) { Counter::Coarse } else { Counter::Lattice });
            return lattice.within(inp_budget, compute_biased);
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
                    compute_biased,
                    simple_mean_log,
                    probe,
                    depth,
                    root,
                );
            }
        }
        probe.count(Counter::Brute);

        self.brute_success_prob(support_index, skip_count, inp_budget, shape.mean, compute_biased)
    }

    /// E[(t - S)+] from one saddlepoint: the inversion of 1/((2/span) sinh(span s/2))^2, which is
    /// 1/s^2 on the lattice, at the lattice points either side of t, joined by a straight line,
    /// which is exact between them. None where the pair of probabilities should answer: outside
    /// the region the saddlepoint takes, no root, or an answer no distribution on the range has.
    pub fn leftover(&self, support_index: i64, skip_count: usize, shape: &Shape, thresh: f64, probe: &mut Probe) -> Option<f64> {
        let &Shape { min_value, max_value, span, min_delta, mean, .. } = shape;
        if thresh > max_value - float_tol(max_value) {
            return Some(thresh - mean);
        }
        if thresh < min_value - float_tol(min_value) {
            return Some(0.0);
        }
        let low = min_value + ((thresh - min_value) / span + FLOAT_TOL).floor() * span;
        let inside = |x: f64| min_value + min_delta + 1.0 < x && x < max_value - min_delta - 1.0;
        if !shape.too_big || max_value - min_value <= min_delta || !inside(low) || !inside(low + span) {
            probe.count(Counter::DirectFallback);
            return None;
        }
        probe.count(Counter::Direct);
        let mut root: Option<(f64, KsTuple)> = None;
        let share = (thresh - low) / span;
        let out = self.leftover_at(support_index, skip_count, shape, low, &mut root, probe).and_then(|at_low| {
            if share < FLOAT_TOL {
                return Some(at_low);
            }
            let at_high = self.leftover_at(support_index, skip_count, shape, low + span, &mut root, probe)?;
            Some(at_low + share * (at_high - at_low))
        });
        if out.is_none() {
            probe.count(Counter::DirectFallback);
        }
        out
    }

    /// At a lattice point. The root after the first starts from the one before.
    fn leftover_at(
        &self,
        support_index: i64,
        skip_count: usize,
        shape: &Shape,
        x: f64,
        root: &mut Option<(f64, KsTuple)>,
        probe: &mut Probe,
    ) -> Option<f64> {
        let &Shape { min_value, max_value, span, min_delta, mean, zero, .. } = shape;
        let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap();
        let out = if ((x - mean) / zero.2.sqrt()).abs() < LEFTOVER_EDGEWORTH_Z {
            probe.count(Counter::Edgeworth);
            edgeworth_leftover(zero, x, span, &normal_dist)
        } else {
            probe.count(Counter::Lugannani);
            let own_guess = self.min_guess_max_triplet(x, min_value, max_value, (zero.1, zero.2, zero.3)).1;
            let mut find = |guess: f64, seed: Option<KsTuple>| {
                self.householder(false, NAN, support_index, skip_count, x, guess, min_value, max_value, min_delta, seed, probe)
            };
            let (theta, _, _, ks, raw) = match root.take() {
                Some((theta, raw)) => find(theta, Some(raw)).or_else(|| find(own_guess, None)),
                None => find(own_guess, None),
            }?;
            *root = Some((theta, raw));
            // all of it at the point the root is of, then along the slope to x
            let at = ks.1;
            let d = at - mean;
            let w = theta.signum() * (2.0 * (theta * at - ks.0)).max(0.0).sqrt();
            let s_span = 2.0 / span * (span * theta / 2.0).sinh();
            let below = normal_dist.cdf(w);
            d * below
                + normal_dist.pdf(w) * (d / w - d / w.powi(3) + 1.0 / (s_span * s_span * ks.2.sqrt()))
                + (x - at) * below
        };
        let slack = 1e-9 * (max_value - min_value);
        let ceiling = (x - min_value) / (max_value - min_value) * (max_value - mean);
        (out.is_finite() && out >= (x - mean).max(0.0) - slack && out <= ceiling + slack).then_some(out)
    }

    /// Lugganani-Rice formula
    pub fn saddlepoint_approximation(
        &self,
        support_index: i64,
        skip_count: usize,
        shape: &Shape,
        inp_budget: f64,
        budget: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        probe: &mut Probe,
        depth: usize,
        root: &mut Option<(f64, KsTuple)>,
    ) -> f64 {
        probe.count(Counter::Saddlepoint);
        let &Shape { min_value, max_value, span, min_delta, .. } = shape;
        let at_zero: KsTuple = if compute_biased { shape.zero.biased(simple_mean_log) } else { shape.zero };
        let k1_zero = at_zero.1;
        let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap();

        // the main formula is singular at the mean. the threshold is complete trial-and-error heuristic
        let (out, trusted) = if (k1_zero - budget).abs() / (k1_zero.max(budget).max(1.0)) < 1e-4 {
            probe.count(Counter::Edgeworth);
            (edgeworth(at_zero, budget, compute_biased, &normal_dist), true)
        } else {
            'root: {
            probe.count(Counter::Lugannani);
            let mut find = |guess: f64, seed: Option<KsTuple>| match self.tuning.root_finder {
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
                    seed,
                    probe,
                ),
            };
            let own_guess =
                || self.min_guess_max_triplet(budget, min_value, max_value, (at_zero.1, at_zero.2, at_zero.3)).1;
            // The biased search starts from the plain root of the same threshold and the
            // cumulants there. Where those are a hair off it can be bracketed away from its
            // root, and starts again on its own.
            let result_opt = match root.take().filter(|_| compute_biased) {
                Some((theta, raw)) => find(theta, Some(raw)).or_else(|| find(own_guess(), None)),
                None => find(own_guess(), None),
            };

            // No root, which a threshold at the very end of the range can do. The series at
            // zero stands in, not trusted, so that the sum is split and tried again.
            let Some(result) = result_opt else {
                break 'root (edgeworth(at_zero, budget, compute_biased, &normal_dist), false);
            };
            let theta_hat = result.0;
            let ks_tuple: KsTuple = result.3;
            if !compute_biased {
                *root = Some((theta_hat, result.4));
            }

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
            gold_memo: AHashMap::new(),
            special_done: None,
            tuning: self.tuning.clone(),
        };

        let mut out: f64 = 0.0;
        for (value, p) in pairs {
            pinned.upgrade_arr[u_index].cost_dist[support_index as usize].pin(value);
            let shape = pinned.shape(support_index, skip_count);
            let this_mean = shape.mean;
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
                    &shape,
                    inp_budget,
                    compute_biased,
                    if compute_biased { this_mean.ln() } else { NAN },
                    probe,
                    depth + 1,
                    &mut None,
                    true,
                );
        }
        out
    }
}
