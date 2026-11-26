//! The exact lattice: the plain probability's (`Lattice`) and the metric's table of every free
//! tap outcome it is small enough for, built from the last upgrade back.

use super::step::StepScan;
use crate::constants::float_tol;
use crate::eval::shape::Shape;
use crate::honing_utils::state_bundle::StateBundle;
use smallvec::SmallVec;

pub struct Lattice {
    min_value: f64,
    span: f64,
    cdf: Vec<f64>,
}

impl Lattice {
    pub fn within(&self, budget: f64) -> f64 {
        let index = ((budget + float_tol(budget) - self.min_value) / self.span).floor();
        if index < 0.0 {
            return 0.0;
        }
        self.cdf[(index as usize).min(self.cdf.len() - 1)]
    }
}

impl StateBundle {
    /// The plain probability's lattice, of the sum `shape` describes. None where it would take
    /// more than `max_lattice_work` multiplications or the gaps are not whole numbers.
    pub fn lattice(&self, support_index: i64, skip_count: usize, shape: &Shape) -> Option<Lattice> {
        let (min_value, span) = (shape.min_value, shape.span);
        if !shape.min_delta.is_finite() || shape.min_delta < 1.0 {
            return None;
        }
        let work = (shape.max_value - min_value) / span * shape.outcomes as f64;
        if work > self.tuning.max_lattice_work as f64 {
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

        let mut below = 0.0;
        let cdf: Vec<f64> = pmf
            .iter()
            .map(|mass| {
                below += mass;
                below
            })
            .collect();
        Some(Lattice {
            min_value,
            span,
            cdf,
        })
    }

    /// The exact lattice's span from each upgrade on, `spans[n - index]`, as far down as it is
    /// small enough: the work only grows towards the first upgrade, so this stops where it is
    /// too much, which for a base material is after a few upgrades. 0 where nothing varies.
    fn exact_spans(&self, support_index: i64) -> SmallVec<[i64; 32]> {
        let n = self.special_state.len();
        let mut spans: SmallVec<[i64; 32]> = SmallVec::new();
        let mut scan = StepScan::default();
        for index in (0..=n).rev() {
            if index < n {
                let this = self.support_from_index(self.special_state[index], support_index);
                scan.add(this, self.tuning.max_lattice_work);
                if !scan.exact {
                    break;
                }
            }
            spans.push(scan.span);
        }
        spans
    }

    /// The outcomes the exact lattice is small enough for, answered into `out`
    pub fn exact_table(
        &self,
        support_index: i64,
        skips: &[usize],
        thresholds: &[f64],
        out: &mut [Option<Vec<(f64, f64)>>],
    ) {
        let n = self.special_state.len();
        let support =
            |index: usize| self.support_from_index(self.special_state[index], support_index);
        let top = thresholds.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // which outcomes the lattice is small enough for
        let spans = self.exact_spans(support_index);
        let wanted: SmallVec<[(usize, usize); 8]> = skips
            .iter()
            .enumerate()
            .filter(|&(_, &skip)| n - skip < spans.len() && spans[n - skip] != 0)
            .map(|(slot, &skip)| (skip, slot))
            .collect();
        let Some(&(first, _)) = wanted.iter().min() else {
            return;
        };
        // by index, as far down as the pass went
        let span = |index: usize| spans[n - index];
        let minimums: Vec<f64> = wanted
            .iter()
            .map(|&(skip, _)| self.find_min_max(support_index, skip).0)
            .collect();

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
                let kept = if reaches[index] < 0.0 {
                    1
                } else {
                    full.min((reaches[index] / unit) as usize + 2)
                };
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
            let last =
                ((self.find_min_max(support_index, index).1 - min_value) / unit).round() as usize;
            let mut answers = vec![(0.0, 0.0); thresholds.len()];
            let mut order: Vec<(usize, usize)> = Vec::with_capacity(thresholds.len());
            for (which, &budget) in thresholds.iter().enumerate() {
                let point = ((budget + float_tol(budget) - min_value) / unit).floor();
                // at or past the top every outcome is below, exactly; summing the masses
                // would leave rounding that a large threshold turns into gold
                if point >= last as f64 {
                    answers[which] = (1.0, 1.0);
                } else if point >= 0.0 {
                    order.push((point as usize, which));
                }
            }
            order.sort();
            let (mut below, mut biased_below) = (0.0, 0.0);
            let mut next_point = 0;
            for &(point, which) in &order {
                while next_point <= point && next_point < pmf.len() {
                    below += pmf[next_point];
                    biased_below +=
                        (min_value + next_point as f64 * unit) * pmf[next_point] * inv_mean;
                    next_point += 1;
                }
                answers[which] = (below, biased_below);
            }
            out[wanted[at].1] = Some(answers);
        }
    }
}
