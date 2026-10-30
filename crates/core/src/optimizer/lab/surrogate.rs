//! A moment surrogate of the metric, searched hard and checked by the true one.
//!
//! Each upgrade's use of a material is independent of the others', so per free tap outcome the
//! mean and cumulants of every material's total are sums. The surrogate takes each material's
//! expected gold from those by an Edgeworth leftover, plus a linear correction that matches its
//! marginal prices to the true shadow prices at the last true state. It gives every normal
//! upgrade its best plan against tables of what each amount is worth (the best response's
//! recursion, copied here so it can take tables), ranks advanced, free tap order and chest
//! moves, and runs whole descents (with kick and repair joint moves) from restarts that kick
//! the prices or the state. The true metric checks every proposal before it is kept, and the
//! true best response polishes where the surrogate has nothing better; the last share of the
//! budget also tries wider responses and single block flips ranked by the true response.
//! See `arena_data/lab/iteration1/attempts/surrogate.md`.

use super::evaluate;
use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::constants::{FLOAT_TOL, SPECIAL_TOL, float_tol};
use crate::materials::NUM_BASE_MATS;
use crate::optimizer::response::{Model, Response};
use crate::optimizer::{Previous, Solved};
use crate::probe::Probe;
use crate::state::OneState;
use crate::state_bundle::{StateBundle, StateEssence};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use smallvec::{SmallVec, smallvec};
use statrs::function::erf::erfc;
use std::cell::{Cell, RefCell};

/// the leftover takes the third and fourth cumulants (Edgeworth), not the normal alone
const SKEW: bool = true;
/// match the surrogate's marginal prices to the true shadow prices
const CALIBRATE: bool = true;
/// amounts a surrogate table is worked out at, the rest on straight lines
const POINTS: usize = 24;
/// rounds of a surrogate descent
const DESCENT: usize = 6;
/// surrogate-ranked moves of a kind the true metric checks
const TOP: usize = 3;
/// true evaluations at effort 1: a base and per upgrade
const BASE_EVALS: f64 = 75000.0;
const PER_UPGRADE: f64 = 3000.0;
/// restarts in a row without a better best before stopping, at effort 1
const STALL: f64 = 40.0;
/// a price kick multiplies prices by up to e to this, either way
const KICK: f64 = 0.7;
/// plans taking an id combination on every tap a response also improves from
const START_COMBOS: usize = 12;
/// kick and repair trials a restart makes, at effort 1
const KICK_TRIALS: f64 = 24.0;
/// single block flips of a plan the true metric checks
const FLIP_TOP: usize = 4;
/// the share of the budget kept for the last phase
const FLIP_SHARE: f64 = 0.25;
/// what the surrogate's work counts for: gold functions and recursion steps an evaluation
const GOLDS_PER_EVAL: f64 = 6000.0;
const STEPS_PER_EVAL: f64 = 2000.0;

#[derive(Clone, Copy, Default, Debug)]
struct Mom {
    mean: f64,
    var: f64,
    k3: f64,
    k4: f64,
    min: f64,
    max: f64,
}

impl Mom {
    fn of(pairs: &[(f64, f64)]) -> Mom {
        let total: f64 = pairs.iter().map(|p| p.1).sum();
        let mean = pairs.iter().map(|p| p.0 * p.1).sum::<f64>() / total;
        let (mut m2, mut m3, mut m4) = (0.0, 0.0, 0.0);
        let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
        for &(x, p) in pairs {
            let d = x - mean;
            m2 += p * d * d;
            m3 += p * d * d * d;
            m4 += p * d * d * d * d;
            min = min.min(x);
            max = max.max(x);
        }
        let (m2, m3, m4) = (m2 / total, m3 / total, m4 / total);
        Mom {
            mean,
            var: m2,
            k3: m3,
            k4: m4 - 3.0 * m2 * m2,
            min,
            max,
        }
    }
    fn add(&self, o: &Mom) -> Mom {
        Mom {
            mean: self.mean + o.mean,
            var: self.var + o.var,
            k3: self.k3 + o.k3,
            k4: self.k4 + o.k4,
            min: self.min + o.min,
            max: self.max + o.max,
        }
    }
    fn sub(&self, o: &Mom) -> Mom {
        Mom {
            mean: self.mean - o.mean,
            var: (self.var - o.var).max(0.0),
            k3: self.k3 - o.k3,
            k4: self.k4 - o.k4,
            min: self.min - o.min,
            max: self.max - o.max,
        }
    }
    fn shift(&self, a: f64) -> Mom {
        Mom {
            mean: self.mean + a,
            min: self.min + a,
            max: self.max + a,
            ..*self
        }
    }
}

