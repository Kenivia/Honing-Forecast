// use hf_core::energy::prob_to_maximize_exact;
use hf_core::helpers::{compute_eqv_gold_values, eqv_gold_unlock};
use hf_core::parser::{PreparationOutputs, Upgrade};
use hf_core::saddlepoint_approximation::prob_to_maximize;
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

/// Rotate the prefix `v[0..=end_index]` of `v` by `amount`.
///
/// - `v` is modified in-place.
/// - `end_index` is inclusive (so prefix length = `end_index + 1`).
/// - `rotate_left = true` rotates left; `false` rotates right.
/// - `amount` may be >= prefix length; it's reduced modulo prefix length.
/// - The function does nothing when `amount % prefix_len == 0` or prefix_len == 0.
// pub fn rotate_prefix_bool_vec(
//     v: &mut Vec<bool>,
//     end_index: usize,
//     rotate_left: bool,
//     amount: usize,
// ) {
//     let n = end_index + 1; // prefix length
//     if n == 0 {
//         return;
//     } // (this can only happen if end_index == usize::MAX, but keep for completeness)
//     if amount == 0 {
//         return;
//     }

//     let k = amount % n;
//     if k == 0 {
//         return;
//     }

//     // Convert a right rotation into an equivalent left rotation.
//     let left_k = if rotate_left { k } else { (n - k) % n };

//     // Copy prefix into temporary buffer and write rotated values back.
//     let mut tmp = Vec::with_capacity(n);
//     // initialize with false to set length
//     tmp.resize(n, false);

//     // new[i] = old[(i + left_k) % n]
//     for i in 0..n {
//         tmp[i] = v[(i + left_k) % n];
//     }

//     // write rotated prefix back into v[0..n]
//     for i in 0..n {
//         v[i] = tmp[i];
//     }
// }

