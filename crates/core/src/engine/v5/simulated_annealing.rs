use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
#[cfg(target_arch = "wasm32")]
use crate::send_progress::send_progress;
use crate::state_bundle::StateBundle;
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IteratorRandom;
// use std::f64::{MAX, MIN};

// #[cfg(not(target_arch = "wasm32"))]
use crate::helpers::Timer;

fn acceptance<R: Rng>(
    new: f64,
    old: f64,
    temperature: f64,
    biggest_gap_seen: f64,
    rng: &mut R,
) -> bool {
    let delta: f64 = (old - new) / biggest_gap_seen.max(1.0);
    let prob: f64 = if delta <= 0.0 {
        1.0
    } else if temperature <= 0.0 {
        0.0
    } else {
        // canonical Boltzmann acceptance with k = 1
        let p = (-delta / (temperature)).exp();
        // numeric safety clamp
        if p.is_finite() {
            p.min(1.0).max(0.0)
        } else {
            0.0
        }
    };

    rng.random_bool(prob)
}

pub struct AdaptiveScaler {
    /// The current scaling factor (starts at a reasonable guess, e.g., 1.0 or 100.0)
    pub current_scale: f64,
    /// How many uphill (bad) moves we've seen in this batch
    uphill_count: usize,
    /// How many of those uphill moves we actually accepted
    accepted_count: usize,
    /// How many moves to observe before updating the scale (e.g., 100)
    batch_size: usize,
    /// Damping factor (0.0 to 1.0). Lower = smoother, Higher = more responsive.
    /// Recommended: 0.1 to 0.2
    learning_rate: f64,
}

impl AdaptiveScaler {
    pub fn new(initial_guess: f64, batch_size: usize) -> Self {
        Self {
            current_scale: initial_guess,
            uphill_count: 0,
            accepted_count: 0,
            batch_size,
            learning_rate: 0.1,
        }
    }

    /// Call this AFTER you have decided whether to accept/reject.
    /// Returns: true if the scale was updated this step.
    pub fn update_stats(
        &mut self,
        is_uphill_move: bool,
        is_accepted: bool,
        progress: f64, // 0.0 (start) to 1.0 (end)
    ) {
        // We only care about "bad" (uphill) moves for tuning Boltzmann
        if !is_uphill_move {
            return;
            // return false;
        }

        self.uphill_count += 1;
        if is_accepted {
            self.accepted_count += 1;
        }

        // Only update scale once we have a full batch
        if self.uphill_count >= self.batch_size {
            self.recalibrate(progress);
            // Reset counters
            self.uphill_count = 0;
            self.accepted_count = 0;

            // return true;
        }
        // false
    }

    fn recalibrate(&mut self, target_rate: f64) {
        // 1. Calculate Measured Rate
        // Clamp to avoid log(0) or log(1) math errors
        let measured_rate =
            (self.accepted_count as f64 / self.uphill_count as f64).clamp(0.001, 0.999);

        // 2. Calculate Target Rate
        // Exponential decay: Start at 80% acceptance, end at 0.1%
        // let start_rate: f64 = 0.80;
        // let end_rate: f64 = 0.001;
        // Interpolate target based on progress
        // This is a simple log-linear decay for the target rate
        // let decay = (end_rate / start_rate).ln();
        // let target_rate = start_rate * (decay * progress).exp();

        // 3. The Robust Update Formula
        // S_new = S_old * (ln(measured) / ln(target))^learning_rate
        let ratio: f64 = measured_rate.ln() / target_rate.ln();

        // Safety clamp on the update ratio to prevent explosions
        // (e.g. don't let it grow/shrink by more than 2x in a single update)
        // let safe_ratio = ratio.clamp(0.5, 2.0);

        // Apply damped update
        let adjustment = ratio.powf(self.learning_rate);

        self.current_scale *= adjustment;
    }
}
pub fn my_pmf(max_len: usize, expected: f64, ratio: f64) -> Vec<f64> {
    // If NaN/inf, return a safe default: all mass at 0
    let mut v = vec![0.0; max_len + 1];
    for (index, v) in v.iter_mut().enumerate() {
        *v = ratio.powf(index as f64 - expected);
    }
    v
}

fn my_weighted_rand<R: Rng>(
    length: usize,
    temp: f64,
    init_temp: f64,
    offset: usize,
    ratio: f64,
    upgrade_len: usize,
    rng: &mut R,
) -> usize {
    let res = WeightedIndex::new(my_pmf(
        (length - 1).max(1),
        (temp / init_temp * length as f64 - 1.0)
            .min(length as f64)
            .max(1.0 / upgrade_len as f64),
        ratio,
    ));
    if res.is_err() {
        dbg!(length, temp, init_temp, offset, ratio);
    }
    res.unwrap().sample(rng) + offset
}

