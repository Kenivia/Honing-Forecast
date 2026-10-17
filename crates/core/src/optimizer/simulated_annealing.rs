use super::response::{Model, Response};
use super::scaler::AdaptiveScaler;
use super::{Previous, Solved};

#[cfg(feature = "wasm")]
use crate::js_interface::send_progress;
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use crate::state_bundle::StateEssence;
#[cfg(feature = "wasm")]
use crate::timer::Timer;

use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::constants::SPECIAL_TOL;
use crate::state::OneState;
use ahash::AHashSet;
use ordered_float::OrderedFloat;
use smallvec::smallvec;

use priority_queue::DoublePriorityQueue;
use rand::Rng;
use std::collections::BTreeMap;

use super::one_batch::SolverStateBundle;

pub fn my_push(
    queue: &mut DoublePriorityQueue<StateEssence, OrderedFloat<f64>>,
    new_item: StateEssence,
    new_metric: OrderedFloat<f64>,
    max_best_size: usize,
) {
    if queue.len() < max_best_size {
        queue.push(new_item, new_metric);
        return;
    }

    if new_metric > *queue.peek_min().unwrap().1 {
        queue.pop_min();
        queue.push(new_item, new_metric);
    }
}

fn compute_upgrade_impact(state_bundle: &mut StateBundle) -> Vec<f64> {
    let mut weights = Vec::with_capacity(state_bundle.upgrade_arr.len());

    for upgrade in &state_bundle.upgrade_arr {
        let mut magnitude: f64 = 0.01;
        let prep = &state_bundle.prep_output;
        let plan_bands = &prep.bands[prep.optimizer_plan];
        for (support_index, support) in upgrade
            .cost_dist
            .iter()
            .enumerate()
            .filter(|(row, _)| prep.table.is_base(*row))
        {
            magnitude += plan_bands[support_index].last().unwrap().1
                * support
                    .access_collapsed(false)
                    .iter()
                    .map(|(s, p)| s * p)
                    .sum::<f64>(); // essentially avg with 0 budget, idk kinda makes sense to me
        }
        weights.push(magnitude);
    }

    let sum: f64 = weights.iter().sum::<f64>();
    weights.iter_mut().map(|x| *x / sum).collect()
}

/// Sends a wasm progress update if at least 1 second has passed since the last one.
/// Returns the updated `last_progress_sec` (unchanged if no update was sent).
#[cfg(feature = "wasm")]
fn maybe_send_wasm_progress(
    done: f64,
    floor: f64,
    solver_bundle: &mut SolverStateBundle,
    timer: &Timer,
    last_progress_sec: f64,
) -> f64 {
    let pct = (100.0 * done).clamp(0.01, 100.0);
    let elapsed = timer.elapsed_sec();

    // a later chain says nothing until it beats the ones before
    if elapsed - last_progress_sec < 1.0
        || f64::from(*solver_bundle.best_n_states.peek_max().unwrap().1) <= floor
    {
        return last_progress_sec;
    }

    // Materialise the best known state before shipping it to the frontend.
    solver_bundle.state_bundle.clone_from_essence(
        solver_bundle.best_n_states.peek_max().unwrap().0,
        solver_bundle.best_n_states.peek_max().unwrap().1,
    );
    let mut dummy_probe = Probe::default();
    solver_bundle
        .state_bundle
        .optimizer_average_gold_metric(&mut dummy_probe);
    solver_bundle.state_bundle.set_latest_special_probs();

    send_progress(Some(&solver_bundle.state_bundle), pct);
    elapsed
}

fn start_solver<R: Rng>(
    rng: &mut R,
    state_bundle: &mut StateBundle,
    probe: &mut Probe,
) -> SolverStateBundle {
    let clock = probe.clock();
    state_bundle.metric = state_bundle.metric_router(probe);
    probe.evaluated(clock);
    state_bundle.set_latest_special_probs();

    let scaler = AdaptiveScaler::new(state_bundle.metric.abs(), 50);
    let upgrade_impacts = compute_upgrade_impact(state_bundle);

    let mut best_n_states: DoublePriorityQueue<StateEssence, OrderedFloat<f64>> =
        DoublePriorityQueue::new();
    my_push(
        &mut best_n_states,
        state_bundle.to_essence(),
        OrderedFloat(state_bundle.metric),
        state_bundle.tuning.max_best_size,
    );
    SolverStateBundle::initialize(
        state_bundle,
        scaler,
        rng.next_u64(),
        &best_n_states,
        &upgrade_impacts,
    )
}