/// E[(t - S)+]
fn leftover(t: f64, m: &Mom) -> f64 {
    if t >= m.max - float_tol(m.max) {
        return t - m.mean;
    }
    if t <= m.min + float_tol(m.min) {
        return 0.0;
    }
    let trivial = (t - m.mean).max(0.0);
    if m.var <= 1e-18 * (1.0 + m.mean * m.mean) {
        return trivial;
    }
    let s = m.var.sqrt();
    let z = (t - m.mean) / s;
    let cdf = 0.5 * erfc(-z / std::f64::consts::SQRT_2);
    let pdf = (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let series = if SKEW {
        let (g3, g4) = (m.k3 / (s * s * s), m.k4 / (m.var * m.var));
        1.0 + g3 / 6.0 * z
            + g4 / 24.0 * (z * z - 1.0)
            + g3 * g3 / 72.0 * (z.powi(4) - 6.0 * z * z + 3.0)
    } else {
        1.0
    };
    (s * (z * cdf + pdf * series)).clamp(trivial, t - m.min)
}

/// expected gold of a material whose total has these moments
fn gold_of(bands: &[(f64, f64)], m: &Mom) -> f64 {
    COST.set(COST.get() + 1.0 / GOLDS_PER_EVAL);
    let (last_thresh, last_price) = *bands.last().unwrap();
    let mut out = last_price * (last_thresh - m.mean);
    for pair in bands.windows(2) {
        let gap = pair[0].1 - pair[1].1;
        if gap.abs() >= FLOAT_TOL {
            out += gap * leftover(pair[1].0, m);
        }
    }
    out
}

fn is_linear(bands: &[(f64, f64)]) -> bool {
    !bands
        .windows(2)
        .any(|pair| (pair[0].1 - pair[1].1).abs() >= FLOAT_TOL)
}

/// f at up to POINTS of the amounts, on straight lines between
fn sampled(amounts: &[f64], f: impl Fn(f64) -> f64) -> Vec<f64> {
    let last = amounts.len() - 1;
    if last == 0 {
        return vec![f(amounts[0])];
    }
    let mut points: Vec<usize> = if amounts.len() <= POINTS {
        (0..=last).collect()
    } else {
        (0..POINTS).map(|p| p * last / (POINTS - 1)).collect()
    };
    points.dedup();
    let values: Vec<f64> = points.iter().map(|&at| f(amounts[at])).collect();
    let mut pair = 0;
    (0..=last)
        .map(|at| {
            while pair + 2 < points.len() && at > points[pair + 1] {
                pair += 1;
            }
            let (low, high) = (points[pair], points[pair + 1]);
            let share = (at - low) as f64 / (high - low) as f64;
            values[pair] + share * (values[pair + 1] - values[pair])
        })
        .collect()
}

/// The surrogate: every upgrade's moments per row, the free tap outcomes, the correction.
struct Sur {
    honed: Vec<Vec<Mom>>,
    skipped: Vec<Vec<f64>>,
    order: Vec<usize>,
    w: Vec<(usize, f64)>,
    corr: Vec<f64>,
    price: Vec<f64>,
    rows: usize,
}

impl Sur {
    fn new(b: &StateBundle) -> Sur {
        let rows = b.prep_output.bands[b.prep_output.optimizer_plan]
            .len()
            .min(b.upgrade_arr[0].cost_dist.len());
        let n = b.upgrade_arr.len();
        Sur {
            honed: vec![vec![Mom::default(); rows]; n],
            skipped: vec![vec![0.0; rows]; n],
            order: (0..n).collect(),
            w: vec![(0, 1.0)],
            corr: vec![0.0; rows],
            price: vec![0.0; rows],
            rows,
        }
    }

    /// from the upgrade's supports, which must be of its state
    fn refresh(&mut self, b: &StateBundle, u: usize) {
        for row in 0..self.rows {
            let support = &b.upgrade_arr[u].cost_dist[row];
            self.honed[u][row] = Mom::of(support.access_collapsed(false));
            self.skipped[u][row] = support.access_min(true);
        }
    }

    fn refresh_order(&mut self, b: &mut StateBundle) {
        b.compute_special_probs(false);
        self.order.clone_from(&b.special_state);
        self.w = b
            .special_probs()
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, p)| p >= SPECIAL_TOL)
            .collect();
    }

    fn refresh_all(&mut self, b: &mut StateBundle) {
        b.update_prob_dist();
        b.update_cost_dist();
        for u in 0..b.upgrade_arr.len() {
            self.refresh(b, u);
        }
        self.refresh_order(b);
    }

    /// the moments of each row's total, per live free tap outcome
    fn sums(&self) -> Vec<Vec<Mom>> {
        let n = self.order.len();
        let mut out = vec![vec![Mom::default(); self.rows]; self.w.len()];
        let mut suffix = vec![Mom::default(); n + 1];
        for row in 0..self.rows {
            for i in (0..n).rev() {
                suffix[i] = suffix[i + 1].add(&self.honed[self.order[i]][row]);
            }
            let (mut pre, mut i) = (0.0, 0);
            for (slot, &(k, _)) in self.w.iter().enumerate() {
                while i < k {
                    pre += self.skipped[self.order[i]][row];
                    i += 1;
                }
                out[slot][row] = suffix[k].shift(pre);
            }
        }
        out
    }

    fn total(&self, b: &StateBundle) -> f64 {
        self.total_with(b, true)
    }

    fn total_with(&self, b: &StateBundle, corrected: bool) -> f64 {
        let bands = &b.prep_output.bands[b.prep_output.optimizer_plan];
        let agg = self.sums();
        let mut out = 0.0;
        for (slot, &(_, w)) in self.w.iter().enumerate() {
            for row in 0..self.rows {
                let m = &agg[slot][row];
                let corr = if corrected { self.corr[row] } else { 0.0 };
                out += w * (gold_of(&bands[row], m) + corr * m.mean);
            }
        }
        out
    }

    /// after a true evaluation of the state as it is
    fn calibrate(&mut self, b: &StateBundle, probe: &mut Probe) {
        let shadow = b.shadow_prices(probe);
        let bands = &b.prep_output.bands[b.prep_output.optimizer_plan];
        let agg = self.sums();
        for row in 0..self.rows.min(shadow.len()) {
            self.price[row] = shadow[row];
            if !CALIBRATE {
                continue;
            }
            let mut slope = 0.0;
            for (slot, &(_, w)) in self.w.iter().enumerate() {
                let m = &agg[slot][row];
                let h = 0.5f64.max(1e-4 * m.mean.abs()).max(1e-3 * m.var.sqrt());
                slope += w
                    * (gold_of(&bands[row], &m.shift(h)) - gold_of(&bands[row], &m.shift(-h)))
                    / (2.0 * h);
            }
            self.corr[row] = -shadow[row] - slope;
        }
    }

    /// what each amount a normal upgrade can use is worth to it, by the surrogate
    fn table(&self, b: &StateBundle, u: usize) -> Option<Table> {
        let up = &b.upgrade_arr[u];
        let ji = &b.prep_output.juice_info;
        let model = Model::of(up, ji, b.block());
        if model.pools.iter().all(|pool| pool.is_empty()) {
            return None;
        }
        let place = self.order.iter().position(|&x| x == u).unwrap();
        let slots: Vec<(usize, f64)> = self
            .w
            .iter()
            .enumerate()
            .filter(|(_, w)| w.0 <= place)
            .map(|(slot, &(_, w))| (slot, w))
            .collect();
        if slots.is_empty() {
            return None;
        }
        let weight: f64 = slots.iter().map(|s| s.1).sum();
        let agg = self.sums();
        let bands = &b.prep_output.bands[b.prep_output.optimizer_plan];
        let worth = |row: usize, amounts: &[f64]| -> Vec<f64> {
            let bd = &bands[row];
            let corr = self.corr[row];
            if is_linear(bd) {
                let price = bd.last().unwrap().1;
                return amounts
                    .iter()
                    .map(|a| (corr - price) * weight * a)
                    .collect();
            }
            let rests: Vec<(f64, Mom)> = slots
                .iter()
                .map(|&(slot, w)| (w, agg[slot][row].sub(&self.honed[u][row])))
                .collect();
            sampled(amounts, |a| {
                rests
                    .iter()
                    .map(|(w, rest)| w * gold_of(bd, &rest.shift(a)))
                    .sum::<f64>()
                    + corr * weight * a
            })
        };
        let len = model.len;
        let mut base_worth = vec![0.0; len + 1];
        for row in (0..NUM_BASE_MATS).filter(|&row| up.costs[row] > 0.0 && row < self.rows) {
            let start = if up.unlocked {
                0.0
            } else {
                up.unlock_costs[row]
            };
            let amounts: Vec<f64> = (0..=len)
                .map(|t| start + t as f64 * up.costs[row])
                .collect();
            for (sum, x) in base_worth.iter_mut().zip(worth(row, &amounts)) {
                *sum += x;
            }
        }
        let juice_worth = model
            .pools
            .iter()
            .flatten()
            .map(|&id| {
                let row = b.prep_output.table.juice_row(id);
                let amount = ji
                    .access(id, up.piece_type_usize, up.upgrade_index)
                    .normal_amt_used as f64;
                let amounts: Vec<f64> = (0..=len).map(|t| t as f64 * amount).collect();
                let values = worth(row, &amounts);
                values.iter().map(|x| x - values[0]).collect()
            })
            .collect();
        Some(Table::assemble(
            model,
            base_worth,
            juice_worth,
            b.tuning.response_mixed,
        ))
    }
}

