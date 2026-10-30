//! price: coordinate the upgrades by material prices, then polish on the true metric.
//! Price trajectories (linear best responses, prices pulled toward the shadow prices of the
//! plans they give, coordinate steps on each kinked material) make start points; a primal
//! phase of true responses, advanced, order and chest moves polishes them; price kicks after.

use super::{check, evaluate};
use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::optimizer::response::{Model, Response};
use crate::optimizer::{Previous, Solved};
use crate::probe::Probe;
use crate::state_bundle::{StateBundle, StateEssence};
use ordered_float::OrderedFloat;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use smallvec::smallvec;

/// work at effort 1, in evaluations: a constant and per upgrade
const BASE_WORK: f64 = 28000.0;
const UPGRADE_WORK: f64 = 2600.0;
/// share of the work for the price trajectories
const PRICE_SHARE: f64 = 0.25;
/// shares for polishing the best start, and the next few together
const FIRST_SHARE: f64 = 0.35;
const OTHER_SHARE: f64 = 0.2;
/// share kept for the last deep polish of the best
const FINAL_SHARE: f64 = 0.1;
const TRAJECTORY_STEPS: usize = 6;
/// what an evaluation costs per upgrade it recomputes, over its base of one
const CHANGED_WORK: f64 = 0.05;
const SHADOW_WORK: f64 = 2.0;
const MIN_RESPONSE_WORK: f64 = 0.1;
/// the most steps a response takes for the best plan of all pools at once, past which it improves pool by pool
const RESPONSE_JOINT: usize = 500_000;
/// what working out one advanced plan's mean use costs
const ADV_USAGE_WORK: f64 = 0.02;
const POLISH_ROUNDS: usize = 8;
/// free tap positions reached less often than this are left alone
const ORDER_TOL: f64 = 1e-4;
/// an order move that loses less than this share is tried again with the moved upgrade replanned
const REPLAN_GAP: f64 = 2e-3;
/// the kicks by where their share of a roll ends, and their names for `Tuning::lab_off`
const KICKS: [(f64, &str); 6] = [
    (0.15, "kchest"),
    (0.3, "kwindow"),
    (0.45, "kadv"),
    (0.65, "kexchange"),
    (0.8, "kpiece"),
    (1.0, "kperturb"),
];

fn gain(new: f64, old: f64) -> bool {
    old == f64::NEG_INFINITY || new > old + old.abs() * 1e-10 + 1e-9
}

struct Run {
    sb: StateBundle,
    models: Vec<Option<Model>>,
    normal: Vec<usize>,
    adv: Vec<usize>,
    effort: f64,
    work: f64,
    budget: f64,
    best: (f64, StateEssence),
    hashes: Vec<u64>,
    rng: SmallRng,
    /// per advanced upgrade, the mean use of each row by each of its plans, once asked for
    usage: Vec<Vec<Vec<f64>>>,
    /// what responses take as their joint limit now
    joint: usize,
}

/// an advanced plan by its number, slot 0 the fastest
fn adv_plan(mut id: usize, slots: usize) -> Vec<usize> {
    (0..slots)
        .map(|_| {
            let k = id % (MAX_ADV_STATE + 1);
            id /= MAX_ADV_STATE + 1;
            k
        })
        .collect()
}

impl Run {
    /// a piece left out by `Tuning::lab_off`
    fn off(&self, piece: &str) -> bool {
        self.sb.tuning.lab_off.split('+').any(|name| name == piece)
    }

    /// a piece added by `Tuning::lab_on`
    fn on(&self, piece: &str) -> bool {
        self.sb.tuning.lab_on.split('+').any(|name| name == piece)
    }

    /// whether the kick a roll draws is left out
    fn kick_off(&self, roll: f64) -> bool {
        let at = KICKS
            .iter()
            .position(|&(end, _)| roll < end)
            .unwrap_or(KICKS.len() - 1);
        self.off(KICKS[at].1)
    }

    fn eval(&mut self, probe: &mut Probe) -> f64 {
        let mut changed = 0;
        for (hash, upgrade) in self.hashes.iter_mut().zip(&self.sb.upgrade_arr) {
            if *hash != upgrade.state.hash {
                *hash = upgrade.state.hash;
                changed += 1;
            }
        }
        self.work += 1.0 + CHANGED_WORK * changed as f64;
        let metric = evaluate(&mut self.sb, probe);
        self.sb.metric = metric;
        if gain(metric, self.best.0) {
            self.best = (metric, self.sb.to_essence());
        }
        metric
    }

