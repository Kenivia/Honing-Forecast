//! Memetic search: a population of whole plans, children recombined along the problem's
//! structure (piece types, pieces or upgrades with their free tap places, chest stacks), each
//! improved by a short local search (best responses, advanced, order and chest moves) before it
//! competes by crowding; restarts by prices when it converges, the best polished in between.
//! See `arena_data/lab/iteration1/attempts/memetic.md`.

use super::check;
use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::chest::ChestStates;
use crate::optimizer::response::Response;
use crate::optimizer::{Previous, Solved};
use crate::probe::Probe;
use crate::state::OneState;
use crate::state_bundle::StateBundle;
use rand::Rng;
use smallvec::smallvec;

/// evaluations a unit of effort buys: a base and so many an upgrade
const BUDGET_BASE: f64 = 20000.0;
const BUDGET_PER_UPGRADE: f64 = 3000.0;
/// best responses a child's local search takes
const CHILD_RESPONSES: usize = 3;
/// children without a new best before a restart, at effort 1
const STALL: f64 = 140.0;
/// restarts in a row that find no new best before stopping, at effort 1
const RESTARTS: f64 = 3.0;
const MUTATE: f64 = 0.5;

#[derive(Clone)]
struct Ind {
    plans: Vec<Vec<OneState>>,
    order: Vec<usize>,
    chests: ChestStates,
    metric: f64,
}

struct Ctx<'a> {
    bundle: StateBundle,
    probe: &'a mut Probe,
    spent: f64,
    normal: Vec<usize>,
    adv: Vec<usize>,
    /// chest keys with a choice
    stacks: Vec<String>,
    free: bool,
    crossover: bool,
}