/// With the upgrades of the best state as they are, reopens chests of each stack as another
/// option for as long as that helps, in steps that halve. The chain goes on from the result.
/// Returns the evaluations it took.
fn chest_polish(solver_bundle: &mut SolverStateBundle, probe: &mut Probe) -> i64 {
    let stacks: Vec<(String, usize)> = solver_bundle
        .state_bundle
        .prep_output
        .chests
        .iter()
        .filter(|(_, chest)| chest.options.len() > 1 && chest.count > 0)
        .map(|(key, chest)| (key.clone(), chest.options.len()))
        .collect();
    // nothing to choose, and the chain is left where it is
    if stacks.is_empty() {
        return 0;
    }
    let (essence, best_metric) = solver_bundle.best_n_states.peek_max().unwrap();
    let state_bundle = &mut solver_bundle.state_bundle;
    state_bundle.clone_from_essence(essence, best_metric);
    let start = state_bundle.metric;
    let mut evals: i64 = 0;
    let mut improved = true;
    while improved {
        improved = false;
        for (key, options) in &stacks {
            for (from, to) in (0..*options).flat_map(|from| (0..*options).map(move |to| (from, to)))
            {
                let mut step = state_bundle.chest_states[key][from];
                while from != to && step > 0 {
                    let moved = step.min(state_bundle.chest_states[key][from]);
                    let state = state_bundle.chest_states.get_mut(key).unwrap();
                    state[from] -= moved;
                    state[to] += moved;
                    state_bundle.apply_chest_state(false);
                    let clock = probe.clock();
                    let metric = state_bundle.metric_router(probe);
                    probe.evaluated(clock);
                    evals += 1;
                    if moved > 0 && metric > state_bundle.metric {
                        state_bundle.metric = metric;
                        improved = true;
                    } else {
                        let state = state_bundle.chest_states.get_mut(key).unwrap();
                        state[from] += moved;
                        state[to] -= moved;
                        state_bundle.apply_chest_state(false);
                        step /= 2;
                    }
                }
            }
        }
    }
    if state_bundle.metric > start {
        my_push(
            &mut solver_bundle.best_n_states,
            state_bundle.to_essence(),
            OrderedFloat(state_bundle.metric),
            state_bundle.tuning.max_best_size,
        );
        probe.put("best", state_bundle.metric);
    }
    solver_bundle
        .prev_state
        .my_clone_from(&solver_bundle.state_bundle);
    evals
}

/// One normal upgrade takes its best plan against the rest, where the metric agrees: the
/// bundle's metric says whether it did. Returns what it took, in evaluations.
fn respond_one(state_bundle: &mut StateBundle, index: usize, probe: &mut Probe) -> f64 {
    let response = Response::of(state_bundle, index, probe);
    let before = state_bundle.upgrade_arr[index].state.payload.clone();
    let now = response.model.canonical(&response.model.indices(&before));
    let plan = response.better(&now, state_bundle.tuning.response_joint).1;
    let work = response.work(state_bundle.tuning.response_steps);
    if now == plan {
        return work;
    }
    state_bundle.upgrade_arr[index]
        .state
        .update_payload(response.model.plan(&plan));
    let metric = state_bundle.metric_router(probe);
    if metric > state_bundle.metric {
        state_bundle.metric = metric;
    } else {
        state_bundle.upgrade_arr[index].state.update_payload(before);
    }
    work + 1.0
}