// ---- the best response's recursion over tables of worth, as in `optimizer/response.rs` ----

struct Table {
    model: Model,
    places: Vec<Vec<usize>>,
    base_worth: Vec<f64>,
    juice_worth: Vec<Vec<f64>>,
    mixed: bool,
}

#[derive(Default)]
struct Memo {
    cells: Vec<(u32, u8, f64)>,
    stamp: u32,
}

thread_local! {
    static MEMO: RefCell<Memo> = RefCell::new(Memo::default());
    /// what the surrogate has cost on this thread, in evaluations
    static COST: Cell<f64> = const { Cell::new(0.0) };
}

struct Kept {
    options: Vec<usize>,
    worth: Vec<f64>,
}

type Picks = SmallVec<[(usize, usize, usize, usize); 3]>;

fn clash(picks: &Picks, set: usize) -> bool {
    (0..picks.len())
        .any(|a| (0..a).any(|b| set >> a & 1 == 1 && set >> b & 1 == 1 && picks[a].3 == picks[b].3))
}

impl Table {
    fn assemble(
        model: Model,
        base_worth: Vec<f64>,
        juice_worth: Vec<Vec<f64>>,
        mixed: bool,
    ) -> Table {
        let mut next = 0;
        let places = model
            .pools
            .iter()
            .map(|ids| {
                next += ids.len();
                (next - ids.len()..next).collect()
            })
            .collect();
        Table {
            model,
            places,
            base_worth,
            juice_worth,
            mixed,
        }
    }

    fn score(&self, indices: &[usize]) -> f64 {
        let model = &self.model;
        let (mut artisan, mut alive, mut out) = (model.artisan, 1.0, 0.0);
        let mut used = vec![0; self.juice_worth.len()];
        let done = |taps: usize, used: &[usize]| -> f64 {
            self.base_worth[taps]
                + used
                    .iter()
                    .zip(&self.juice_worth)
                    .map(|(&times, worth)| worth[times])
                    .sum::<f64>()
        };
        for count in 0.. {
            if artisan >= 1.0 || count + 1 >= model.len {
                return out + alive * done(count + 1, &used);
            }
            let chance = model.chance(count, model.extra[indices[count]]);
            if chance == 1.0 {
                return out + alive * done(count + 1, &used);
            }
            for (pool, stride) in model.strides.iter().enumerate() {
                if let Some(which) =
                    (indices[count] / stride % (model.pools[pool].len() + 1)).checked_sub(1)
                {
                    used[self.places[pool][which]] += 1;
                }
            }
            out += alive * chance * done(count + 1, &used);
            alive *= 1.0 - chance;
            artisan = model.step(artisan, chance);
        }
        unreachable!()
    }

    fn slot(&self, picks: &Picks, count: usize, used: [usize; 3]) -> usize {
        picks
            .iter()
            .zip(used)
            .rev()
            .fold(0, |at, (pick, times)| at * (pick.2 + 1) + times)
            * (self.model.len + 1)
            + count
    }

    fn done(
        &self,
        picks: &Picks,
        kept: &Kept,
        charged: usize,
        taps: usize,
        used: [usize; 3],
    ) -> f64 {
        self.base_worth[taps]
            + kept.worth[charged]
            + picks
                .iter()
                .zip(used)
                .map(|(pick, times)| self.juice_worth[pick.1][times])
                .sum::<f64>()
    }

    fn taken(&self, picks: &Picks, set: usize, mut used: [usize; 3]) -> (usize, [usize; 3]) {
        let mut option = 0;
        for (at, pick) in picks.iter().enumerate() {
            if set >> at & 1 == 1 {
                option += pick.0;
                used[at] += 1;
            }
        }
        (option, used)
    }

    #[allow(clippy::too_many_arguments)]
    fn play(
        &self,
        picks: &Picks,
        kept: &Kept,
        start: usize,
        mut artisan: f64,
        mut used: [usize; 3],
        set: usize,
        memo: &mut Memo,
    ) -> f64 {
        let model = &self.model;
        let (mut value, mut alive) = (0.0, 1.0);
        for count in start..start + model.block {
            if artisan >= 1.0 || count + 1 >= model.len {
                return value + alive * self.done(picks, kept, count, count + 1, used);
            }
            let (option, next) = self.taken(picks, set, used);
            let chance = model.chance(count, model.extra[kept.options[count] + option]);
            if chance == 1.0 {
                return value + alive * self.done(picks, kept, count, count + 1, used);
            }
            value += alive * chance * self.done(picks, kept, count + 1, count + 1, next);
            alive *= 1.0 - chance;
            artisan = model.step(artisan, chance);
            used = next;
        }
        value + alive * self.go(picks, kept, start + model.block, artisan, used, memo)
    }

    fn go(
        &self,
        picks: &Picks,
        kept: &Kept,
        count: usize,
        artisan: f64,
        used: [usize; 3],
        memo: &mut Memo,
    ) -> f64 {
        let model = &self.model;
        if artisan >= 1.0 || count + 1 >= model.len {
            return self.done(picks, kept, count, count + 1, used);
        }
        let slot = self.slot(picks, count, used);
        if memo.cells[slot].0 == memo.stamp {
            return memo.cells[slot].2;
        }
        COST.set(COST.get() + (model.block << picks.len()) as f64 / STEPS_PER_EVAL);
        let mut best = (f64::NEG_INFINITY, 0);
        for set in (0..1usize << picks.len()).filter(|&set| !clash(picks, set)) {
            let value = self.play(picks, kept, count, artisan, used, set, memo);
            if set == 0 || value > best.0 + best.0.abs() * 1e-13 {
                best = (value, set as u8);
            }
        }
        memo.cells[slot] = (memo.stamp, best.1, best.0);
        best.0
    }

    fn best_of(&self, picks: &Picks, kept: &Kept) -> (f64, Vec<usize>) {
        let model = &self.model;
        let size = picks.iter().map(|pick| pick.2 + 1).product::<usize>() * (model.len + 1);
        let mut memo = MEMO.take();
        if memo.cells.len() < size {
            memo.cells.resize(size, (0, 0, 0.0));
        }
        memo.stamp = memo.stamp.wrapping_add(1);
        if memo.stamp == 0 {
            memo.cells.fill((0, 0, 0.0));
            memo.stamp = 1;
        }
        let value = self.go(picks, kept, 0, model.artisan, [0; 3], &mut memo);
        let mut indices = kept.options.clone();
        let (mut artisan, mut used) = (model.artisan, [0; 3]);
        let mut set = 0;
        for count in 0.. {
            if artisan >= 1.0 || count + 1 >= model.len {
                break;
            }
            if count % model.block == 0 {
                let (stamp, chosen, _) = memo.cells[self.slot(picks, count, used)];
                if stamp != memo.stamp {
                    break;
                }
                set = chosen;
            }
            let (option, next) = self.taken(picks, set as usize, used);
            indices[count] += option;
            let chance = model.chance(count, model.extra[indices[count]]);
            if chance == 1.0 {
                break;
            }
            used = next;
            artisan = model.step(artisan, chance);
        }
        MEMO.set(memo);
        (value, model.filled(&indices))
    }

