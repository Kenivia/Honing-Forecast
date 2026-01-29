// use hf_core::saddlepoint_approximation::average::DEBUG_AVERAGE;
#[cfg(target_arch = "wasm32")]
use hf_core::send_progress::send_progress;
use hf_core::state_bundle::StateBundle;
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
// use std::f64::{MAX, MIN};

pub static INIT_TEMP: f64 = 333.3;
pub static RESOLUTION_CUTOFF_TEMP: f64 = 33.3;
pub static MAX_BEST_SIZE: usize = 10;
pub static ITERS_PER_TEMP: i64 = 7;
pub static MAX_ITERS: i64 = 7000;
pub static BATCH_SIZE: i64 = 500;
pub static ALPHA: f64 = 0.99;
pub fn my_pmf(max_len: usize, expected: f64, ratio: f64) -> Vec<f64> {
    // If NaN/inf, return a safe default: all mass at 0
    let mut v = vec![0.0; max_len + 1];
    for (index, v) in v.iter_mut().enumerate() {
        *v = ratio.powf(index as f64 - expected);
    }
    v
}

pub fn my_weighted_rand(
    length: usize,
    temp: f64,

    offset: usize,
    ratio: f64,
    upgrade_len: usize,
    rng: &mut SmallRng,
) -> usize {
    let res = WeightedIndex::new(my_pmf(
        (length - 1).max(1),
        (temp / INIT_TEMP * length as f64 - 1.0)
            .min(length as f64)
            .max(1.0 / upgrade_len as f64),
        ratio,
    ));
    if res.is_err() {
        dbg!(length, temp, INIT_TEMP, offset, ratio);
    }
    res.unwrap().sample(rng) + offset
}

/// New signature: accepts `resolution` to control block size
pub fn neighbour(state_bundle: &mut StateBundle, temp: f64, resolution: usize, rng: &mut SmallRng) {
    let u_len = state_bundle.upgrade_arr.len();
    // PART 1: Special State (Unchanged by resolution as requested)
    if state_bundle.special_state.len() > 1 {
        let want_to_swap: usize = my_weighted_rand(
            state_bundle.special_state.len(),
            temp / 2.0,
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
        let want_to_flip: usize =
            my_weighted_rand(num_blocks, temp, 0, 0.8 / eff_resolution as f64, u_len, rng);

        let want_to_book: usize =
            my_weighted_rand(num_blocks, temp, 1, 0.8 / eff_resolution as f64, u_len, rng);

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

pub fn new_temp(temp: f64) -> f64 {
    let new: f64 = temp * ALPHA;
    if new < 0.05 {
        return 0.0;
    }
    return new;
}
