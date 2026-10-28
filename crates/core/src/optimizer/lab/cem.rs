//! Cross-entropy method: a probability model over the decisions, sampled in batches and refit
//! to the best of each batch, then polished by best responses. See `arena_data/lab/attempts/cem.md`.

use super::super::response::{Model, Response};
use super::super::{Previous, Solved};
use super::{check, evaluate};
use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::chest::ChestStates;
use crate::probe::Probe;
use crate::state::OneState;
use crate::state_bundle::StateBundle;
use rand::Rng;
use std::collections::BTreeMap;

// evaluations per unit of effort: a constant and a share per upgrade
const BUDGET_BASE: f64 = 10000.0;
const BUDGET_PER_UPGRADE: f64 = 1300.0;
// spread of each model from the start plan in turn, of each about the best, and rounds
// without a new best that end one
const SPREADS: [f64; 3] = [1.0, 2.0, 0.5];
const NARROW: f64 = 0.5;
const STALL_ROUNDS: usize = 10;
// share of the parts one sample redraws at most, and the chance it redraws one piece type's normals
const PICK_SHARE: f64 = 0.25;
const GROUP_PICK: f64 = 0.2;
const ELITE_SHARE: f64 = 0.25;
const MEAN_SMOOTH: f64 = 0.7;
const SPREAD_SMOOTH: f64 = 0.5;
const PROB_FLOOR: f64 = 0.02;
// Plackett-Luce weight a position of the order is worth: at first, and the range the refit keeps it in
const ORDER_SCALE: f64 = 0.5;
const ORDER_MIN: f64 = 0.3;
const ORDER_MAX: f64 = 3.0;
const IMPROVE_TOL: f64 = 1e-9;
// most passes of a polish, and the gain of a pass under which it stops
const POLISH_PASSES: usize = 12;
const POLISH_TOL: f64 = 1e-7;

/// one complete plan
#[derive(Clone)]
struct Plan {
    payloads: Vec<Vec<OneState>>,
    order: Vec<usize>,
    chests: ChestStates,
    metric: f64,
}

/// what one pool of one normal upgrade samples: id (0 none), front taps, back taps before the pity
#[derive(Clone, Copy)]
struct PoolDraw {
    cat: usize,
    front: f64,
    back: f64,
}

/// what a sample drew, kept for the refit, and which parts it put in its plan
#[derive(Clone)]
struct Draw {
    picked: Vec<bool>,
    pools: Vec<Vec<PoolDraw>>,
    groups: Vec<f64>,
    adv: Vec<Vec<usize>>,
    order: Vec<f64>,
    chests: Vec<Vec<f64>>,
}

struct Gauss {
    mu: f64,
    sigma: f64,
    floor: f64,
}

struct PoolModel {
    probs: Vec<f64>,
    front: Gauss,
    back: Gauss,
}

struct NormalModel {
    index: usize,
    model: Model,
    group: usize,
    scale: f64,
    pools: Vec<PoolModel>,
}

struct ChestModel {
    key: String,
    count: i64,
    logits: Vec<Gauss>,
}

/// The model: per normal upgrade and pool an id and two streak lengths, a shift of the fronts
/// per piece type, per advanced slot a categorical, Plackett-Luce weights for the order, and a
/// logistic normal split per chest stack.
struct Cem {
    normals: Vec<NormalModel>,
    groups: Vec<Gauss>,
    advs: Vec<(usize, Vec<Vec<f64>>)>,
    order: Option<Vec<f64>>,
    chests: Vec<ChestModel>,
}