    fn pick(&self, pool: usize, which: usize) -> (usize, usize, usize, usize) {
        let option = self.model.strides[pool] * (which + 1);
        (
            option,
            self.places[pool][which],
            self.model.pity(&vec![option; self.model.len]).0,
            pool,
        )
    }

    fn kept(&self, indices: &[usize], without: &[usize]) -> Kept {
        let model = &self.model;
        let mut options = indices.to_vec();
        let mut used = vec![0; self.juice_worth.len()];
        let mut worth = vec![0.0; model.len + 1];
        for tap in 0..model.len {
            for (pool, stride) in model.strides.iter().enumerate() {
                let Some(which) =
                    (indices[tap] / stride % (model.pools[pool].len() + 1)).checked_sub(1)
                else {
                    continue;
                };
                if without.contains(&pool) {
                    options[tap] -= stride * (which + 1);
                } else {
                    used[self.places[pool][which]] += 1;
                }
            }
            worth[tap + 1] = used
                .iter()
                .zip(&self.juice_worth)
                .map(|(&times, worth)| worth[times])
                .sum();
        }
        Kept { options, worth }
    }

    fn better(&self, indices: &[usize], joint_steps: usize) -> (f64, Vec<usize>) {
        let model = &self.model;
        let (mut cells, mut choices) = (model.len + 1, 1);
        for (pool, ids) in model
            .pools
            .iter()
            .enumerate()
            .filter(|(_, ids)| !ids.is_empty())
        {
            cells *= 2 * (self.pick(pool, 0).2 + 1);
            choices *= ids.len();
        }
        if cells / 3 * choices > joint_steps {
            return self.improve(indices);
        }
        let found = self.best();
        if choices == 1 || !self.mixed {
            return found;
        }
        self.improve(if self.score(indices) > found.0 {
            indices
        } else {
            &found.1
        })
    }

    fn best(&self) -> (f64, Vec<usize>) {
        let model = &self.model;
        let used_pools: Vec<usize> = (0..model.pools.len())
            .filter(|&pool| !model.pools[pool].is_empty())
            .collect();
        let kept = self.kept(&vec![0; model.len], &[]);
        let mut which = vec![0; used_pools.len()];
        let mut best = (f64::NEG_INFINITY, vec![0; model.len]);
        loop {
            let mut picks: Picks = used_pools
                .iter()
                .zip(&which)
                .map(|(&pool, &which)| self.pick(pool, which))
                .collect();
            while picks.iter().map(|pick| pick.2 + 1).product::<usize>() * (model.len + 1)
                > 2_000_000
            {
                picks.pop();
            }
            let found = self.best_of(&picks, &kept);
            if found.0 > best.0 {
                best = (found.0, model.canonical(&found.1));
            }
            let Some(at) =
                (0..which.len()).find(|&at| which[at] + 1 < model.pools[used_pools[at]].len())
            else {
                return best;
            };
            which[at] += 1;
            which[..at].fill(0);
        }
    }

    fn improve(&self, indices: &[usize]) -> (f64, Vec<usize>) {
        let model = &self.model;
        let mut now = (self.score(indices), model.filled(&model.canonical(indices)));
        let used_pools: Vec<usize> = (0..model.pools.len())
            .filter(|&pool| !model.pools[pool].is_empty())
            .collect();
        for _ in 0..4 {
            let mut moved = false;
            for &pool in &used_pools {
                let kept = self.kept(&now.1, &[pool]);
                let ids = model.pools[pool].len();
                let alone = (0..ids)
                    .map(|which| std::iter::once(self.pick(pool, which)).collect::<Picks>());
                let pairs = (0..ids)
                    .flat_map(|a| (0..a).map(move |b| (a, b)))
                    .filter(|_| self.mixed)
                    .map(|(a, b)| {
                        [self.pick(pool, b), self.pick(pool, a)]
                            .into_iter()
                            .collect::<Picks>()
                    });
                for picks in alone.chain(pairs) {
                    if picks.iter().map(|pick| pick.2 + 1).product::<usize>() * (model.len + 1)
                        > 2_000_000
                    {
                        continue;
                    }
                    let found = self.best_of(&picks, &kept);
                    if found.0 > now.0 + now.0.abs() * 1e-12 {
                        now = found;
                        moved = true;
                    }
                }
            }
            if !moved || used_pools.len() < 2 && model.pools[used_pools[0]].len() < 2 {
                break;
            }
        }
        (now.0, model.canonical(&now.1))
    }
}

// ---- the search ----

/// how far the surrogate's predictions of true changes were right
#[derive(Default)]
struct Agree {
    n: f64,
    sign: f64,
    sx: f64,
    sy: f64,
    sxx: f64,
    syy: f64,
    sxy: f64,
    proposed: f64,
    confirmed: f64,
}

impl Agree {
    fn add(&mut self, predicted: f64, actual: f64, scale: f64) {
        let (x, y) = (predicted / scale, actual / scale);
        self.n += 1.0;
        if (x > 0.0) == (y > 0.0) {
            self.sign += 1.0;
        }
        self.sx += x;
        self.sy += y;
        self.sxx += x * x;
        self.syy += y * y;
        self.sxy += x * y;
        if x > 0.0 {
            self.proposed += 1.0;
            if y > 0.0 {
                self.confirmed += 1.0;
            }
        }
    }
    fn corr(&self) -> f64 {
        let n = self.n;
        let cov = self.sxy - self.sx * self.sy / n;
        let vx = self.sxx - self.sx * self.sx / n;
        let vy = self.syy - self.sy * self.sy / n;
        if vx <= 0.0 || vy <= 0.0 {
            return f64::NAN;
        }
        cov / (vx * vy).sqrt()
    }
}

fn better_than(new: f64, old: f64) -> bool {
    new > old + 1e-12 * old.abs()
}

struct Run {
    b: StateBundle,
    sur: Sur,
    rng: SmallRng,
    work: f64,
    budget: f64,
    best: (StateEssence, f64),
    agree: Agree,
    normals: Vec<usize>,
    advs: Vec<usize>,
    /// the surrogate's cost on this thread when the solve began
    cost0: f64,
    /// the last phase, where the true metric also checks single block flips
    flipping: bool,
}

impl Run {
    fn eval(&mut self, probe: &mut Probe) -> f64 {
        self.work += 1.0;
        let metric = evaluate(&mut self.b, probe);
        self.b.metric = metric;
        if metric > self.best.1 {
            self.best = (self.b.to_essence(), metric);
        }
        metric
    }

    fn spent(&self) -> f64 {
        self.work + COST.get() - self.cost0
    }

    fn scale(&self) -> f64 {
        self.b.metric.abs().max(1.0)
    }

    fn load(&mut self, e: &StateEssence) {
        for (up, payload) in self.b.upgrade_arr.iter_mut().zip(&e.state_arr) {
            if up.state.payload != *payload {
                up.state.update_payload(payload.clone());
            }
        }
        self.b.special_state.clone_from(&e.special_state);
        self.b.set_chest_state(&e.chest_states);
    }

