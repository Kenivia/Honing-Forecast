//! The whole distribution of a sum, exactly, where the lattice it lives on is small.
//!
//! The saddlepoint formula needs many comparable upgrades, and is far off without any
//! sign of it where one upgrade outweighs the rest. That is mostly juices and books,
//! whose totals are small numbers, so few lattice points. Where it is too big, the metric
//! takes a coarse lattice of a set number of points a standard deviation (`Coarse`), and the
//! plain probability the saddlepoint formula.

use crate::constants::float_tol;
use crate::core::probability::shape::Shape;
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;
use num::Integer;
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

/// A sum on a grid: `pmf[i]` is the mass at grid point `start + i`, counted from the sum's
/// minimum, and `added` the variance the grid added.
struct Grid {
    start: usize,
    pmf: Vec<f64>,
    added: f64,
}

/// Each outcome's mass is split between the two grid points around it in proportion, which
/// keeps every mean and adds a known variance. Grid points from `keep` on are dropped, and
/// mass under 1e-16 at either end. None where it would take more than `max_work`
/// multiplications or nothing is left.
fn grid<'a>(
    sets: impl Iterator<Item = &'a Vec<(f64, f64)>>,
    step: f64,
    keep: usize,
    max_work: usize,
    probe: &mut Probe,
) -> Option<Grid> {
    let (mut pmf, mut start, mut added, mut work) = (vec![1.0], 0usize, 0.0, 0usize);
    let mut cells: Vec<f64> = Vec::new();
    for pairs in sets {
        if pairs.len() < 2 {
            continue;
        }
        let low = pairs[0].0;
        cells.clear();
        for &(value, p) in pairs.iter() {
            let at = (value - low) / step;
            let mut below = at.floor();
            let mut share = at - below;
            if share > 1.0 - 1e-9 {
                (below, share) = (below + 1.0, 0.0);
            } else if share < 1e-9 {
                share = 0.0;
            }
            let below = below as usize;
            if cells.len() < below + 2 {
                cells.resize(below + 2, 0.0);
            }
            cells[below] += p * (1.0 - share);
            cells[below + 1] += p * share;
            added += p * share * (1.0 - share) * step * step;
        }
        let end = (start + pmf.len() + cells.len() - 1).min(keep);
        if end <= start {
            return None;
        }
        let mut next = vec![0.0; end - start];
        for (offset, &weight) in cells.iter().enumerate() {
            if offset >= next.len() {
                break;
            }
            if weight == 0.0 {
                continue;
            }
            let len = (next.len() - offset).min(pmf.len());
            for (target, mass) in next[offset..offset + len].iter_mut().zip(&pmf[..len]) {
                *target += mass * weight;
            }
            work += len;
        }
        if work > max_work {
            probe.count(Counter::CoarseTooBig);
            return None;
        }
        let (mut front, mut tail) = (0, 0.0);
        while front + 1 < next.len() && tail + next[front] < COARSE_TAIL {
            tail += next[front];
            front += 1;
        }
        let (mut back, mut tail) = (next.len(), 0.0);
        while back > front + 1 && tail + next[back - 1] < COARSE_TAIL {
            tail += next[back - 1];
            back -= 1;
        }
        next.truncate(back);
        next.drain(..front);
        start += front;
        pmf = next;
    }
    Some(Grid { start, pmf, added })
}

/// A sum the exact lattice is too big for, on a grid of about `Tuning::coarse_cells` points a
/// standard deviation. Each outcome's mass is split between the two grid points around it in
/// proportion, which keeps every mean: so t F - mean F' is exactly E[(t - S')+] of the gridded
/// sum S', and S' only has `added` more variance, which is taken off as half of it times the
/// density. Mass under 1e-16 at either end is dropped.
pub struct Coarse {
    min_value: f64,
    max_value: f64,
    mean: f64,
    step: f64,
    /// the grid point of `pmf[0]`, counted from the minimum
    start: usize,
    pmf: Vec<f64>,
    cdf: Vec<f64>,
    /// running sums of grid point times mass
    moment: Vec<f64>,
    added: f64,
}

const COARSE_TAIL: f64 = 1e-16;

impl Coarse {
    /// E[(x - G)+] of the gridded sum G, x counted from the minimum
    fn leftover(&self, x: f64) -> f64 {
        let at = x / self.step;
        let index = at.floor() - self.start as f64;
        if index < 0.0 {
            return 0.0;
        }
        let index = (index as usize).min(self.cdf.len() - 1);
        self.step * (at * self.cdf[index] - self.moment[index])
    }

