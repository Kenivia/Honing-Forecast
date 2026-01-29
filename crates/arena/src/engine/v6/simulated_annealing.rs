use super::core::{neighbour, new_temp};
use super::scaler::AdaptiveScaler;
use hf_core::saddlepoint_approximation::average::DEBUG_AVERAGE;
#[cfg(target_arch = "wasm32")]
use hf_core::send_progress::send_progress;
use hf_core::state_bundle::StateBundle;
use rand::Rng;

#[cfg(not(target_arch = "wasm32"))]
use crate::timer::Timer;

struct SolverStateBundle {
    state_bundle: StateBundle,
    scaler: AdaptiveScaler,
    init_temp: f64,
    temp: f64,
}
pub fn solve<R: Rng>(
    rng: &mut R,
    metric_type: i64,
    mut state_bundle: StateBundle,
    performance: &mut hf_core::performance::Performance,
) -> StateBundle {
    let timer = Timer::start();

    let init_temp: f64 = if DEBUG_AVERAGE { -1.0 } else { 333.0 };
    let mut temp: f64 = init_temp;

    // Calculate max state length to establish the starting "coarse" resolution
    let max_len = state_bundle
        .upgrade_arr
        .iter()
        .map(|u| u.state.len())
        .max()
        .unwrap_or(state_bundle.min_resolution)
        .max(state_bundle.min_resolution);

    let temp_schedule_start = 333.0_f64;
    let temp_schedule_cutoff = 33.0_f64; // Below this temp, resolution is pinned to min

    if DEBUG_AVERAGE {
        neighbour(&mut state_bundle, temp, init_temp, max_len, rng);
    }
    state_bundle.metric = state_bundle.metric_router(metric_type, performance);
    let mut prev_state: StateBundle = state_bundle.clone();

    let iterations_per_temp = 69;
    let mut count: i64 = 0;
    let alpha: f64 = 0.99;
    // let mut highest_seen: f64 = MIN;
    // let mut lowest_seen: f64 = MAX;
    let mut best_state_so_far: StateBundle = state_bundle.clone();
    let mut temps_without_improvement = 1;
    let mut total_count: i64 = 0;
    let mut scaler = AdaptiveScaler::new(state_bundle.metric.abs(), 50);

    while temp >= 0.0 {
        let current_resolution = if temp > temp_schedule_cutoff {
            // Logarithmic interpolation from Max Len -> Min Res
            let log_curr = temp.ln();
            let log_start = temp_schedule_start.ln();
            let log_end = temp_schedule_cutoff.ln();

            // 0.0 at cutoff, 1.0 at start
            let ratio = ((log_curr - log_end) / (log_start - log_end)).clamp(0.0, 1.0);

            ((state_bundle.min_resolution as f64
                + (max_len as f64 - state_bundle.min_resolution as f64) * ratio)
                / state_bundle.min_resolution as f64)
                .floor() as usize
                * state_bundle.min_resolution
        } else {
            state_bundle.min_resolution
        };

        neighbour(&mut state_bundle, temp, init_temp, current_resolution, rng);
        state_bundle.metric = state_bundle.metric_router(metric_type, performance);

        // highest_seen = highest_seen.max(state_bundle.metric);
        // lowest_seen = lowest_seen.min(state_bundle.metric);

        if state_bundle.metric > best_state_so_far.metric {
            best_state_so_far.my_clone_from(&state_bundle);
            temps_without_improvement = 0;
        }

        let delta = (prev_state.metric - state_bundle.metric) / scaler.current_scale;
        let is_uphill = delta < 0.0; // Assuming maximization? Adjust if minimization.
        let accepted = if !is_uphill {
            true
        } else {
            let prob = (-delta.abs()).exp();
            rng.random_bool(prob)
        };
        if accepted {
            prev_state.my_clone_from(&state_bundle);
        } else {
            state_bundle.my_clone_from(&prev_state);
        }
        scaler.update_stats(is_uphill, accepted, (temp / init_temp) * 0.69);
        count += 1;
        if count > iterations_per_temp {
            count = 0;
            if temps_without_improvement as f64 > (1.0 * temp).max(3.0) {
                state_bundle.my_clone_from(&best_state_so_far);
                temps_without_improvement = 0;
            }
            temps_without_improvement += 1;
            temp = new_temp(temp, alpha);
        }
        if total_count % 1000 == 0 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                performance.best_history.push((
                    timer.elapsed_sec(),
                    total_count,
                    best_state_so_far.metric,
                ));
            }

            #[cfg(target_arch = "wasm32")]
            {
                send_progress(
                    &best_state_so_far.clone(),
                    (100.0
                        * (total_count as f64
                            / (iterations_per_temp as f64 * (0.05_f64 / init_temp).ln()
                                / 0.99_f64.ln())))
                    .min(100.0),
                )
            }
        }
        total_count += 1;
    }

    best_state_so_far
}
