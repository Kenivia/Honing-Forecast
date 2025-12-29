// use hf_core::energy::prob_to_maximize_exact;

use hf_core::normal_sa::normal_honing_sa_wrapper;
use hf_core::parser::PreparationOutput;
use hf_core::state::StateBundle;
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IteratorRandom;

// use std::collections::HashMap;

fn acceptance<R: Rng>(new: f64, old: f64, temperature: f64, rng: &mut R) -> bool {
    let delta: f64 = old - new;
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
pub fn my_pmf(max_len: usize, expected: f64) -> Vec<f64> {
    // If NaN/inf, return a safe default: all mass at 0
    let mut v = vec![0.0; max_len + 1];
    for (index, v) in v.iter_mut().enumerate() {
        *v = 0.8_f64.powf(index as f64 - expected);
    }
    v
}

fn neighbour<R: Rng>(
    state_bundle: &mut StateBundle,
    prep_output: &PreparationOutput,
    temp: f64,
    init_temp: f64,
    rng: &mut R,
) {
    for (u_index, upgrade_state) in state_bundle.state.iter_mut().enumerate() {
        let upgrade = &prep_output.upgrade_arr[u_index];
        let choice_len: usize = upgrade.books_avail as usize;

        let want_to_flip: usize = WeightedIndex::new(my_pmf(
            (upgrade_state.len() - 1).max(1),
            ((temp / init_temp).cbrt() * upgrade_state.len() as f64 - 1.0)
                .min(upgrade_state.len() as f64)
                .max(0.0),
            // .max(),
        ))
        .unwrap()
        .sample(rng)
            + 1;

        let want_to_book: usize = WeightedIndex::new(my_pmf(
            (upgrade_state.len() - 1).max(1),
            ((temp / init_temp).cbrt() * upgrade_state.len() as f64 - 1.0)
                .min(upgrade_state.len() as f64)
                .max(0.0),
            // .max(),
        ))
        .unwrap()
        .sample(rng)
            + 1;
        let flip_target = rng.random_bool(0.5);

        // dbg!(want_to_flip);
        let mut want_to_flip_indices: Vec<usize> =
            (0..upgrade_state.len()).choose_multiple(rng, want_to_flip);
        want_to_flip_indices.sort();
        let mut flipped_index: usize = 0;

        let book_target: usize = rng.random_range(0..=choice_len);
        let mut want_to_book_indices: Vec<usize> =
            (0..upgrade_state.len()).choose_multiple(rng, want_to_book);
        want_to_book_indices.sort();
        let mut booked_index: usize = 0;

        let mut artisan: f64 = 0.0;
        for (s_index, (juice, book)) in upgrade_state.iter_mut().enumerate() {
            if artisan >= 1.0 || s_index == 0 {
                (*juice, *book) = (false, 0);
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
                if *book != book_target {
                    booked_index += 1;
                    *book = book_target;
                } else {
                    want_to_book_indices[booked_index] = s_index + 1;
                }
            }
            artisan += (46.51_f64 / 100.0)
                * upgrade.artisan_rate
                * (upgrade.base_chance
                    + if *juice {
                        prep_output.juice_info.chances[upgrade.upgrade_index][0]
                    } else {
                        0.0
                    }
                    + if *book > 0 {
                        prep_output.juice_info.chances[upgrade.upgrade_index][(*book + 1) as usize]
                    } else {
                        0.0
                    });
        }
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
    prep_output: &mut PreparationOutput,
    rng: &mut R,
    states_evaled: &mut i64,
) -> StateBundle {
    let init_temp: f64 = 333.0; // 0.969 = ~32
    // let mut cache: HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>> = HashMap::new();
    let mut temp: f64 = init_temp;
    let mut state: Vec<Vec<(bool, usize)>> = Vec::with_capacity(prep_output.upgrade_arr.len());
    let mut starting_special: Vec<usize> = Vec::with_capacity(prep_output.upgrade_arr.len() * 50);
    for (index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        // state.push(vec![rng.random_bool(0.5); upgrade.support_lengths[0]]);
        state.push(vec![(false, 0); upgrade.prob_dist_len]);
        for _ in 0..50 {
            starting_special.push(index);
        }
    }

    let mut state_bundle: StateBundle = StateBundle {
        state: state,
        names: prep_output
            .upgrade_arr
            .iter()
            .map(|x| {
                let mut string: String = if x.is_normal_honing {
                    "".to_owned()
                } else {
                    "adv_".to_owned()
                };
                string += if x.is_weapon { "weap_" } else { "armor_" };
                string += &x.upgrade_index.to_string();
                string
            })
            .collect::<Vec<String>>(),
        state_index: vec![],
        prob: -1.0,
        special_state: starting_special,
        log_prob_dist_arr: vec![],
        gold_costs_arr: vec![],
    };

    state_bundle.prob = normal_honing_sa_wrapper(&mut state_bundle, prep_output, states_evaled);
    let mut prev_state: StateBundle = state_bundle.clone();

    let iterations_per_temp = 69;
    // let mut temperature_level_k = 0;
    let mut count: i32 = 0;
    let alpha: f64 = 0.99;

    let mut best_state_so_far: StateBundle = state_bundle.clone();

    let mut temps_without_improvement = 1;
    while temp >= 0.0 {
        neighbour(&mut state_bundle, prep_output, temp, init_temp, rng);
        state_bundle.prob = normal_honing_sa_wrapper(&mut state_bundle, prep_output, states_evaled);

        if state_bundle.prob > best_state_so_far.prob {
            best_state_so_far = state_bundle.clone();
            temps_without_improvement = 0;
            // println!(
            //     "Temp: {:.6} Best prob: {:.6} Best state: \n{}",
            //     temp,
            //     (best_state_so_far.prob * 100.0),
            //     // prob_to_maximize_exact(
            //     //     &best_state_so_far,
            //     //     &mut prep_output.upgrade_arr,
            //     //     0.0,
            //     //     &prep_output.price_arr,
            //     //     compute_eqv_gold_values(&prep_output.budgets, &prep_output.price_arr)
            //     //         - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
            //     //     0
            //     // ),
            //     encode_all(&&best_state_so_far)
            // );
        }

        if acceptance(state_bundle.prob, prev_state.prob, temp, rng) {
            prev_state = state_bundle.clone();
        } else {
            state_bundle.clone_from(&prev_state);
        }
        count += 1;
        if count > iterations_per_temp {
            count = 0;
            // temperature_level_k += 1;

            // println!(
            //     "Temp: {:.6} Prob: {:.6} Best prob: {:.6}",
            //     temp,
            //     (prev_prob * 100.0),
            //     (best_prob_so_far * 100.0),
            //     // prob_to_maximize_exact(
            //     //     &best_state_so_far,
            //     //     &mut prep_output.upgrade_arr,
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
                state_bundle.clone_from(&best_state_so_far);
                temps_without_improvement = 0;
                // dbg!("restarted");
            }
            temps_without_improvement += 1;
            temp = new_temp(temp, alpha);
        }
        if best_state_so_far.prob >= 1.0 {
            break; // optimize average later
        }
    }
    // println!(
    //     "Temp: {} Prob: {} State (Final): \n{} ",
    //     temp.to_string(),
    //     (best_state_so_far.prob * 100.0).to_string(),
    //     encode_all(&best_state_so_far)
    // );

    // println!(
    //     "Exact value: {:.6}",
    //     prob_to_maximize_exact(
    //         &best_state_so_far,
    //         &mut prep_output.upgrade_arr,
    //         0.0,
    //         &prep_output.price_arr,
    //         compute_eqv_gold_values(&prep_output.budgets, &prep_output.price_arr)
    //             - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
    //         0
    //     )
    // );
    best_state_so_far
}
