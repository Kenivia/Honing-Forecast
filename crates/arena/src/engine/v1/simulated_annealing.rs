// use hf_core::energy::prob_to_maximize_exact;

use hf_core::saddlepoint_approximation::average::DEBUG_AVERAGE;
use hf_core::state_bundle::StateBundle;
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IteratorRandom;
use std::f64::{MAX, MIN};
use std::fs::File;
use std::io::Write;

// use std::collections::HashMap;

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

/// Return the PMF of Binomial(n, p) as a Vec<f64> where
/// p is chosen so that E[Binomial(n,p)] = `expected` (i.e. p = expected / n).
///
/// - `max_len` is n (number of trials).
/// - `expected` is the desired expected value (can be non-integer).
///
/// The returned Vec has length `max_len + 1` with pmf[k] = P(X = k).
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
    rng: &mut R,
) -> usize {
    // dbg!(length, temp, init_temp, offset, ratio,);
    let res = WeightedIndex::new(my_pmf(
        (length - 1).max(1),
        ((temp / (init_temp)).cbrt() * length as f64 - 1.0)
            .min(length as f64 - 1.0)
            .max(0.0),
        ratio, // .max(),
    ));
    if res.is_err() {
        dbg!(length, temp, init_temp, offset, ratio);
    }
    res.unwrap().sample(rng) + offset
}
fn neighbour<R: Rng>(state_bundle: &mut StateBundle, temp: f64, init_temp: f64, rng: &mut R) {
    if state_bundle.special_state.len() > 1 {
        let want_to_swap: usize = my_weighted_rand(
            state_bundle.special_state.len(),
            temp / 2.0,
            init_temp,
            1,
            0.8,
            rng,
        );
        for _ in 0..want_to_swap {
            let want_to_swap_indices: Vec<usize> =
                (0..state_bundle.special_state.len()).choose_multiple(rng, 2);
            let first = want_to_swap_indices[0];
            let second = want_to_swap_indices[1];
            let temp = state_bundle.special_state[first];
            state_bundle.special_state[first] = state_bundle.special_state[second];
            state_bundle.special_state[second] = temp;
        }
    }

    for upgrade in state_bundle.upgrade_arr.iter_mut() {
        let choice_len: usize = upgrade.books_avail as usize;
        let state = &mut upgrade.state;

        let want_to_flip: usize = my_weighted_rand(state.len(), temp, init_temp, 1, 0.8, rng);

        let want_to_book: usize = my_weighted_rand(state.len(), temp, init_temp, 1, 0.8, rng);

        let flip_target = rng.random_bool(0.5);

        // dbg!(want_to_flip);
        let mut want_to_flip_indices: Vec<usize> =
            (0..state.len() - 1).choose_multiple(rng, want_to_flip);
        want_to_flip_indices.sort();
        let mut flipped_index: usize = 0;

        let ids = &state_bundle.prep_output.juice_info.ids[upgrade.upgrade_index];
        let book_target_index: usize = rng.random_range(0..=choice_len);
        let book_target_id: usize = ids[book_target_index];
        let mut want_to_book_indices: Vec<usize> =
            (0..state.len() - 1).choose_multiple(rng, want_to_book);
        want_to_book_indices.sort();
        let mut booked_index: usize = 0;

        let mut artisan: f64 = 0.0;
        let base_chance = upgrade.base_chance;
        let artisan_rate = upgrade.artisan_rate;

        let juice_chances = &state_bundle.prep_output.juice_info.chances_id;

        // dbg!(&state, choice_len);
        for (s_index, (juice, id)) in state.iter_mut().enumerate() {
            if artisan >= 1.0 || s_index == 0 {
                (*juice, *id) = (false, 0);
                continue;
            }
            if flipped_index < want_to_flip_indices.len()
                && s_index == want_to_flip_indices[flipped_index]
            {
                if *juice != flip_target {
                    flipped_index += 1;
                    *juice = flip_target;
                } else {
                    want_to_flip_indices[flipped_index] = s_index + 1;
                }
            }

            if booked_index < want_to_book_indices.len()
                && s_index == want_to_book_indices[booked_index]
            {
                if *id != book_target_id {
                    booked_index += 1;
                    *id = book_target_id;
                } else {
                    want_to_book_indices[booked_index] = s_index + 1;
                }
            }
            artisan += (46.51_f64 / 100.0)
                * artisan_rate
                * (base_chance
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
    return new; // this is very much subject to change
}

pub fn solve<R: Rng>(
    rng: &mut R,
    metric_type: i64, // note that we're always trying to maximize this metric which is why i'm not calling it energy
    mut state_bundle: StateBundle,
) -> StateBundle {
    let init_temp: f64 = if DEBUG_AVERAGE { -1.0 } else { -333.0 };
    // let init_temp: f64 = -1.0; // 0.969 = ~32
    // let mut cache: HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>> = HashMap::new();
    let mut temp: f64 = init_temp;

    state_bundle.metric = state_bundle.metric_router(metric_type);
    let mut prev_state: StateBundle = state_bundle.clone();

    let iterations_per_temp = 69;
    // let mut temperature_level_k = 0;
    let mut count: i64 = 0;
    let alpha: f64 = 0.99;
    let mut highest_seen: f64 = MIN;
    let mut lowest_seen: f64 = MAX;
    let mut best_state_so_far: StateBundle = state_bundle.clone();

    let mut temps_without_improvement = 1;
    while temp >= 0.0 {
        neighbour(&mut state_bundle, temp, init_temp, rng);
        state_bundle.metric = state_bundle.metric_router(metric_type);
        highest_seen = highest_seen.max(state_bundle.metric);
        lowest_seen = lowest_seen.min(state_bundle.metric);
        if state_bundle.metric > best_state_so_far.metric {
            best_state_so_far.my_clone_from(&state_bundle);
            temps_without_improvement = 0;
            // println!(
            //     "Temp: {:.6} Best prob: {:.6} Best state: \n{}",
            //     temp,
            //     (best_state_so_far.prob * 100.0),
            //     // prob_to_maximize_exact(
            //     //     &best_state_so_far,
            //     //     &mut upgrade_arr,
            //     //     0.0,
            //     //     &prep_output.price_arr,
            //     //     compute_eqv_gold_values(&prep_output.budgets, &prep_output.price_arr)
            //     //         - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
            //     //     0
            //     // ),
            //     encode_all(&&best_state_so_far)
            // );
        }

        if acceptance(
            state_bundle.metric,
            prev_state.metric,
            temp,
            highest_seen - lowest_seen,
            rng,
        ) {
            prev_state.my_clone_from(&state_bundle);
        } else {
            state_bundle.my_clone_from(&prev_state);
        }
        count += 1;
        if count > iterations_per_temp {
            count = 0;
            // temperature_level_k += 1;

            // println!(
            //     "Temp: {:.6} Prob: {:.6} Best prob: {:.6}",
            //     temp,
            //     prev_state.metric,
            //     (best_state_so_far.metric),
            //     // prob_to_maximize_exact(
            //     //     &best_state_so_far,
            //     //     &mut upgrade_arr,
            //     //     0.0,
            //     //     &prep_output.price_arr,
            //     //     compute_eqv_gold_values(&prep_output.budgets, &prep_output.price_arr)
            //     //         - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
            //     //     0
            //     // ),
            // );
            if temps_without_improvement as f64 > (1.0 * temp).max(3.0)
            // || (best_prob_so_far - prev_prob) > 0.005
            {
                state_bundle.my_clone_from(&best_state_so_far);
                temps_without_improvement = 0;
                // dbg!("restarted");
            }
            temps_without_improvement += 1;
            temp = new_temp(temp, alpha);
        }
    }
    // println!(
    //     "Temp: {} Prob: {} State (Final): \n{} ",
    //     temp.to_string(),
    //     (best_state_so_far.metric).to_string(),
    //     best_state_so_far.encode_all(),
    // );

    // println!(
    //     "Exact value: {:.6}",
    //     prob_to_maximize_exact(
    //         &best_state_so_far,
    //         &mut upgrade_arr,
    //         0.0,
    //         &prep_output.price_arr,
    //         compute_eqv_gold_values(&prep_output.budgets, &prep_output.price_arr)
    //             - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
    //         0
    //     )
    // );

    if metric_type == 1 {
        let mut results: Vec<(f64, f64)> = Vec::new();
        best_state_so_far.update_dist();
        best_state_so_far.update_individual_support();

        let (soft_low_limit, mut guess, soft_high_limit) =
            best_state_so_far.min_guess_max_triplet(4, 0);

        let mean_var = {
            let out = best_state_so_far.ks(
                0.0,
                &(false, true, true, true, true),
                true,
                best_state_so_far.simple_avg_var(4, 0).0,
                4,
                0,
            );
            dbg!(out);
            (out.1, out.2)
        };
        for i in 0..100_000 {
            let theta =
                i as f64 / 100_000_f64 * (soft_high_limit - soft_low_limit) + soft_low_limit;
            let res = best_state_so_far
                .ks(
                    theta,
                    &(false, true, false, false, false),
                    true,
                    mean_var.0.ln(),
                    4,
                    0,
                )
                .1
                - state_bundle.prep_output.budgets[4] as f64;
            results.push((theta, res));
        }
        let json_data: Vec<Vec<f64>> = results.iter().map(|(x, y)| vec![*x, *y]).collect();

        let json_string = serde_json::to_string_pretty(&json_data).unwrap();

        let mut file = File::create("results.json").unwrap();
        file.write_all(json_string.as_bytes()).unwrap();
    }
    best_state_so_far
}
