//! The probability at the bottom of the evaluator against an exact convolution.
//!
//! `verify` checks the metric a payload asks for. This asks the evaluator for the
//! probability at many thresholds of every material instead: around the mean, where the
//! saddlepoint formula is singular, at quantiles, at the ends of the range and in between.
//! And the expected leftover E[(t - S)+], which is what the metric takes of the two, in
//! standard deviations of the sum: a coarse lattice is only meant to get that right.

use crate::report::table;
use crate::scenario::Scenario;
use hf_core::optimizer::scramble;
use hf_core::probe::{Counter, Probe};
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use rand::prelude::*;
use rayon::prelude::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;

thread_local! {
    static LAST_PANIC: RefCell<String> = RefCell::new(String::new());
}

/// multiplications an exact convolution may take before its material is left out
const CONVOLUTION_CAP: f64 = 3e8;
const SCRAMBLE_MOVES: [usize; 3] = [20, 200, 2000];
const ERROR_EDGES: [f64; 5] = [1e-6, 1e-4, 1e-2, 1e-1, f64::INFINITY];

struct Exact {
    min: f64,
    max: f64,
    gap: f64,
    cdf: Vec<f64>,
    biased_cdf: Vec<f64>,
    mean: f64,
    sd: f64,
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

/// the distribution of the sum on its own lattice
fn exact(pairs: &[Vec<(f64, f64)>]) -> Result<Exact, &'static str> {
    const UNIT: f64 = 1000.0;
    let mut gap: i64 = 0;
    let mut offsets: Vec<Vec<(i64, f64)>> = Vec::new();
    for arr in pairs {
        let mut this: Vec<(i64, f64)> = Vec::with_capacity(arr.len());
        for &(value, p) in arr {
            let scaled = value * UNIT;
            if (scaled - scaled.round()).abs() > 1e-6 {
                return Err("materials left out: values off the grid");
            }
            this.push((scaled.round() as i64, p));
        }
        let low = this.iter().map(|x| x.0).min().unwrap();
        for x in this.iter_mut() {
            x.0 -= low;
            gap = gcd(gap, x.0);
        }
        offsets.push(this);
    }
    if gap == 0 {
        return Err("materials left out: constant");
    }
    let (mut cost, mut len) = (0.0, 1.0);
    for arr in &offsets {
        if arr.len() > 1 {
            cost += len * arr.len() as f64;
        }
        len += (arr.iter().map(|x| x.0).max().unwrap() / gap) as f64;
    }
    if cost > CONVOLUTION_CAP {
        return Err("materials left out: convolution too big");
    }

    let mut pmf: Vec<f64> = vec![1.0];
    for arr in offsets.iter().filter(|arr| arr.len() > 1) {
        let top = (arr.iter().map(|x| x.0).max().unwrap() / gap) as usize;
        let mut next = vec![0.0; pmf.len() + top];
        for &(offset, p) in arr {
            for (i, mass) in pmf.iter().enumerate() {
                next[i + (offset / gap) as usize] += mass * p;
            }
        }
        pmf = next;
    }

    let end = |pick: fn(f64, f64) -> f64, start: f64| -> f64 {
        pairs
            .iter()
            .map(|arr| arr.iter().map(|x| x.0).fold(start, pick))
            .sum()
    };
    let min = end(f64::min, f64::INFINITY);
    let max = end(f64::max, f64::NEG_INFINITY);
    let gap = gap as f64 / UNIT;
    let total: f64 = pmf.iter().sum();
    let value = |i: usize| min + i as f64 * gap;
    let mean: f64 = pmf
        .iter()
        .enumerate()
        .map(|(i, p)| value(i) * p)
        .sum::<f64>()
        / total;
    let var: f64 = pmf
        .iter()
        .enumerate()
        .map(|(i, p)| (value(i) - mean).powi(2) * p)
        .sum::<f64>()
        / total;
    let (mut below, mut biased_below) = (0.0, 0.0);
    let mut cdf = Vec::with_capacity(pmf.len());
    let mut biased_cdf = Vec::with_capacity(pmf.len());
    for (i, p) in pmf.iter().enumerate() {
        below += p / total;
        biased_below += value(i) * p / total / mean;
        cdf.push(below);
        biased_cdf.push(biased_below);
    }
    Ok(Exact {
        min,
        max,
        gap,
        cdf,
        biased_cdf,
        mean,
        sd: var.sqrt(),
    })
}

