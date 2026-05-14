use super::scaler::AdaptiveScaler;

#[cfg(feature = "wasm")]
use crate::js_interface::send_progress;
use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use crate::state_bundle::StateEssence;

use ordered_float::OrderedFloat;

// use rayon::iter::IntoParallelRefMutIterator;
// use rayon::iter::ParallelIterator;

use priority_queue::DoublePriorityQueue;
use rand::Rng;

use super::constants::*;
use super::one_batch::SolverStateBundle;
use crate::timer::Timer;

pub fn my_push(
    queue: &mut DoublePriorityQueue<StateEssence, OrderedFloat<f64>>,
    new_item: StateEssence,
    new_metric: OrderedFloat<f64>,
) {
    if queue.len() < MAX_BEST_SIZE {
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
        for (support_index, support) in upgrade.cost_dist.iter().enumerate().take(7) {
            magnitude += state_bundle.prep_output.optimizer_material_info[support_index]
                .last()
                .unwrap()
                .1
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

fn send_initial_progress(
    #[allow(unused)] timer: &Timer,
    #[allow(unused)] eqv_wall_time_iters: i64,
    solver_bundle: &SolverStateBundle,
    #[allow(unused)] overall_performance: &mut Performance,
) {
    #[cfg(feature = "run_tests")]
    overall_performance.best_history.push((
        timer.elapsed_sec(),
        eqv_wall_time_iters,
        f64::from(*solver_bundle.best_n_states.peek_max().unwrap().1),
    ));

    #[cfg(feature = "wasm")]
    send_progress(Some(&solver_bundle.state_bundle), 0.01);
}

#[cfg(feature = "run_tests")]
fn record_run_test_progress(
    timer: &Timer,
    eqv_wall_time_iters: i64,
    best_metric: f64,
    overall_performance: &mut Performance,
) {
    overall_performance
        .best_history
        .push((timer.elapsed_sec(), eqv_wall_time_iters, best_metric));
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
    let pct = (100.0 * (eqv_wall_time_iters as f64 / MAX_ITERS as f64)).clamp(0.01, 100.0);
    let elapsed = timer.elapsed_sec();

    if elapsed - last_progress_sec < 1.0 {
        return last_progress_sec;
    }

    // Materialise the best known state before shipping it to the frontend.
    solver_bundle.state_bundle.clone_from_essence(
        solver_bundle.best_n_states.peek_max().unwrap().0,
        solver_bundle.best_n_states.peek_max().unwrap().1,
    );
    let mut dummy_performance = Performance::new();
    solver_bundle
        .state_bundle
        .optimizer_average_gold_metric(&mut dummy_performance);
    solver_bundle.state_bundle.set_latest_special_probs();

    send_progress(Some(&solver_bundle.state_bundle), pct);
    elapsed
}
pub fn solve<R: Rng>(
    rng: &mut R,
    mut state_bundle: StateBundle,
    overall_performance: &mut Performance,
) -> StateBundle {
    let timer = Timer::start();

    let max_state_len = state_bundle
        .upgrade_arr
        .iter()
        .map(|u| u.state.len())
        .max()
        .unwrap_or(state_bundle.min_resolution)
        .max(state_bundle.min_resolution);

    state_bundle.metric = state_bundle.metric_router(overall_performance);
    state_bundle.set_latest_special_probs();

    if state_bundle.upgrade_arr.is_empty() {
        return state_bundle;
    }

    let mut eqv_wall_time_iters: i64 = 0;
    let scaler = AdaptiveScaler::new(state_bundle.metric.abs(), 50);

    let upgrade_impacts = compute_upgrade_impact(&mut state_bundle);

    let init_essence = state_bundle.to_essence();
    let init_metric = state_bundle.metric;

    let mut best_n_states: DoublePriorityQueue<StateEssence, OrderedFloat<f64>> =
        DoublePriorityQueue::new();
    my_push(&mut best_n_states, init_essence, OrderedFloat(init_metric));
    let mut solver_bundle: SolverStateBundle = SolverStateBundle::initialize(
        &state_bundle,
        scaler.clone(),
        max_state_len,
        Performance::new(),
        rng.next_u64(),
        &best_n_states,
        &upgrade_impacts,
    );

    send_initial_progress(
        &timer,
        eqv_wall_time_iters,
        &solver_bundle,
        overall_performance,
    );

    #[allow(unused)]
    let mut last_run_test_count: i64 = 0;
    let mut last_progress_sec: f64 = 0.0;

    while eqv_wall_time_iters < MAX_ITERS {
        let mutate_special = if solver_bundle.temps_without_improvement as f64
            > (10.0 * solver_bundle.progress()).max(3.0)
        {
            solver_bundle.perform_crossover();
            solver_bundle.temps_without_improvement = 0;
            false
        } else {
            solver_bundle.neighbour()
        };

        solver_bundle.state_bundle.metric = solver_bundle
            .state_bundle
            .metric_router(&mut solver_bundle.performance);

        let best_metric = f64::from(*solver_bundle.best_n_states.peek_max().unwrap().1);
        if solver_bundle.state_bundle.metric > best_metric {
            if mutate_special {
                solver_bundle.special_affinity =
                    (solver_bundle.special_affinity * SPECIAL_AFFINITY_GROWTH).min(1.0);
            }
            my_push(
                &mut solver_bundle.best_n_states,
                solver_bundle.state_bundle.to_essence(),
                OrderedFloat(solver_bundle.state_bundle.metric),
            );
            solver_bundle.temps_without_improvement = 0;
        } else if mutate_special {
            solver_bundle.special_affinity *= SPECIAL_AFFINITY_DECAY;
        }

        let delta = (solver_bundle.prev_state.metric - solver_bundle.state_bundle.metric)
            / solver_bundle.scaler.current_scale;
        let is_uphill = delta > 0.0;
        let accepted = !is_uphill || solver_bundle.rng.random_bool((-delta.abs()).exp());

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

        #[cfg(feature = "run_tests")]
        if eqv_wall_time_iters - last_run_test_count >= 2000 {
            record_run_test_progress(
                &timer,
                eqv_wall_time_iters,
                best_metric,
                overall_performance,
            );
            last_run_test_count = eqv_wall_time_iters;
        }

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
    solver_bundle.state_bundle
}
