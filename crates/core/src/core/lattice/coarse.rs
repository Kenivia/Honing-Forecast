//! The coarse lattice: a sum the exact lattice is too big for, on a grid of about
//! `Tuning::coarse_cells` points a standard deviation

use crate::constants::float_tol;
use crate::core::common::mix;
use crate::core::shape::Shape;
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;
use ahash::AHashMap;
use smallvec::SmallVec;

/// An upgrade's coarse grid cells and the variance they add, by upgrade, state, material and
/// step (`Tuning::coarse_cache`). A copy of the state starts without it.
#[derive(Default, Debug)]
pub struct CoarseMemo(pub AHashMap<u64, (Vec<f64>, f64)>);

impl Clone for CoarseMemo {
    fn clone(&self) -> Self {
        CoarseMemo::default()
    }
}

const COARSE_MEMO_SIZE: usize = 1 << 15;

/// A sum on a grid: `pmf[i]` is the mass at grid point `start + i`, counted from the sum's
/// minimum, and `added` the variance the grid added.
struct Grid {
    start: usize,
    pmf: Vec<f64>,
    added: f64,
}

impl Grid {
    fn new() -> Grid {
        Grid {
            start: 0,
            pmf: vec![1.0],
            added: 0.0,
        }
    }

    /// One upgrade's outcomes added to the sum: each outcome's mass is split between the two
    /// grid points around it in proportion, which keeps every mean and adds a known variance.
    /// Grid points from `keep` on are dropped, and mass under 1e-16 at either end. The
    /// multiplications it took, or None where nothing is left.
    fn add(
        &mut self,
        pairs: &[(f64, f64)],
        step: f64,
        keep: usize,
        cells: &mut Vec<f64>,
    ) -> Option<usize> {
        bin(pairs, step, cells, &mut self.added);
        self.convolve(cells, keep)
    }

    /// The sum with an upgrade's grid cells added (`bin`)
    fn convolve(&mut self, cells: &[f64], keep: usize) -> Option<usize> {
        let mut work = 0;
        let (start, pmf) = (self.start, &self.pmf);
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
        self.start += front;
        self.pmf = next;
        Some(work)
    }
}

/// One upgrade's outcomes on the grid, counted from its lowest: each outcome's mass split between
/// the two grid points around it in proportion, the variance that adds summed into `added`
fn bin(pairs: &[(f64, f64)], step: f64, cells: &mut Vec<f64>, added: &mut f64) {
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
        *added += p * share * (1.0 - share) * step * step;
    }
}