    /// puts whatever was given into a state the game allows
    fn fix(&mut self) {
        let b = &mut self.b;
        let block = b.block();
        let n = b.upgrade_arr.len();
        let ji = &b.prep_output.juice_info;
        for up in b.upgrade_arr.iter_mut() {
            let mut payload = up.state.payload.clone();
            if up.is_normal_honing {
                payload.resize(up.clean_prob_dist_len, smallvec![]);
                up.state.payload = payload.clone();
                let model = Model::of(up, ji, block);
                payload = model.plan(&model.filled(&model.indices(&payload)));
            } else {
                let len = ji.adv_uindex_to_id[up.piece_type_usize][up.upgrade_index].len();
                payload.resize(len, smallvec![0]);
                for slot in payload.iter_mut() {
                    let v = slot.first().copied().unwrap_or(0).min(MAX_ADV_STATE);
                    *slot = smallvec![v];
                }
            }
            up.state.update_payload(payload);
        }
        let mut order = b.special_state.clone();
        order.sort();
        if order != (0..n).collect::<Vec<_>>() {
            b.special_state = (0..n).collect();
        }
        let mut states = b.chest_states.clone();
        states.retain(|key, _| b.prep_output.chests.contains_key(key));
        for (key, chest) in &b.prep_output.chests {
            if !states.get(key).is_some_and(|s| chest.is_valid(s)) {
                states.insert(key.clone(), chest.default_state());
            }
        }
        b.set_chest_state(&states);
    }

    fn refresh_dist(&mut self, u: usize) {
        let StateBundle {
            upgrade_arr,
            adv_cache,
            prep_output,
            ..
        } = &mut self.b;
        let up = &mut upgrade_arr[u];
        up.update_this_prob_dist(adv_cache, &prep_output.juice_info);
        up.update_this_support(&prep_output.juice_info);
        self.sur.refresh(&self.b, u);
    }

    fn set_payload(&mut self, u: usize, payload: Vec<OneState>) {
        self.b.upgrade_arr[u].state.update_payload(payload);
        self.refresh_dist(u);
    }

    fn set_order(&mut self, order: &[usize]) {
        self.b.special_state.clear();
        self.b.special_state.extend_from_slice(order);
        self.sur.refresh_order(&mut self.b);
    }

    fn set_chests(&mut self, states: &crate::chest::ChestStates) {
        self.b.set_chest_state(states);
        self.sur.refresh_order(&mut self.b);
    }

    /// keeps the free tap cache from growing over many orders tried
    fn prune(&mut self) {
        let size: usize = self.b.special_cache.values().map(|m| m.len()).sum();
        if size > 4000 {
            self.b.special_cache.clear();
            self.b.special_done = None;
            self.sur.refresh_order(&mut self.b);
        }
    }

    fn calibrate(&mut self, probe: &mut Probe) {
        self.work += 1.0;
        self.sur.calibrate(&self.b, probe);
    }

    /// a normal upgrade's surrogate best plan; checked by the true metric where `verify`
    fn sur_normal(&mut self, u: usize, verify: bool, probe: &mut Probe) -> bool {
        let Some(table) = self.sur.table(&self.b, u) else {
            return false;
        };
        let model = &table.model;
        let current = model.indices(&self.b.upgrade_arr[u].state.payload);
        let (_, mut found) = if self.flipping {
            widest(
                model,
                &current,
                |at| table.better(at, self.b.tuning.response_joint),
                |at| table.improve(at),
                |at| table.score(at),
            )
        } else {
            table.better(&current, self.b.tuning.response_joint)
        };
        if model.canonical(&found) != model.canonical(&model.filled(&found)) {
            found = model.filled(&found);
        }
        if model.canonical(&found) == model.canonical(&current) {
            return false;
        }
        let plan = model.plan(&found);
        let old = self.b.upgrade_arr[u].state.payload.clone();
        let (g0, t0) = (self.sur.total(&self.b), self.b.metric);
        self.set_payload(u, plan);
        let g1 = self.sur.total(&self.b);
        if !verify {
            if better_than(g1, g0) {
                return true;
            }
            self.set_payload(u, old);
            return false;
        }
        let t1 = self.eval(probe);
        self.agree.add(g1 - g0, t1 - t0, t0.abs().max(1.0));
        if better_than(t1, t0) {
            return true;
        }
        self.set_payload(u, old);
        self.b.metric = t0;
        let model = model.clone();
        self.blend(u, &model, &current, &found, probe)
    }

    /// the true best response of a normal upgrade
    fn true_normal(&mut self, u: usize, probe: &mut Probe) -> bool {
        let model = Model::of(
            &self.b.upgrade_arr[u],
            &self.b.prep_output.juice_info,
            self.b.block(),
        );
        if model.pools.iter().all(|pool| pool.is_empty()) {
            return false;
        }
        let response = Response::of(&mut self.b, u, probe);
        let model = &response.model;
        let current = model.indices(&self.b.upgrade_arr[u].state.payload);
        // the wide search of plans is paid in steps, so the true response takes it last only
        let (_, mut found) = if self.flipping {
            widest(
                model,
                &current,
                |at| response.better(at, self.b.tuning.response_joint),
                |at| response.improve(at),
                |at| response.score(at),
            )
        } else {
            response.better(&current, self.b.tuning.response_joint)
        };
        self.work += response.work(self.b.tuning.response_steps);
        if model.canonical(&found) != model.canonical(&model.filled(&found)) {
            found = model.filled(&found);
        }
        let model = model.clone();
        if model.canonical(&found) != model.canonical(&current)
            && response.score(&found) > response.score(&current) + 1e-12 * self.scale()
        {
            let plan = model.plan(&found);
            let old = self.b.upgrade_arr[u].state.payload.clone();
            let t0 = self.b.metric;
            self.set_payload(u, plan);
            let t1 = self.eval(probe);
            if better_than(t1, t0) {
                return true;
            }
            self.set_payload(u, old);
            self.b.metric = t0;
            if self.blend(u, &model, &current, &found, probe) {
                return true;
            }
        }
        self.flipping && self.flips(u, &model, &current, &|at| response.score(at), probe)
    }

    /// Every block of the live taps given another choice of one pool, ranked by a response's
    /// score; the best few checked by the true metric.
    fn flips(
        &mut self,
        u: usize,
        model: &Model,
        current: &[usize],
        score: &dyn Fn(&[usize]) -> f64,
        probe: &mut Probe,
    ) -> bool {
        let pity = model.pity(current).0;
        let mut ranked: Vec<(f64, Vec<usize>)> = Vec::new();
        for start in (0..pity.min(model.len)).step_by(model.block) {
            for (pool, ids) in model.pools.iter().enumerate() {
                let stride = model.strides[pool];
                let now = current[start] / stride % (ids.len() + 1);
                for choice in (0..=ids.len()).filter(|&choice| choice != now) {
                    let mut plan = current.to_vec();
                    for tap in start..(start + model.block).min(model.len) {
                        let here = plan[tap] / stride % (ids.len() + 1);
                        plan[tap] = plan[tap] - here * stride + choice * stride;
                    }
                    let value = score(&plan);
                    ranked.push((value, plan));
                }
            }
        }
        ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
        let old = self.b.upgrade_arr[u].state.payload.clone();
        let t0 = self.b.metric;
        let mut best: Option<(f64, Vec<OneState>)> = None;
        for (_, plan) in ranked.into_iter().take(FLIP_TOP) {
            let plan = model.plan(&model.filled(&plan));
            self.set_payload(u, plan.clone());
            let t = self.eval(probe);
            if better_than(t, best.as_ref().map_or(t0, |b| b.0)) {
                best = Some((t, plan));
            }
        }
        match best {
            Some((t, plan)) => {
                self.set_payload(u, plan);
                self.b.metric = t;
                true
            }
            None => {
                self.set_payload(u, old);
                self.b.metric = t0;
                false
            }
        }
    }

