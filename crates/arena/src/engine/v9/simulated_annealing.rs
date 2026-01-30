use super::scaler::AdaptiveScaler;
use hf_core::performance::Performance;
use hf_core::saddlepoint_approximation::average::DEBUG_AVERAGE;
#[cfg(target_arch = "wasm32")]
use hf_core::send_progress::send_progress;
use hf_core::state_bundle::StateBundle;
use hf_core::state_bundle::StateEssence;

use ordered_float::OrderedFloat;

use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;

use priority_queue::DoublePriorityQueue;
use rand::Rng;

use super::core::BATCH_SIZE;
use super::core::MAX_BEST_SIZE;
use super::core::MAX_ITERS;
use super::one_batch::SolverStateBundle;
// #[cfg(not(target_arch = "wasm32"))]
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

pub fn solve<R: Rng>(
    rng: &mut R,
    metric_type: i64,
    mut state_bundle: StateBundle,
    overall_performance: &mut hf_core::performance::Performance,
) -> StateBundle {
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

    state_bundle.metric = state_bundle.metric_router(metric_type, overall_performance);

    let mut eqv_wall_time_iters: i64 = 0;
    let mut last_total_count: i64 = 0;
    let scaler = AdaptiveScaler::new(state_bundle.metric.abs(), 50);

    let mut overall_best_n_states: DoublePriorityQueue<StateEssence, OrderedFloat<f64>> =
        DoublePriorityQueue::new();
    overall_best_n_states.push(state_bundle.to_essence(), OrderedFloat(state_bundle.metric));

    let actual_thread_num: i64 = 16;
    // if state_bundle.num_threads == 0 {
    //     16
    // } else {
    //     state_bundle.num_threads
    // };
    let mut solver_arr: Vec<SolverStateBundle> = Vec::with_capacity(actual_thread_num as usize);
    for _ in 0..actual_thread_num {
        let solver_state_bundle: SolverStateBundle = SolverStateBundle::initialize(
            &state_bundle,
            scaler.clone(),
            max_state_len,
            Performance::new(),
            rng.next_u64(),
            metric_type,
            &overall_best_n_states,
        );
        solver_arr.push(solver_state_bundle);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        overall_performance.best_history.push((
            timer.elapsed_sec(),
            eqv_wall_time_iters * actual_thread_num,
            f64::from(*overall_best_n_states.peek_max().unwrap().1),
        ));
    }
    // vec![state_bundle.clone(); state_bundle.num_threads];
    while eqv_wall_time_iters < MAX_ITERS {
        solver_arr
            .par_iter_mut()
            .for_each(|x| x.one_batch(BATCH_SIZE));

        let mut new_scale = 0.0;
        for solver in solver_arr.iter_mut() {
            overall_performance.aggregate_counts(&solver.performance);
            solver.performance = Performance::new();
            for (state, metric) in solver.best_n_states.drain() {
                my_push(&mut overall_best_n_states, state, metric);
            }
            new_scale += solver.scaler.current_scale;
        }
        new_scale /= actual_thread_num as f64;
        for solver in solver_arr.iter_mut() {
            solver.best_n_states = overall_best_n_states.clone();
            solver.scaler.current_scale = new_scale;
        }

        if eqv_wall_time_iters * actual_thread_num - last_total_count * actual_thread_num >= 1000 {
            last_total_count = eqv_wall_time_iters;
            #[cfg(not(target_arch = "wasm32"))]
            {
                overall_performance.best_history.push((
                    timer.elapsed_sec(),
                    eqv_wall_time_iters * actual_thread_num as i64,
                    f64::from(*overall_best_n_states.peek_max().unwrap().1),
                ));
            }

            #[cfg(target_arch = "wasm32")]
            {
                state_bundle.clone_from_essence(
                    overall_best_n_states.peek_max().unwrap().0,
                    overall_best_n_states.peek_max().unwrap().1,
                );
                let mut dummy_performance = Performance::new();
                state_bundle.metric_router(metric_type, &mut dummy_performance);
                send_progress(
                    &state_bundle.clone(),
                    (100.0 * (eqv_wall_time_iters as f64 / MAX_ITERS as f64)).min(100.0),
                )
            }
        }
        eqv_wall_time_iters += BATCH_SIZE;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        overall_performance.best_history.push((
            timer.elapsed_sec(),
            eqv_wall_time_iters * actual_thread_num as i64,
            f64::from(*overall_best_n_states.peek_max().unwrap().1),
        ));
    }
    let best_pair = overall_best_n_states.peek_max().unwrap();
    state_bundle.clone_from_essence(best_pair.0, best_pair.1);
    state_bundle
}