/// The sum of these upgrades on a grid (`Grid::add`). None where it would take more than
/// `max_work` multiplications or nothing is left.
fn grid<'a>(
    sets: impl Iterator<Item = &'a Vec<(f64, f64)>>,
    step: f64,
    keep: usize,
    max_work: usize,
    probe: &mut Probe,
) -> Option<Grid> {
    let (mut out, mut work, mut cells) = (Grid::new(), 0usize, Vec::new());
    for pairs in sets {
        if pairs.len() < 2 {
            continue;
        }
        work += out.add(pairs, step, keep, &mut cells)?;
        if work > max_work {
            probe.count(Counter::CoarseTooBig);
            return None;
        }
    }
    Some(out)
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
    /// (shape, reach, step) of an outcome the coarse lattice answers; None where it is trivial
    fn coarse_wanted(
        &self,
        support_index: i64,
        skip: usize,
        top: f64,
    ) -> Option<(Shape, f64, f64)> {
        let shape = self.shape(support_index, skip);
        let reach = top + float_tol(top) - shape.min_value;
        let sd = shape.zero.2.sqrt();
        if reach < 0.0 || !(sd > 0.0) || !shape.min_delta.is_finite() {
            return None;
        }
        let step = self.coarse_step(sd, shape.min_delta, shape.span);
        Some((shape, reach, step))
    }

    /// The outcomes the exact lattice was too big for, each on a coarse lattice of its own at
    /// `coarse_cells` grid points a standard deviation, a whole number of spans where there is a
    /// lattice, so outcomes on it stay put
    pub fn coarse_table(
        &self,
        support_index: i64,
        skips: &[usize],
        thresholds: &[f64],
        out: &mut [Option<Vec<(f64, f64)>>],
        mut coarse: Option<&mut CoarseMemo>,
        shared_step: Option<f64>,
        probe: &mut Probe,
    ) {
        if self.tuning.coarse_cells <= 0.0 {
            return;
        }
        let cache = self.tuning.coarse_cache && self.tuning.coarse_shared;
        let top = thresholds.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        // (skip, slot, shape, reach, step) of each outcome to answer
        let mut wanted: SmallVec<[(usize, usize, Shape, f64, f64); 8]> = SmallVec::new();
        for (slot, &skip) in skips.iter().enumerate() {
            if out[slot].is_some() {
                continue;
            }
            let Some((shape, reach, step)) = self.coarse_wanted(support_index, skip, top) else {
                continue;
            };
            if self.tuning.coarse_shared {
                wanted.push((skip, slot, shape, reach, step));
                continue;
            }
            if let Some(coarse) = self.coarse(support_index, skip, &shape, reach, step, probe) {
                out[slot] = Some(coarse.answers(thresholds));
            }
        }
        if wanted.is_empty() {
            return;
        }
        // shared: one pass from the last upgrade back on the finest of their steps, each outcome
        // read off as the pass reaches it
        let step = shared_step.unwrap_or(wanted.iter().fold(f64::INFINITY, |a, w| a.min(w.4)));
        let first = wanted.iter().map(|w| w.0).min().unwrap();
        let n = self.special_state.len();
        let mut keeps = vec![0usize; n + 1];
        let mut keep = 0usize;
        for (index, slot) in keeps.iter_mut().enumerate().skip(first) {
            if let Some(w) = wanted.iter().find(|w| w.0 == index) {
                keep = keep.max((w.3 / step) as usize + 4);
            }
            *slot = keep;
        }
        let (mut sum, mut work, mut cells) = (Grid::new(), 0usize, Vec::new());
        for index in (first..n).rev() {
            let u_index = self.special_state[index];
            let pairs = self
                .support_from_index(u_index, support_index)
                .access_collapsed(false);
            if pairs.len() >= 2 {
                let more =
                    if cache {
                        // each upgrade's cells and variance apart, kept by what they are made of
                        let made_of = mix(u_index as u64, self.upgrade_arr[u_index].state.hash);
                        let key = mix(mix(made_of, support_index as u64), step.to_bits());
                        let hit = coarse.as_deref().and_then(|memo| memo.0.get(&key)).map(
                            |(kept, part)| {
                                cells.clear();
                                cells.extend_from_slice(kept);
                                *part
                            },
                        );
                        let part = match hit {
                            Some(part) => part,
                            None => {
                                let mut part = 0.0;
                                bin(pairs, step, &mut cells, &mut part);
                                if let Some(memo) = coarse.as_deref_mut() {
                                    if memo.0.len() >= COARSE_MEMO_SIZE {
                                        memo.0.clear();
                                    }
                                    memo.0.insert(key, (cells.clone(), part));
                                }
                                part
                            }
                        };
                        sum.added += part;
                        sum.convolve(&cells, keeps[index])
                    } else {
                        sum.add(pairs, step, keeps[index], &mut cells)
                    };
                let Some(more) = more else {
                    break;
                };
                work += more;
                if work > self.tuning.max_coarse_work {
                    probe.count(Counter::CoarseTooBig);
                    break;
                }
            }
            for w in wanted.iter().filter(|w| w.0 == index) {
                probe.count(Counter::Coarse);
                let grid = Grid {
                    start: sum.start,
                    pmf: sum.pmf.clone(),
                    added: sum.added,
                };
                out[w.1] = Some(Coarse::of(grid, &w.2, step).answers(thresholds));
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
        let grid = grid(
            self.extract_collapsed_pair(support_index, skip_count),
            step,
            (reach / step) as usize + 4,
            self.tuning.max_coarse_work,
            probe,
        )?;
        probe.count(Counter::Coarse);
        Some(Coarse::of(grid, shape, step))
    }
}

impl Coarse {
    fn of(Grid { start, pmf, added }: Grid, shape: &Shape, step: f64) -> Coarse {
        let (mut below, mut moment) = (0.0, 0.0);
        let mut cdf = Vec::with_capacity(pmf.len());
        let mut moments = Vec::with_capacity(pmf.len());
        for (index, mass) in pmf.iter().enumerate() {
            below += mass;
            moment += (start + index) as f64 * mass;
            cdf.push(below);
            moments.push(moment);
        }
        Coarse {
            min_value: shape.min_value,
            max_value: shape.max_value,
            mean: shape.mean,
            step,
            start,
            pmf,
            cdf,
            moment: moments,
            added,
        }
    }

    /// the pair at each threshold, (1, 1) at or past the top
    fn answers(&self, thresholds: &[f64]) -> Vec<(f64, f64)> {
        let top_value = self.max_value - float_tol(self.max_value);
        thresholds
            .iter()
            .map(|&t| {
                if t >= top_value {
                    (1.0, 1.0)
                } else {
                    self.pair(t)
                }
            })
            .collect()
    }
}