fn normal<R: Rng>(rng: &mut R) -> f64 {
    let u: f64 = rng.random::<f64>().max(1e-300);
    let v: f64 = rng.random();
    (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
}

fn categorical<R: Rng>(rng: &mut R, probs: &[f64]) -> usize {
    let mut at = rng.random::<f64>() * probs.iter().sum::<f64>();
    for (k, p) in probs.iter().enumerate() {
        at -= p;
        if at < 0.0 {
            return k;
        }
    }
    probs.len() - 1
}

impl Gauss {
    fn new(mu: f64, sigma: f64, floor: f64) -> Gauss {
        Gauss {
            mu,
            sigma: sigma.max(floor),
            floor,
        }
    }
    fn draw<R: Rng>(&self, rng: &mut R) -> f64 {
        self.mu + self.sigma * normal(rng)
    }
    fn fit(&mut self, values: &[f64]) {
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        self.mu = (1.0 - MEAN_SMOOTH) * self.mu + MEAN_SMOOTH * mean;
        self.sigma =
            ((1.0 - SPREAD_SMOOTH) * self.sigma + SPREAD_SMOOTH * var.sqrt()).max(self.floor);
    }
}

fn fit_probs(probs: &mut [f64], picks: &[usize]) {
    let n = picks.len() as f64;
    for (k, p) in probs.iter_mut().enumerate() {
        let freq = picks.iter().filter(|&&x| x == k).count() as f64 / n;
        *p = ((1.0 - MEAN_SMOOTH) * *p + MEAN_SMOOTH * freq).max(PROB_FLOOR);
    }
    let sum: f64 = probs.iter().sum();
    probs.iter_mut().for_each(|p| *p /= sum);
}

/// a categorical peaked at one choice, with some mass on its neighbours and the rest spread
fn peaked(size: usize, at: usize, peak: f64) -> Vec<f64> {
    let mut out = vec![(1.0 - peak) * 0.5 / size as f64; size];
    for (k, p) in out.iter_mut().enumerate() {
        let d = (k as i64 - at as i64).unsigned_abs() as f64;
        *p += (1.0 - peak) * 0.5 * (-d).exp() / 2.0;
    }
    out[at] += peak;
    let sum: f64 = out.iter().sum();
    out.iter_mut().for_each(|p| *p /= sum);
    out
}

fn round_block(x: f64, block: usize, len: usize) -> usize {
    let b = block as f64;
    (((x.max(0.0) / b).round() * b) as usize).min(len)
}

/// the plan of one normal upgrade from what its pools drew
fn build(n: &NormalModel, draws: &[PoolDraw], group: f64) -> Vec<OneState> {
    let model = &n.model;
    let len = model.len;
    let mut idx = vec![0usize; len];
    let mut fronts = vec![0usize; draws.len()];
    for (p, d) in draws.iter().enumerate() {
        if d.cat == 0 {
            continue;
        }
        let opt = model.strides[p] * d.cat;
        fronts[p] = round_block(d.front + group * n.scale, model.block, len);
        idx[..fronts[p]].iter_mut().for_each(|x| *x += opt);
    }
    let pity = model.pity(&idx).0;
    for (p, d) in draws.iter().enumerate() {
        if d.cat == 0 || d.back < 0.5 {
            continue;
        }
        let opt = model.strides[p] * d.cat;
        let start = round_block(pity as f64 - 1.0 - d.back, model.block, len);
        if start + 1 < pity {
            idx[start.max(fronts[p])..]
                .iter_mut()
                .for_each(|x| *x += opt);
        }
    }
    model.plan(&model.canonical(&model.filled(&idx)))
}

/// the draw a plan of one normal upgrade comes from, about: its most used id, front and back streaks
fn derive(n: &NormalModel, payload: &[OneState]) -> Vec<PoolDraw> {
    let model = &n.model;
    let idx = model.canonical(&model.indices(payload));
    let (pity, by_chance) = model.pity(&idx);
    let live = if by_chance { pity } else { pity - 1 };
    let which = |p: usize, t: usize| idx[t] / model.strides[p] % (model.pools[p].len() + 1);
    let mut out = Vec::new();
    let mut fronts_only = vec![0usize; model.len];
    for p in 0..model.pools.len() {
        let mut counts = vec![0; model.pools[p].len() + 1];
        (0..live).for_each(|t| counts[which(p, t)] += 1);
        let cat = (1..counts.len())
            .max_by_key(|&k| (counts[k], usize::MAX - k))
            .filter(|&k| counts[k] > 0)
            .unwrap_or(0);
        let front = (0..live).take_while(|&t| which(p, t) != 0).count();
        if cat > 0 {
            fronts_only[..front]
                .iter_mut()
                .for_each(|x| *x += model.strides[p] * cat);
        }
        out.push(PoolDraw {
            cat,
            front: front as f64,
            back: 0.0,
        });
    }
    let front_pity = model.pity(&fronts_only).0;
    for (p, d) in out.iter_mut().enumerate() {
        let tail = (0..live).rev().take_while(|&t| which(p, t) != 0).count();
        let start = live - tail;
        if d.cat > 0 && tail > 0 && start > d.front as usize {
            d.back = (front_pity as f64 - 1.0 - start as f64).max(0.0);
        }
    }
    out
}

fn plan_of(bundle: &StateBundle) -> Plan {
    Plan {
        payloads: bundle
            .upgrade_arr
            .iter()
            .map(|u| u.state.payload.clone())
            .collect(),
        order: bundle.special_state.clone(),
        chests: bundle.chest_states.clone(),
        metric: bundle.metric,
    }
}

fn apply(bundle: &mut StateBundle, plan: &Plan) {
    for (u, payload) in bundle.upgrade_arr.iter_mut().zip(&plan.payloads) {
        if u.state.payload != *payload {
            u.state.update_payload(payload.clone());
        }
    }
    bundle.special_state.clone_from(&plan.order);
    bundle.set_chest_state(&plan.chests);
}

fn split_chest(count: i64, logits: &[f64]) -> Vec<i64> {
    let top = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<f64> = logits.iter().map(|z| (z - top).exp()).collect();
    let sum: f64 = weights.iter().sum();
    let exact: Vec<f64> = weights.iter().map(|w| w / sum * count as f64).collect();
    let mut out: Vec<i64> = exact.iter().map(|x| x.floor() as i64).collect();
    let mut rest: Vec<usize> = (0..out.len()).collect();
    rest.sort_by(|&a, &b| {
        (exact[b] - exact[b].floor())
            .partial_cmp(&(exact[a] - exact[a].floor()))
            .unwrap()
            .then(a.cmp(&b))
    });
    let short = count - out.iter().sum::<i64>();
    for k in 0..short as usize {
        out[rest[k % rest.len()]] += 1;
    }
    out
}

/// the best of the samples given, by metric, at least two, or none where there are fewer
fn top_of(mut with: Vec<&(f64, Draw)>) -> Vec<&Draw> {
    if with.len() < 2 {
        return Vec::new();
    }
    with.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let keep = ((with.len() as f64 * ELITE_SHARE).ceil() as usize).max(2);
    with.into_iter().take(keep).map(|s| &s.1).collect()
}

impl Cem {
    /// a model centred on a plan
    fn around(bundle: &StateBundle, plan: &Plan, spread: f64) -> Cem {
        let juice_info = &bundle.prep_output.juice_info;
        let block = bundle.block();
        let mut group_of: BTreeMap<usize, usize> = BTreeMap::new();
        let mut normals = Vec::new();
        let mut advs = Vec::new();
        for (index, u) in bundle.upgrade_arr.iter().enumerate() {
            if !u.is_normal_honing {
                let slots = plan.payloads[index]
                    .iter()
                    .map(|s| peaked(MAX_ADV_STATE + 1, s[0], 0.5))
                    .collect();
                advs.push((index, slots));
                continue;
            }
            let model = Model::of(u, juice_info, block);
            let scale = model.len as f64;
            let next = group_of.len();
            let group = *group_of.entry(u.piece_type_usize).or_insert(next);
            let mut n = NormalModel {
                index,
                model,
                group,
                scale,
                pools: Vec::new(),
            };
            let floor = 0.3 * block as f64;
            let draws = derive(&n, &plan.payloads[index]);
            n.pools = draws
                .iter()
                .enumerate()
                .map(|(p, d)| {
                    let size = n.model.pools[p].len() + 1;
                    let probs = if size == 1 {
                        vec![1.0]
                    } else {
                        let mut probs = vec![0.3 / (size - 1) as f64; size];
                        probs[d.cat] += 0.7;
                        let sum: f64 = probs.iter().sum();
                        probs.iter().map(|p| p / sum).collect()
                    };
                    PoolModel {
                        probs,
                        front: Gauss::new(
                            d.front,
                            spread * (block as f64).max(0.15 * scale),
                            floor,
                        ),
                        back: Gauss::new(d.back, spread * (block as f64).max(0.1 * scale), floor),
                    }
                })
                .collect();
            normals.push(n);
        }
        let groups = (0..group_of.len())
            .map(|_| Gauss::new(0.0, 0.08 * spread, 0.005))
            .collect();
        let order = bundle.affords_a_free_tap().then(|| {
            let mut w = vec![0.0; plan.order.len()];
            for (pos, &i) in plan.order.iter().enumerate() {
                w[i] = -(pos as f64) * ORDER_SCALE / spread.max(0.25);
            }
            w
        });
        let chests = bundle
            .prep_output
            .chests
            .iter()
            .filter(|(_, c)| c.options.len() > 1 && c.count > 0)
            .map(|(key, c)| {
                let state = &plan.chests[key];
                let k = state.len() as f64;
                ChestModel {
                    key: key.clone(),
                    count: c.count,
                    logits: state
                        .iter()
                        .map(|&x| {
                            let share = 0.9 * x as f64 / c.count as f64 + 0.1 / k;
                            Gauss::new(share.ln(), 1.5 * spread, 0.05)
                        })
                        .collect(),
                }
            })
            .collect();
        Cem {
            normals,
            groups,
            advs,
            order,
            chests,
        }
    }

    /// the parts a sample can redraw: normals, advanced, the order, chests
    fn parts(&self) -> usize {
        self.normals.len() + self.advs.len() + self.order.is_some() as usize + self.chests.len()
    }

    /// the most parts one sample redraws
    fn reach(&self) -> usize {
        ((self.parts() as f64 * PICK_SHARE).ceil() as usize)
            .max(2)
            .min(self.parts())
    }

    /// which parts one sample redraws: a few at random, or every normal of one piece type
    fn pick<R: Rng>(&self, rng: &mut R) -> Vec<bool> {
        let parts = self.parts();
        let mut picked = vec![false; parts];
        if !self.groups.is_empty() && rng.random::<f64>() < GROUP_PICK {
            let group = rng.random_range(0..self.groups.len());
            for (k, n) in self.normals.iter().enumerate() {
                picked[k] = n.group == group;
            }
            return picked;
        }
        let count = rng.random_range(1..=self.reach());
        let mut all: Vec<usize> = (0..parts).collect();
        for k in 0..count {
            let at = rng.random_range(k..parts);
            all.swap(k, at);
            picked[all[k]] = true;
        }
        picked
    }

    /// a plan that is the base but for the parts it redraws from the model
    fn sample<R: Rng>(&self, rng: &mut R, base: &Plan) -> (Plan, Draw) {
        let mut picked = self.pick(rng);
        let mut plan = base.clone();
        let at = self.normals.len() + self.advs.len();
        let mut order_keys = Vec::new();
        if let Some(w) = &self.order {
            let keys: Vec<f64> = w
                .iter()
                .map(|w| w - (-rng.random::<f64>().max(1e-300).ln()).ln())
                .collect();
            let mut order: Vec<usize> = (0..w.len()).collect();
            order.sort_by(|&a, &b| keys[b].partial_cmp(&keys[a]).unwrap().then(a.cmp(&b)));
            order_keys = vec![0.0; w.len()];
            for (pos, &i) in order.iter().enumerate() {
                order_keys[i] = pos as f64;
            }
            if picked[at] {
                // the normals the new order moves most are redrawn with it
                let mut moved: Vec<(usize, usize)> = self
                    .normals
                    .iter()
                    .enumerate()
                    .map(|(k, n)| {
                        let was = base.order.iter().position(|&i| i == n.index).unwrap();
                        let now = order_keys[n.index] as usize;
                        (was.abs_diff(now), k)
                    })
                    .filter(|m| m.0 > 0)
                    .collect();
                moved.sort_by(|a, b| b.cmp(a));
                for &(_, k) in moved.iter().take(self.reach() / 2) {
                    picked[k] = true;
                }
                plan.order = order;
            }
        }
        let groups: Vec<f64> = self.groups.iter().map(|g| g.draw(rng)).collect();
        let pools = self
            .normals
            .iter()
            .enumerate()
            .map(|(k, n)| {
                let draws: Vec<PoolDraw> = n
                    .pools
                    .iter()
                    .map(|p| PoolDraw {
                        cat: categorical(rng, &p.probs),
                        front: p.front.draw(rng),
                        back: p.back.draw(rng),
                    })
                    .collect();
                if picked[k] {
                    plan.payloads[n.index] = build(n, &draws, groups[n.group]);
                }
                draws
            })
            .collect();
        let at = self.normals.len();
        let adv = self
            .advs
            .iter()
            .enumerate()
            .map(|(k, (index, slots))| {
                let picks: Vec<usize> = slots.iter().map(|p| categorical(rng, p)).collect();
                if picked[at + k] {
                    plan.payloads[*index] = picks.iter().map(|&k| smallvec::smallvec![k]).collect();
                }
                picks
            })
            .collect();
        let at = at + self.advs.len();
        let at = at + self.order.is_some() as usize;
        let chests = self
            .chests
            .iter()
            .enumerate()
            .map(|(k, c)| {
                let z: Vec<f64> = c.logits.iter().map(|g| g.draw(rng)).collect();
                if picked[at + k] {
                    plan.chests.insert(c.key.clone(), split_chest(c.count, &z));
                }
                z
            })
            .collect();
        (
            plan,
            Draw {
                picked,
                pools,
                groups,
                adv,
                order: order_keys,
                chests,
            },
        )
    }

    /// each part refit to the best of the samples that redrew it
    fn refit(&mut self, scored: &[(f64, Draw)]) {
        let elites = |part: usize| top_of(scored.iter().filter(|s| s.1.picked[part]).collect());
        for g in 0..self.groups.len() {
            let top = top_of(
                scored
                    .iter()
                    .filter(|s| {
                        self.normals
                            .iter()
                            .enumerate()
                            .any(|(k, n)| n.group == g && s.1.picked[k])
                    })
                    .collect(),
            );
            if !top.is_empty() {
                self.groups[g].fit(&top.iter().map(|d| d.groups[g]).collect::<Vec<_>>());
            }
        }
        for k in 0..self.normals.len() {
            let top = elites(k);
            if top.is_empty() {
                continue;
            }
            let n = &mut self.normals[k];
            let (group, scale) = (n.group, n.scale);
            let mu = self.groups[group].mu;
            for (p, pm) in n.pools.iter_mut().enumerate() {
                if pm.probs.len() > 1 {
                    fit_probs(
                        &mut pm.probs,
                        &top.iter().map(|d| d.pools[k][p].cat).collect::<Vec<_>>(),
                    );
                }
                // the front with what its group drew past the group's mean
                pm.front.fit(
                    &top.iter()
                        .map(|d| d.pools[k][p].front + (d.groups[group] - mu) * scale)
                        .collect::<Vec<_>>(),
                );
                pm.back
                    .fit(&top.iter().map(|d| d.pools[k][p].back).collect::<Vec<_>>());
            }
        }
        let at = self.normals.len();
        for k in 0..self.advs.len() {
            let top = elites(at + k);
            if top.is_empty() {
                continue;
            }
            for (s, probs) in self.advs[k].1.iter_mut().enumerate() {
                fit_probs(probs, &top.iter().map(|d| d.adv[k][s]).collect::<Vec<_>>());
            }
        }
        let at = at + self.advs.len();
        if self.order.is_some() {
            let top = elites(at);
            if let (Some(w), false) = (&mut self.order, top.is_empty()) {
                let k = top.len() as f64;
                let stats: Vec<(f64, f64)> = (0..w.len())
                    .map(|i| {
                        let mean = top.iter().map(|d| d.order[i]).sum::<f64>() / k;
                        let var = top.iter().map(|d| (d.order[i] - mean).powi(2)).sum::<f64>() / k;
                        (mean, var.sqrt())
                    })
                    .collect();
                // sharper where the elites agree on the order
                let spread = stats.iter().map(|s| s.1).sum::<f64>() / w.len() as f64;
                let sharp = (1.0 / (spread + 0.25)).clamp(ORDER_MIN, ORDER_MAX);
                for (w, (mean, _)) in w.iter_mut().zip(stats) {
                    *w = (1.0 - MEAN_SMOOTH) * *w - MEAN_SMOOTH * sharp * mean;
                }
            }
        }
        let at = at + self.order.is_some() as usize;
        for k in 0..self.chests.len() {
            let top = elites(at + k);
            if top.is_empty() {
                continue;
            }
            for (j, g) in self.chests[k].logits.iter_mut().enumerate() {
                g.fit(&top.iter().map(|d| d.chests[k][j]).collect::<Vec<_>>());
            }
        }
    }
}

/// the best plan, and the plan the search now goes on from
struct Run<'a> {
    work: StateBundle,
    best: Plan,
    cur: Plan,
    spent: f64,
    probe: &'a mut Probe,
}

