// use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
#[cfg(target_arch = "wasm32")]
use crate::send_progress::send_progress;
use crate::state_bundle::StateBundle;
use ordered_float::Float;
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
// use std::f64::{MAX, MIN};

pub const INIT_TEMP: f64 = 333.3;
pub const RESOLUTION_CUTOFF_TEMP: f64 = 33.3;
pub const MAX_BEST_SIZE: usize = 10;
pub const ITERS_PER_TEMP: i64 = 7;
pub const MAX_ITERS: i64 = 7000;
pub const BATCH_SIZE: i64 = 500;
pub const ALPHA: f64 = 0.99;
pub const NON_IMPACT_WEIGHT: f64 = 3.0 / 333.3;
pub const SPECIAL_START_CHANCE: f64 = 1.0;
pub const MIN_SPECIAL_CHANCE: f64 = 0.05;
pub const CONCURRENT_COUNT: i64 = 16;

pub const JUICE_TEMP_FACTOR: f64 = 3.0;
pub const SPECIAL_TEMP_FACTOR: f64 = 1.0;

pub const BLOCK_SIZE_MULTIPLIER: f64 = 1.0;

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
        (temp / INIT_TEMP * (length as f64 - 1.0))
            .min(length as f64)
            .max(1.0 / upgrade_len as f64),
        ratio,
    ));
    if res.is_err() {
        dbg!(length, temp, INIT_TEMP, offset, ratio);
    }
    res.unwrap().sample(rng) + offset
}

// New struct fields for SolverStateBundle
// pub upgrade_weights: WeightedIndex<f64>,

pub fn neighbour(
    state_bundle: &mut StateBundle,
    temp: f64,
    resolution: usize,
    rng: &mut SmallRng,
    upgrade_weights: &Vec<f64>,
) {
    let juice_temp_factor = (temp * JUICE_TEMP_FACTOR + 1.0).min(INIT_TEMP).ln() / INIT_TEMP.ln();
    let special_temp_factor =
        (temp * SPECIAL_TEMP_FACTOR + 1.0).min(INIT_TEMP).ln() / INIT_TEMP.ln();
    let mutate_special = rng.random_bool(
        ((SPECIAL_START_CHANCE - MIN_SPECIAL_CHANCE) * special_temp_factor + MIN_SPECIAL_CHANCE)
            * (1.0 - state_bundle.special_cache[&state_bundle.special_state][0]),
        //
    ) && state_bundle.special_state.len() > 1;

    if mutate_special {
        for _ in 0..(special_temp_factor * state_bundle.special_state.len() as f64).ceil() as usize
        {
            let len = state_bundle.special_state.len();
            let max_dist_f = (len as f64) * special_temp_factor; // min 10% range
            let max_dist = (max_dist_f as usize).max(1);

            let idx1 = rng.random_range(0..len - 1);
            // Pick idx2 within max_dist of idx1
            let offset = rng.random_range(1..=max_dist);
            let idx2 = if rng.random_bool(0.5) {
                (idx1 + offset).min(len - 1)
            } else {
                idx1.saturating_sub(offset)
            };

            state_bundle.special_state.swap(idx1, idx2);
        }
    } else {
        let num_upgrades = state_bundle.upgrade_arr.len();
        let max_mutations = (num_upgrades as f64 * juice_temp_factor).ceil() as usize;
        let num_to_mutate = rng.random_range(1..=max_mutations.max(1));
        let mut already_mutated: Vec<bool> = vec![false; state_bundle.upgrade_arr.len()];

        for _ in 0..num_to_mutate {
            let u_idx = WeightedIndex::new(upgrade_weights.iter().zip(already_mutated.iter()).map(
                |(x, alr)| {
                    if *alr {
                        0.0
                    } else {
                        x * juice_temp_factor + NON_IMPACT_WEIGHT * (1.0 - juice_temp_factor)
                    }
                },
            ))
            .unwrap()
            .sample(rng);
            already_mutated[u_idx] = true;
            let upgrade = &mut state_bundle.upgrade_arr[u_idx];

            if upgrade.succeeded {
                continue;
            }

            let mut effective_len = 0;

            let mut artisan: f64 = 0.0;
            let mut count: f64 = 0.1;
            let base_chance = upgrade.base_chance;
            let artisan_rate = upgrade.artisan_rate;
            let juice_chances = &state_bundle.prep_output.juice_info.chances_id;

            for (i, (juice, id)) in upgrade.state.iter().enumerate() {
                // ... insert your specific probability logic here ...
                // cumulative_prob += ...;
                if artisan >= 1.0 {
                    effective_len = i + 1;
                    break;
                }
                artisan += (46.51_f64 / 100.0)
                    * artisan_rate
                    * (base_chance * (1.0 + count)
                        + if *juice {
                            juice_chances[0][upgrade.upgrade_index]
                        } else {
                            0.0
                        }
                        + if *id > 0 {
                            juice_chances[*id][upgrade.upgrade_index]
                        } else {
                            0.0
                        });
                if count < 1.0 {
                    count += 0.1;
                }
            }
            if effective_len == 0 {
                effective_len = upgrade.state.len();
            }

            let block_size = resolution.max(1);
            let num_blocks = (effective_len + block_size - 1) / block_size;

            // Pick a block
            let block_idx = rng.random_range(0..num_blocks);

            let start_idx = block_idx * block_size;

            // let actual_block_size = block_size
            //     * (num_blocks as f64 * juice_temp_factor * BLOCK_SIZE_MULTIPLIER).ceil() as usize;
            let end_idx = (start_idx + block_size).min(upgrade.state.len());
            let juice_target =
                // if rng.random_bool((temp / INIT_TEMP * JUICE_BOOK_MULTIPLIER).min(1.0).max(0.1)) {
                    rng.random_bool(0.5);
            // } else {
            // !upgrade.state[rng.random_range(start_idx..end_idx)].0;
            // };

            let book_target =
                // if rng.random_bool((temp / INIT_TEMP * JUICE_BOOK_MULTIPLIER).min(1.0).max(0.1)) 
                {
                    let ids = &state_bundle.prep_output.juice_info.ids[upgrade.upgrade_index];
                    let ind = rng.random_range(0..ids.len());
                   let out =  ids[ind];
                //    if out ==   upgrade.state[ind].1{ ids[(ind + 1) % ids.len()]} else 
                   { out}
                // } else {
                //     upgrade.state[rng.random_range(start_idx..end_idx)].1
                };
            for i in start_idx..end_idx {
                if i < upgrade.alr_failed {
                    continue;
                }
                upgrade.state[i].0 = juice_target;
                upgrade.state[i].1 = book_target;
            }

            upgrade.state.update_hash();
        }
    }
}

pub fn new_temp(temp: f64) -> f64 {
    let new: f64 = temp * ALPHA;
    if new < 0.05 {
        return 0.0;
    }
    return new;
}
