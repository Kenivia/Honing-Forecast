use super::scaler::AdaptiveScaler;

use crate::performance::Performance;
#[cfg(feature = "wasm")]
use crate::send_progress::send_progress;
use crate::state_bundle::StateBundle;
use crate::state_bundle::StateEssence;

use ordered_float::OrderedFloat;

// use rayon::iter::IntoParallelRefMutIterator;
// use rayon::iter::ParallelIterator;

use priority_queue::DoublePriorityQueue;
use rand::Rng;

use super::constants::*;
use super::one_batch::SolverStateBundle;
#[cfg(not(feature = "wasm"))]
use crate::helpers::Timer;

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
    let probs = state_bundle.compute_leftover_probs();

    let mut weights = Vec::with_capacity(state_bundle.upgrade_arr.len());

    for upgrade in &state_bundle.upgrade_arr {
        let mut magnitude: f64 = 0.01;
        for (index, p) in probs.iter().take(7).enumerate() {
            magnitude += (state_bundle.prep_output.leftover_values[index] * p
                + state_bundle.prep_output.price_arr[index] * (1.0 - p))
                * upgrade.cost_dist[index]
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
pub fn solve<R: Rng>(
    rng: &mut R,

    mut state_bundle: StateBundle,
    overall_performance: &mut Performance,
) -> StateBundle {
    #[cfg(not(feature = "wasm"))]
    let timer = Timer::start();

    // let init_temp: f64 = if DEBUG_AVERAGE { -1.0 } else { 333.0 };
    // let mut temp: f64 = init_temp;

    // Calculate max state length to establish the starting "coarse" resolution
    let max_state_len = state_bundle
        .upgrade_arr
        .iter()
        .map(|u| u.state.len())
        .max()
        .unwrap_or(state_bundle.min_resolution)
        .max(state_bundle.min_resolution);

    state_bundle.metric = state_bundle.metric_router(overall_performance);
    if state_bundle.upgrade_arr.is_empty() {
        return state_bundle;
    }

    let mut eqv_wall_time_iters: i64 = 0;
    let mut last_total_count: i64 = 0;
    let scaler = AdaptiveScaler::new(state_bundle.metric.abs(), 50);

    let mut overall_best_n_states: DoublePriorityQueue<StateEssence, OrderedFloat<f64>> =
        DoublePriorityQueue::new();
    overall_best_n_states.push(state_bundle.to_essence(), OrderedFloat(state_bundle.metric));

    let actual_thread_num: i64 = CONCURRENT_COUNT;
    // if state_bundle.num_threads == 0 {
    //     16
    // } else {
    //     state_bundle.num_threads
    // };
    let upgrade_impacts = compute_upgrade_impact(&mut state_bundle);
    // let mut solver_arr: Vec<SolverStateBundle> = Vec::with_capacity(actual_thread_num as usize);

    // for _ in 0..actual_thread_num {
    let mut solver_bundle: SolverStateBundle = SolverStateBundle::initialize(
        &state_bundle,
        scaler.clone(),
        max_state_len,
        Performance::new(),
        rng.next_u64(),
        &overall_best_n_states,
        &upgrade_impacts,
    );
    // solver_arr.push(solver_state_bundle);
    // }
    #[cfg(not(feature = "wasm"))]
    {
        overall_performance.best_history.push((
            timer.elapsed_sec(),
            eqv_wall_time_iters * actual_thread_num,
            f64::from(*overall_best_n_states.peek_max().unwrap().1),
        ));
    }
    // vec![state_bundle.clone(); state_bundle.num_threads];
    while eqv_wall_time_iters < MAX_ITERS {
        // solver_arr.iter_mut().for_each(|x| x.one_batch(BATCH_SIZE));
        // web_sys::console::log_1(&format!("{:?}", eqv_wall_time_iters).into());
        let mutate_special: bool;
        if solver_bundle.temps_without_improvement as f64
            > (10.0 * solver_bundle.progress()).max(3.0)
        {
            solver_bundle.perform_crossover();
            solver_bundle.temps_without_improvement = 0;
            mutate_special = false;
        } else {
            mutate_special = solver_bundle.neighbour();
        }

        solver_bundle.state_bundle.metric = solver_bundle
            .state_bundle
            .metric_router(&mut solver_bundle.performance);
        // if solver_bundle.state_bundle.metric_type == 1 {
        //     my_dbg!(solver_bundle.state_bundle.metric, mutate_special);
        // }

        // highest_seen = highest_seen.max(state_bundle.metric);
        // lowest_seen = lowest_seen.min(state_bundle.metric);

        if OrderedFloat(solver_bundle.state_bundle.metric)
            > *solver_bundle.best_n_states.peek_max().unwrap().1
        {
            if mutate_special {
                solver_bundle.special_affinity *= SPECIAL_AFFINITY_GROWTH;
                solver_bundle.special_affinity = solver_bundle.special_affinity.min(1.0);
            }
            my_push(
                &mut solver_bundle.best_n_states,
                solver_bundle.state_bundle.to_essence(),
                OrderedFloat(solver_bundle.state_bundle.metric),
            );
            solver_bundle.temps_without_improvement = 0;
            // my_dbg!(
            //     solver_bundle.state_bundle.metric,
            //     &solver_bundle.best_n_states
            // );
        } else if mutate_special {
            solver_bundle.special_affinity *= SPECIAL_AFFINITY_DECAY;
        }
        let delta = (solver_bundle.prev_state.metric - solver_bundle.state_bundle.metric)
            / solver_bundle.scaler.current_scale;
        let is_uphill = delta > 0.0;
        let accepted = if !is_uphill {
            true
        } else {
            let prob = (-delta.abs()).exp();
            solver_bundle.rng.random_bool(prob)
        };
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

        if eqv_wall_time_iters * actual_thread_num - last_total_count * actual_thread_num >= 1000 {
            last_total_count = eqv_wall_time_iters;
            #[cfg(not(feature = "wasm"))]
            {
                overall_performance.best_history.push((
                    timer.elapsed_sec(),
                    eqv_wall_time_iters * actual_thread_num,
                    f64::from(*solver_bundle.best_n_states.peek_max().unwrap().1),
                ));
            }

            #[cfg(feature = "wasm")]
            {
                solver_bundle.state_bundle.clone_from_essence(
                    solver_bundle.best_n_states.peek_max().unwrap().0,
                    solver_bundle.best_n_states.peek_max().unwrap().1,
                );
                let mut dummy_performance = Performance::new();
                solver_bundle
                    .state_bundle
                    .average_gold_metric_with_breakdown(&mut dummy_performance);
                solver_bundle.state_bundle.set_latest_special_probs();
                solver_bundle
                    .state_bundle
                    .metric_router(overall_performance);
                send_progress(
                    &solver_bundle.state_bundle.clone(),
                    (100.0 * (eqv_wall_time_iters as f64 / MAX_ITERS as f64)).min(100.0),
                )
            }
        }
    }

    let best_pair = solver_bundle.best_n_states.peek_max().unwrap();
    solver_bundle
        .state_bundle
        .clone_from_essence(best_pair.0, best_pair.1);
    solver_bundle.state_bundle
}