impl Ctx<'_> {
    fn eval(&mut self) -> f64 {
        self.spent += 1.0;
        let metric = super::evaluate(&mut self.bundle, self.probe);
        self.bundle.metric = metric;
        metric
    }

    fn load(&mut self, ind: &Ind) {
        for (upgrade, plan) in self.bundle.upgrade_arr.iter_mut().zip(&ind.plans) {
            if upgrade.state.payload != *plan {
                upgrade.state.update_payload(plan.clone());
            }
        }
        self.bundle.special_state.clone_from(&ind.order);
        self.bundle.set_chest_state(&ind.chests);
    }

    fn snapshot(&self, metric: f64) -> Ind {
        Ind {
            plans: self
                .bundle
                .upgrade_arr
                .iter()
                .map(|upgrade| upgrade.state.payload.clone())
                .collect(),
            order: self.bundle.special_state.clone(),
            chests: self.bundle.chest_states.clone(),
            metric,
        }
    }

    /// the best response of one normal upgrade, applied; whether it moved
    fn respond(&mut self, index: usize) -> bool {
        let response = Response::of(&mut self.bundle, index, self.probe);
        let now = response
            .model
            .indices(&self.bundle.upgrade_arr[index].state.payload);
        let (_, found) = response.better(&now, self.bundle.tuning.response_joint);
        self.spent += response.work(self.bundle.tuning.response_steps);
        let model = &response.model;
        if model.canonical(&now) == found {
            return false;
        }
        self.bundle.upgrade_arr[index]
            .state
            .update_payload(model.plan(&found));
        true
    }

    /// best responses on these upgrades, kept only if the metric agrees
    fn sweep(&mut self, from: f64, list: &[usize]) -> (f64, bool) {
        let before = self.snapshot(from);
        let mut moved = false;
        for &index in list {
            moved |= self.respond(index);
        }
        if !moved {
            return (from, false);
        }
        let metric = self.eval();
        if metric > from {
            (metric, true)
        } else {
            self.load(&before);
            (from, false)
        }
    }

    fn adv_move<R: Rng>(&mut self, rng: &mut R, from: f64) -> f64 {
        let index = self.adv[rng.random_range(0..self.adv.len())];
        let slots = self.bundle.upgrade_arr[index].state.payload.len();
        if slots == 0 {
            return from;
        }
        let slot = rng.random_range(0..slots);
        let old = self.bundle.upgrade_arr[index].state.payload.clone();
        let step = [1i64, -1, 2, -2, 4, -4][rng.random_range(0..6)];
        let next = (old[slot][0] as i64 + step).clamp(0, MAX_ADV_STATE as i64) as usize;
        if next == old[slot][0] {
            return from;
        }
        let mut new = old.clone();
        new[slot] = smallvec![next];
        self.try_plan(index, new, old, from)
    }

    fn try_plan(&mut self, index: usize, new: Vec<OneState>, old: Vec<OneState>, from: f64) -> f64 {
        self.bundle.upgrade_arr[index].state.update_payload(new);
        let metric = self.eval();
        if metric > from {
            return metric;
        }
        self.bundle.upgrade_arr[index].state.update_payload(old);
        from
    }

    fn order_move<R: Rng>(&mut self, rng: &mut R, from: f64) -> f64 {
        let old = self.bundle.special_state.clone();
        let len = old.len();
        let mut new = old.clone();
        let at = rng.random_range(0..len);
        let item = new.remove(at);
        let shift = rng.random_range(1..=3.min(len - 1)) as i64;
        let to = if rng.random_bool(0.5) {
            (at as i64 - shift).max(0)
        } else {
            (at as i64 + shift).min(len as i64 - 1)
        };
        new.insert(to as usize, item);
        if new == old {
            return from;
        }
        self.bundle.special_state = new;
        let metric = self.eval();
        if metric > from {
            return metric;
        }
        self.bundle.special_state = old;
        from
    }

    fn chest_move<R: Rng>(&mut self, rng: &mut R, from: f64) -> f64 {
        let key = self.stacks[rng.random_range(0..self.stacks.len())].clone();
        let old = self.bundle.chest_states.clone();
        let mut new = old.clone();
        if !shift_chests(rng, &mut new.get_mut(&key).unwrap()) {
            return from;
        }
        self.bundle.set_chest_state(&new);
        let metric = self.eval();
        if metric > from {
            return metric;
        }
        self.bundle.set_chest_state(&old);
        from
    }

    /// a short local search of a child, from its evaluated state
    fn improve<R: Rng>(&mut self, rng: &mut R, from: f64, focus: &[usize]) -> f64 {
        let mut metric = from;
        if !self.normal.is_empty() {
            let mut list: Vec<usize> = focus.to_vec();
            shuffle(rng, &mut list);
            list.truncate(CHILD_RESPONSES);
            while list.len() < CHILD_RESPONSES.min(self.normal.len()) {
                let pick = self.normal[rng.random_range(0..self.normal.len())];
                if !list.contains(&pick) {
                    list.push(pick);
                }
            }
            metric = self.sweep(metric, &list).0;
        }
        if !self.adv.is_empty() {
            metric = self.adv_move(rng, metric);
        }
        if self.free && self.bundle.special_state.len() > 1 {
            metric = self.order_move(rng, metric);
        }
        if !self.stacks.is_empty() {
            metric = self.chest_move(rng, metric);
        }
        metric
    }

    /// a thorough local search of the current state
    fn polish<R: Rng>(&mut self, rng: &mut R, from: f64, budget: f64) -> f64 {
        let mut metric = from;
        for _ in 0..4 {
            let start = metric;
            let mut list = self.normal.clone();
            shuffle(rng, &mut list);
            // a sweep in pieces, so that one bad response does not lose the rest
            for chunk in list.chunks(4) {
                if self.spent >= budget {
                    break;
                }
                metric = self.sweep(metric, chunk).0;
            }
            metric = self.adv_polish(metric, budget);
            if self.free && self.bundle.special_state.len() > 1 {
                metric = self.order_polish(metric, budget);
            }
            metric = self.chest_polish(metric, budget);
            if metric <= start + start.abs() * 1e-12 || self.spent >= budget {
                break;
            }
        }
        metric
    }

    /// each advanced slot to its best index, the others as they are
    fn adv_polish(&mut self, from: f64, budget: f64) -> f64 {
        let mut metric = from;
        for index in self.adv.clone() {
            for slot in 0..self.bundle.upgrade_arr[index].state.payload.len() {
                let start = self.bundle.upgrade_arr[index].state.payload.clone();
                let mut found = start.clone();
                for next in 0..=MAX_ADV_STATE {
                    if next == start[slot][0] || self.spent >= budget {
                        continue;
                    }
                    let mut new = start.clone();
                    new[slot] = smallvec![next];
                    self.bundle.upgrade_arr[index]
                        .state
                        .update_payload(new.clone());
                    let tried = self.eval();
                    if tried > metric {
                        metric = tried;
                        found = new;
                    }
                }
                self.bundle.upgrade_arr[index].state.update_payload(found);
            }
        }
        metric
    }

    /// each normal upgrade to a better place in the order, within a window, its plan and the
    /// displaced one's answering the move
    fn order_polish(&mut self, from: f64, budget: f64) -> f64 {
        let mut metric = from;
        let len = self.bundle.special_state.len();
        let window = if len <= 12 { len } else { 6 };
        for item in self.normal.clone() {
            let base = self.snapshot(metric);
            let at = base.order.iter().position(|&x| x == item).unwrap();
            // near, the front, and either side of where free tapping stops
            let stop = self.bundle.special_invalid_index.unwrap_or(len);
            let mut places: Vec<usize> = (at.saturating_sub(window)..=(at + window).min(len - 1))
                .chain(0..3.min(len))
                .chain(stop.saturating_sub(1)..(stop + 2).min(len))
                .chain([len - 1])
                .collect();
            // beside the next and the last level of its piece
            let upgrades = &self.bundle.upgrade_arr;
            let (piece, level) = (upgrades[item].piece_index, upgrades[item].upgrade_index);
            for (to, &other) in base.order.iter().enumerate() {
                let next = &upgrades[other];
                if other != item && next.is_normal_honing && next.piece_index == piece {
                    let place = if next.upgrade_index > level {
                        to
                    } else {
                        to + 1
                    };
                    places.push(if place > at { place - 1 } else { place }.min(len - 1));
                }
            }
            places.sort();
            places.dedup();
            for to in places {
                if to == at || self.spent >= budget {
                    continue;
                }
                let mut new = base.order.clone();
                new.remove(at);
                new.insert(to, item);
                if new == base.order {
                    continue;
                }
                let displaced = base.order[to];
                self.bundle.special_state = new;
                let tried = self.eval();
                if tried > metric {
                    metric = tried;
                    break;
                }
                let mut list = vec![item];
                if self.bundle.upgrade_arr[displaced].is_normal_honing {
                    list.push(displaced);
                }
                let (tried, kept) = self.sweep(f64::NEG_INFINITY, &list);
                if kept && tried > metric {
                    metric = tried;
                    break;
                }
                self.load(&base);
            }
        }
        // one upgrade out of the free taps and one in, plans answering
        let inside = places(&self.bundle, &self.bundle.special_state);
        let (ins, outs): (Vec<usize>, Vec<usize>) =
            self.normal.iter().partition(|&&index| inside[index] < 2.0);
        for &a in &ins {
            for &b in &outs {
                if self.spent >= budget {
                    return metric;
                }
                let base = self.snapshot(metric);
                let new = toggle(&self.bundle, toggle(&self.bundle, base.order.clone(), a), b);
                self.bundle.special_state = new;
                let tried = self.eval();
                if tried > metric {
                    metric = tried;
                    continue;
                }
                let (tried, kept) = self.sweep(f64::NEG_INFINITY, &[a, b]);
                if kept && tried > metric {
                    metric = tried;
                } else {
                    self.load(&base);
                }
            }
        }
        // runs of two or three to the front or to the end
        let len = self.bundle.special_state.len();
        for size in [2, 3] {
            for at in 0..len.saturating_sub(size - 1) {
                for front in [true, false] {
                    if self.spent >= budget {
                        return metric;
                    }
                    let base = self.snapshot(metric);
                    let run: Vec<usize> = base.order[at..at + size].to_vec();
                    let rest = base.order.iter().copied().filter(|x| !run.contains(x));
                    let new: Vec<usize> = if front {
                        run.iter().copied().chain(rest).collect()
                    } else {
                        rest.chain(run.iter().copied()).collect()
                    };
                    if new == base.order {
                        continue;
                    }
                    self.bundle.special_state = new;
                    let tried = self.eval();
                    if tried > metric {
                        metric = tried;
                        continue;
                    }
                    let list: Vec<usize> = run
                        .iter()
                        .copied()
                        .filter(|&index| self.bundle.upgrade_arr[index].is_normal_honing)
                        .collect();
                    let (tried, kept) = self.sweep(f64::NEG_INFINITY, &list);
                    if kept && tried > metric {
                        metric = tried;
                    } else {
                        self.load(&base);
                    }
                }
            }
        }
        metric
    }

    /// each chest stack by moves between options, halving the amount moved
    fn chest_polish(&mut self, from: f64, budget: f64) -> f64 {
        let mut metric = from;
        for key in self.stacks.clone() {
            let options = self.bundle.chest_states[&key].len();
            let mut step = (self.bundle.prep_output.chests[&key].count / 2).max(1);
            while self.spent < budget {
                let mut moved = false;
                for a in 0..options {
                    for b in (0..options).filter(|&b| b != a) {
                        while self.spent < budget && self.bundle.chest_states[&key][a] > 0 {
                            let old = self.bundle.chest_states.clone();
                            let mut new = old.clone();
                            let state = new.get_mut(&key).unwrap();
                            let amount = step.min(state[a]);
                            state[a] -= amount;
                            state[b] += amount;
                            self.bundle.set_chest_state(&new);
                            let tried = self.eval();
                            if tried > metric {
                                metric = tried;
                                moved = true;
                            } else {
                                self.bundle.set_chest_state(&old);
                                break;
                            }
                        }
                    }
                }
                if !moved {
                    if step == 1 {
                        break;
                    }
                    step /= 2;
                }
            }
        }
        self.chest_pairs(metric, budget)
    }

    /// one chest moved in each of two places at once, until no pair betters it
    fn chest_pairs(&mut self, from: f64, budget: f64) -> f64 {
        let mut metric = from;
        let mut moves: Vec<(String, usize, usize)> = Vec::new();
        for key in &self.stacks {
            let options = self.bundle.chest_states[key].len();
            for a in 0..options {
                for b in (0..options).filter(|&b| b != a) {
                    moves.push((key.clone(), a, b));
                }
            }
        }
        let mut moved = true;
        while moved {
            moved = false;
            for x in 0..moves.len() {
                for y in x + 1..moves.len() {
                    if self.spent >= budget {
                        return metric;
                    }
                    let old = self.bundle.chest_states.clone();
                    let mut new = old.clone();
                    let mut fits = true;
                    for (key, a, b) in [&moves[x], &moves[y]] {
                        let state = new.get_mut(key).unwrap();
                        if state[*a] == 0 {
                            fits = false;
                            break;
                        }
                        state[*a] -= 1;
                        state[*b] += 1;
                    }
                    if !fits || new == old {
                        continue;
                    }
                    self.bundle.set_chest_state(&new);
                    let tried = self.eval();
                    if tried > metric {
                        metric = tried;
                        moved = true;
                    } else {
                        self.bundle.set_chest_state(&old);
                    }
                }
            }
        }
        metric
    }
}