impl Exact {
    fn at(&self, budget: f64, biased: bool) -> f64 {
        let index = ((budget + 1e-9 - self.min) / self.gap).floor();
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

    fn quantile(&self, q: f64) -> f64 {
        let index = self
            .cdf
            .partition_point(|&below| below < q)
            .min(self.cdf.len() - 1);
        self.min + index as f64 * self.gap
    }

    fn budgets(&self, rng: &mut StdRng) -> Vec<(f64, &'static str)> {
        let mut out: Vec<(f64, &'static str)> = Vec::new();
        for eps in [
            0.0, 1e-12, 1e-9, 1e-7, 1e-6, 1e-5, 5e-5, 9e-5, 1.1e-4, 2e-4, 1e-3, 1e-2,
        ] {
            out.push((self.mean * (1.0 + eps), "mean"));
            out.push((self.mean * (1.0 - eps), "mean"));
        }
        let nearest = self.min + ((self.mean - self.min) / self.gap).round() * self.gap;
        for step in -3..=3 {
            out.push((nearest + step as f64 * self.gap, "mean"));
        }
        for z in [0.01, 0.03, 0.1, 0.3] {
            out.push((self.mean + z * self.sd, "near"));
            out.push((self.mean - z * self.sd, "near"));
        }
        for q in [
            1e-6,
            1e-4,
            1e-3,
            0.01,
            0.05,
            0.1,
            0.25,
            0.5,
            0.75,
            0.9,
            0.95,
            0.99,
            0.999,
            1.0 - 1e-6,
        ] {
            out.push((self.quantile(q), "quantile"));
        }
        for step in 0..5 {
            out.push((self.min + step as f64 * self.gap, "edge"));
            out.push((self.max - step as f64 * self.gap, "edge"));
        }
        for _ in 0..10 {
            out.push((
                self.min + rng.random::<f64>() * (self.max - self.min),
                "uniform",
            ));
        }
        out
    }
}

#[derive(Default, Clone)]
struct Bucket {
    calls: u64,
    histogram: [u64; 5],
    max_error: f64,
    sum_error: f64,
    /// negative is an evaluator that is short, which for the leftover is optimistic
    sum_signed: f64,
    seconds: f64,
    root_iters: u64,
    max_root_iters: u64,
}

#[derive(Default)]
pub struct Summary {
    /// by (method, region, kind)
    buckets: BTreeMap<(&'static str, &'static str, &'static str), Bucket>,
    pub panics: BTreeMap<String, u64>,
    notes: BTreeMap<&'static str, u64>,
    /// (error, what it was)
    worst: Vec<(f64, String)>,
}

impl Summary {
    fn merge(mut self, other: Summary) -> Summary {
        for (key, theirs) in other.buckets {
            let mine = self.buckets.entry(key).or_default();
            mine.calls += theirs.calls;
            for (a, b) in mine.histogram.iter_mut().zip(theirs.histogram) {
                *a += b;
            }
            mine.max_error = mine.max_error.max(theirs.max_error);
            mine.sum_error += theirs.sum_error;
            mine.sum_signed += theirs.sum_signed;
            mine.seconds += theirs.seconds;
            mine.root_iters += theirs.root_iters;
            mine.max_root_iters = mine.max_root_iters.max(theirs.max_root_iters);
        }
        for (key, count) in other.panics {
            *self.panics.entry(key).or_default() += count;
        }
        for (key, count) in other.notes {
            *self.notes.entry(key).or_default() += count;
        }
        self.worst.extend(other.worst);
        self.worst.sort_by(|a, b| b.0.total_cmp(&a.0));
        self.worst.truncate(15);
        self
    }

    fn note(&mut self, key: &'static str) {
        *self.notes.entry(key).or_default() += 1;
    }

    fn panicked(&mut self, during: &str) {
        let at = LAST_PANIC.with(|last| last.borrow().clone());
        *self.panics.entry(format!("{at} ({during})")).or_default() += 1;
    }

    /// the largest error of a method that is meant to be exact
    #[cfg(test)]
    pub fn max_error_of(&self, method: &str) -> f64 {
        self.buckets
            .iter()
            .filter(|(key, _)| key.0 == method)
            .map(|(_, b)| b.max_error)
            .fold(0.0, f64::max)
    }
}

fn sweep(name: &str, state_bundle: &StateBundle, rng: &mut StdRng, summary: &mut Summary) {
    let special_probs = state_bundle.special_probs().clone();
    let mut skips: Vec<usize> = (0..special_probs.len())
        .filter(|&i| special_probs[i] >= 1e-7)
        .collect();
    if skips.len() > 3 {
        skips = vec![skips[0], skips[skips.len() / 2], skips[skips.len() - 1]];
    }
    for &skip in &skips {
        for material in 0..state_bundle.upgrade_arr[0].cost_dist.len() {
            let pairs: Vec<Vec<(f64, f64)>> = state_bundle
                .extract_collapsed_pair(material as i64, skip)
                .cloned()
                .collect();
            let reference = match exact(&pairs) {
                Ok(reference) => reference,
                Err(why) => {
                    summary.note(why);
                    continue;
                }
            };
            summary.note("materials swept");
            for (budget, region) in reference.budgets(rng) {
                for kind in ["plain", "leftover"] {
                    let mut probe = Probe::default();
                    let start = Instant::now();
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        if kind == "leftover" {
                            return state_bundle.expected_leftover(
                                material as i64,
                                skip,
                                budget,
                                &mut probe,
                            );
                        }
                        state_bundle.saddlepoint_approximation_wrapper(
                            material as i64,
                            skip,
                            budget,
                            &mut probe,
                        )
                    }));
                    let seconds = start.elapsed().as_secs_f64();
                    let count = |counter: Counter| probe.data.counts[counter as usize] as u64;
                    let method = if count(Counter::Coarse) > 0 {
                        "coarse"
                    } else if count(Counter::Direct) > count(Counter::DirectFallback) {
                        "direct"
                    } else if count(Counter::Conditioned) > 0 {
                        "conditioned"
                    } else if count(Counter::Edgeworth) > 0 {
                        "edgeworth"
                    } else if count(Counter::Saddlepoint) > 0 {
                        "lugannani"
                    } else if count(Counter::Lattice) > 0 {
                        "lattice"
                    } else if count(Counter::Brute) > 0 {
                        "brute"
                    } else {
                        "trivial"
                    };
                    let bucket = summary.buckets.entry((method, region, kind)).or_default();
                    bucket.calls += 1;
                    bucket.seconds += seconds;
                    bucket.root_iters += count(Counter::RootIter);
                    bucket.max_root_iters = bucket.max_root_iters.max(count(Counter::RootIter));
                    let Ok(got) = result else {
                        summary.panicked(method);
                        continue;
                    };
                    let truth = if kind == "leftover" {
                        budget * reference.at(budget, false)
                            - reference.mean * reference.at(budget, true)
                    } else {
                        reference.at(budget, false)
                    };
                    let signed = if kind == "leftover" {
                        (got - truth) / reference.sd
                    } else {
                        got - truth
                    };
                    let error = signed.abs();
                    bucket.histogram[ERROR_EDGES
                        .iter()
                        .position(|&edge| error < edge)
                        .unwrap_or(4)] += 1;
                    bucket.max_error = bucket.max_error.max(error);
                    bucket.sum_error += error;
                    bucket.sum_signed += signed;
                    if summary.worst.len() < 15 || error > summary.worst[summary.worst.len() - 1].0
                    {
                        let upgrades = pairs.iter().filter(|arr| arr.len() > 1).count();
                        summary.worst.push((
                            error,
                            format!(
                                "{name} material {material} skip {skip} {method} {region} {kind} budget {budget} got {got:.5} exact {truth:.5}, {upgrades} upgrades, {} lattice points",
                                reference.cdf.len(),
                            ),
                        ));
                        summary.worst.sort_by(|a, b| b.0.total_cmp(&a.0));
                        summary.worst.truncate(15);
                    }
                }
            }
        }
    }
}