/// From the best state, each normal upgrade in turn takes its best plan against the rest
/// where the metric agrees, for a few passes or until one moves nothing. The chain goes on
/// from the result. Returns what it took, in evaluations.
fn response_polish(solver_bundle: &mut SolverStateBundle, sweeps: usize, probe: &mut Probe) -> i64 {
    let (essence, best_metric) = solver_bundle.best_n_states.peek_max().unwrap();
    let state_bundle = &mut solver_bundle.state_bundle;
    state_bundle.clone_from_essence(essence, best_metric);
    let start = state_bundle.metric;
    let mut work: f64 = 0.0;
    for _ in 0..sweeps {
        let mut moved = false;
        for index in 0..state_bundle.upgrade_arr.len() {
            if !state_bundle.upgrade_arr[index].is_normal_honing {
                continue;
            }
            let before = state_bundle.metric;
            work += respond_one(state_bundle, index, probe);
            moved |= state_bundle.metric > before;
        }
        if !moved {
            break;
        }
    }
    if state_bundle.metric > start {
        my_push(
            &mut solver_bundle.best_n_states,
            state_bundle.to_essence(),
            OrderedFloat(state_bundle.metric),
            state_bundle.tuning.max_best_size,
        );
        probe.put("best", state_bundle.metric);
    }
    // a move that was turned down leaves its supports behind
    state_bundle.metric = state_bundle.metric_router(probe);
    solver_bundle
        .prev_state
        .my_clone_from(&solver_bundle.state_bundle);
    work.ceil() as i64
}

/// Starts from the free tap order that takes the upgrades by what a free tap saves for what it
/// takes, either whole or one piece's next level at a time, where one beats the order given.
fn greedy_order(state_bundle: &mut StateBundle, probe: &mut Probe) {
    let mut best = (
        state_bundle.metric_router(probe),
        state_bundle.special_state.clone(),
    );
    if !state_bundle.affords_a_free_tap() {
        return;
    }
    let impact = compute_upgrade_impact(state_bundle);
    let worth = |index: usize| -> f64 {
        let this = &state_bundle.upgrade_arr[index];
        if this.is_normal_honing {
            impact[index] * this.base_chance / this.special_cost.max(1) as f64
        } else {
            -1.0
        }
    };
    let mut whole: Vec<usize> = (0..impact.len()).collect();
    whole.sort_by(|&a, &b| worth(b).total_cmp(&worth(a)));
    // the lowest level left of each piece, the best of those first
    let mut left = whole.clone();
    let mut stepped: Vec<usize> = Vec::new();
    while !left.is_empty() {
        let at = (0..left.len())
            .filter(|&at| {
                let this = &state_bundle.upgrade_arr[left[at]];
                !left.iter().any(|&other| {
                    let other = &state_bundle.upgrade_arr[other];
                    other.is_normal_honing == this.is_normal_honing
                        && other.piece_index == this.piece_index
                        && other.upgrade_index < this.upgrade_index
                })
            })
            .max_by(|&a, &b| worth(left[a]).total_cmp(&worth(left[b])))
            .unwrap();
        stepped.push(left.remove(at));
    }
    for order in [whole, stepped] {
        state_bundle.special_state = order;
        let metric = state_bundle.metric_router(probe);
        if metric > best.0 {
            best = (metric, state_bundle.special_state.clone());
        }
    }
    state_bundle.special_state = best.1;
}

/// From the best state, each upgrade in turn goes to each place of the free tap order that the
/// budget reaches, for as long as that helps. The chain goes on from the result. Returns the
/// evaluations it took.
fn order_polish(solver_bundle: &mut SolverStateBundle, probe: &mut Probe) -> i64 {
    let (essence, best_metric) = solver_bundle.best_n_states.peek_max().unwrap();
    let state_bundle = &mut solver_bundle.state_bundle;
    state_bundle.clone_from_essence(essence, best_metric);
    if !state_bundle.affords_a_free_tap() {
        return 0;
    }
    let start = state_bundle.metric;
    let count = state_bundle.special_state.len();
    let normal: Vec<usize> = (0..count)
        .filter(|&index| state_bundle.upgrade_arr[index].is_normal_honing)
        .collect();
    let mut evals: i64 = 0;
    for _ in 0..5 {
        let mut improved = false;
        let mut tried: AHashSet<Vec<usize>> = AHashSet::new();
        for place in 0..count {
            // an upgrade past the last one a free tap can reach is as good as not in the order
            state_bundle.compute_special_probs(false);
            let reach = state_bundle
                .special_probs()
                .iter()
                .rposition(|&chance| chance >= SPECIAL_TOL)
                .unwrap_or(0);
            if place > reach {
                break;
            }
            for &index in &normal {
                let before = state_bundle.special_state.clone();
                state_bundle.special_state.retain(|&this| this != index);
                state_bundle.special_state.insert(place, index);
                state_bundle.clean_special_state();
                if state_bundle.special_state == before
                    || !tried.insert(state_bundle.special_state.clone())
                {
                    state_bundle.special_state = before;
                    continue;
                }
                let clock = probe.clock();
                let metric = state_bundle.metric_router(probe);
                probe.evaluated(clock);
                evals += 1;
                if metric > state_bundle.metric + state_bundle.metric.abs() * 1e-12 {
                    state_bundle.metric = metric;
                    improved = true;
                    tried.clear();
                } else {
                    state_bundle.special_state = before;
                }
            }
        }
        if !improved {
            break;
        }
    }
    if state_bundle.metric > start {
        my_push(
            &mut solver_bundle.best_n_states,
            state_bundle.to_essence(),
            OrderedFloat(state_bundle.metric),
            state_bundle.tuning.max_best_size,
        );
        probe.put("best", state_bundle.metric);
    }
    state_bundle.metric = state_bundle.metric_router(probe);
    solver_bundle
        .prev_state
        .my_clone_from(&solver_bundle.state_bundle);
    evals
}