    /// Plans between the one an upgrade has and one proposed for it that the truth turned
    /// down: the proposal on one pool only, or up to some tap only. Kept where truly better.
    fn blend(
        &mut self,
        u: usize,
        model: &Model,
        current: &[usize],
        found: &[usize],
        probe: &mut Probe,
    ) -> bool {
        let pool_of = |option: usize, pool: usize| {
            option / model.strides[pool] % (model.pools[pool].len() + 1) * model.strides[pool]
        };
        let mut candidates: Vec<Vec<usize>> = Vec::new();
        if model.pools.iter().filter(|ids| !ids.is_empty()).count() > 1 {
            for pool in 0..model.pools.len() {
                candidates.push(
                    (0..model.len)
                        .map(|tap| {
                            current[tap] - pool_of(current[tap], pool) + pool_of(found[tap], pool)
                        })
                        .collect(),
                );
            }
        }
        for share in [1, 2, 3] {
            let cut = model.len * share / 4 / model.block * model.block;
            candidates.push(
                (0..model.len)
                    .map(|tap| if tap < cut { found[tap] } else { current[tap] })
                    .collect(),
            );
        }
        let old = self.b.upgrade_arr[u].state.payload.clone();
        let t0 = self.b.metric;
        let mut best: Option<(f64, Vec<OneState>)> = None;
        let mut seen = vec![model.canonical(current), model.canonical(found)];
        for candidate in candidates {
            let candidate = model.filled(&candidate);
            if seen.contains(&model.canonical(&candidate)) {
                continue;
            }
            seen.push(model.canonical(&candidate));
            let plan = model.plan(&candidate);
            self.set_payload(u, plan.clone());
            let t = self.eval(probe);
            if better_than(t, best.as_ref().map_or(t0, |b| b.0)) {
                best = Some((t, plan));
            }
        }
        match best {
            Some((t, plan)) => {
                self.set_payload(u, plan);
                self.b.metric = t;
                true
            }
            None => {
                self.set_payload(u, old);
                self.b.metric = t0;
                false
            }
        }
    }

    /// Verifies the best few of the surrogate's candidates, or takes the best where not
    /// `verify`. A candidate is applied by `apply` and undone by `apply(None)`.
    fn choose<C: Clone>(
        &mut self,
        mut ranked: Vec<(f64, C)>,
        g0: f64,
        verify: bool,
        probe: &mut Probe,
        apply: &dyn Fn(&mut Run, Option<&C>),
    ) -> bool {
        ranked.retain(|c| better_than(c.0, g0));
        ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
        if ranked.is_empty() {
            return false;
        }
        if !verify {
            apply(self, Some(&ranked[0].1));
            return true;
        }
        let t0 = self.b.metric;
        let mut best: Option<(f64, usize)> = None;
        for (at, (g, c)) in ranked.iter().take(TOP).enumerate() {
            apply(self, Some(c));
            let t = self.eval(probe);
            self.agree.add(g - g0, t - t0, t0.abs().max(1.0));
            if better_than(t, best.map_or(t0, |b| b.0)) {
                best = Some((t, at));
            }
            apply(self, None);
        }
        match best {
            Some((t, at)) => {
                apply(self, Some(&ranked[at].1));
                self.b.metric = t;
                true
            }
            None => {
                self.b.metric = t0;
                false
            }
        }
    }

    fn adv_move(&mut self, u: usize, verify: bool, probe: &mut Probe) -> bool {
        let old = self.b.upgrade_arr[u].state.payload.clone();
        let g0 = self.sur.total(&self.b);
        let mut ranked = Vec::new();
        for slot in 0..old.len() {
            for v in (0..=MAX_ADV_STATE).filter(|&v| v != old[slot][0]) {
                let mut payload = old.clone();
                payload[slot] = smallvec![v];
                self.set_payload(u, payload.clone());
                ranked.push((self.sur.total(&self.b), payload));
            }
        }
        self.set_payload(u, old.clone());
        let apply = move |run: &mut Run, c: Option<&Vec<OneState>>| {
            run.set_payload(u, c.cloned().unwrap_or_else(|| old.clone()))
        };
        self.choose(ranked, g0, verify, probe, &apply)
    }

    fn greedy_order(&self) -> Vec<usize> {
        let b = &self.b;
        let budget = b.prep_output.special_budget;
        let value = |u: usize| -> f64 {
            let up = &b.upgrade_arr[u];
            let saved: f64 = (0..self.sur.rows)
                .map(|row| {
                    self.sur.price[row] * (self.sur.honed[u][row].mean - self.sur.skipped[u][row])
                })
                .sum();
            saved * up.base_chance / up.special_cost.max(1) as f64
        };
        let mut free: Vec<(f64, usize)> = (0..b.upgrade_arr.len())
            .filter(|&u| {
                b.upgrade_arr[u].is_normal_honing && b.upgrade_arr[u].special_cost <= budget
            })
            .map(|u| (value(u), u))
            .collect();
        free.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut order: Vec<usize> = free.iter().map(|x| x.1).collect();
        // a lower level of a piece before a higher one, in the places the piece got
        for piece in 0..7 {
            let at: Vec<usize> = (0..order.len())
                .filter(|&i| b.upgrade_arr[order[i]].piece_index == piece)
                .collect();
            let mut ups: Vec<usize> = at.iter().map(|&i| order[i]).collect();
            ups.sort_by_key(|&u| b.upgrade_arr[u].upgrade_index);
            for (&i, &u) in at.iter().zip(&ups) {
                order[i] = u;
            }
        }
        let rest: Vec<usize> = (0..b.upgrade_arr.len())
            .filter(|u| !order.contains(u))
            .collect();
        order.extend(rest);
        order
    }

