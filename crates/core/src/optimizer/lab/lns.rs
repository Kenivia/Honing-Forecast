//! lns: large neighbourhood search. From the current state, free a structured part of it (two
//! upgrades, a piece type, a piece, the users of a scarce material, the advanced upgrades, a
//! window or the tail of the free tap order, a chest stack with the upgrades that use what it
//! gives) and rebuild it jointly with the rest fixed: an emptied start, a noisy price start, or
//! a ladder of prices whose plans are combined, then best responses in turn until none moves.
//! Record-to-record acceptance; neighbourhoods weighted by what they gain for their work.
//! See `arena_data/lab/attempts/lns.md`.

use super::{check, evaluate};
use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::materials::NUM_BASE_MATS;
use crate::optimizer::response::{Model, Response};
use crate::optimizer::{Previous, Solved};
use crate::probe::Probe;
use crate::state::{OneState, get_pool_ids};
use crate::state_bundle::{StateBundle, StateEssence};
use ordered_float::OrderedFloat;
use rand::Rng;
use rand::seq::SliceRandom;
use smallvec::smallvec;

/// work at lab_effort 1, in evaluations
const BASE_WORK: f64 = 18000.0;
const WORK_PER_UPGRADE: f64 = 2400.0;
/// shares of the budget for the first rebuild of everything and the last polish
const START_SHARE: f64 = 0.35;
const FINAL_SHARE: f64 = 0.1;
const MAX_FREED: usize = 10;
const SWEEPS: usize = 3;
const PRICE_NOISE: f64 = 0.35;
/// record-to-record deviation at the start, as a share of the best metric, to nothing at the end
const DEVIATION: f64 = 3e-4;
const REACTION: f64 = 0.2;
const MIN_WEIGHT: f64 = 0.05;
const RESTART_AFTER: usize = 20;
/// stop once this share of the budget has gone by without the best moving
const STALL: f64 = 0.45;
/// upgrades tried at each place when the free tap order is built again inside a move
const BUILD_TRIES: usize = 8;
/// what the ladder multiplies the prices of the materials it is about by
const LADDER: [f64; 6] = [0.0, 0.3, 0.6, 1.0, 1.6, 3.0];
const LADDER_ROUNDS: usize = 3;
/// pairs of advanced upgrades a ladder move tries shares between
const ADV_TRANSFERS: usize = 12;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Hood {
    Pair,
    Adv,
    PieceType,
    Piece,
    Material,
    Order,
    /// the free tap order built again from a place on
    OrderTail,
    Chest,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Start {
    Empty,
    Price,
    /// candidate plans of each from prices on a ladder, the best combination of them
    Ladder,
    Kept,
}

const ARMS: [(Hood, Start); 13] = [
    (Hood::Adv, Start::Ladder),
    (Hood::OrderTail, Start::Kept),
    (Hood::Pair, Start::Ladder),
    (Hood::PieceType, Start::Ladder),
    (Hood::Material, Start::Ladder),
    (Hood::PieceType, Start::Price),
    (Hood::PieceType, Start::Empty),
    (Hood::Piece, Start::Price),
    (Hood::Piece, Start::Empty),
    (Hood::Material, Start::Price),
    (Hood::Material, Start::Empty),
    (Hood::Order, Start::Kept),
    (Hood::Chest, Start::Kept),
];

struct Search<'a> {
    probe: &'a mut Probe,
    work: f64,
}