impl Run<'_> {
    fn score(&mut self, plan: &mut Plan) -> f64 {
        apply(&mut self.work, plan);
        let metric = evaluate(&mut self.work, self.probe);
        self.spent += 1.0;
        plan.metric = metric;
        if metric > self.cur.metric {
            self.cur = plan.clone();
        }
        if metric > self.best.metric {
            self.best = plan.clone();
        }
        metric
    }

    /// rounds of one model about the best, until it stalls or the budget is spent
    fn cem<R: Rng>(&mut self, rng: &mut R, budget: f64, spread: f64) {
        let stop = self.spent + budget;
        let mut cem = Cem::around(&self.work, &self.cur, spread);
        // each part redrawn some four times a batch
        let mean_pick = (1.0 + cem.reach() as f64) / 2.0;
        let batch = ((4.0 * cem.parts() as f64 / mean_pick).ceil() as usize).clamp(12, 48);
        let mut stalled = 0;
        while self.spent + batch as f64 <= stop && stalled < STALL_ROUNDS {
            let before = self.cur.metric;
            let base = self.cur.clone();
            let mut scored: Vec<(f64, Draw)> = Vec::with_capacity(batch);
            for _ in 0..batch {
                let (mut plan, draw) = cem.sample(rng, &base);
                let metric = self.score(&mut plan);
                scored.push((metric, draw));
            }
            if self.cur.metric > before + before.abs() * IMPROVE_TOL {
                stalled = 0;
            } else {
                stalled += 1;
            }
            cem.refit(&scored);
        }
    }

    /// best responses, advanced, order and chest moves on the best, until none gains or the budget is spent
    fn polish<R: Rng>(&mut self, rng: &mut R, budget: f64) {
        let stop = self.spent + budget;
        let steps = self.work.tuning.response_steps;
        let joint = self.work.tuning.response_joint;
        let n = self.cur.payloads.len();
        for pass in 0..POLISH_PASSES {
            let before = self.cur.metric;
            let mut order: Vec<usize> = (0..n).collect();
            for k in (1..n).rev() {
                order.swap(k, rng.random_range(0..=k));
            }
            for &i in &order {
                if self.spent >= stop {
                    return;
                }
                if self.work.upgrade_arr[i].is_normal_honing {
                    let mut plan = self.cur.clone();
                    apply(&mut self.work, &plan);
                    let response = Response::of(&mut self.work, i, self.probe);
                    let model = &response.model;
                    let now = model.indices(&plan.payloads[i]);
                    let (_, found) = response.better(&now, joint);
                    self.spent += response.work(steps);
                    let found = model.plan(&found);
                    if found != plan.payloads[i] {
                        plan.payloads[i] = found;
                        self.score(&mut plan);
                    }
                } else {
                    for s in 0..self.cur.payloads[i].len() {
                        for d in [-2i64, -1, 1, 2] {
                            let mut plan = self.cur.clone();
                            let k = plan.payloads[i][s][0] as i64 + d;
                            if k < 0 || k > MAX_ADV_STATE as i64 {
                                continue;
                            }
                            plan.payloads[i][s][0] = k as usize;
                            self.score(&mut plan);
                        }
                    }
                }
            }
            self.polish_order();
            self.polish_chests(pass);
            if self.cur.metric <= before + before.abs() * POLISH_TOL || self.spent >= stop {
                return;
            }
        }
    }

    /// every move of one upgrade in the order into or out of the part free taps reach
    fn polish_order(&mut self) {
        if !self.work.affords_a_free_tap() {
            return;
        }
        let plan = self.cur.clone();
        apply(&mut self.work, &plan);
        evaluate(&mut self.work, self.probe);
        let mut tail = 1.0;
        let mut reach = 0;
        for p in self.work.special_probs() {
            tail -= p;
            if tail < 1e-4 {
                break;
            }
            reach += 1;
        }
        let n = plan.order.len();
        let reach = (reach + 2).min(n).min(10);
        // the prefix rebuilt a place at a time, each the best of every upgrade there
        let mut built = plan.clone();
        for at in 0..reach {
            let mut pick = (f64::NEG_INFINITY, built.order.clone());
            for j in at..n {
                let mut order = built.order.clone();
                let item = order.remove(j);
                order.insert(at, item);
                let mut trial = Plan {
                    order: order.clone(),
                    ..built.clone()
                };
                let metric = self.score(&mut trial);
                if metric > pick.0 {
                    pick = (metric, order);
                }
            }
            built.order = pick.1;
        }
        for j in 0..n {
            for to in (0..n).filter(|&to| to != j && (to < reach || j < reach)) {
                let mut plan = self.cur.clone();
                let mut order = plan.order.clone();
                let item = order.remove(j);
                order.insert(to, item);
                plan.order = order;
                self.score(&mut plan);
            }
        }
    }

    /// per stack: every chest on one option, then moves between two options while they gain, halving
    fn polish_chests(&mut self, pass: usize) {
        let chests: Vec<(String, i64, usize)> = self
            .work
            .prep_output
            .chests
            .iter()
            .filter(|(_, c)| c.options.len() > 1 && c.count > 0)
            .map(|(k, c)| (k.clone(), c.count, c.options.len()))
            .collect();
        for (key, count, options) in chests {
            if pass == 0 {
                for a in 0..options {
                    let mut plan = self.cur.clone();
                    let mut state = vec![0; options];
                    state[a] = count;
                    if plan.chests[&key] != state {
                        plan.chests.insert(key.clone(), state);
                        self.score(&mut plan);
                    }
                }
            }
            let mut step = (count >> (pass + 2)).max(1);
            loop {
                for a in 0..options {
                    for b in (0..options).filter(|&b| b != a) {
                        loop {
                            let before = self.cur.metric;
                            let mut plan = self.cur.clone();
                            let state = plan.chests.get_mut(&key).unwrap();
                            let moved = step.min(state[a]);
                            if moved == 0 {
                                break;
                            }
                            state[a] -= moved;
                            state[b] += moved;
                            if self.score(&mut plan) <= before + before.abs() * IMPROVE_TOL {
                                break;
                            }
                        }
                    }
                }
                if step == 1 {
                    break;
                }
                step = (step / 2).max(1);
            }
        }
    }
}

