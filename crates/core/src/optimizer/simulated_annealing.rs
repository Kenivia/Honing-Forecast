use super::scaler::AdaptiveScaler;
use super::{Previous, Solved};

#[cfg(feature = "wasm")]
use crate::js_interface::send_progress;
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use crate::state_bundle::StateEssence;
#[cfg(feature = "wasm")]
use crate::timer::Timer;

use ordered_float::OrderedFloat;

use priority_queue::DoublePriorityQueue;
use rand::Rng;

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
    eqv_wall_time_iters: i64,
    solver_bundle: &mut SolverStateBundle,
    timer: &Timer,
    last_progress_sec: f64,
) -> f64 {
    let max_iters = solver_bundle.state_bundle.tuning.max_iters;
    let pct = (100.0 * (eqv_wall_time_iters as f64 / max_iters as f64)).clamp(0.01, 100.0);
    let elapsed = timer.elapsed_sec();

    if elapsed - last_progress_sec < 1.0 {
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
    // placeholder: the chests are opened at random, once
    state_bundle.randomize_chest_state(rng);
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

/// A random valid state some moves away from the given one, for testing the evaluator
/// away from the optimum.
pub fn scramble<R: Rng>(rng: &mut R, mut state_bundle: StateBundle, moves: usize) -> StateBundle {
    if state_bundle.upgrade_arr.is_empty() {
        return state_bundle;
    }
    let mut probe = Probe::default();
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
    _previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    #[cfg(feature = "wasm")]
    let timer = Timer::start();

    if state_bundle.upgrade_arr.is_empty() {
        state_bundle.metric = state_bundle.metric_router(probe);
        state_bundle.set_latest_special_probs();
        return Solved {
            best: state_bundle,
            blob: Vec::new(),
        };
    }

    let mut solver_bundle = start_solver(rng, &mut state_bundle, probe);
    let tuning = state_bundle.tuning.clone();
    let mut eqv_wall_time_iters: i64 = 0;

    probe.at(0);
    probe.put("best", state_bundle.metric);

    #[cfg(feature = "wasm")]
    send_progress(Some(&solver_bundle.state_bundle), 0.01);
    #[cfg(feature = "wasm")]
    let mut last_progress_sec: f64 = 0.0;

    while eqv_wall_time_iters < tuning.max_iters {
        probe.at(eqv_wall_time_iters + 1);
        let restarted = solver_bundle.temps_without_improvement as f64
            > (10.0 * solver_bundle.progress()).max(3.0);
        let mutate_special = if restarted {
            solver_bundle.perform_crossover();
            solver_bundle.temps_without_improvement = 0;
            false
        } else {
            solver_bundle.neighbour()
        };

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
                eqv_wall_time_iters,
                &mut solver_bundle,
                &timer,
                last_progress_sec,
            );
        }
    }

    solver_bundle.state_bundle.clone_from_essence(
        &solver_bundle.best_n_states.peek_max().unwrap().0,
        solver_bundle.best_n_states.peek_max().unwrap().1,
    );

    let blob: Vec<u8> = Vec::new();
    assert!(blob.len() <= tuning.max_blob_bytes);
    Solved {
        best: solver_bundle.state_bundle,
        blob,
    }
}