/// From the best state, each entry of each advanced upgrade's strategy takes the best of its
/// values with the rest as it is, until none moves. The chain goes on from the result. Returns
/// the evaluations it took.
fn adv_polish(solver_bundle: &mut SolverStateBundle, probe: &mut Probe) -> i64 {
    let (essence, best_metric) = solver_bundle.best_n_states.peek_max().unwrap();
    let state_bundle = &mut solver_bundle.state_bundle;
    state_bundle.clone_from_essence(essence, best_metric);
    let start = state_bundle.metric;
    let advanced: Vec<usize> = (0..state_bundle.upgrade_arr.len())
        .filter(|&index| !state_bundle.upgrade_arr[index].is_normal_honing)
        .collect();
    let mut evals: i64 = 0;
    for _ in 0..if advanced.is_empty() { 0 } else { 3 } {
        let before = state_bundle.metric;
        for &index in &advanced {
            for entry in 0..state_bundle.upgrade_arr[index].state.len() {
                let kept = state_bundle.upgrade_arr[index].state[entry][0];
                let mut best = kept;
                for value in (0..=MAX_ADV_STATE).filter(|&value| value != kept) {
                    let state = &mut state_bundle.upgrade_arr[index].state;
                    state[entry][0] = value;
                    state.update_hash();
                    let metric = state_bundle.metric_router(probe);
                    evals += 1;
                    if metric > state_bundle.metric {
                        (state_bundle.metric, best) = (metric, value);
                    }
                }
                let state = &mut state_bundle.upgrade_arr[index].state;
                state[entry][0] = best;
                state.update_hash();
            }
        }
        if state_bundle.metric <= before {
            break;
        }
    }
    if state_bundle.metric > start {
        my_push(
            &mut solver_bundle.best_n_states,
            state_bundle.to_essence(),
            OrderedFloat(state_bundle.metric),
            state_bundle.tuning.max_best_size,
        );
        probe.put("best", state_bundle.metric);
    }
    state_bundle.metric = state_bundle.metric_router(probe);
    solver_bundle
        .prev_state
        .my_clone_from(&solver_bundle.state_bundle);
    evals
}

/// The polishes in turn, from the best state, for as many rounds as any of them moves it.
/// Returns what they took, in evaluations.
fn polish(solver_bundle: &mut SolverStateBundle, probe: &mut Probe) -> i64 {
    let tuning = solver_bundle.state_bundle.tuning.clone();
    let mut cost = 0;
    for _ in 0..tuning.polish_rounds {
        let before = *solver_bundle.best_n_states.peek_max().unwrap().1;
        cost += order_polish(solver_bundle, probe);
        if tuning.response_polish > 0 {
            cost += response_polish(solver_bundle, tuning.response_polish, probe);
        }
        cost += adv_polish(solver_bundle, probe);
        cost += chest_polish(solver_bundle, probe);
        if *solver_bundle.best_n_states.peek_max().unwrap().1 <= before {
            break;
        }
    }
    if tuning.polish_rounds == 0 && tuning.response_polish > 0 {
        cost += response_polish(solver_bundle, tuning.response_polish, probe);
    }
    cost
}