fn shuffle<R: Rng, T>(rng: &mut R, list: &mut [T]) {
    for at in (1..list.len()).rev() {
        list.swap(at, rng.random_range(0..=at));
    }
}

/// moves some chests of a stack from one option to another
fn shift_chests<R: Rng>(rng: &mut R, state: &mut [i64]) -> bool {
    let from: Vec<usize> = (0..state.len()).filter(|&at| state[at] > 0).collect();
    if from.is_empty() || state.len() < 2 {
        return false;
    }
    let a = from[rng.random_range(0..from.len())];
    let mut b = rng.random_range(0..state.len() - 1);
    if b >= a {
        b += 1;
    }
    let amount = match rng.random_range(0..3) {
        0 => state[a],
        1 => (state[a] / 2).max(1),
        _ => 1,
    };
    state[a] -= amount;
    state[b] += amount;
    true
}

/// a plan on the block grid, kept where it already is
fn fit(bundle: &mut StateBundle, index: usize) {
    let block = bundle.block();
    let juice_info = &bundle.prep_output.juice_info;
    let upgrade = &mut bundle.upgrade_arr[index];
    if !upgrade.is_normal_honing {
        return;
    }
    let model = crate::optimizer::response::Model::of(upgrade, juice_info, block);
    let indices = model.indices(&upgrade.state.payload);
    if model.canonical(&indices) == model.canonical(&model.filled(&indices)) {
        return;
    }
    let (piece_type, level) = (upgrade.piece_type_usize, upgrade.upgrade_index);
    upgrade
        .state
        .snap_to_block(block, piece_type, level, juice_info);
    let indices = model.indices(&upgrade.state.payload);
    if model.canonical(&indices) != model.canonical(&model.filled(&indices)) {
        upgrade
            .state
            .update_payload(model.plan(&model.filled(&indices)));
    }
}