    /// The probability of being within t, and the biased one that makes t F - mean F' the
    /// expected leftover less the bias.
    fn pair(&self, t: f64) -> (f64, f64) {
        if t > self.max_value - float_tol(self.max_value) {
            return (1.0, 1.0);
        }
        if t < self.min_value - float_tol(self.min_value) {
            return (0.0, 0.0);
        }
        let (half, min) = (self.step / 2.0, self.min_value);
        let raw = (self.leftover(t + half - min) - self.leftover(t - half - min)) / self.step;
        let plain = raw.clamp(0.0, 1.0);
        let x = t - min;
        let near = (x / self.step).round() as i64 - self.start as i64;
        let density: f64 = (near - 2..=near + 2)
            .filter_map(|i| usize::try_from(i).ok().and_then(|i| self.pmf.get(i)))
            .sum::<f64>()
            / (5.0 * self.step);
        // no less than what any distribution of that mean has
        let leftover =
            (self.leftover(x) - 0.5 * self.added * density).max((t - self.mean).max(0.0));
        (plain, (t * plain - leftover) / self.mean)
    }
}

impl StateBundle {
    /// The plain probability's lattice. None where it would take more than `max_lattice_work`
    /// multiplications or the gaps are not whole numbers.
    pub fn lattice(&self, support_index: i64, skip_count: usize) -> Option<Lattice> {
        let (min_value, max_value) = self.find_min_max(support_index, skip_count);
        let (span, min_delta) = self.lattice_span(support_index, skip_count);
        if !min_delta.is_finite() {
            return None;
        }
        if min_delta < 1.0 {
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

    /// The lattice's answers at every threshold, plain and biased, for each of these free
    /// tap outcomes at once. Skipping k upgrades leaves the sum of the rest, so the sums are
    /// built from the last upgrade backwards and each one is the next with one more upgrade.
    /// Nothing past the largest threshold is kept. None where `lattice` would be.
    pub fn lattice_table(
        &self,
        support_index: i64,
        skips: &[usize],
        thresholds: &[f64],
        probe: &mut Probe,
    ) -> Vec<Option<Vec<(f64, f64)>>> {
        let n = self.special_state.len();
        let support =
            |index: usize| self.support_from_index(self.special_state[index], support_index);
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
                if min_delta < 1.0
                    || range / (span.max(1) as f64) * outcomes as f64
                        > self.tuning.max_lattice_work as f64
                {
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
            self.fallback_table(support_index, skips, thresholds, &mut out, probe);
            return out;
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
        self.fallback_table(support_index, skips, thresholds, &mut out, probe);
        out
    }

    /// The outcomes the exact lattice was too big for, each on a coarse lattice of its own at
    /// `coarse_cells` grid points a standard deviation, a whole number of spans where there is a
    /// lattice, so outcomes on it stay put
    fn fallback_table(
        &self,
        support_index: i64,
        skips: &[usize],
        thresholds: &[f64],
        out: &mut [Option<Vec<(f64, f64)>>],
        probe: &mut Probe,
    ) {
        if self.tuning.coarse_cells <= 0.0 {
            return;
        }
        let top = thresholds.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        for (slot, &skip) in skips.iter().enumerate() {
            if out[slot].is_some() {
                continue;
            }
            let shape = self.shape(support_index, skip);
            let reach = top + float_tol(top) - shape.min_value;
            let sd = shape.zero.2.sqrt();
            if reach < 0.0 || !(sd > 0.0) || !shape.min_delta.is_finite() {
                continue;
            }
            let mut step = sd / self.tuning.coarse_cells;
            if shape.min_delta >= 1.0 {
                step = shape.span * (step / shape.span).floor().max(1.0);
            }
            if let Some(coarse) = self.coarse(support_index, skip, &shape, reach, step, probe) {
                let top_value = shape.max_value - float_tol(shape.max_value);
                out[slot] = Some(
                    thresholds
                        .iter()
                        .map(|&t| {
                            if t >= top_value {
                                (1.0, 1.0)
                            } else {
                                coarse.pair(t)
                            }
                        })
                        .collect(),
                );
            }
        }
    }

    /// The sum on a grid of this step, kept as far as `reach` above its minimum. None where the
    /// sum has one outcome or it would take more than `max_coarse_work`.
    fn coarse(
        &self,
        support_index: i64,
        skip_count: usize,
        shape: &Shape,
        reach: f64,
        step: f64,
        probe: &mut Probe,
    ) -> Option<Coarse> {
        let Grid { start, pmf, added } = grid(
            self.extract_collapsed_pair(support_index, skip_count),
            step,
            (reach / step) as usize + 4,
            self.tuning.max_coarse_work,
            probe,
        )?;
        probe.count(Counter::Coarse);
        let (mut below, mut moment) = (0.0, 0.0);
        let mut cdf = Vec::with_capacity(pmf.len());
        let mut moments = Vec::with_capacity(pmf.len());
        for (index, mass) in pmf.iter().enumerate() {
            below += mass;
            moment += (start + index) as f64 * mass;
            cdf.push(below);
            moments.push(moment);
        }
        Some(Coarse {
            min_value: shape.min_value,
            max_value: shape.max_value,
            mean: shape.mean,
            step,
            start,
            pmf,
            cdf,
            moment: moments,
            added,
        })
    }
}