/// A random valid state some moves away from the given one, for testing the evaluator
/// away from the optimum.
pub fn scramble<R: Rng>(rng: &mut R, mut state_bundle: StateBundle, moves: usize) -> StateBundle {
    if state_bundle.upgrade_arr.is_empty() {
        return state_bundle;
    }
    let mut probe = Probe::default();
    state_bundle.randomize_chest_state(rng);
    let mut solver_bundle = start_solver(rng, &mut state_bundle, &mut probe);
    for _ in 0..moves {
        solver_bundle.neighbour();
        // the next move reads the free tap probabilities of this state
        solver_bundle.state_bundle.metric = solver_bundle.state_bundle.metric_router(&mut probe);
    }
    solver_bundle.state_bundle
}

pub fn solve<R: Rng>(
    rng: &mut R,
    mut state_bundle: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    if state_bundle.upgrade_arr.is_empty() {
        state_bundle.metric = state_bundle.metric_router(probe);
        state_bundle.set_latest_special_probs();
        return Solved {
            best: state_bundle,
            blob: Vec::new(),
        };
    }
    let tuning = state_bundle.tuning.clone();
    // A chain needs longer the more upgrades there are, and what is left over is better spent
    // on more chains than on longer ones.
    let chain_iters =
        (tuning.chain_iters_per_upgrade * state_bundle.upgrade_arr.len() as f64) as i64;
    let chains = match tuning.chains {
        0 => (tuning.max_iters
            / chain_iters
                .max(tuning.chain_iters_min)
                .min(tuning.max_iters / 4)
                .max(1))
        .max(1),
        chains => chains as i64,
    };

    // A plan to go on from: the one that came with the payload or the one of the run before,
    // where either beats the empty plan, which is where every other chain starts.
    let given = state_bundle.clone();
    for upgrade in state_bundle.upgrade_arr.iter_mut() {
        let slot: OneState = if upgrade.is_normal_honing {
            smallvec![]
        } else {
            smallvec![0]
        };
        upgrade
            .state
            .update_payload(vec![slot; upgrade.state.len()]);
    }
    state_bundle.special_state = (0..state_bundle.upgrade_arr.len()).collect();
    let mut warm: Option<StateBundle> = None;
    let mut floor = state_bundle.metric_router(probe);
    let carried = previous.map(|previous| {
        let mut other = state_bundle.clone();
        previous.carry(&mut other);
        other
    });
    for mut other in [Some(given), carried]
        .into_iter()
        .flatten()
        .filter(|_| tuning.carry)
    {
        let metric = other.metric_router(probe);
        if metric > floor {
            (floor, warm) = (metric, Some(other));
        }
    }
    if tuning.greedy_order {
        greedy_order(&mut state_bundle, probe);
    }

    // Several shorter chains share the evaluations, each polished, and the best is kept. The
    // first goes on from a carried plan where there is one, the rest start afresh.
    let mut results: Vec<StateBundle> = Vec::new();
    if let Some(warm) = warm.as_mut().filter(|_| tuning.carry_polish) {
        // a carried plan is often a polish away from the best
        warm.tuning.max_iters = 0;
        *warm = anneal(rng, warm, probe, 0, chains, f64::NEG_INFINITY);
        results.push(warm.clone());
    }
    let mut stalled = 0;
    for chain in 0..chains {
        let floor = results
            .iter()
            .map(|result| result.metric)
            .fold(f64::NEG_INFINITY, f64::max);
        let mut start = state_bundle.clone();
        start.tuning.max_iters = tuning.max_iters / chains;
        if let Some(warm) = warm.as_ref().filter(|_| chain == 0) {
            start = warm.clone();
            start.tuning.max_iters =
                (tuning.max_iters as f64 / chains as f64 * tuning.carry_iters) as i64;
        }
        results.push(anneal(rng, &mut start, probe, chain, chains, floor));
        // chains that end on the same state are unlikely to be beaten by another, and so is a
        // carried plan that the last few chains did not better
        let top = results
            .iter()
            .map(|result| result.metric)
            .fold(f64::NEG_INFINITY, f64::max);
        let close = top.abs().max(1.0) * 1e-7;
        let same = results
            .iter()
            .filter(|result| top - result.metric <= close)
            .count();
        stalled = if top > floor + close { 0 } else { stalled + 1 };
        if tuning.agree_exit > 0 && same >= tuning.agree_exit
            || warm.is_some() && tuning.carry_stall > 0 && stalled >= tuning.carry_stall
        {
            break;
        }
    }
    results.sort_by(|a, b| b.metric.total_cmp(&a.metric));
    let mut best = results[0].clone();
    let merged = tuning.merge && merge(&mut best, &results[1..], probe);
    refine(rng, &mut best, merged, probe);
    best.tuning = tuning;
    let blob: Vec<u8> = Vec::new();
    assert!(blob.len() <= best.tuning.max_blob_bytes);
    Solved { best, blob }
}