fn normal(rng: &mut impl Rng) -> f64 {
    let u: f64 = rng.random::<f64>().max(1e-12);
    let v: f64 = rng.random();
    (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
}

fn pick(rng: &mut impl Rng, weights: &[f64]) -> Option<usize> {
    let total: f64 = weights.iter().sum();
    if !(total > 0.0) {
        return None;
    }
    let mut at = rng.random::<f64>() * total;
    for (index, &weight) in weights.iter().enumerate() {
        if weight > 0.0 && at < weight {
            return Some(index);
        }
        at -= weight;
    }
    weights.iter().rposition(|&weight| weight > 0.0)
}

/// up to `count` of the indices, by weight, without repeats
fn pick_some(rng: &mut impl Rng, mut weights: Vec<f64>, count: usize) -> Vec<usize> {
    let mut out = Vec::new();
    while out.len() < count {
        let Some(at) = pick(rng, &weights) else { break };
        weights[at] = 0.0;
        out.push(at);
    }
    out
}

fn model_of(sb: &StateBundle, index: usize) -> Model {
    Model::of(
        &sb.upgrade_arr[index],
        &sb.prep_output.juice_info,
        sb.block(),
    )
}

fn has_choice(sb: &StateBundle, index: usize) -> bool {
    let upgrade = &sb.upgrade_arr[index];
    !upgrade.is_normal_honing || model_of(sb, index).options.len() > 1
}

fn uses(sb: &StateBundle, index: usize, row: usize) -> bool {
    let upgrade = &sb.upgrade_arr[index];
    let juice_info = &sb.prep_output.juice_info;
    let (piece_type, level) = (upgrade.piece_type_usize, upgrade.upgrade_index);
    if row < NUM_BASE_MATS {
        return upgrade.cost_dist[row].mean > 0.0
            || (upgrade.is_normal_honing && upgrade.costs[row] > 0.0);
    }
    let id = row - NUM_BASE_MATS;
    if upgrade.is_normal_honing {
        get_pool_ids(piece_type, level, juice_info)
            .iter()
            .flatten()
            .any(|&other| other == id)
    } else {
        juice_info.adv_uindex_to_id[piece_type][level].contains(&id)
    }
}

/// every piece's normal upgrades in level order, on the places they hold
fn fix_order(sb: &StateBundle, order: &mut [usize]) {
    for piece in 0..7 {
        let places: Vec<usize> = (0..order.len())
            .filter(|&at| {
                let upgrade = &sb.upgrade_arr[order[at]];
                upgrade.is_normal_honing && upgrade.piece_index == piece
            })
            .collect();
        let mut held: Vec<usize> = places.iter().map(|&at| order[at]).collect();
        held.sort_by_key(|&index| sb.upgrade_arr[index].upgrade_index);
        for (&at, index) in places.iter().zip(held) {
            order[at] = index;
        }
    }
}

/// the state put on the rules where it is off them
fn make_valid(sb: &mut StateBundle) {
    let block = sb.block();
    let juice_info = sb.prep_output.juice_info.clone();
    for upgrade in sb.upgrade_arr.iter_mut() {
        let (piece_type, level) = (upgrade.piece_type_usize, upgrade.upgrade_index);
        if upgrade.is_normal_honing {
            let model = Model::of(upgrade, &juice_info, block);
            let indices = model.indices(&upgrade.state.payload);
            if model.canonical(&indices) != model.canonical(&model.filled(&indices)) {
                upgrade
                    .state
                    .snap_to_block(block, piece_type, level, &juice_info);
            }
        } else {
            let slots = juice_info.adv_uindex_to_id[piece_type][level].len();
            let payload = &upgrade.state.payload;
            if payload.len() != slots
                || payload
                    .iter()
                    .any(|slot| slot.len() != 1 || slot[0] > MAX_ADV_STATE)
            {
                upgrade.state.update_payload(vec![smallvec![0]; slots]);
            }
        }
    }
    let mut sorted = sb.special_state.clone();
    sorted.sort();
    if sorted != (0..sb.upgrade_arr.len()).collect::<Vec<_>>() {
        sb.special_state = (0..sb.upgrade_arr.len()).collect();
    }
    let mut states = sb.chest_states.clone();
    states.retain(|key, _| sb.prep_output.chests.contains_key(key));
    for (key, chest) in &sb.prep_output.chests {
        if !states.get(key).is_some_and(|state| chest.is_valid(state)) {
            states.insert(key.clone(), chest.default_state());
        }
    }
    sb.set_chest_state(&states);
}

impl Search<'_> {
    fn evaluate(&mut self, sb: &mut StateBundle) -> f64 {
        self.work += 1.0;
        let metric = evaluate(sb, self.probe);
        sb.metric = metric;
        metric
    }

    fn prices(&mut self, sb: &mut StateBundle) -> Vec<f64> {
        self.evaluate(sb);
        self.work += 2.0;
        sb.shadow_prices(self.probe)
    }

    /// this normal upgrade's best plan with the rest as it is; whether it moved
    fn respond(&mut self, sb: &mut StateBundle, index: usize) -> bool {
        if !has_choice(sb, index) {
            return false;
        }
        let response = Response::of(sb, index, self.probe);
        let model = &response.model;
        let now = model.indices(&sb.upgrade_arr[index].state.payload);
        let (_, found) = response.better(&now, sb.tuning.response_joint);
        self.work += response.work(sb.tuning.response_steps);
        let (was, is) = (response.score(&now), response.score(&found));
        if is > was + was.abs() * 1e-12 {
            sb.upgrade_arr[index]
                .state
                .update_payload(model.plan(&found));
            return true;
        }
        false
    }

    /// the same at fixed prices, taken whatever it is
    fn respond_linear(&mut self, sb: &mut StateBundle, index: usize, prices: &[f64]) {
        if !has_choice(sb, index) {
            return;
        }
        let response = Response::linear(sb, index, prices);
        let model = &response.model;
        let now = model.indices(&sb.upgrade_arr[index].state.payload);
        let (_, found) = response.better(&now, sb.tuning.response_joint);
        self.work += response.work(sb.tuning.response_steps);
        let plan: Vec<OneState> = model.plan(&found);
        if plan != sb.upgrade_arr[index].state.payload {
            sb.upgrade_arr[index].state.update_payload(plan);
        }
    }

    /// an advanced upgrade's indices, one slot at a time over every value, until none moves
    fn adv_scan(&mut self, sb: &mut StateBundle, index: usize) -> bool {
        let mut now = self.evaluate(sb);
        let mut moved = false;
        for _ in 0..2 {
            let mut moved_now = false;
            for slot in 0..sb.upgrade_arr[index].state.payload.len() {
                let at = sb.upgrade_arr[index].state.payload[slot][0];
                let mut best = (now, at);
                for value in (0..=MAX_ADV_STATE).filter(|&value| value != at) {
                    let mut payload = sb.upgrade_arr[index].state.payload.clone();
                    payload[slot] = smallvec![value];
                    sb.upgrade_arr[index].state.update_payload(payload);
                    let metric = self.evaluate(sb);
                    if metric > best.0 + best.0.abs() * 1e-12 {
                        best = (metric, value);
                    }
                }
                let mut payload = sb.upgrade_arr[index].state.payload.clone();
                payload[slot] = smallvec![best.1];
                sb.upgrade_arr[index].state.update_payload(payload);
                if best.1 != at {
                    now = best.0;
                    moved_now = true;
                }
            }
            moved |= moved_now;
            if !moved_now {
                break;
            }
        }
        moved
    }

    /// best responses in turn over the set until none moves; the metric after
    fn sweep(
        &mut self,
        sb: &mut StateBundle,
        set: &[usize],
        sweeps: usize,
        rng: &mut impl Rng,
    ) -> f64 {
        let mut order = set.to_vec();
        for _ in 0..sweeps {
            order.shuffle(rng);
            let mut moved = false;
            for &index in &order {
                moved |= if sb.upgrade_arr[index].is_normal_honing {
                    self.respond(sb, index)
                } else {
                    self.adv_scan(sb, index)
                };
            }
            if !moved {
                break;
            }
        }
        self.evaluate(sb)
    }

    /// the set rebuilt together from the start given
    fn rebuild(
        &mut self,
        sb: &mut StateBundle,
        set: &[usize],
        start: Start,
        row: Option<usize>,
        rng: &mut impl Rng,
    ) -> f64 {
        {
            match start {
                Start::Empty => {
                    for &index in set {
                        let upgrade = &mut sb.upgrade_arr[index];
                        let len = upgrade.state.payload.len();
                        let empty = if upgrade.is_normal_honing {
                            vec![smallvec![]; len]
                        } else {
                            vec![smallvec![0]; len]
                        };
                        if upgrade.state.payload != empty {
                            upgrade.state.update_payload(empty);
                        }
                    }
                }
                Start::Price => {
                    let mut prices = self.prices(sb);
                    for price in prices.iter_mut() {
                        *price *= (PRICE_NOISE * normal(rng)).exp();
                    }
                    for &index in set {
                        if sb.upgrade_arr[index].is_normal_honing {
                            self.respond_linear(sb, index, &prices);
                        }
                    }
                }
                Start::Ladder => {
                    self.adv_transfers(sb, set, rng);
                    self.ladder(sb, set, row);
                }
                Start::Kept => {}
            }
        }
        self.sweep(sb, set, SWEEPS, rng)
    }

    /// Each normal upgrade of the set gets candidate plans, its best at the shadow prices with
    /// those of the row (or of every juice and book) scaled along the ladder; then the best
    /// pair of them where there are two, else each in turn takes its best candidate.
    fn ladder(&mut self, sb: &mut StateBundle, set: &[usize], row: Option<usize>) {
        let base = self.prices(sb);
        let normals: Vec<usize> = set
            .iter()
            .copied()
            .filter(|&index| sb.upgrade_arr[index].is_normal_honing && has_choice(sb, index))
            .collect();
        if normals.is_empty() {
            return;
        }
        let mut candidates: Vec<Vec<Vec<OneState>>> = normals
            .iter()
            .map(|&index| vec![sb.upgrade_arr[index].state.payload.clone()])
            .collect();
        let mut by_scale: Vec<Vec<usize>> = Vec::new();
        for scale in LADDER {
            let mut prices = base.clone();
            for (at, price) in prices.iter_mut().enumerate() {
                if row.map_or(at >= NUM_BASE_MATS, |row| row == at) {
                    *price *= scale;
                }
            }
            by_scale.push(Vec::new());
            for (k, &index) in normals.iter().enumerate() {
                let response = Response::linear(sb, index, &prices);
                let model = &response.model;
                let now = model.indices(&sb.upgrade_arr[index].state.payload);
                let (_, found) = response.better(&now, sb.tuning.response_joint);
                self.work += response.work(sb.tuning.response_steps);
                let plan = model.plan(&found);
                let at = match candidates[k].iter().position(|other| *other == plan) {
                    Some(at) => at,
                    None => {
                        candidates[k].push(plan);
                        candidates[k].len() - 1
                    }
                };
                by_scale.last_mut().unwrap().push(at);
            }
        }
        let set_plan = |sb: &mut StateBundle, index: usize, plan: &Vec<OneState>| {
            if sb.upgrade_arr[index].state.payload != *plan {
                sb.upgrade_arr[index].state.update_payload(plan.clone());
            }
        };
        let mut now = self.evaluate(sb);
        if normals.len() == 2 {
            let mut best = (now, 0, 0);
            for a in 0..candidates[0].len() {
                set_plan(sb, normals[0], &candidates[0][a]);
                for b in 0..candidates[1].len() {
                    if a == 0 && b == 0 {
                        continue;
                    }
                    set_plan(sb, normals[1], &candidates[1][b]);
                    let metric = self.evaluate(sb);
                    if metric > best.0 {
                        best = (metric, a, b);
                    }
                }
            }
            set_plan(sb, normals[0], &candidates[0][best.1]);
            set_plan(sb, normals[1], &candidates[1][best.2]);
            return;
        }
        // every upgrade at one scale together, then each in turn
        let mut chosen = vec![0; normals.len()];
        for picks in &by_scale {
            for (k, &at) in picks.iter().enumerate() {
                set_plan(sb, normals[k], &candidates[k][at]);
            }
            let metric = self.evaluate(sb);
            if metric > now + now.abs() * 1e-12 {
                now = metric;
                chosen.clone_from(picks);
            }
        }
        for (k, &at) in chosen.iter().enumerate() {
            set_plan(sb, normals[k], &candidates[k][at]);
        }
        for _ in 0..LADDER_ROUNDS {
            let mut moved = false;
            for k in 0..normals.len() {
                for c in 0..candidates[k].len() {
                    if c == chosen[k] {
                        continue;
                    }
                    set_plan(sb, normals[k], &candidates[k][c]);
                    let metric = self.evaluate(sb);
                    if metric > now + now.abs() * 1e-12 {
                        now = metric;
                        chosen[k] = c;
                        moved = true;
                    }
                }
                set_plan(sb, normals[k], &candidates[k][chosen[k]]);
            }
            if !moved {
                break;
            }
        }
    }

    /// Shares of a slot moved between two advanced upgrades of the set, kept where better.
    fn adv_transfers(&mut self, sb: &mut StateBundle, set: &[usize], rng: &mut impl Rng) {
        let advs: Vec<usize> = set
            .iter()
            .copied()
            .filter(|&index| !sb.upgrade_arr[index].is_normal_honing)
            .collect();
        if advs.len() < 2 {
            return;
        }
        let mut now = self.evaluate(sb);
        for _ in 0..ADV_TRANSFERS {
            let a = advs[rng.random_range(0..advs.len())];
            let b = advs[rng.random_range(0..advs.len())];
            let slots = sb.upgrade_arr[a].state.payload.len();
            if a == b || slots != sb.upgrade_arr[b].state.payload.len() {
                continue;
            }
            let slot = rng.random_range(0..slots);
            let (was_a, was_b) = (
                sb.upgrade_arr[a].state.payload.clone(),
                sb.upgrade_arr[b].state.payload.clone(),
            );
            let mut best: Option<(f64, Vec<OneState>, Vec<OneState>)> = None;
            for step in [1, 2, 3, 5] {
                let (x, y) = (was_a[slot][0], was_b[slot][0]);
                if x < step || y + step > MAX_ADV_STATE {
                    continue;
                }
                let (mut pa, mut pb) = (was_a.clone(), was_b.clone());
                pa[slot] = smallvec![x - step];
                pb[slot] = smallvec![y + step];
                sb.upgrade_arr[a].state.update_payload(pa.clone());
                sb.upgrade_arr[b].state.update_payload(pb.clone());
                let metric = self.evaluate(sb);
                if metric > best.as_ref().map_or(now, |best| best.0) + now.abs() * 1e-12 {
                    best = Some((metric, pa, pb));
                }
            }
            let (pa, pb) = match best {
                Some((metric, pa, pb)) => {
                    now = metric;
                    (pa, pb)
                }
                None => (was_a, was_b),
            };
            sb.upgrade_arr[a].state.update_payload(pa);
            sb.upgrade_arr[b].state.update_payload(pb);
        }
    }

    fn adv_set(&self, sb: &StateBundle, rng: &mut impl Rng) -> Vec<usize> {
        let weights = (0..sb.upgrade_arr.len())
            .map(|index| (!sb.upgrade_arr[index].is_normal_honing) as u8 as f64)
            .collect();
        let set = pick_some(rng, weights, MAX_FREED);
        if set.len() < 2 { Vec::new() } else { set }
    }

    fn pair_set(&self, sb: &StateBundle, rng: &mut impl Rng) -> Vec<usize> {
        let normals: Vec<usize> = (0..sb.upgrade_arr.len())
            .filter(|&index| sb.upgrade_arr[index].is_normal_honing && has_choice(sb, index))
            .collect();
        if normals.len() < 2 {
            return Vec::new();
        }
        let first = normals[rng.random_range(0..normals.len())];
        let kind = sb.upgrade_arr[first].piece_type_usize;
        let weights: Vec<f64> = normals
            .iter()
            .map(|&index| {
                if index == first {
                    0.0
                } else if sb.upgrade_arr[index].piece_type_usize == kind {
                    1.0
                } else {
                    0.1
                }
            })
            .collect();
        let second = normals[pick(rng, &weights).unwrap()];
        vec![first, second]
    }

    fn piece_type_set(&self, sb: &StateBundle, rng: &mut impl Rng) -> Vec<usize> {
        let types: Vec<usize> = (0..sb.upgrade_arr.len())
            .filter(|&index| has_choice(sb, index))
            .map(|index| sb.upgrade_arr[index].piece_type_usize)
            .collect();
        let Some(&chosen) = types.get(rng.random_range(0..types.len().max(1))) else {
            return Vec::new();
        };
        let weights = (0..sb.upgrade_arr.len())
            .map(|index| {
                (sb.upgrade_arr[index].piece_type_usize == chosen && has_choice(sb, index)) as u8
                    as f64
            })
            .collect();
        pick_some(rng, weights, MAX_FREED)
    }

    fn piece_set(&self, sb: &StateBundle, rng: &mut impl Rng) -> Vec<usize> {
        let pieces: Vec<usize> = (0..sb.upgrade_arr.len())
            .filter(|&index| has_choice(sb, index))
            .map(|index| sb.upgrade_arr[index].piece_index)
            .collect();
        let Some(&chosen) = pieces.get(rng.random_range(0..pieces.len().max(1))) else {
            return Vec::new();
        };
        let weights = (0..sb.upgrade_arr.len())
            .map(|index| {
                (sb.upgrade_arr[index].piece_index == chosen && has_choice(sb, index)) as u8 as f64
            })
            .collect();
        pick_some(rng, weights, MAX_FREED)
    }

    /// the users of a material the plan is short of or long on, the heaviest users likelier
    fn material_set(
        &mut self,
        sb: &mut StateBundle,
        rng: &mut impl Rng,
    ) -> (Option<usize>, Vec<usize>) {
        let prices = self.prices(sb);
        let bands = &sb.prep_output.bands[sb.prep_output.optimizer_plan];
        let rows = sb.prep_output.table.len();
        let used: Vec<f64> = (0..rows)
            .map(|row| {
                sb.upgrade_arr
                    .iter()
                    .map(|upgrade| upgrade.cost_dist[row].mean)
                    .sum()
            })
            .collect();
        let users: Vec<Vec<usize>> = (0..rows)
            .map(|row| {
                (0..sb.upgrade_arr.len())
                    .filter(|&index| has_choice(sb, index) && uses(sb, index, row))
                    .collect()
            })
            .collect();
        let values: Vec<f64> = (0..rows).map(|row| prices[row].abs() * used[row]).collect();
        let mean = values.iter().sum::<f64>() / rows as f64 + 1e-9;
        let weights: Vec<f64> = (0..rows)
            .map(|row| {
                let varies = bands[row]
                    .windows(2)
                    .any(|pair| (pair[0].1 - pair[1].1).abs() > 1e-9);
                if varies && !users[row].is_empty() {
                    1.0 + values[row] / mean
                } else {
                    0.0
                }
            })
            .collect();
        let Some(row) = pick(rng, &weights) else {
            return (None, Vec::new());
        };
        let weights = (0..sb.upgrade_arr.len())
            .map(|index| {
                if users[row].contains(&index) {
                    1.0 + sb.upgrade_arr[index].cost_dist[row].mean
                        / (used[row] / users[row].len() as f64 + 1e-9)
                } else {
                    0.0
                }
            })
            .collect();
        (Some(row), pick_some(rng, weights, MAX_FREED))
    }

    /// a few upgrades of the free tap order where it is reached taken out, with one from
    /// anywhere, and put back one at a time where each does best
    fn order_move(&mut self, sb: &mut StateBundle, rng: &mut impl Rng) -> f64 {
        self.evaluate(sb);
        let probs = sb.special_probs().clone();
        let mut tail = 0.0;
        let mut reach = 0;
        for k in (0..probs.len()).rev() {
            tail += probs[k];
            if tail > 1e-6 {
                reach = k;
                break;
            }
        }
        let len = sb.special_state.len();
        if len < 2 {
            return sb.metric;
        }
        let region = (reach + 2).min(len);
        let width = rng.random_range(1..=3).min(region);
        let start = rng.random_range(0..=region - width);
        let mut order: Vec<usize> = sb.special_state.clone();
        let mut taken: Vec<usize> = order.drain(start..start + width).collect();
        if order.len() > start && rng.random_bool(0.7) {
            let at = rng.random_range(start..order.len());
            taken.push(order.remove(at));
        }
        taken.shuffle(rng);
        let mut moved = Vec::new();
        for index in taken {
            let top = (region + 1).min(order.len());
            let mut places: Vec<usize> = if top <= 8 {
                (0..=top).collect()
            } else {
                let mut places: Vec<usize> =
                    (start.saturating_sub(2)..=(start + width + 2).min(top)).collect();
                places.extend((0..4).map(|_| rng.random_range(0..=top)));
                places
            };
            places.push(order.len());
            places.sort();
            places.dedup();
            let mut best: Option<(f64, Vec<usize>)> = None;
            for at in places {
                let mut trial = order.clone();
                trial.insert(at, index);
                sb.special_state = trial;
                let metric = self.evaluate(sb);
                if best.as_ref().is_none_or(|best| metric > best.0) {
                    best = Some((metric, sb.special_state.clone()));
                }
            }
            order = best.unwrap().1;
            moved.push(index);
        }
        sb.special_state = order;
        moved.retain(|&index| has_choice(sb, index));
        self.sweep(sb, &moved, 1, rng)
    }

    /// The free tap order one place at a time from the front: each place the upgrade that does
    /// best there, the rest as they were, while the place is reached.
    fn build_order(
        &mut self,
        sb: &mut StateBundle,
        from: usize,
        tries: usize,
        rng: &mut impl Rng,
    ) -> Vec<usize> {
        let kept = sb.special_state.clone();
        let mut order = kept.clone();
        let mut now = self.evaluate(sb);
        for place in from..order.len() {
            sb.special_state = order.clone();
            self.evaluate(sb);
            let probs = sb.special_probs();
            let reached: f64 = probs.iter().skip(place).sum();
            if reached < 1e-6 || place >= sb.special_invalid_index.unwrap() + 1 {
                break;
            }
            let mut best = (now, order.clone());
            let mut ats: Vec<usize> = (place + 1..order.len()).collect();
            ats.shuffle(rng);
            ats.truncate(tries);
            for at in ats {
                if !sb.upgrade_arr[order[at]].is_normal_honing {
                    continue;
                }
                let mut trial = order.clone();
                let index = trial.remove(at);
                trial.insert(place, index);
                sb.special_state = trial;
                let metric = self.evaluate(sb);
                if metric > best.0 {
                    best = (metric, sb.special_state.clone());
                }
            }
            now = best.0;
            order = best.1;
        }
        sb.special_state = kept;
        order
    }

    /// the free tap order from a place where it is reached on built again, then the upgrades
    /// whose place moved rebuilt
    fn order_tail(&mut self, sb: &mut StateBundle, rng: &mut impl Rng) -> f64 {
        self.evaluate(sb);
        let probs = sb.special_probs().clone();
        let mut tail = 0.0;
        let mut reach = 0;
        for k in (0..probs.len()).rev() {
            tail += probs[k];
            if tail > 1e-6 {
                reach = k;
                break;
            }
        }
        let was = sb.special_state.clone();
        let from = rng.random_range(0..=reach.min(was.len() - 1));
        sb.special_state = self.build_order(sb, from, BUILD_TRIES, rng);
        let top = (reach + 2).min(was.len());
        let moved: Vec<f64> = (0..sb.upgrade_arr.len())
            .map(|index| {
                let place = |order: &[usize]| order.iter().position(|&x| x == index).unwrap();
                let (a, b) = (place(&was), place(&sb.special_state));
                (a != b && a.min(b) < top && has_choice(sb, index)) as u8 as f64
            })
            .collect();
        let set = pick_some(rng, moved, MAX_FREED);
        self.sweep(sb, &set, 1, rng)
    }

    /// the upgrades with a choice where the free taps reach, and the next two
    fn reached(&mut self, sb: &mut StateBundle) -> Vec<usize> {
        self.evaluate(sb);
        let probs = sb.special_probs();
        let mut tail = 0.0;
        let mut reach = 0;
        for k in (0..probs.len()).rev() {
            tail += probs[k];
            if tail > 1e-6 {
                reach = k;
                break;
            }
        }
        sb.special_state
            .iter()
            .take(reach + 2)
            .copied()
            .filter(|&index| has_choice(sb, index))
            .take(MAX_FREED)
            .collect()
    }

    /// up to `MAX_FREED` upgrades that use any of these rows
    fn users(&self, sb: &StateBundle, rows: &[usize], rng: &mut impl Rng) -> Vec<usize> {
        let weights: Vec<f64> = (0..sb.upgrade_arr.len())
            .map(|index| {
                (has_choice(sb, index) && rows.iter().any(|&row| uses(sb, index, row))) as u8 as f64
            })
            .collect();
        pick_some(rng, weights, MAX_FREED)
    }

    /// one chest stack opened anew, then the upgrades that use what it gives rebuilt
    fn chest_move(
        &mut self,
        sb: &mut StateBundle,
        which: Option<usize>,
        rng: &mut impl Rng,
    ) -> f64 {
        let keys: Vec<String> = sb
            .prep_output
            .chests
            .iter()
            .filter(|(_, chest)| chest.options.len() > 1 && chest.count > 0)
            .map(|(key, _)| key.clone())
            .collect();
        let mut now = self.evaluate(sb);
        if keys.is_empty() {
            return now;
        }
        let key =
            keys[which.unwrap_or_else(|| rng.random_range(0..keys.len())) % keys.len()].clone();
        let chest = sb.prep_output.chests[&key].clone();
        let options = chest.options.len();
        let try_state =
            |search: &mut Self, sb: &mut StateBundle, state: Vec<i64>, now: &mut f64| {
                let mut trial = sb.chest_states.clone();
                trial.insert(key.clone(), state);
                let kept = sb.chest_states.clone();
                sb.set_chest_state(&trial);
                let metric = search.evaluate(sb);
                if metric > *now {
                    *now = metric;
                } else {
                    sb.set_chest_state(&kept);
                }
            };
        // All on one option, with the free tap order built again for it where the stack gives
        // free taps and the upgrades that use what it gives rebuilt; then halving shares moved
        // between two options, the plans as they are.
        let leaps = chest
            .options
            .iter()
            .flat_map(|option| option.keys())
            .any(|label| *label == sb.prep_output.special_label);
        let start = sb.to_essence();
        let mut best = (now, start.clone());
        let rows: Vec<usize> = chest
            .options
            .iter()
            .flat_map(|option| option.keys())
            .filter_map(|label| sb.prep_output.table.get_row(label))
            .collect();
        let users = self.users(sb, &rows, rng);
        for option in 0..options {
            let mut state = vec![0; options];
            state[option] = chest.count;
            let mut trial = sb.chest_states.clone();
            trial.insert(key.clone(), state);
            sb.set_chest_state(&trial);
            if leaps {
                sb.special_state = self.build_order(sb, 0, BUILD_TRIES, rng);
            }
            let mut users = users.clone();
            if leaps {
                for index in self.reached(sb) {
                    if !users.contains(&index) {
                        users.push(index);
                    }
                }
            }
            let metric = self.sweep(sb, &users, 1, rng);
            if metric > best.0 {
                best = (metric, sb.to_essence());
            }
            sb.clone_from_essence(&start, &OrderedFloat(now));
        }
        sb.clone_from_essence(&best.1, &OrderedFloat(best.0));
        now = best.0;
        let pairs: Vec<(usize, usize)> = (0..options)
            .flat_map(|from| (0..options).map(move |to| (from, to)))
            .filter(|(from, to)| from != to)
            .collect();
        let mut step = chest.count / 2;
        while step > 0 {
            let some: Vec<(usize, usize)> = if pairs.len() <= 6 {
                pairs.clone()
            } else {
                (0..6)
                    .map(|_| pairs[rng.random_range(0..pairs.len())])
                    .collect()
            };
            for (from, to) in some {
                let mut state = sb.chest_states[&key].clone();
                if state[from] < step {
                    continue;
                }
                state[from] -= step;
                state[to] += step;
                try_state(self, sb, state, &mut now);
            }
            step /= 2;
        }
        let rows: Vec<usize> = chest
            .options
            .iter()
            .flat_map(|option| option.keys())
            .filter_map(|label| sb.prep_output.table.get_row(label))
            .collect();
        let set = self.users(sb, &rows, rng);
        self.sweep(sb, &set, 2, rng)
    }
}