/// what one material a unit is bought at
fn market(bundle: &StateBundle) -> Vec<f64> {
    let prep = &bundle.prep_output;
    prep.bands[prep.optimizer_plan]
        .iter()
        .map(|bands| bands.last().map_or(0.0, |band| band.1))
        .collect()
}

fn normal_sample<R: Rng>(rng: &mut R) -> f64 {
    let u: f64 = rng.random::<f64>().max(1e-12);
    let v: f64 = rng.random();
    (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
}

/// every normal upgrade's best plan alone at these prices
fn linear_plans(ctx: &mut Ctx, prices: &[f64]) {
    for at in 0..ctx.normal.len() {
        let index = ctx.normal[at];
        let response = Response::linear(&ctx.bundle, index, prices);
        let now = response
            .model
            .indices(&ctx.bundle.upgrade_arr[index].state.payload);
        let (_, found) = response.better(&now, ctx.bundle.tuning.response_joint);
        ctx.spent += response.work(ctx.bundle.tuning.response_steps);
        if response.model.canonical(&now) != found {
            let plan = response.model.plan(&found);
            ctx.bundle.upgrade_arr[index].state.update_payload(plan);
        }
    }
}

/// an order by what a leapstone saves on each upgrade at these prices
fn price_order(ctx: &Ctx, prices: &[f64]) -> Vec<usize> {
    let bundle = &ctx.bundle;
    let mut keyed: Vec<(f64, usize)> = ctx
        .normal
        .iter()
        .map(|&index| {
            let response = Response::linear(bundle, index, prices);
            let indices = response
                .model
                .indices(&bundle.upgrade_arr[index].state.payload);
            let upgrade = &bundle.upgrade_arr[index];
            let saved = -response.score(&indices) * upgrade.base_chance
                / upgrade.special_cost.max(1) as f64;
            (saved, index)
        })
        .collect();
    keyed.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut order: Vec<usize> = keyed.into_iter().map(|pair| pair.1).collect();
    order.extend(ctx.adv.iter().copied());
    order
}

fn distance(a: &Ind, b: &Ind, free: bool) -> usize {
    a.plans.iter().zip(&b.plans).filter(|(x, y)| x != y).count()
        + (free && a.order != b.order) as usize
        + a.chests
            .iter()
            .filter(|(key, state)| b.chests[*key] != **state)
            .count()
}

/// the group an upgrade goes to a child in, at one of three grains
fn group(bundle: &StateBundle, index: usize, grain: usize) -> usize {
    let upgrade = &bundle.upgrade_arr[index];
    match grain {
        0 => upgrade.piece_type_usize,
        1 => upgrade.piece_index,
        _ => index,
    }
}

/// each upgrade's place in an order as a share of the free tapped run, past 2 where free taps stop
fn places(bundle: &StateBundle, order: &[usize]) -> Vec<f64> {
    let mut highest = [-1i64; 7];
    let mut valid = Vec::new();
    let mut out = vec![0.0; order.len()];
    for (at, &index) in order.iter().enumerate() {
        let upgrade = &bundle.upgrade_arr[index];
        if upgrade.is_normal_honing && highest[upgrade.piece_index] <= upgrade.upgrade_index as i64
        {
            highest[upgrade.piece_index] = upgrade.upgrade_index as i64;
            valid.push(index);
        } else {
            out[index] = 2.0 + at as f64 / order.len() as f64;
        }
    }
    for (rank, &index) in valid.iter().enumerate() {
        out[index] = rank as f64 / valid.len() as f64;
    }
    out
}

/// an upgrade out of the free taps to the end, or into them beside the levels of its piece
fn toggle(bundle: &StateBundle, mut order: Vec<usize>, item: usize) -> Vec<usize> {
    let inside = places(bundle, &order)[item] < 2.0;
    order.retain(|&x| x != item);
    if inside {
        order.push(item);
        return order;
    }
    let upgrades = &bundle.upgrade_arr;
    let (piece, level) = (upgrades[item].piece_index, upgrades[item].upgrade_index);
    let at = order
        .iter()
        .position(|&x| {
            let other = &upgrades[x];
            other.is_normal_honing && other.piece_index == piece && other.upgrade_index > level
        })
        .unwrap_or(0);
    order.insert(at, item);
    order
}

/// order crossover: a segment of one, the rest in the other's order
fn order_crossover<R: Rng>(rng: &mut R, a: &[usize], b: &[usize]) -> Vec<usize> {
    let len = a.len();
    let (mut lo, mut hi) = (rng.random_range(0..len), rng.random_range(0..len));
    if lo > hi {
        std::mem::swap(&mut lo, &mut hi);
    }
    let kept = &a[lo..=hi];
    let mut rest = b.iter().copied().filter(|x| !kept.contains(x));
    (0..len)
        .map(|at| {
            if (lo..=hi).contains(&at) {
                a[at]
            } else {
                rest.next().unwrap()
            }
        })
        .collect()
}

fn tournament<R: Rng>(rng: &mut R, pop: &[Ind], not: Option<usize>) -> usize {
    let mut pick = || loop {
        let at = rng.random_range(0..pop.len());
        if Some(at) != not || pop.len() < 2 {
            return at;
        }
    };
    let (x, y) = (pick(), pick());
    if pop[x].metric >= pop[y].metric { x } else { y }
}

/// a change to one part of the child, for diversity
fn mutate<R: Rng>(rng: &mut R, ctx: &Ctx, child: &mut Ind) -> Vec<usize> {
    let mut kinds = Vec::new();
    if !ctx.normal.is_empty() {
        kinds.push(0);
    }
    if !ctx.adv.is_empty() {
        kinds.push(1);
    }
    if ctx.free && child.order.len() > 1 {
        kinds.push(2);
    }
    if !ctx.stacks.is_empty() {
        kinds.push(3);
    }
    if kinds.is_empty() {
        return Vec::new();
    }
    match kinds[rng.random_range(0..kinds.len())] {
        0 => {
            // one block of one upgrade takes another option, filled
            let index = ctx.normal[rng.random_range(0..ctx.normal.len())];
            let upgrade = &ctx.bundle.upgrade_arr[index];
            let model = crate::optimizer::response::Model::of(
                upgrade,
                &ctx.bundle.prep_output.juice_info,
                ctx.bundle.block(),
            );
            let mut indices = model.indices(&child.plans[index]);
            let live = model.pity(&indices).0.saturating_sub(1).max(1);
            let start = rng.random_range(0..live) / model.block * model.block;
            let option = rng.random_range(0..model.options.len());
            let end = if rng.random_bool(0.5) {
                (start + model.block).min(indices.len())
            } else {
                indices.len()
            };
            for slot in &mut indices[start..end] {
                *slot = option;
            }
            child.plans[index] = model.plan(&model.canonical(&model.filled(&indices)));
            vec![index]
        }
        1 => {
            let index = ctx.adv[rng.random_range(0..ctx.adv.len())];
            let plan = &mut child.plans[index];
            if !plan.is_empty() {
                let slot = rng.random_range(0..plan.len());
                plan[slot] = smallvec![rng.random_range(0..=MAX_ADV_STATE)];
            }
            Vec::new()
        }
        2 => {
            let len = child.order.len();
            let (a, b) = (rng.random_range(0..len), rng.random_range(0..len));
            child.order.swap(a, b);
            Vec::new()
        }
        _ => {
            let key = &ctx.stacks[rng.random_range(0..ctx.stacks.len())];
            shift_chests(rng, child.chests.get_mut(key).unwrap());
            Vec::new()
        }
    }
}

/// a child of two parents, and the upgrades where the parents' plans met
fn recombine<R: Rng>(rng: &mut R, ctx: &Ctx, a: &Ind, b: &Ind) -> (Ind, Vec<usize>) {
    let mut child = a.clone();
    let grain = rng.random_range(0..3);
    let groups = ctx.bundle.upgrade_arr.len().max(7);
    let take: Vec<bool> = (0..groups).map(|_| rng.random_bool(0.5)).collect();
    let mut from_b = Vec::new();
    for index in 0..a.plans.len() {
        if take[group(&ctx.bundle, index, grain)] && a.plans[index] != b.plans[index] {
            child.plans[index].clone_from(&b.plans[index]);
            from_b.push(index);
        }
    }
    if ctx.free && a.order.len() > 1 {
        child.order = if rng.random_bool(0.7) {
            // each upgrade's place in the free taps from the parent its group came from
            let (at_a, at_b) = (places(&ctx.bundle, &a.order), places(&ctx.bundle, &b.order));
            let mut keyed: Vec<(f64, usize)> = (0..a.order.len())
                .map(|index| {
                    let from_b = take[group(&ctx.bundle, index, grain)];
                    (if from_b { at_b[index] } else { at_a[index] }, index)
                })
                .collect();
            keyed.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
            keyed.into_iter().map(|pair| pair.1).collect()
        } else {
            order_crossover(rng, &a.order, &b.order)
        };
    }
    for key in &ctx.stacks {
        if rng.random_bool(0.5) {
            child.chests.insert(key.clone(), b.chests[key].clone());
        }
    }
    // the upgrades whose neighbours changed: those of the pieces that mixed
    let pieces: Vec<usize> = from_b
        .iter()
        .map(|&index| ctx.bundle.upgrade_arr[index].piece_type_usize)
        .collect();
    let focus = ctx
        .normal
        .iter()
        .copied()
        .filter(|&index| pieces.contains(&ctx.bundle.upgrade_arr[index].piece_type_usize))
        .collect();
    (child, focus)
}

/// a plan of every part from a base, the normal upgrades by prices between the shadow and the market ones
fn price_seed<R: Rng>(
    rng: &mut R,
    ctx: &mut Ctx,
    base: &Ind,
    shadow: &[f64],
    market: &[f64],
) -> Ind {
    ctx.load(base);
    let lean: f64 = rng.random();
    let prices: Vec<f64> = shadow
        .iter()
        .zip(market)
        .map(|(&s, &m)| {
            let mix = (lean + 0.3 * normal_sample(rng)).clamp(0.0, 1.0);
            ((1.0 - mix) * s + mix * m) * (0.3 * normal_sample(rng)).exp()
        })
        .collect();
    if !ctx.normal.is_empty() {
        linear_plans(ctx, &prices);
    }
    if ctx.free && rng.random_bool(0.5) {
        ctx.bundle.special_state = price_order(ctx, &prices);
    }
    if ctx.free && ctx.normal.len() > 1 {
        // a few upgrades in or out of the free taps
        for _ in 0..rng.random_range(1..=3) {
            let item = ctx.normal[rng.random_range(0..ctx.normal.len())];
            let order = std::mem::take(&mut ctx.bundle.special_state);
            ctx.bundle.special_state = toggle(&ctx.bundle, order, item);
        }
    }
    for key in &ctx.stacks.clone() {
        if rng.random_bool(0.5) {
            let mut chests = ctx.bundle.chest_states.clone();
            let options = chests[key].len();
            let state = chests.get_mut(key).unwrap();
            let total: i64 = state.iter().sum();
            state.fill(0);
            state[rng.random_range(0..options)] = total;
            ctx.bundle.set_chest_state(&chests);
        }
    }
    for _ in 0..2 {
        if !ctx.adv.is_empty() {
            let index = ctx.adv[rng.random_range(0..ctx.adv.len())];
            let slots = ctx.bundle.upgrade_arr[index].state.payload.len();
            if slots > 0 {
                let mut plan = ctx.bundle.upgrade_arr[index].state.payload.clone();
                plan[rng.random_range(0..slots)] = smallvec![rng.random_range(0..=MAX_ADV_STATE)];
                ctx.bundle.upgrade_arr[index].state.update_payload(plan);
            }
        }
    }
    let metric = ctx.eval();
    ctx.snapshot(metric)
}

pub fn solve<R: Rng>(
    rng: &mut R,
    state_bundle: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    run(rng, state_bundle, previous, probe, true)
}

/// `crossover` off makes each child a mutated copy of one parent, for the ablation
fn run<R: Rng>(
    rng: &mut R,
    mut bundle: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
    crossover: bool,
) -> Solved {
    if bundle.upgrade_arr.is_empty() {
        bundle.metric = super::evaluate(&mut bundle, probe);
        return Solved {
            best: bundle,
            blob: Vec::new(),
        };
    }
    let effort = bundle.tuning.lab_effort;
    let count = bundle.upgrade_arr.len();
    let budget = effort * (BUDGET_BASE + BUDGET_PER_UPGRADE * count as f64);
    let carried = previous.map(|previous| previous.carried(&bundle));
    let normal: Vec<usize> = (0..count)
        .filter(|&index| bundle.upgrade_arr[index].is_normal_honing)
        .collect();
    let adv: Vec<usize> = (0..count)
        .filter(|&index| !bundle.upgrade_arr[index].is_normal_honing)
        .collect();
    let stacks: Vec<String> = bundle
        .prep_output
        .chests
        .iter()
        .filter(|(_, chest)| chest.options.len() > 1 && chest.count > 0)
        .map(|(key, _)| key.clone())
        .collect();
    // a chest may give the leapstones
    let free = bundle.affords_a_free_tap()
        || bundle.prep_output.chests.values().any(|chest| {
            chest
                .options
                .iter()
                .any(|option| option.contains_key(&bundle.prep_output.special_label))
        });
    let mut ctx = Ctx {
        bundle,
        probe,
        spent: 0.0,
        normal,
        adv,
        stacks,
        free,
        crossover,
    };

    // the given and carried plans, on the grid, as candidates
    let mut seeds: Vec<Ind> = Vec::new();
    let mut givens: Vec<StateBundle> = vec![ctx.bundle.clone()];
    givens.extend(carried.into_iter().flatten());
    for given in givens {
        let ind = Ind {
            plans: given
                .upgrade_arr
                .iter()
                .map(|upgrade| upgrade.state.payload.clone())
                .collect(),
            order: given.special_state.clone(),
            chests: given.chest_states.clone(),
            metric: 0.0,
        };
        if ind.plans.len() != count {
            continue;
        }
        ctx.load(&ind);
        for &index in &ctx.normal.clone() {
            fit(&mut ctx.bundle, index);
        }
        if check::invalid(&ctx.bundle).is_some() {
            continue;
        }
        let metric = ctx.eval();
        seeds.push(ctx.snapshot(metric));
    }
    if seeds.is_empty() {
        // nothing given is a plan the game allows: start from nothing
        for &index in &ctx.normal.clone() {
            let len = ctx.bundle.upgrade_arr[index].clean_prob_dist_len;
            ctx.bundle.upgrade_arr[index]
                .state
                .update_payload(vec![smallvec![]; len]);
        }
        for &index in &ctx.adv.clone() {
            let slots = ctx.bundle.upgrade_arr[index].state.payload.len();
            ctx.bundle.upgrade_arr[index]
                .state
                .update_payload(vec![smallvec![0]; slots]);
        }
        let chests: ChestStates = ctx
            .bundle
            .prep_output
            .chests
            .iter()
            .map(|(key, chest)| (key.clone(), chest.default_state()))
            .collect();
        ctx.bundle.set_chest_state(&chests);
        ctx.bundle.special_state = (0..count).collect();
        let metric = ctx.eval();
        seeds.push(ctx.snapshot(metric));
    }
    seeds.sort_by(|a, b| b.metric.total_cmp(&a.metric));

    // the population: the seeds, then plans by perturbed prices
    let size = ((4.0 + 4.0 * effort).round() as usize).clamp(4, 16);
    let mut pop: Vec<Ind> = Vec::new();
    for seed in &seeds {
        if pop.iter().all(|ind| distance(ind, seed, free) > 0) && pop.len() < size {
            pop.push(seed.clone());
        }
    }
    ctx.load(&seeds[0]);
    ctx.eval();
    let shadow = ctx.bundle.shadow_prices(ctx.probe);
    let market = market(&ctx.bundle);
    let mut tries = 0;
    while pop.len() < size && tries < 2 * size && ctx.spent < budget * 0.3 {
        tries += 1;
        let base = seeds[rng.random_range(0..seeds.len())].clone();
        let ind = price_seed(rng, &mut ctx, &base, &shadow, &market);
        if pop.iter().all(|other| distance(other, &ind, free) > 0) {
            pop.push(ind);
        }
    }

    // the best seed polished, as the population's anchor
    let mut best = 0;
    for at in 0..pop.len() {
        if pop[at].metric > pop[best].metric {
            best = at;
        }
    }
    ctx.load(&pop[best].clone());
    let polish_cap = ctx.spent + budget * 0.25;
    let metric = ctx.polish(rng, pop[best].metric, polish_cap);
    pop[best] = ctx.snapshot(metric);
    if free && ctx.normal.len() > 1 {
        // the order by what a leapstone saves at the anchor's prices, plans answering it
        let prices = ctx.bundle.shadow_prices(ctx.probe);
        ctx.bundle.special_state = price_order(&ctx, &prices);
        let start = ctx.eval();
        let metric = ctx.polish(rng, start, polish_cap + budget * 0.1);
        let other = ctx.snapshot(metric);
        if metric > pop[best].metric {
            pop[best] = other;
        } else if pop.len() < size + 1 {
            pop.push(other);
        }
    }

    // generations, one child at a time
    let stall_limit = (STALL * effort).ceil() as usize;
    let restarts = (RESTARTS * effort).ceil() as usize;
    let (mut stall, mut dry) = (0, 0);
    let mut since = f64::NEG_INFINITY;
    let reserve = budget * 0.8;
    while ctx.spent < reserve && pop.len() > 1 {
        let top = pop
            .iter()
            .map(|ind| ind.metric)
            .fold(f64::NEG_INFINITY, f64::max);
        if stall >= stall_limit {
            // converged: stop after restarts that found nothing, else new members by prices
            dry = if since.is_finite() && top <= since + since.abs() * 1e-9 {
                dry + 1
            } else {
                0
            };
            since = top;
            if dry >= restarts {
                break;
            }
            let best = (0..pop.len())
                .max_by(|&a, &b| pop[a].metric.total_cmp(&pop[b].metric).then(b.cmp(&a)))
                .unwrap();
            ctx.load(&pop[best].clone());
            let cap = (ctx.spent + budget * 0.1).min(reserve);
            let metric = ctx.polish(rng, pop[best].metric, cap);
            pop[best] = ctx.snapshot(metric);
            let anchor = pop[best].clone();
            for at in (0..pop.len()).filter(|&at| at != best) {
                let seed = price_seed(rng, &mut ctx, &anchor, &shadow, &market);
                // a pass of responses, so that it can compete
                let mut list = ctx.normal.clone();
                shuffle(rng, &mut list);
                let mut metric = seed.metric;
                for chunk in list.chunks(4) {
                    metric = ctx.sweep(metric, chunk).0;
                }
                pop[at] = ctx.snapshot(metric);
            }
            stall = 0;
            continue;
        }
        let pa = tournament(rng, &pop, None);
        let (mut child, mut focus) = if ctx.crossover {
            let pb = tournament(rng, &pop, Some(pa));
            recombine(rng, &ctx, &pop[pa], &pop[pb])
        } else {
            (pop[pa].clone(), Vec::new())
        };
        let same = pop.iter().any(|ind| distance(ind, &child, free) == 0);
        if same || !ctx.crossover || rng.random_bool(MUTATE) {
            focus.extend(mutate(rng, &ctx, &mut child));
        }
        ctx.load(&child);
        let metric = ctx.eval();
        let metric = ctx.improve(rng, metric, &focus);
        let child = ctx.snapshot(metric);
        let near = (0..pop.len())
            .min_by_key(|&at| (distance(&pop[at], &child, free), at))
            .unwrap();
        if child.metric > pop[near].metric {
            pop[near] = child;
        }
        let now = pop
            .iter()
            .map(|ind| ind.metric)
            .fold(f64::NEG_INFINITY, f64::max);
        if now > top + top.abs() * 1e-9 {
            stall = 0;
        } else {
            stall += 1;
        }
    }

    // the best polished with what is left
    let mut best = 0;
    for at in 0..pop.len() {
        if pop[at].metric > pop[best].metric {
            best = at;
        }
    }
    let winner = pop[best].clone();
    ctx.load(&winner);
    let metric = ctx.eval();
    ctx.polish(rng, metric, budget.max(ctx.spent + 1.0));
    let mut out = ctx.bundle;
    if check::invalid(&out).is_some() {
        // never expected; the best seed is a plan the game allows
        let seed = &seeds[0];
        for (upgrade, plan) in out.upgrade_arr.iter_mut().zip(&seed.plans) {
            upgrade.state.update_payload(plan.clone());
        }
        out.special_state.clone_from(&seed.order);
        out.set_chest_state(&seed.chests);
    }
    out.metric = super::evaluate(&mut out, ctx.probe);
    Solved {
        best: out,
        blob: Vec::new(),
    }
}