/// The exchanges, then the polishes over what they or a merge moved, for as long as the
/// exchanges move anything.
fn refine<R: Rng>(rng: &mut R, best: &mut StateBundle, mut moved: bool, probe: &mut Probe) {
    let exchanges = best.tuning.exchange;
    for _ in 0..3 {
        moved |= exchanges && exchange(best, probe);
        if !moved {
            break;
        }
        let before = best.metric;
        best.tuning.max_iters = 0;
        *best = anneal(rng, best, probe, 1, 1, f64::NEG_INFINITY);
        if !exchanges || best.metric <= before {
            break;
        }
        moved = false;
    }
}

/// Takes into the best chain's result what the other chains ended on, one piece at a time: the
/// free tap order, and each upgrade's plan, wherever the metric agrees. Chains often find
/// different parts of the best plan. Returns whether anything was taken.
fn merge(best: &mut StateBundle, others: &[StateBundle], probe: &mut Probe) -> bool {
    best.metric = best.metric_router(probe);
    let start = best.metric;
    for _ in 0..3 {
        let before = best.metric;
        for other in others {
            if other.special_state != best.special_state {
                let kept = std::mem::replace(&mut best.special_state, other.special_state.clone());
                let metric = best.metric_router(probe);
                if metric > best.metric {
                    best.metric = metric;
                } else {
                    best.special_state = kept;
                }
            }
            // the upgrades of a piece type share their juices and books, so their plans go together
            let count = best.upgrade_arr.len();
            let kinds: Vec<usize> = best
                .upgrade_arr
                .iter()
                .map(|upgrade| upgrade.piece_type_usize)
                .collect();
            let blocks = (0..3).map(|kind| {
                (0..count)
                    .filter(|&index| kinds[index] == kind)
                    .collect::<Vec<usize>>()
            });
            for block in blocks.chain((0..count).map(|index| vec![index])) {
                let differs = |best: &StateBundle, index: usize| {
                    other.upgrade_arr[index].state.hash != best.upgrade_arr[index].state.hash
                };
                if !block.iter().any(|&index| differs(best, index)) {
                    continue;
                }
                let kept: Vec<Vec<OneState>> = block
                    .iter()
                    .map(|&index| best.upgrade_arr[index].state.payload.clone())
                    .collect();
                for &index in &block {
                    if differs(best, index) {
                        best.upgrade_arr[index]
                            .state
                            .update_payload(other.upgrade_arr[index].state.payload.clone());
                    }
                }
                let metric = best.metric_router(probe);
                if metric > best.metric {
                    best.metric = metric;
                    continue;
                }
                for (&index, plan) in block.iter().zip(kept) {
                    if best.upgrade_arr[index].state.payload != plan {
                        best.upgrade_arr[index].state.update_payload(plan);
                    }
                }
            }
        }
        if best.metric <= before {
            break;
        }
    }
    best.metric = best.metric_router(probe);
    best.metric > start
}