fn one_step(name: &str, base: StateBundle, scrambles: usize) -> Summary {
    let mut summary = Summary::default();
    if base.upgrade_arr.is_empty() {
        return summary;
    }
    let mut rng = StdRng::seed_from_u64(name.bytes().fold(7u64, |seed, byte| {
        seed.wrapping_mul(131).wrapping_add(byte as u64)
    }));
    for moves in std::iter::once(0).chain(SCRAMBLE_MOVES.into_iter().take(scrambles)) {
        let made = catch_unwind(AssertUnwindSafe(|| {
            let mut this = if moves == 0 {
                base.clone()
            } else {
                scramble(&mut rng, base.clone(), moves)
            };
            this.update_prob_dist();
            this.update_cost_dist();
            this.compute_special_probs(false);
            this
        }));
        let Ok(mut state_bundle) = made else {
            summary.panicked("scrambling");
            continue;
        };
        summary.note("states");
        if catch_unwind(AssertUnwindSafe(|| {
            state_bundle.metric_router(&mut Probe::default())
        }))
        .is_err()
        {
            summary.panicked("the metric");
        }
        sweep(name, &state_bundle, &mut rng, &mut summary);
    }
    summary
}

pub fn run(scenarios: &[Scenario], scrambles: usize, tuning: &Tuning) -> Summary {
    // a panic is counted by where it was, not printed
    std::panic::set_hook(Box::new(|info| {
        let at = info
            .location()
            .map_or(String::new(), |at| format!("{}:{}", at.file(), at.line()));
        LAST_PANIC.with(|last| *last.borrow_mut() = at);
    }));
    let steps: Vec<(String, &hf_core::payload::Payload)> = scenarios
        .iter()
        .flat_map(|scenario| {
            scenario
                .steps
                .iter()
                .enumerate()
                .map(|(step, payload)| (format!("{}:{step}", scenario.name), payload))
        })
        .collect();
    let summary = steps
        .par_iter()
        .map(|(name, payload)| {
            let mut state_bundle = StateBundle::init_from_payload((*payload).clone());
            state_bundle.tuning = tuning.clone();
            one_step(name, state_bundle, scrambles)
        })
        .reduce(Summary::default, Summary::merge);
    let _ = std::panic::take_hook();
    summary
}