    fn order_move(&mut self, verify: bool, probe: &mut Probe) -> bool {
        if self.b.upgrade_arr.len() < 2 || !self.b.affords_a_free_tap() {
            return false;
        }
        self.prune();
        let old = self.b.special_state.clone();
        let g0 = self.sur.total(&self.b);
        let n = old.len();
        let budget = self.b.prep_output.special_budget;
        let reach = (self.sur.w.last().unwrap().0 + 1).min(n - 1);
        let mut candidates: Vec<Vec<usize>> = vec![self.greedy_order()];
        for i in 0..n {
            let up = &self.b.upgrade_arr[old[i]];
            if !up.is_normal_honing || up.special_cost > budget {
                continue;
            }
            for j in 0..=reach {
                if j != i {
                    let mut order = old.clone();
                    let x = order.remove(i);
                    order.insert(j, x);
                    candidates.push(order);
                }
            }
            // out of the free taps altogether
            let mut order = old.clone();
            let x = order.remove(i);
            order.push(x);
            candidates.push(order);
        }
        let mut ranked = Vec::new();
        for order in candidates {
            if order == old {
                continue;
            }
            self.set_order(&order);
            ranked.push((self.sur.total(&self.b), order));
        }
        self.set_order(&old);
        let apply = move |run: &mut Run, c: Option<&Vec<usize>>| run.set_order(c.unwrap_or(&old));
        self.choose(ranked, g0, verify, probe, &apply)
    }

    fn chest_move(&mut self, verify: bool, probe: &mut Probe) -> bool {
        let old = self.b.chest_states.clone();
        let g0 = self.sur.total(&self.b);
        let mut candidates = Vec::new();
        for (key, chest) in &self.b.prep_output.chests {
            let state = &old[key];
            for a in 0..state.len() {
                if state[a] == 0 {
                    continue;
                }
                let mut amounts: Vec<i64> = vec![state[a], (state[a] + 1) / 2, state[a] / 8, 1];
                amounts.retain(|&x| x > 0);
                amounts.dedup();
                for to in (0..chest.options.len()).filter(|&to| to != a) {
                    for &amount in &amounts {
                        let mut states = old.clone();
                        let s = states.get_mut(key).unwrap();
                        s[a] -= amount;
                        s[to] += amount;
                        candidates.push(states);
                    }
                }
            }
        }
        if candidates.is_empty() {
            return false;
        }
        let mut ranked = Vec::new();
        for states in candidates {
            self.set_chests(&states);
            ranked.push((self.sur.total(&self.b), states));
        }
        self.set_chests(&old);
        let apply = move |run: &mut Run, c: Option<&crate::chest::ChestStates>| {
            run.set_chests(c.unwrap_or(&old))
        };
        let moved = self.choose(ranked, g0, verify, probe, &apply);
        self.prune();
        moved
    }

    /// the surrogate alone, from the state as it is
    /// On the surrogate alone: one upgrade is put on a plan far from its own (improved from
    /// an id combination on every tap), every other normal upgrade answers it, then it
    /// answers them. Kept where the surrogate is better: a joint move no single response makes.
    fn kick_repair(&mut self, effort: f64, probe: &mut Probe) -> bool {
        let mut trials = (KICK_TRIALS * effort).ceil() as usize;
        let mut moved = false;
        let mut g0 = self.sur.total(&self.b);
        let mut normals = self.normals.clone();
        normals.shuffle(&mut self.rng);
        for &u in &normals {
            let Some(table) = self.sur.table(&self.b, u) else {
                continue;
            };
            let model = &table.model;
            let current = model.canonical(&model.indices(&self.b.upgrade_arr[u].state.payload));
            let mut combos: Vec<usize> = vec![0];
            for (pool, ids) in model.pools.iter().enumerate() {
                combos = combos
                    .iter()
                    .flat_map(|&base| {
                        (0..=ids.len()).map(move |which| base + which * model.strides[pool])
                    })
                    .collect();
            }
            let mut plans: Vec<Vec<usize>> = Vec::new();
            for &option in combos.iter().take(START_COMBOS + 1) {
                let found = model.canonical(&table.improve(&vec![option; model.len]).1);
                if found != current && !plans.contains(&found) {
                    plans.push(found);
                }
            }
            for plan in plans {
                if trials == 0 {
                    return moved;
                }
                trials -= 1;
                let old: Vec<Vec<OneState>> = self
                    .normals
                    .iter()
                    .map(|&v| self.b.upgrade_arr[v].state.payload.clone())
                    .collect();
                self.set_payload(u, model.plan(&model.filled(&plan)));
                for &v in normals.iter().filter(|&&v| v != u) {
                    self.sur_normal(v, false, probe);
                }
                self.sur_normal(u, false, probe);
                let g = self.sur.total(&self.b);
                if better_than(g, g0) {
                    g0 = g;
                    moved = true;
                    break;
                }
                for (&v, payload) in self.normals.clone().iter().zip(old) {
                    if self.b.upgrade_arr[v].state.payload != payload {
                        self.set_payload(v, payload);
                    }
                }
            }
        }
        moved
    }

    fn descend(&mut self, rounds: usize, probe: &mut Probe) {
        for _ in 0..rounds {
            let mut moved = false;
            let mut normals = self.normals.clone();
            normals.shuffle(&mut self.rng);
            for u in normals {
                moved |= self.sur_normal(u, false, probe);
            }
            for u in self.advs.clone() {
                moved |= self.adv_move(u, false, probe);
            }
            for _ in 0..3 {
                if !self.order_move(false, probe) {
                    break;
                }
                moved = true;
            }
            for _ in 0..4 {
                if !self.chest_move(false, probe) {
                    break;
                }
                moved = true;
            }
            if !moved {
                break;
            }
        }
    }

    /// moves the true metric keeps, the surrogate's first
    fn polish(&mut self, probe: &mut Probe) {
        while self.spent() < self.budget {
            self.eval(probe);
            self.calibrate(probe);
            let mut moved = false;
            let mut normals = self.normals.clone();
            normals.shuffle(&mut self.rng);
            for u in normals {
                if self.spent() >= self.budget {
                    return;
                }
                if self.sur_normal(u, true, probe) || self.true_normal(u, probe) {
                    moved = true;
                }
            }
            for u in self.advs.clone() {
                moved |= self.adv_move(u, true, probe);
            }
            moved |= self.order_move(true, probe);
            moved |= self.chest_move(true, probe);
            if !moved {
                break;
            }
        }
    }