/// Moves uses of a juice or a book from one upgrade to another that takes the same, in amounts
/// that double, wherever the metric agrees: the last taps that take it of the one, the first
/// that do not of the other. Each upgrade having its best plan against the rest leaves these.
/// Returns whether anything moved.
fn exchange(best: &mut StateBundle, probe: &mut Probe) -> bool {
    // a streak length would have to be kept on both sides
    if best.block() > 1 {
        return false;
    }
    best.metric = best.metric_router(probe);
    let start = best.metric;
    let normal: Vec<usize> = (0..best.upgrade_arr.len())
        .filter(|&index| best.upgrade_arr[index].is_normal_honing)
        .collect();
    let models: Vec<Model> = normal
        .iter()
        .map(|&index| Model::of(&best.upgrade_arr[index], &best.prep_output.juice_info, 1))
        .collect();
    // by id: the upgrades that can take it, and what taking it adds to an option's index there
    let mut users: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
    for (at, model) in models.iter().enumerate() {
        for (pool, ids) in model.pools.iter().enumerate() {
            for (which, &id) in ids.iter().enumerate() {
                users.entry(id).or_default().push((
                    at,
                    model.strides[pool],
                    (which + 1) * model.strides[pool],
                ));
            }
        }
    }
    let width = |model: &Model, stride: usize| {
        model.pools[model
            .strides
            .iter()
            .position(|&this| this == stride)
            .unwrap()]
        .len()
            + 1
    };
    for _ in 0..2 {
        let before = best.metric;
        for users in users.values().filter(|users| users.len() > 1) {
            for &(from, from_stride, from_option) in users {
                for &(to, to_stride, to_option) in users.iter().filter(|user| user.0 != from) {
                    let (a, b) = (&models[from], &models[to]);
                    let (wide_a, wide_b) = (width(a, from_stride), width(b, to_stride));
                    for amount in (0..7).map(|power| 1usize << power) {
                        let kept = (
                            best.upgrade_arr[normal[from]].state.payload.clone(),
                            best.upgrade_arr[normal[to]].state.payload.clone(),
                        );
                        let mut plan_a = a.canonical(&a.indices(&kept.0));
                        let mut plan_b = b.canonical(&b.indices(&kept.1));
                        let given: Vec<usize> = (0..a.len)
                            .rev()
                            .filter(|&tap| {
                                plan_a[tap] / from_stride % wide_a * from_stride == from_option
                            })
                            .take(amount)
                            .collect();
                        let (pity, by_chance) = b.pity(&plan_b);
                        let taken: Vec<usize> = (0..pity - !by_chance as usize)
                            .filter(|&tap| plan_b[tap] / to_stride % wide_b == 0)
                            .take(amount)
                            .collect();
                        if given.len() < amount || taken.len() < amount {
                            break;
                        }
                        given.iter().for_each(|&tap| plan_a[tap] -= from_option);
                        taken.iter().for_each(|&tap| plan_b[tap] += to_option);
                        best.upgrade_arr[normal[from]]
                            .state
                            .update_payload(a.plan(&plan_a));
                        best.upgrade_arr[normal[to]]
                            .state
                            .update_payload(b.plan(&plan_b));
                        let metric = best.metric_router(probe);
                        if metric > best.metric + best.metric.abs() * 1e-12 {
                            best.metric = metric;
                        } else {
                            best.upgrade_arr[normal[from]].state.update_payload(kept.0);
                            best.upgrade_arr[normal[to]].state.update_payload(kept.1);
                        }
                    }
                }
            }
        }
        if best.metric <= before {
            break;
        }
    }
    best.metric = best.metric_router(probe);
    best.metric > start
}