/// New signature: accepts `resolution` to control block size
fn neighbour<R: Rng>(
    state_bundle: &mut StateBundle,
    temp: f64,
    init_temp: f64,
    resolution: usize,
    rng: &mut R,
) {
    let u_len = state_bundle.upgrade_arr.len();
    // PART 1: Special State (Unchanged by resolution as requested)
    if state_bundle.special_state.len() > 1 {
        let want_to_swap: usize = my_weighted_rand(
            state_bundle.special_state.len(),
            temp / 2.0,
            init_temp,
            0,
            0.5,
            u_len,
            rng,
        );
        for _ in 0..want_to_swap {
            let want_to_swap_indices: Vec<usize> =
                (0..state_bundle.special_state.len()).choose_multiple(rng, 2);
            let first = want_to_swap_indices[0];
            let second = want_to_swap_indices[1];

            if !(state_bundle.upgrade_arr[first].succeeded
                || state_bundle.upgrade_arr[second].succeeded)
            {
                let temp = state_bundle.special_state[first];
                state_bundle.special_state[first] = state_bundle.special_state[second];
                state_bundle.special_state[second] = temp;
            }
        }
    }

    // PART 2: Upgrade Arrays (Affected by resolution)
    let eff_resolution = resolution.max(1);

    for upgrade in state_bundle.upgrade_arr.iter_mut() {
        if upgrade.succeeded {
            continue;
        }
        let choice_len: usize = upgrade.books_avail as usize;
        let state = &mut upgrade.state;
        let state_len = state.len();

        // Calculate number of blocks based on resolution
        // e.g., Len 25, Res 10 -> 3 Blocks (0..10, 10..20, 20..25)
        let num_blocks = (state_len + eff_resolution - 1) / eff_resolution;

        // Adjust weighted rand to pick number of BLOCKS to flip, not individual items
        let want_to_flip: usize = my_weighted_rand(
            num_blocks,
            temp,
            init_temp,
            0,
            0.8 / eff_resolution as f64,
            u_len,
            rng,
        );

        let want_to_book: usize = my_weighted_rand(
            num_blocks,
            temp,
            init_temp,
            1,
            0.8 / eff_resolution as f64,
            u_len,
            rng,
        );

        let flip_target = rng.random_bool(0.5);

        // Choose which blocks to target
        let mut want_to_flip_blocks: Vec<usize> =
            (0..num_blocks).choose_multiple(rng, want_to_flip);
        want_to_flip_blocks.sort();

        // Choose which blocks to book
        let ids: &Vec<usize> = &state_bundle.prep_output.juice_info.ids[upgrade.upgrade_index];
        let book_target_index: usize = rng.random_range(0..=choice_len);
        let book_target_id: usize = ids[book_target_index];

        let mut want_to_book_blocks: Vec<usize> =
            (0..num_blocks).choose_multiple(rng, want_to_book);
        want_to_book_blocks.sort();

        // let mut artisan: f64 = 0.0;
        // let mut count: f64 = 0.1;
        // let base_chance = upgrade.base_chance;
        // let artisan_rate = upgrade.artisan_rate;
        // let juice_chances = &state_bundle.prep_output.juice_info.chances_id;

        // Pointers for sorted block lists
        let mut flip_ptr = 0;
        let mut book_ptr = 0;

        for (s_index, (juice, id)) in state.iter_mut().enumerate() {
            // Determine which block this index belongs to
            let current_block = s_index / eff_resolution;

            // --- Mutate Juice (based on Block) ---
            // Advance pointer if we passed the stored block index
            while flip_ptr < want_to_flip_blocks.len()
                && want_to_flip_blocks[flip_ptr] < current_block
            {
                flip_ptr += 1;
            }
            // If current block is selected, apply target
            if flip_ptr < want_to_flip_blocks.len()
                && want_to_flip_blocks[flip_ptr] == current_block
                && s_index >= upgrade.alr_failed
            // Respect alr_failed boundary
            {
                *juice = flip_target;
            }

            // --- Mutate Book ID (based on Block) ---
            while book_ptr < want_to_book_blocks.len()
                && want_to_book_blocks[book_ptr] < current_block
            {
                book_ptr += 1;
            }
            if book_ptr < want_to_book_blocks.len()
                && want_to_book_blocks[book_ptr] == current_block
                && s_index >= upgrade.alr_failed
            {
                *id = book_target_id;
            }

            // asrtisan before
            // artisan += (46.51_f64 / 100.0)
            //     * artisan_rate
            //     * (base_chance * (1.0 + count)
            //         + if *juice {
            //             juice_chances[0][upgrade.upgrade_index]
            //         } else {
            //             0.0
            //         }
            //         + if *id > 0 {
            //             juice_chances[*id][upgrade.upgrade_index]
            //         } else {
            //             0.0
            //         });
            // if count < 1.0 {
            //     count += 0.1;
            // }

            // artisan after
        }
        state.update_hash();
    }
}

fn new_temp(temp: f64, alpha: f64) -> f64 {
    if temp == 0.0 {
        return -6.9;
    }
    let new: f64 = temp * alpha;
    if new < 0.05 {
        return 0.0;
    }
    return new;
}

pub fn solve<R: Rng>(
    rng: &mut R,
    mut state_bundle: StateBundle,
    performance: &mut crate::performance::Performance,
) -> StateBundle {
    // #[cfg(not(target_arch = "wasm32"))]
    // {
    let timer = Timer::start();
    // }

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
    state_bundle.metric = state_bundle.metric_router(performance);
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
        state_bundle.metric = state_bundle.metric_router(performance);

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