pub fn report(summary: &Summary) {
    for (note, count) in &summary.notes {
        println!("{note}: {count}");
    }
    let mut out = table(&[
        "method",
        "region",
        "kind",
        "calls",
        "<1e-6",
        "<1e-4",
        "<1e-2",
        "<1e-1",
        ">=1e-1",
        "max error",
        "mean error",
        "mean signed",
        "us/call",
        "root iters",
        "max iters",
    ]);
    for ((method, region, kind), bucket) in &summary.buckets {
        let mut row = vec![
            method.to_string(),
            region.to_string(),
            kind.to_string(),
            bucket.calls.to_string(),
        ];
        row.extend(bucket.histogram.iter().map(|count| count.to_string()));
        row.extend([
            format!("{:.1e}", bucket.max_error),
            format!("{:.1e}", bucket.sum_error / bucket.calls as f64),
            format!("{:+.1e}", bucket.sum_signed / bucket.calls as f64),
            format!("{:.1}", bucket.seconds / bucket.calls as f64 * 1e6),
            format!("{:.2}", bucket.root_iters as f64 / bucket.calls as f64),
            bucket.max_root_iters.to_string(),
        ]);
        out.add_row(row);
    }
    println!("{out}");
    println!(
        "leftover errors are in standard deviations of the sum; a negative mean signed leftover is optimistic"
    );
    println!("worst:");
    for (error, what) in &summary.worst {
        println!("  {error:.4}  {what}");
    }
    println!("panics: {}", summary.panics.values().sum::<u64>());
    for (at, count) in &summary.panics {
        println!("  {count:>8}  {at}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::dummy;

    #[test]
    fn probabilities_hold_up_under_a_sweep() {
        let summary = run(&[dummy()], 1, &Tuning::default());
        assert!(summary.panics.is_empty(), "{:?}", summary.panics);
        assert!(summary.max_error_of("brute") < 1e-9);
        assert!(summary.max_error_of("lattice") < 1e-9);
        assert!(summary.max_error_of("trivial") < 1e-9);
    }
}