    fn load(&mut self, essence: &StateEssence) {
        let metric = OrderedFloat(self.sb.metric);
        self.sb.clone_from_essence(essence, &metric);
        for upgrade in self.sb.upgrade_arr.iter_mut() {
            upgrade.state.streak_cache = None;
        }
    }

    fn restore(&mut self, essence: &StateEssence, probe: &mut Probe) {
        self.load(essence);
        self.eval(probe);
    }

    fn shadow(&mut self, probe: &mut Probe) -> Vec<f64> {
        self.work += SHADOW_WORK;
        self.sb.shadow_prices(probe)
    }

    /// lowest and highest price a unit of each row can cost
    fn bounds(&self) -> Vec<(f64, f64)> {
        let prep = &self.sb.prep_output;
        prep.bands[prep.optimizer_plan]
            .iter()
            .map(|bands| {
                bands
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |a, b| {
                        (a.0.min(b.1), a.1.max(b.1))
                    })
            })
            .collect()
    }

    fn indices(&self, index: usize) -> Vec<usize> {
        let model = self.models[index].as_ref().unwrap();
        model.indices(&self.sb.upgrade_arr[index].state.payload)
    }

    fn set_plan(&mut self, index: usize, indices: &[usize]) -> bool {
        let model = self.models[index].as_ref().unwrap();
        let plan = model.plan(indices);
        let state = &mut self.sb.upgrade_arr[index].state;
        if state.payload == plan {
            return false;
        }
        state.update_payload(plan);
        true
    }

    /// the mean use of each row by every plan of an advanced upgrade, from its supports alone
    fn adv_usage(&mut self, index: usize) -> &Vec<Vec<f64>> {
        if self.usage[index].is_empty() {
            let slots = self.sb.upgrade_arr[index].state.payload.len();
            let kept = self.sb.upgrade_arr[index].state.payload.clone();
            let plans = (MAX_ADV_STATE + 1).pow(slots as u32);
            let sb = &mut self.sb;
            let upgrade = &mut sb.upgrade_arr[index];
            let mut out = Vec::with_capacity(plans);
            for id in 0..plans {
                let plan = adv_plan(id, slots);
                upgrade
                    .state
                    .update_payload(plan.iter().map(|&k| smallvec![k]).collect());
                upgrade.update_this_prob_dist(&mut sb.adv_cache, &sb.prep_output.juice_info);
                upgrade.update_this_support(&sb.prep_output.juice_info);
                out.push(
                    upgrade
                        .cost_dist
                        .iter()
                        .map(|support| support.mean)
                        .collect(),
                );
            }
            upgrade.state.update_payload(kept);
            self.work += plans as f64 * ADV_USAGE_WORK;
            self.usage[index] = out;
        }
        &self.usage[index]
    }

    /// Every listed upgrade takes its best plan at these prices: a normal one by the linear
    /// response, an advanced one of two slots or fewer by its mean use. Whether any moved.
    fn priced(&mut self, which: &[usize], prices: &[f64]) -> bool {
        let joint = self.sb.tuning.response_joint;
        let steps = self.sb.tuning.response_steps;
        let mut moved = false;
        for &index in which {
            if self.models[index].is_none() {
                let slots = self.sb.upgrade_arr[index].state.payload.len();
                if slots > 2 {
                    continue;
                }
                let usage = self.adv_usage(index);
                let cost =
                    |id: usize| -> f64 { usage[id].iter().zip(prices).map(|(u, p)| u * p).sum() };
                let top = (0..usage.len())
                    .min_by(|&a, &b| cost(a).total_cmp(&cost(b)))
                    .unwrap();
                let plan: Vec<_> = adv_plan(top, slots).iter().map(|&k| smallvec![k]).collect();
                let state = &mut self.sb.upgrade_arr[index].state;
                if state.payload != plan {
                    state.update_payload(plan);
                    moved = true;
                }
                continue;
            }
            let now = self.indices(index);
            let response = Response::linear(&self.sb, index, prices);
            let (_, found) = response.better(&now, joint);
            self.work += response.work(steps).max(MIN_RESPONSE_WORK);
            moved |= self.set_plan(index, &found);
        }
        moved
    }

    /// every chest stack with a choice all on its option worth the most at these prices
    fn priced_chests(&mut self, prices: &[f64], probe: &mut Probe) {
        let special = self.special_value(probe);
        let prep = &self.sb.prep_output;
        let mut states = self.sb.chest_states.clone();
        for (key, chest) in &prep.chests {
            if chest.options.len() < 2 {
                continue;
            }
            let worth = |option: &std::collections::BTreeMap<String, f64>| -> f64 {
                option
                    .iter()
                    .map(|(label, amount)| {
                        if *label == prep.special_label {
                            amount * special
                        } else {
                            prep.table
                                .get_row(label)
                                .map_or(0.0, |row| amount * prices[row])
                        }
                    })
                    .sum()
            };
            let mut top = 0;
            for at in 1..chest.options.len() {
                if worth(&chest.options[at]) > worth(&chest.options[top]) {
                    top = at;
                }
            }
            let mut state = vec![0; chest.options.len()];
            state[top] = chest.count;
            states.insert(key.clone(), state);
        }
        self.sb.set_chest_state(&states);
    }

    /// what one more free tap leapstone is worth at the state last evaluated, by a difference
    fn special_value(&mut self, probe: &mut Probe) -> f64 {
        let delta = self
            .normal
            .iter()
            .map(|&i| self.sb.upgrade_arr[i].special_cost)
            .min()
            .unwrap_or(1)
            .max(1);
        let base = evaluate(&mut self.sb, probe);
        self.sb.prep_output.special_budget += delta;
        let up = evaluate(&mut self.sb, probe);
        self.sb.prep_output.special_budget -= delta;
        self.work += 2.0;
        self.sb.metric = evaluate(&mut self.sb, probe);
        ((up - base) / delta as f64).max(0.0)
    }

    /// the true best response of a normal upgrade put on it, unchecked; whether it moved
    fn respond_raw(&mut self, index: usize, probe: &mut Probe) -> bool {
        let joint = self.joint;
        let steps = self.sb.tuning.response_steps;
        let now = self.indices(index);
        let response = Response::of(&mut self.sb, index, probe);
        let (_, found) = response.better(&now, joint);
        self.work += response.work(steps).max(MIN_RESPONSE_WORK);
        let model = self.models[index].as_ref().unwrap();
        if model.canonical(&found) == model.canonical(&now) {
            return false;
        }
        self.set_plan(index, &found)
    }

    fn respond(&mut self, index: usize, probe: &mut Probe) {
        let (old, old_metric) = (
            self.sb.upgrade_arr[index].state.payload.clone(),
            self.sb.metric,
        );
        if !self.respond_raw(index, probe) {
            return;
        }
        if !gain(self.eval(probe), old_metric) {
            self.sb.upgrade_arr[index].state.update_payload(old);
            self.sb.metric = old_metric;
        }
    }

    /// one advanced slot to its best index, the others as they are
    fn adv_slot(&mut self, index: usize, slot: usize, probe: &mut Probe) {
        let now = self.sb.upgrade_arr[index].state.payload[slot][0];
        let mut top = (self.sb.metric, now);
        for k in (0..=MAX_ADV_STATE).filter(|&k| k != now) {
            if self.work >= self.budget {
                break;
            }
            let state = &mut self.sb.upgrade_arr[index].state;
            state.payload[slot] = smallvec![k];
            state.update_hash();
            let metric = self.eval(probe);
            if gain(metric, top.0) {
                top = (metric, k);
            }
        }
        let state = &mut self.sb.upgrade_arr[index].state;
        state.payload[slot] = smallvec![top.1];
        state.update_hash();
        self.sb.metric = top.0;
    }

    /// every advanced plan within two of this one in each slot, the best kept
    fn adv_window(&mut self, index: usize, probe: &mut Probe) {
        let now: Vec<usize> = self.sb.upgrade_arr[index]
            .state
            .payload
            .iter()
            .map(|slot| slot[0])
            .collect();
        let mut plans: Vec<Vec<usize>> = vec![Vec::new()];
        for &k in &now {
            plans = plans
                .into_iter()
                .flat_map(|plan| {
                    (k.saturating_sub(2)..=(k + 2).min(MAX_ADV_STATE)).map(move |k| {
                        let mut plan = plan.clone();
                        plan.push(k);
                        plan
                    })
                })
                .collect();
        }
        let mut top = (self.sb.metric, now.clone());
        for plan in plans.into_iter().filter(|plan| *plan != now) {
            if self.work >= self.budget {
                break;
            }
            let state = &mut self.sb.upgrade_arr[index].state;
            state.update_payload(plan.iter().map(|&k| smallvec![k]).collect());
            if gain(self.eval(probe), top.0) {
                top = (self.sb.metric, plan);
            }
        }
        let state = &mut self.sb.upgrade_arr[index].state;
        state.update_payload(top.1.iter().map(|&k| smallvec![k]).collect());
        self.sb.metric = top.0;
    }

    /// moves of one upgrade to a free tap position that is reached, first improvement
    fn order_moves(&mut self, probe: &mut Probe) {
        if self.off("order") || !self.sb.affords_a_free_tap() || self.normal.len() < 2 {
            return;
        }
        self.sb.compute_special_probs(false);
        let probs = self.sb.special_probs().clone();
        let mut reach = 0;
        let mut tail: f64 = probs.iter().sum();
        for &p in &probs {
            if tail < ORDER_TOL {
                break;
            }
            reach += 1;
            tail -= p;
        }
        let n = self.sb.upgrade_arr.len();
        let reach = reach.min(n);
        let mut moves: Vec<(usize, usize)> = (0..reach)
            .flat_map(|to| {
                (0..n)
                    .filter(move |&from| from != to)
                    .map(move |from| (from, to))
            })
            .collect();
        moves.shuffle(&mut self.rng);
        moves.truncate(((40 + 6 * n) as f64 * self.effort.sqrt()) as usize);
        let mut replans = if self.off("replan") { 0 } else { reach + 2 };
        for (from, to) in moves {
            if self.work >= self.budget {
                break;
            }
            let (old, old_metric) = (self.sb.special_state.clone(), self.sb.metric);
            let id = self.sb.special_state.remove(from);
            self.sb.special_state.insert(to, id);
            if self.sb.special_state == old {
                continue;
            }
            let plain = self.eval(probe);
            let close = old_metric - plain <= REPLAN_GAP * old_metric.abs();
            if gain(plain, old_metric) || self.models[id].is_none() || !close || replans == 0 {
                if !gain(self.sb.metric, old_metric) {
                    self.sb.special_state = old;
                    self.sb.metric = old_metric;
                }
                continue;
            }
            // the moved upgrade replanned for where it now is, where the move alone lost little
            replans -= 1;
            let plan = self.sb.upgrade_arr[id].state.payload.clone();
            if self.respond_raw(id, probe) && gain(self.eval(probe), old_metric) {
                continue;
            }
            self.sb.upgrade_arr[id].state.update_payload(plan);
            self.sb.special_state = old;
            self.sb.metric = old_metric;
        }
    }

    /// moves of some chests of a stack from one option to another, first improvement
    fn chest_moves(&mut self, probe: &mut Probe) {
        let keys: Vec<String> = self
            .sb
            .prep_output
            .chests
            .iter()
            .filter(|(_, chest)| chest.options.len() > 1)
            .map(|(key, _)| key.clone())
            .collect();
        for key in keys {
            let options = self.sb.prep_output.chests[&key].options.len();
            let mut pairs: Vec<(usize, usize)> = (0..options)
                .flat_map(|a| (0..options).filter(move |&b| b != a).map(move |b| (a, b)))
                .collect();
            pairs.shuffle(&mut self.rng);
            pairs.truncate((12.0 * self.effort.sqrt()) as usize + 2);
            for (a, b) in pairs {
                let have = self.sb.chest_states[&key][a];
                // all of it, then random amounts or a third at a time
                let mut amounts = vec![have];
                if self.on("chestrand") {
                    if have > 1 {
                        amounts.extend((0..4).map(|_| self.rng.random_range(1..have)));
                    }
                } else {
                    while amounts.last().unwrap() / 3 > 0 {
                        amounts.push(amounts.last().unwrap() / 3);
                    }
                }
                for amount in amounts.into_iter().filter(|&amount| amount > 0) {
                    if self.work >= self.budget {
                        return;
                    }
                    let (old, old_metric) = (self.sb.chest_states.clone(), self.sb.metric);
                    let mut states = old.clone();
                    let state = states.get_mut(&key).unwrap();
                    state[a] -= amount;
                    state[b] += amount;
                    self.sb.set_chest_state(&states);
                    if gain(self.eval(probe), old_metric) {
                        break;
                    }
                    self.sb.set_chest_state(&old);
                    self.sb.metric = old_metric;
                }
            }
        }
    }

    /// primal rounds from the state as it is, until a round gains nothing or the limit
    /// Rounds use the default joint limit of the responses until one gains nothing, then the
    /// larger one where deep.
    fn polish(&mut self, limit: f64, deep: bool, probe: &mut Probe) {
        let limit = limit.min(self.budget);
        self.joint = self.sb.tuning.response_joint;
        for round in 0..POLISH_ROUNDS {
            let start = self.sb.metric;
            let mut order = self.normal.clone();
            order.shuffle(&mut self.rng);
            for index in order {
                if self.work >= limit {
                    return;
                }
                self.respond(index, probe);
            }
            for index in self.adv.clone() {
                if self.work >= limit {
                    return;
                }
                if round == 0 {
                    for slot in 0..self.sb.upgrade_arr[index].state.payload.len() {
                        self.adv_slot(index, slot, probe);
                    }
                }
                self.adv_window(index, probe);
            }
            if self.work >= limit {
                return;
            }
            self.order_moves(probe);
            self.chest_moves(probe);
            if !gain(self.sb.metric, start) {
                if !deep || self.joint >= RESPONSE_JOINT {
                    return;
                }
                self.joint = RESPONSE_JOINT;
            }
        }
    }

    /// prices pulled toward the shadow prices of the plans they give; the best state seen
    fn trajectory(
        &mut self,
        mut prices: Vec<f64>,
        chests: bool,
        limit: f64,
        probe: &mut Probe,
    ) -> (f64, StateEssence, Vec<f64>) {
        let mut top = (f64::NEG_INFINITY, self.sb.to_essence(), prices.clone());
        for step in 0..TRAJECTORY_STEPS {
            if self.work >= limit {
                break;
            }
            let all: Vec<usize> = (0..self.sb.upgrade_arr.len()).collect();
            let moved = self.priced(&all, &prices);
            if chests {
                self.priced_chests(&prices, probe);
            }
            let metric = self.eval(probe);
            if metric > top.0 {
                top = (metric, self.sb.to_essence(), prices.clone());
            }
            if !moved && step > 0 {
                break;
            }
            let shadow = self.shadow(probe);
            let bounds = self.bounds();
            let alpha = 1.0 / (step as f64 + 2.0);
            for (row, price) in prices.iter_mut().enumerate() {
                *price = ((1.0 - alpha) * *price + alpha * shadow[row])
                    .clamp(bounds[row].0, bounds[row].1);
            }
        }
        top
    }

    /// rows each upgrade pays for (an advanced one once its use is known)
    fn rows(&self, index: usize) -> Vec<usize> {
        if self.models[index].is_none() {
            let usage = &self.usage[index];
            return (0..self.sb.prep_output.table.len())
                .filter(|&row| usage.iter().any(|plan| plan[row] > 0.0))
                .collect();
        }
        let upgrade = &self.sb.upgrade_arr[index];
        let table = &self.sb.prep_output.table;
        let mut out: Vec<usize> = (0..crate::materials::NUM_BASE_MATS)
            .filter(|&row| upgrade.costs[row] > 0.0 || upgrade.unlock_costs[row] > 0.0)
            .collect();
        let model = self.models[index].as_ref().unwrap();
        out.extend(model.pools.iter().flatten().map(|&id| table.juice_row(id)));
        out
    }

    /// Two upgrades that share a kinked row priced apart on it, one cheap and one dear, so
    /// that the stock moves from one to the other. Whether there was such a pair.
    fn exchange(&mut self, probe: &mut Probe) -> bool {
        let bounds = self.bounds();
        let users: Vec<(usize, Vec<usize>)> = (0..bounds.len())
            .filter(|&row| bounds[row].1 - bounds[row].0 > 1e-9)
            .map(|row| {
                let which: Vec<usize> = (0..self.sb.upgrade_arr.len())
                    .filter(|&i| self.rows(i).contains(&row))
                    .collect();
                (row, which)
            })
            .filter(|(_, which)| which.len() > 1)
            .collect();
        if users.is_empty() {
            return false;
        }
        let (row, which) = &users[self.rng.random_range(0..users.len())];
        let a = which[self.rng.random_range(0..which.len())];
        let mut b = which[self.rng.random_range(0..which.len() - 1)];
        if b == a {
            b = which[which.len() - 1];
        }
        let shadow = self.shadow(probe);
        let (lo, hi) = bounds[*row];
        let mut cheap = shadow.clone();
        cheap[*row] = lo + self.rng.random::<f64>() * (shadow[*row] - lo).max(0.0);
        let mut dear = shadow;
        dear[*row] = hi - self.rng.random::<f64>() * (hi - dear[*row]).max(0.0);
        self.priced(&[a], &cheap);
        self.priced(&[b], &dear);
        true
    }

    /// prices moved at random: a kinked row anywhere between its ends, the rest a little
    fn perturbed(&mut self, prices: &[f64], share: f64) -> Vec<f64> {
        let bounds = self.bounds();
        prices
            .iter()
            .zip(&bounds)
            .map(|(&price, &(lo, hi))| {
                if self.rng.random::<f64>() >= share {
                    price
                } else if hi - lo > 1e-9 {
                    lo + self.rng.random::<f64>() * (hi - lo)
                } else {
                    price * (0.6 * (self.rng.random::<f64>() - 0.5)).exp()
                }
            })
            .collect()
    }

    /// each kinked row's price stepped toward either end, kept where the priced state gains
    fn coordinate(
        &mut self,
        start: (f64, StateEssence, Vec<f64>),
        limit: f64,
        probe: &mut Probe,
    ) -> (f64, StateEssence, Vec<f64>) {
        let (mut metric, essence, mut prices) = start;
        self.restore(&essence, probe);
        metric = metric.max(self.sb.metric);
        let mut essence = essence;
        let bounds = self.bounds();
        for row in 0..prices.len() {
            let (lo, hi) = bounds[row];
            if hi - lo <= 1e-9 {
                continue;
            }
            for target in [lo, hi] {
                if self.work >= limit {
                    return (metric, essence, prices);
                }
                let mut tried = prices.clone();
                tried[row] = 0.5 * (prices[row] + target);
                self.priced(&(0..self.sb.upgrade_arr.len()).collect::<Vec<_>>(), &tried);
                let found = self.eval(probe);
                if gain(found, metric) {
                    (metric, essence, prices) = (found, self.sb.to_essence(), tried);
                } else {
                    self.restore(&essence, probe);
                }
            }
        }
        (metric, essence, prices)
    }
}