/// One chain, polished: the best state it saw. `chain` of `chains` and `floor`, the best of the
/// chains before, are for reporting progress.
fn anneal<R: Rng>(
    rng: &mut R,
    state_bundle: &mut StateBundle,
    probe: &mut Probe,
    chain: i64,
    chains: i64,
    floor: f64,
) -> StateBundle {
    #[cfg(feature = "wasm")]
    let timer = Timer::start();
    let _ = floor;
    let mut solver_bundle = start_solver(rng, state_bundle, probe);
    let tuning = state_bundle.tuning.clone();
    let mut eqv_wall_time_iters: i64 = 0;
    let offset = chain * tuning.max_iters;

    probe.at(offset);
    probe.put("best", state_bundle.metric);

    #[cfg(feature = "wasm")]
    if chain == 0 {
        send_progress(Some(&solver_bundle.state_bundle), 0.01);
    }
    #[cfg(feature = "wasm")]
    let mut last_progress_sec: f64 = 0.0;

    let mut next_polish: i64 = 0;
    let mut next_response: i64 = tuning.response_every;
    while eqv_wall_time_iters < tuning.max_iters {
        probe.at(offset + eqv_wall_time_iters + 1);
        if tuning.chest_polish_every > 0 && eqv_wall_time_iters >= next_polish {
            let evals = chest_polish(&mut solver_bundle, probe);
            solver_bundle.count += evals;
            eqv_wall_time_iters += evals;
            next_polish = eqv_wall_time_iters + tuning.chest_polish_every;
        }
        if tuning.response_every > 0 && eqv_wall_time_iters >= next_response {
            let cost = response_polish(&mut solver_bundle, 1, probe);
            solver_bundle.count += cost;
            eqv_wall_time_iters += cost;
            next_response = eqv_wall_time_iters + tuning.response_every;
        }
        let restarted = solver_bundle.temps_without_improvement as f64
            > (10.0 * solver_bundle.progress()).max(3.0);
        let mutate_special = if restarted {
            solver_bundle.perform_crossover();
            solver_bundle.temps_without_improvement = 0;
            false
        } else {
            solver_bundle.neighbour()
        };

        let extra = std::mem::take(&mut solver_bundle.extra_cost);
        solver_bundle.count += extra;
        eqv_wall_time_iters += extra;

        let clock = probe.clock();
        solver_bundle.state_bundle.metric = solver_bundle.state_bundle.metric_router(probe);
        probe.evaluated(clock);

        let best_metric = f64::from(*solver_bundle.best_n_states.peek_max().unwrap().1);
        if solver_bundle.state_bundle.metric > best_metric {
            if mutate_special {
                solver_bundle.special_affinity =
                    (solver_bundle.special_affinity * tuning.special_affinity_growth).min(1.0);
            }
            my_push(
                &mut solver_bundle.best_n_states,
                solver_bundle.state_bundle.to_essence(),
                OrderedFloat(solver_bundle.state_bundle.metric),
                tuning.max_best_size,
            );
            solver_bundle.temps_without_improvement = 0;
            probe.put("best", solver_bundle.state_bundle.metric);
        } else if mutate_special {
            solver_bundle.special_affinity *= tuning.special_affinity_decay;
        }

        let delta = (solver_bundle.prev_state.metric - solver_bundle.state_bundle.metric)
            / solver_bundle.scaler.current_scale;
        let is_uphill = delta > 0.0;
        let accepted = !is_uphill || solver_bundle.rng.random_bool((-delta.abs()).exp());

        if probe.detail() {
            probe.put("metric", solver_bundle.state_bundle.metric);
            probe.put("accepted", accepted as i64 as f64);
            probe.put("scale", solver_bundle.scaler.current_scale);
            probe.put("special_affinity", solver_bundle.special_affinity);
            if restarted {
                probe.put("restart", 1.0);
            }
            if mutate_special {
                probe.put("special_move", 1.0);
            }
        }

        if accepted {
            solver_bundle
                .prev_state
                .my_clone_from(&solver_bundle.state_bundle);
        } else {
            solver_bundle
                .state_bundle
                .my_clone_from(&solver_bundle.prev_state);
        }
        solver_bundle
            .scaler
            .update_stats(is_uphill, accepted, solver_bundle.lam_rate());
        solver_bundle.count += 1;
        solver_bundle.temps_without_improvement += 1;
        eqv_wall_time_iters += 1;

        #[cfg(feature = "wasm")]
        {
            last_progress_sec = maybe_send_wasm_progress(
                (chain as f64 + eqv_wall_time_iters as f64 / tuning.max_iters as f64)
                    / chains as f64,
                floor,
                &mut solver_bundle,
                &timer,
                last_progress_sec,
            );
        }
    }
    let _ = chains;

    polish(&mut solver_bundle, probe);
    solver_bundle.state_bundle.clone_from_essence(
        &solver_bundle.best_n_states.peek_max().unwrap().0,
        solver_bundle.best_n_states.peek_max().unwrap().1,
    );
    solver_bundle.state_bundle
}