/// free tap orders by what a free tap saves for what it costs: as they come, and with every
/// piece in level order
fn greedy_orders(search: &mut Search, sb: &mut StateBundle) -> [Vec<usize>; 2] {
    let prices = search.prices(sb);
    let rows = sb.prep_output.table.len();
    let mut ratios: Vec<(f64, usize)> = (0..sb.upgrade_arr.len())
        .map(|index| {
            let upgrade = &sb.upgrade_arr[index];
            if !upgrade.is_normal_honing {
                return (f64::NEG_INFINITY, index);
            }
            let saved: f64 = (0..rows)
                .map(|row| prices[row] * upgrade.cost_dist[row].mean)
                .sum();
            let cost = upgrade.special_cost.max(1) as f64 / upgrade.base_chance.max(1e-9);
            (saved / cost, index)
        })
        .collect();
    ratios.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let order: Vec<usize> = ratios.into_iter().map(|pair| pair.1).collect();
    let mut sorted = order.clone();
    fix_order(sb, &mut sorted);
    [order, sorted]
}

pub fn solve<R: Rng>(
    rng: &mut R,
    mut sb: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    let effort = sb.tuning.lab_effort;
    let count = sb.upgrade_arr.len();
    let mut search = Search { probe, work: 0.0 };
    make_valid(&mut sb);
    if count == 0 {
        search.evaluate(&mut sb);
        return Solved {
            best: sb,
            blob: Vec::new(),
        };
    }
    let budget = effort * (BASE_WORK + WORK_PER_UPGRADE * count as f64);

    // the given, carried and empty plans, the best of them
    let mut candidates: Vec<StateEssence> = vec![sb.to_essence()];
    if let Some(previous) = previous {
        for mut other in previous.carried(&sb) {
            make_valid(&mut other);
            if check::invalid(&other).is_none() {
                candidates.push(other.to_essence());
            }
        }
    }
    let mut empty = sb.to_essence();
    for (slots, upgrade) in empty.state_arr.iter_mut().zip(&sb.upgrade_arr) {
        if upgrade.is_normal_honing {
            slots.iter_mut().for_each(|slot| slot.clear());
        }
    }
    candidates.push(empty);
    let mut best = (f64::NEG_INFINITY, candidates[0].clone());
    for candidate in &candidates {
        sb.clone_from_essence(candidate, &OrderedFloat(0.0));
        let metric = search.evaluate(&mut sb);
        if metric > best.0 {
            best = (metric, sb.to_essence());
        }
    }
    sb.clone_from_essence(&best.1, &OrderedFloat(best.0));
    let mut now = best.0;
    let keep_best = |sb: &StateBundle, metric: f64, best: &mut (f64, StateEssence)| {
        if metric > best.0 {
            *best = (metric, sb.to_essence());
        }
    };

    // a first rebuild of everything: free tap order, every upgrade in turn, chests, the
    // free tap order again
    if sb.affords_a_free_tap() {
        let built = search.build_order(&mut sb, 0, usize::MAX, rng);
        for order in greedy_orders(&mut search, &mut sb)
            .into_iter()
            .chain([built])
        {
            let kept = sb.special_state.clone();
            sb.special_state = order;
            let metric = search.evaluate(&mut sb);
            if metric > now {
                now = metric;
            } else {
                sb.special_state = kept;
            }
        }
    }
    let all: Vec<usize> = (0..count).filter(|&index| has_choice(&sb, index)).collect();
    {
        let mut order = all.clone();
        for _ in 0..SWEEPS {
            if search.work > START_SHARE * budget {
                break;
            }
            order.shuffle(rng);
            let mut moved = false;
            for &index in &order {
                moved |= if sb.upgrade_arr[index].is_normal_honing {
                    search.respond(&mut sb, index)
                } else {
                    search.adv_scan(&mut sb, index)
                };
            }
            if !moved {
                break;
            }
        }
        now = search.evaluate(&mut sb);
    }
    if now < best.0 {
        sb.clone_from_essence(&best.1, &OrderedFloat(best.0));
        now = best.0;
    }
    keep_best(&sb, now, &mut best);
    for which in 0..sb
        .prep_output
        .chests
        .values()
        .filter(|chest| chest.options.len() > 1)
        .count()
    {
        let before = sb.to_essence();
        let metric = search.chest_move(&mut sb, Some(which), rng);
        if metric >= now {
            now = metric;
        } else {
            sb.clone_from_essence(&before, &OrderedFloat(now));
        }
    }
    keep_best(&sb, now, &mut best);
    if sb.affords_a_free_tap() {
        let kept = sb.special_state.clone();
        sb.special_state = search.build_order(&mut sb, 0, usize::MAX, rng);
        let metric = search.sweep(&mut sb, &all, 1, rng);
        if metric > now {
            now = metric;
        } else {
            sb.clone_from_essence(&best.1, &OrderedFloat(best.0));
            sb.special_state = kept;
            now = search.evaluate(&mut sb);
        }
    }
    keep_best(&sb, now, &mut best);

    // destroy and repair
    let mut weights: Vec<f64> = ARMS
        .iter()
        .map(|&(hood, _)| match hood {
            Hood::Order | Hood::OrderTail => 1.0,
            Hood::Chest => sb
                .prep_output
                .chests
                .values()
                .any(|chest| chest.options.len() > 1) as u8 as f64,
            _ => (!all.is_empty()) as u8 as f64,
        })
        .collect();
    let start_work = search.work;
    let mut since_best = 0;
    let order_arms: Vec<usize> = (0..ARMS.len())
        .filter(|&arm| matches!(ARMS[arm].0, Hood::Order | Hood::OrderTail))
        .collect();
    let mut best_work = search.work;
    let (mut moves, mut spent) = (0, 0.0);
    let end = (1.0 - FINAL_SHARE) * budget;
    while search.work < end {
        if search.work - best_work > STALL * budget {
            break;
        }
        for &arm in &order_arms {
            if !sb.affords_a_free_tap() {
                weights[arm] = 0.0;
            } else if weights[arm] == 0.0 {
                weights[arm] = MIN_WEIGHT;
            }
        }
        let Some(arm) = pick(rng, &weights) else {
            break;
        };
        let (hood, start) = ARMS[arm];
        let before = sb.to_essence();
        let work_before = search.work;
        let metric = match hood {
            Hood::Pair | Hood::Adv | Hood::PieceType | Hood::Piece | Hood::Material => {
                let (row, set) = match hood {
                    Hood::Pair => (None, search.pair_set(&sb, rng)),
                    Hood::Adv => (None, search.adv_set(&sb, rng)),
                    Hood::PieceType => (None, search.piece_type_set(&sb, rng)),
                    Hood::Piece => (None, search.piece_set(&sb, rng)),
                    _ => search.material_set(&mut sb, rng),
                };
                if set.is_empty() {
                    weights[arm] = 0.0;
                    continue;
                }
                search.rebuild(&mut sb, &set, start, row, rng)
            }
            Hood::Order => search.order_move(&mut sb, rng),
            Hood::OrderTail => search.order_tail(&mut sb, rng),
            Hood::Chest => search.chest_move(&mut sb, None, rng),
        };
        let progress = ((search.work - start_work) / (end - start_work).max(1.0)).min(1.0);
        let threshold = best.0 - DEVIATION * best.0.abs().max(1.0) * (1.0 - progress);
        let reward = if metric > best.0 + best.0.abs() * 1e-12 {
            if metric > best.0 + best.0.abs().max(1.0) * 1e-6 {
                best_work = search.work;
            }
            best = (metric, sb.to_essence());
            since_best = 0;
            4.0
        } else if metric > now + now.abs() * 1e-12 {
            2.0
        } else if metric >= threshold && metric < now - now.abs() * 1e-12 {
            0.3
        } else {
            0.0
        };
        moves += 1;
        spent += search.work - work_before;
        let cost = (search.work - work_before) / (spent / moves as f64);
        since_best += 1;
        if metric >= threshold {
            now = metric;
        } else {
            sb.clone_from_essence(&before, &OrderedFloat(now));
        }
        if since_best > RESTART_AFTER && now < best.0 {
            sb.clone_from_essence(&best.1, &OrderedFloat(best.0));
            now = best.0;
            since_best = 0;
        }
        weights[arm] =
            ((1.0 - REACTION) * weights[arm] + REACTION * reward / cost.max(0.1)).max(MIN_WEIGHT);
    }

    // a last pass over every upgrade from the best
    sb.clone_from_essence(&best.1, &OrderedFloat(best.0));
    if search.work < budget && !all.is_empty() {
        let metric = search.sweep(&mut sb, &all, 1, rng);
        keep_best(&sb, metric, &mut best);
    }
    sb.clone_from_essence(&best.1, &OrderedFloat(best.0));
    search.evaluate(&mut sb);
    debug_assert_eq!(check::invalid(&sb), None);
    Solved {
        best: sb,
        blob: Vec::new(),
    }
}