/// every normal plan on the block grid, by streaks first where asked
fn on_grid(sb: &mut StateBundle, models: &[Option<Model>], snap: bool) {
    let block = sb.block();
    for (index, upgrade) in sb.upgrade_arr.iter_mut().enumerate() {
        let Some(model) = &models[index] else {
            continue;
        };
        if snap {
            upgrade.state.snap_to_block(
                block,
                upgrade.piece_type_usize,
                upgrade.upgrade_index,
                &sb.prep_output.juice_info,
            );
        }
        let plan = model.plan(&model.filled(&model.indices(&upgrade.state.payload)));
        if plan != upgrade.state.payload {
            upgrade.state.update_payload(plan);
        }
    }
}

/// the order by what a free tap saves per leapstone, lower levels of a piece first
fn ratio_order(run: &Run, prices: &[f64]) -> Vec<usize> {
    let sb = &run.sb;
    let value = |index: usize| -> f64 {
        let upgrade = &sb.upgrade_arr[index];
        let cost = -Response::linear(sb, index, prices).score(&run.indices(index));
        cost * upgrade.base_chance / upgrade.special_cost.max(1) as f64
    };
    let mut left: Vec<(usize, f64)> = run.normal.iter().map(|&i| (i, value(i))).collect();
    let mut out = Vec::new();
    while !left.is_empty() {
        // the next level of each piece is what may go now
        let mut pick: Option<usize> = None;
        for (at, &(index, worth)) in left.iter().enumerate() {
            let upgrade = &sb.upgrade_arr[index];
            let lowest = left.iter().all(|&(other, _)| {
                let o = &sb.upgrade_arr[other];
                o.piece_index != upgrade.piece_index || o.upgrade_index >= upgrade.upgrade_index
            });
            if lowest && pick.is_none_or(|p| worth > left[p].1) {
                pick = Some(at);
            }
        }
        out.push(left.remove(pick.unwrap()).0);
    }
    out.extend(&run.adv);
    out
}