    /// somewhere else to start from, near the best or not
    fn perturb(&mut self, round: usize) {
        let block = self.b.block();
        match round % 3 {
            1 => {
                for u in self.normals.clone() {
                    let len = self.b.upgrade_arr[u].state.len();
                    self.b.upgrade_arr[u]
                        .state
                        .update_payload(vec![smallvec![]; len]);
                }
                for u in self.advs.clone() {
                    let len = self.b.upgrade_arr[u].state.len();
                    self.b.upgrade_arr[u]
                        .state
                        .update_payload(vec![smallvec![0]; len]);
                }
                let states = self
                    .b
                    .prep_output
                    .chests
                    .iter()
                    .map(|(key, chest)| (key.clone(), chest.random_state(&mut self.rng)))
                    .collect();
                self.b.set_chest_state(&states);
            }
            2 => {
                for u in self.normals.clone() {
                    if self.rng.random::<f64>() > 0.3 {
                        continue;
                    }
                    let up = &self.b.upgrade_arr[u];
                    let model = Model::of(up, &self.b.prep_output.juice_info, block);
                    let mut indices = vec![0; model.len];
                    for (pool, ids) in model.pools.iter().enumerate() {
                        if ids.is_empty() || self.rng.random::<bool>() {
                            continue;
                        }
                        let which = self.rng.random_range(0..ids.len());
                        let length = self.rng.random_range(0..=model.len);
                        for tap in indices.iter_mut().take(length) {
                            *tap += model.strides[pool] * (which + 1);
                        }
                    }
                    let plan = model.plan(&model.filled(&indices));
                    self.b.upgrade_arr[u].state.update_payload(plan);
                }
                let n = self.b.special_state.len();
                for _ in 0..2 {
                    let i = self.rng.random_range(0..n);
                    let j = self.rng.random_range(0..n.min(4));
                    let x = self.b.special_state.remove(i);
                    self.b.special_state.insert(j, x);
                }
            }
            _ => {
                let piece_type = self.rng.random_range(0..3);
                for u in self.normals.clone() {
                    if self.b.upgrade_arr[u].piece_type_usize == piece_type {
                        let len = self.b.upgrade_arr[u].state.len();
                        self.b.upgrade_arr[u]
                            .state
                            .update_payload(vec![smallvec![]; len]);
                    }
                }
                let mut states = self.b.chest_states.clone();
                if let Some((key, chest)) = self
                    .b
                    .prep_output
                    .chests
                    .iter()
                    .filter(|(_, chest)| chest.options.len() > 1)
                    .nth(0)
                {
                    states.insert(key.clone(), chest.random_state(&mut self.rng));
                }
                self.b.set_chest_state(&states);
            }
        }
    }

    fn main(&mut self, effort: f64, probe: &mut Probe) {
        let stall_max = (STALL * effort).ceil().max(1.0) as usize;
        let mut stall = 0;
        for round in 0.. {
            if round > 0 && (self.spent() >= self.budget || stall >= stall_max) {
                break;
            }
            let before = self.best.1;
            let start = self.best.0.clone();
            self.load(&start);
            self.sur.refresh_all(&mut self.b);
            let t0 = self.eval(probe);
            self.calibrate(probe);
            let g0 = self.sur.total(&self.b);
            // every other restart a kick of the prices, the rest a kick of the state
            if round > 0 && round % 2 == 1 {
                let kept = self.sur.corr.clone();
                for row in 0..self.sur.rows {
                    if self.rng.random::<f64>() < 0.5 {
                        let factor = (KICK * (self.rng.random::<f64>() * 2.0 - 1.0)).exp();
                        self.sur.corr[row] -= (factor - 1.0) * self.sur.price[row];
                    }
                }
                self.descend(DESCENT, probe);
                self.sur.corr = kept;
            } else if round > 0 {
                self.perturb(round / 2);
                self.sur.refresh_all(&mut self.b);
            }
            self.descend(DESCENT, probe);
            if self.kick_repair(effort, probe) {
                self.descend(DESCENT, probe);
            }
            let g1 = self.sur.total(&self.b);
            let t1 = self.eval(probe);
            if g1 != g0 {
                self.agree.add(g1 - g0, t1 - t0, t0.abs().max(1.0));
            }
            if round == 0 && t1 < t0 {
                self.load(&start);
                self.sur.refresh_all(&mut self.b);
                self.eval(probe);
            }
            self.polish(probe);
            if better_than(self.best.1, before) {
                stall = 0;
            } else {
                stall += 1;
            }
        }
    }
}

/// a trace row; a NaN would not survive the trip to the driver
fn put(probe: &mut Probe, key: &'static str, value: f64) {
    if value.is_finite() {
        probe.put(key, value);
    }
}

pub fn solve<R: Rng>(
    rng: &mut R,
    state_bundle: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    let mut b = state_bundle;
    if b.upgrade_arr.is_empty() {
        b.metric = evaluate(&mut b, probe);
        return Solved {
            best: b,
            blob: Vec::new(),
        };
    }
    let effort = b.tuning.lab_effort.max(0.0);
    let n = b.upgrade_arr.len();
    let carried: Vec<StateEssence> = previous
        .map(|p| p.carried(&b).iter().map(|c| c.to_essence()).collect())
        .unwrap_or_default();
    let sur = Sur::new(&b);
    let mut run = Run {
        normals: (0..n)
            .filter(|&u| b.upgrade_arr[u].is_normal_honing)
            .collect(),
        flipping: false,
        cost0: COST.get(),
        advs: (0..n)
            .filter(|&u| !b.upgrade_arr[u].is_normal_honing)
            .collect(),
        best: (b.to_essence(), f64::NEG_INFINITY),
        b,
        sur,
        rng: SmallRng::seed_from_u64(rng.random()),
        work: 0.0,
        budget: effort * (BASE_EVALS + PER_UPGRADE * n as f64),
        agree: Agree::default(),
    };
    // the given and carried plans, on the block grid
    run.fix();
    run.eval(probe);
    for essence in &carried {
        run.load(essence);
        run.fix();
        run.eval(probe);
    }
    let full = run.budget;
    run.budget = full * (1.0 - FLIP_SHARE);
    run.main(effort, probe);
    run.budget = full;
    run.flipping = true;
    let best = run.best.0.clone();
    run.load(&best);
    run.sur.refresh_all(&mut run.b);
    run.polish(probe);

    let best = run.best.0.clone();
    run.load(&best);
    run.sur.refresh_all(&mut run.b);
    let metric = evaluate(&mut run.b, probe);
    run.b.metric = metric;
    let raw = run.sur.total_with(&run.b, false);
    put(probe, "sur_n", run.agree.n);
    put(probe, "sur_sign", run.agree.sign / run.agree.n.max(1.0));
    let corr = run.agree.corr();
    if corr.is_finite() {
        put(probe, "sur_corr", corr);
    }
    put(
        probe,
        "sur_confirmed",
        run.agree.confirmed / run.agree.proposed.max(1.0),
    );
    put(probe, "sur_level", (raw - metric) / metric.abs().max(1.0));
    put(probe, "sur_work", run.work);
    put(probe, "sur_spent", run.spent());
    Solved {
        best: run.b,
        blob: Vec::new(),
    }
}

/// The best of `better` from the plan and `improve` from plans that take every id
/// combination on every tap: improving a pool at a time from the plan alone misses plans that
/// change several pools at once.
fn widest(
    model: &Model,
    current: &[usize],
    better: impl Fn(&[usize]) -> (f64, Vec<usize>),
    improve: impl Fn(&[usize]) -> (f64, Vec<usize>),
    score: impl Fn(&[usize]) -> f64,
) -> (f64, Vec<usize>) {
    let mut best = better(current);
    let mut combos: Vec<usize> = vec![0];
    for (pool, ids) in model.pools.iter().enumerate() {
        combos = combos
            .iter()
            .flat_map(|&base| (0..=ids.len()).map(move |which| base + which * model.strides[pool]))
            .collect();
    }
    for &option in combos
        .iter()
        .filter(|&&option| option > 0)
        .take(START_COMBOS)
    {
        let found = improve(&vec![option; model.len]);
        if found.0 > best.0 {
            best = (score(&found.1), found.1);
        }
    }
    best
}