pub fn solve<R: Rng>(
    rng: &mut R,
    mut state_bundle: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    if state_bundle.upgrade_arr.is_empty() {
        state_bundle.metric = evaluate(&mut state_bundle, probe);
        return Solved {
            best: state_bundle,
            blob: Vec::new(),
        };
    }
    let block = state_bundle.block();
    let snap = |bundle: &mut StateBundle| {
        let juice_info = bundle.prep_output.juice_info.clone();
        for u in bundle.upgrade_arr.iter_mut().filter(|u| u.is_normal_honing) {
            u.state
                .snap_to_block(block, u.piece_type_usize, u.upgrade_index, &juice_info);
            u.state.update_hash();
        }
    };
    // the given plan, the carried ones, and nothing used
    let mut carried: Vec<StateBundle> = previous
        .map(|p| p.carried(&state_bundle))
        .unwrap_or_default();
    snap(&mut state_bundle);
    let mut plans = vec![plan_of(&state_bundle)];
    for c in carried.iter_mut() {
        snap(c);
        let mut plan = plan_of(c);
        let chests = &state_bundle.prep_output.chests;
        if plan.chests.len() != chests.len()
            || chests
                .iter()
                .any(|(key, chest)| !plan.chests.get(key).is_some_and(|s| chest.is_valid(s)))
        {
            plan.chests = state_bundle.chest_states.clone();
        }
        plans.push(plan);
    }
    // nothing used, built from scratch so that it is valid whatever was given
    let juice_info = &state_bundle.prep_output.juice_info;
    let empty = Plan {
        payloads: state_bundle
            .upgrade_arr
            .iter()
            .map(|u| {
                if u.is_normal_honing {
                    vec![OneState::new(); u.clean_prob_dist_len]
                } else {
                    let slots =
                        juice_info.adv_uindex_to_id[u.piece_type_usize][u.upgrade_index].len();
                    vec![smallvec::smallvec![0]; slots]
                }
            })
            .collect(),
        order: (0..state_bundle.upgrade_arr.len()).collect(),
        chests: state_bundle
            .prep_output
            .chests
            .iter()
            .map(|(key, chest)| (key.clone(), chest.default_state()))
            .collect(),
        metric: f64::NEG_INFINITY,
    };
    let mut kept = plan_of(&state_bundle);
    kept.payloads.clone_from(&empty.payloads);
    for (u, (payload, given)) in state_bundle
        .upgrade_arr
        .iter()
        .zip(kept.payloads.iter_mut().zip(&plans[0].payloads))
    {
        if !u.is_normal_honing {
            payload.clone_from(given);
        }
    }
    plans.push(kept);
    plans.push(empty);

    let n = state_bundle.upgrade_arr.len() as f64;
    let budget = state_bundle.tuning.lab_effort * (BUDGET_BASE + BUDGET_PER_UPGRADE * n);
    let none = Plan {
        metric: f64::NEG_INFINITY,
        ..plans[0].clone()
    };
    let mut run = Run {
        best: none.clone(),
        cur: none,
        work: state_bundle,
        spent: 0.0,
        probe,
    };
    for plan in plans.iter_mut() {
        apply(&mut run.work, plan);
        if check::invalid(&run.work).is_none() {
            run.score(plan);
        }
    }

    // every normal upgrade's best response at the shadow prices of the best
    {
        let mut plan = run.best.clone();
        apply(&mut run.work, &plan);
        evaluate(&mut run.work, run.probe);
        let prices = run.work.shadow_prices(run.probe);
        run.spent += 1.0;
        for i in 0..plan.payloads.len() {
            if run.work.upgrade_arr[i].is_normal_honing {
                let response = Response::linear(&run.work, i, &prices);
                let (_, found) = response.best();
                plan.payloads[i] = response.model.plan(&found);
            }
        }
        run.score(&mut plan);
    }

    // Cycles of a model until it stalls and a polish of what it found: from the start plan
    // with a wide model, then about the best with a narrow one, in turn.
    let start = run.best.clone();
    let mut polished = vec![start.metric];
    for cycle in 0.. {
        let fresh = cycle % 2 == 0;
        run.cur = if fresh {
            start.clone()
        } else {
            run.best.clone()
        };
        let spread = if fresh {
            SPREADS[(cycle / 2) % SPREADS.len()]
        } else {
            NARROW
        };
        let before = run.spent;
        run.cem(rng, budget - run.spent, spread);
        if run.spent == before {
            break;
        }
        // a plan already polished is not again
        if polished.contains(&run.cur.metric) {
            continue;
        }
        run.polish(rng, budget - run.spent);
        polished.push(run.cur.metric);
    }

    let best = run.best.clone();
    let mut out = run.work;
    apply(&mut out, &best);
    out.metric = evaluate(&mut out, run.probe);
    Solved {
        best: out,
        blob: Vec::new(),
    }
}