fn neighbour<R: Rng>(
    state: &[Vec<bool>],
    upgrade_arr: &[Upgrade],
    temp: f64,
    init_temp: f64,
    rng: &mut R,
) -> Vec<Vec<bool>> {
    let mut new_state: Vec<Vec<bool>> = state.to_vec();
    for (u_index, s) in new_state.iter_mut().enumerate() {
        let want_to_flip: usize = WeightedIndex::new(my_pmf(
            (s.len() - 1).max(1),
            ((temp / init_temp).cbrt() * s.len() as f64 - 1.0)
                .min(s.len() as f64)
                .max(0.0),
            // .max(),
        ))
        .unwrap()
        .sample(rng)
            + 1;
        let flip_target = rng.random_bool(0.5);
        // dbg!(want_to_flip);
        let mut want_to_flip_indices: Vec<usize> = (0..s.len()).choose_multiple(rng, want_to_flip);
        want_to_flip_indices.sort();
        let mut flipped_index: usize = 0;
        let mut true_count: usize = 0;
        for (s_index, bit) in s.iter_mut().enumerate() {
            if s_index
                > upgrade_arr[u_index].support_lengths
                    [true_count.min(upgrade_arr[u_index].support_lengths.len() - 1)]
            {
                *bit = false;
            } else {
                if flipped_index < want_to_flip_indices.len()
                    && s_index == want_to_flip_indices[flipped_index]
                {
                    if *bit == flip_target {
                        flipped_index += 1;
                        *bit = !*bit;
                    } else {
                        want_to_flip_indices[flipped_index] = s_index + 1;
                    }

                    // *bit = rng.random_bool(0.5);
                }
                if *bit {
                    true_count += 1;
                }
            }
        }

        // let rotate_count: usize = WeightedIndex::new(binomial_pmf(
        //     (s.len() as f64).ceil() as usize,
        //     (temp / init_temp * s.len() as f64 - 1.0)
        //         .min(s.len() as f64)
        //         .max(0.1),
        // ))
        // .unwrap()
        // .sample(rng);
        // rotate_prefix_bool_vec(
        //     s,
        //     upgrade_arr[u_index].support_lengths
        //         [true_count.min(upgrade_arr[u_index].support_lengths.len() - 1)]
        //         - 1,
        //     rng.random_bool(0.5),
        //     rotate_count,
        // );
        // dbg!(want_to_flip, flipped_index, rotate_count);
    }
    // dbg!(&new_state);
    new_state
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
fn simulated_annealing<R: Rng>(
    prep_output: &mut PreparationOutputs,
    rng: &mut R,
    states_evaled: &mut i64,
) -> (Vec<Vec<bool>>, f64) {
    let init_temp: f64 = 333.0; // 0.969 = ~32
    // let mut cache: HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>> = HashMap::new();
    let mut temp: f64 = init_temp;
    let mut state: Vec<Vec<bool>> = Vec::with_capacity(prep_output.upgrade_arr.len());
    for upgrade in prep_output.upgrade_arr.iter() {
        // state.push(vec![rng.random_bool(0.5); upgrade.support_lengths[0]]);
        state.push(vec![false; upgrade.support_lengths[0]]);
    }
    let mut prev_prob: f64 = prob_to_maximize(
        &state,
        &mut prep_output.upgrade_arr,
        &prep_output.mats_value,
        compute_eqv_gold_values(&prep_output.budgets, &prep_output.mats_value)
            - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
        states_evaled,
    );

    let iterations_per_temp = 69;
    // let mut temperature_level_k = 0;
    let mut count: i32 = 0;
    let alpha: f64 = 0.99;

    let mut best_state_so_far: Vec<Vec<bool>> = state.clone();
    let mut best_prob_so_far: f64 = prev_prob;
    let mut temps_without_improvement = 1;
    while temp >= 0.0 {
        let new_state: Vec<Vec<bool>> =
            neighbour(&state, &prep_output.upgrade_arr, temp, init_temp, rng);
        // dbg!(
        //     compute_eqv_gold_values(&prep_output.budgets, &prep_output.mats_value),
        //     eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value)
        // );
        // panic!();
        let new_prob: f64 = prob_to_maximize(
            &state,
            &mut prep_output.upgrade_arr,
            &prep_output.mats_value,
            compute_eqv_gold_values(&prep_output.budgets, &prep_output.mats_value)
                - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
            states_evaled,
        );
        // if new_prob > 0.13 {
        //     panic!();
        // }
        if new_prob > best_prob_so_far {
            best_prob_so_far = new_prob;
            best_state_so_far = new_state.clone();
            temps_without_improvement = 0;
            println!(
                "Temp: {:.6} Prob: {:.6} Best prob: {:.6} Best state: \n{}",
                temp,
                (prev_prob * 100.0),
                (best_prob_so_far * 100.0),
                // prob_to_maximize_exact(
                //     &best_state_so_far,
                //     &mut prep_output.upgrade_arr,
                //     0.0,
                //     &prep_output.mats_value,
                //     compute_eqv_gold_values(&prep_output.budgets, &prep_output.mats_value)
                //         - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
                //     0
                // ),
                encode_all(&&best_state_so_far)
            );
        }

        if acceptance(new_prob, prev_prob, temp, rng) {
            state = new_state;
            prev_prob = new_prob;
        }
        count += 1;
        if count > iterations_per_temp {
            count = 0;
            // temperature_level_k += 1;

            println!(
                "Temp: {:.6} Prob: {:.6} Best prob: {:.6}",
                temp,
                (prev_prob * 100.0),
                (best_prob_so_far * 100.0),
                // prob_to_maximize_exact(
                //     &best_state_so_far,
                //     &mut prep_output.upgrade_arr,
                //     0.0,
                //     &prep_output.mats_value,
                //     compute_eqv_gold_values(&prep_output.budgets, &prep_output.mats_value)
                //         - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
                //     0
                // ),
            );
            if temps_without_improvement as f64 > (1.0 * temp).max(3.0)
            // || (best_prob_so_far - prev_prob) > 0.005
            {
                state = best_state_so_far.clone();
                prev_prob = best_prob_so_far.clone();
                temps_without_improvement = 0;
                dbg!("restarted");
            }
            temps_without_improvement += 1;
            temp = new_temp(temp, alpha);
        }
    }
    println!(
        "Temp: {} Prob: {} State (Final): \n{} ",
        temp.to_string(),
        (best_prob_so_far * 100.0).to_string(),
        encode_all(&best_state_so_far)
    );
    // println!(
    //     "Exact value: {:.6}",
    //     prob_to_maximize_exact(
    //         &best_state_so_far,
    //         &mut prep_output.upgrade_arr,
    //         0.0,
    //         &prep_output.mats_value,
    //         compute_eqv_gold_values(&prep_output.budgets, &prep_output.mats_value)
    //             - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
    //         0
    //     )
    // );
    (best_state_so_far, best_prob_so_far)
}

fn encode_one_positions(v1: &[bool]) -> String {
    let mut s1 = String::new();

    for i in 0..v1.len() {
        // first vector row
        if v1[i] {
            s1.push('1');
        } else {
            s1.push('0');
        }
    }

    format!("{s1}")
}
fn encode_all(input: &[Vec<bool>]) -> String {
    let mut strings = Vec::new();
    for i in input.iter() {
        strings.push(encode_one_positions(i));
    }
    strings.join("\n")
}
pub fn solve<R: Rng>(
    states_evaled: &mut i64,
    prep_output: &mut PreparationOutputs,
    rng: &mut R,
) -> (String, f64) {
    let (state, prob) = simulated_annealing(prep_output, rng, states_evaled);
    (encode_all(&state), prob)
}