pub fn solve<R: Rng>(
    rng: &mut R,
    mut sb: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    let n = sb.upgrade_arr.len();
    if n == 0 {
        sb.metric = evaluate(&mut sb, probe);
        return Solved {
            best: sb,
            blob: Vec::new(),
        };
    }
    let block = sb.block();
    let models: Vec<Option<Model>> = sb
        .upgrade_arr
        .iter()
        .map(|u| {
            u.is_normal_honing
                .then(|| Model::of(u, &sb.prep_output.juice_info, block))
        })
        .collect();
    let normal: Vec<usize> = (0..n).filter(|&i| models[i].is_some()).collect();
    let adv: Vec<usize> = (0..n).filter(|&i| models[i].is_none()).collect();
    let effort = sb.tuning.lab_effort.max(0.01);
    let budget = effort * (BASE_WORK + UPGRADE_WORK * n as f64);
    let carried: Vec<StateBundle> = previous.map_or(Vec::new(), |p| p.carried(&sb));

    // the given state, on the grid as it is and by streaks
    on_grid(&mut sb, &models, false);
    let given = sb.to_essence();
    let mut run = Run {
        best: (f64::NEG_INFINITY, given.clone()),
        hashes: vec![0; n],
        rng: SmallRng::seed_from_u64(rng.random()),
        usage: vec![Vec::new(); n],
        joint: 0,
        sb,
        models,
        normal,
        adv,
        effort,
        work: 0.0,
        budget,
    };
    let mut candidates = vec![given.clone()];
    on_grid(&mut run.sb, &run.models, true);
    candidates.push(run.sb.to_essence());
    for other in &carried {
        run.sb.my_clone_from(other);
        on_grid(&mut run.sb, &run.models, false);
        candidates.push(run.sb.to_essence());
        on_grid(&mut run.sb, &run.models, true);
        candidates.push(run.sb.to_essence());
    }
    // nothing used anywhere, always a valid plan
    let mut empty = given.clone();
    let juice_info = &run.sb.prep_output.juice_info;
    for (index, upgrade) in run.sb.upgrade_arr.iter().enumerate() {
        empty.state_arr[index] = if upgrade.is_normal_honing {
            vec![smallvec![]; upgrade.clean_prob_dist_len]
        } else {
            let slots =
                juice_info.adv_uindex_to_id[upgrade.piece_type_usize][upgrade.upgrade_index].len();
            vec![smallvec![0]; slots]
        };
    }
    empty.special_state = (0..n).collect();
    empty.chest_states = run
        .sb
        .prep_output
        .chests
        .iter()
        .map(|(key, chest)| (key.clone(), chest.default_state()))
        .collect();
    candidates.push(empty.clone());
    candidates.dedup();
    for essence in &candidates {
        run.load(essence);
        if check::invalid(&run.sb).is_none() {
            run.eval(probe);
        }
    }
    if run.best.0 == f64::NEG_INFINITY {
        run.restore(&empty, probe);
    }
    let incumbent = run.best.1.clone();
    run.restore(&incumbent, probe);
    let prices = run.shadow(probe);

    // the free tap order by leapstone efficiency
    if !run.off("ratio") && run.sb.affords_a_free_tap() {
        let metric = run.sb.metric;
        let old = run.sb.special_state.clone();
        run.sb.special_state = ratio_order(&run, &prices);
        if !gain(run.eval(probe), metric) {
            run.sb.special_state = old;
            run.sb.metric = metric;
        }
    }

    // price trajectories from the incumbent's shadow prices and from perturbed ones
    let (incumbent_metric, incumbent) = run.best.clone();
    let mut starts = Vec::new();
    {
        let limit = run.budget * PRICE_SHARE;
        let kinked = run.bounds().iter().any(|&(lo, hi)| hi - lo > 1e-9);
        let mut count = 0;
        while !run.off("prices") && run.work < limit * 0.7 && (kinked || count < 4) {
            run.restore(&incumbent, probe);
            let start = if count < 2 {
                prices.clone()
            } else {
                run.perturbed(&prices, 0.6)
            };
            let chests = count % 2 == 0;
            let found = run.trajectory(start, chests, limit * 0.7, probe);
            starts.push(found);
            count += 1;
        }
        starts.sort_by(|a, b| b.0.total_cmp(&a.0));
        if let Some(first) = starts.first().cloned() {
            let found = run.coordinate(first, limit, probe);
            starts.insert(0, found);
        }
        starts.dedup_by(|a, b| (a.0 - b.0).abs() <= 1e-9 * b.0.abs());
    }

    // polish the best of the incumbent and the price starts, then the next few
    let mut pool: Vec<(f64, StateEssence)> = starts.into_iter().map(|s| (s.0, s.1)).collect();
    pool.push((incumbent_metric, incumbent));
    pool.sort_by(|a, b| b.0.total_cmp(&a.0));
    pool.dedup_by(|a, b| (a.0 - b.0).abs() <= 1e-9 * b.0.abs());
    let count = (1.0 + effort).round() as usize;
    for (at, (_, essence)) in pool.iter().take(count + 1).enumerate() {
        if run.work >= run.budget {
            break;
        }
        let share = if at == 0 {
            FIRST_SHARE
        } else {
            OTHER_SHARE / count as f64
        };
        run.restore(essence, probe);
        let limit = run.work + run.budget * share;
        run.polish(limit, true, probe);
    }

    // kicks: part of the upgrades priced at moved shadow prices, polished, kept where better
    let mut stall = 0;
    let stall_max = (30.0 + 40.0 * effort) as usize;
    let kick_end = run.budget * (1.0 - FINAL_SHARE);
    let kicks = !run.off("kicks") && KICKS.iter().any(|&(_, name)| !run.off(name));
    while kicks && run.work < kick_end && stall < stall_max {
        let best = run.best.clone();
        run.restore(&best.1, probe);
        let mut roll = run.rng.random::<f64>();
        while run.kick_off(roll) {
            roll = run.rng.random::<f64>();
        }
        if roll < 0.15
            && run
                .sb
                .prep_output
                .chests
                .values()
                .any(|c| c.options.len() > 1)
        {
            // one stack opened at random
            let keys: Vec<String> = run
                .sb
                .prep_output
                .chests
                .iter()
                .filter(|(_, c)| c.options.len() > 1)
                .map(|(k, _)| k.clone())
                .collect();
            let key = keys[run.rng.random_range(0..keys.len())].clone();
            let mut states = run.sb.chest_states.clone();
            let state = run.sb.prep_output.chests[&key].random_state(&mut run.rng);
            states.insert(key, state);
            run.sb.set_chest_state(&states);
        } else if roll < 0.3 && !run.off("kwindow") && run.sb.affords_a_free_tap() {
            // a window of the free tap order shuffled
            let len = run.sb.special_state.len();
            let at = run.rng.random_range(0..len);
            let end = (at + 2 + run.rng.random_range(0..4)).min(len);
            run.sb.special_state[at..end].shuffle(&mut run.rng);
        } else if roll < 0.45 && !run.off("kadv") && !run.adv.is_empty() {
            // one advanced upgrade anywhere
            let index = run.adv[run.rng.random_range(0..run.adv.len())];
            let slots = run.sb.upgrade_arr[index].state.payload.len();
            let plan: Vec<_> = (0..slots)
                .map(|_| smallvec![run.rng.random_range(0..=MAX_ADV_STATE)])
                .collect();
            // or every advanced upgrade of its piece type on that plan, the normal ones then responding
            let which: Vec<usize> = if run.on("advall") && run.rng.random_bool(0.5) {
                let piece = run.sb.upgrade_arr[index].piece_type_usize;
                run.adv
                    .iter()
                    .copied()
                    .filter(|&i| run.sb.upgrade_arr[i].piece_type_usize == piece)
                    .collect()
            } else {
                vec![index]
            };
            for i in which {
                let len = run.sb.upgrade_arr[i].state.payload.len();
                let mine = (0..len).map(|k| plan[k.min(slots - 1)].clone()).collect();
                run.sb.upgrade_arr[i].state.update_payload(mine);
            }
        } else if roll < 0.65 && !run.off("kexchange") && run.exchange(probe) {
        } else if roll < 0.8 && !run.off("kpiece") && !run.normal.is_empty() {
            // the upgrades of a piece type all take their true response at once
            let piece = run.sb.upgrade_arr[run.normal[run.rng.random_range(0..run.normal.len())]]
                .piece_type_usize;
            let which: Vec<usize> = run
                .normal
                .iter()
                .copied()
                .filter(|&i| run.sb.upgrade_arr[i].piece_type_usize == piece)
                .collect();
            let joint = run.sb.tuning.response_joint;
            let steps = run.sb.tuning.response_steps;
            let mut found = Vec::new();
            for &index in &which {
                let now = run.indices(index);
                let response = Response::of(&mut run.sb, index, probe);
                found.push(response.better(&now, joint).1);
                run.work += response.work(steps).max(MIN_RESPONSE_WORK);
            }
            for (&index, plan) in which.iter().zip(&found) {
                run.set_plan(index, plan);
            }
        } else if !run.off("kperturb") {
            let shadow = run.shadow(probe);
            let kicked = run.perturbed(&shadow, 0.5);
            let mut which: Vec<usize> = (0..n).collect();
            if run.rng.random::<f64>() < 0.7 {
                let piece = run.sb.upgrade_arr[which[run.rng.random_range(0..which.len())]]
                    .piece_type_usize;
                which.retain(|&i| run.sb.upgrade_arr[i].piece_type_usize == piece);
            }
            run.priced(&which, &kicked);
            if run.rng.random::<f64>() < 0.3 {
                run.priced_chests(&kicked, probe);
            }
        }
        run.eval(probe);
        let limit = run.work + run.budget * 0.1;
        run.polish(limit, false, probe);
        if gain(run.best.0, best.0) {
            stall = 0;
        } else {
            stall += 1;
        }
    }

    // the best polished once more with the larger joint limit
    let best = run.best.1.clone();
    run.restore(&best, probe);
    run.polish(run.budget, true, probe);

    let best = run.best.1.clone();
    run.load(&best);
    if check::invalid(&run.sb).is_some() {
        run.load(&empty);
    }
    let mut out = run.sb;
    out.metric = evaluate(&mut out, probe);
    Solved {
        best: out,
        blob: Vec::new(),
    }
}
