use crate::parser::PreparationOutputs;
use crate::{helpers::compute_gold_cost_from_raw, parser::Upgrade};
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IteratorRandom;
use std::collections::HashMap;
fn state_dist(base: f64, artisan_rate: f64, extra_arr: &[bool], extra_amount: f64) -> Vec<f64> {
    let mut raw_chances: Vec<f64> = Vec::new();
    let mut artisan: f64 = 0.0_f64;
    let mut count: usize = 0;

    loop {
        let min_count: f64 = std::cmp::min(count, 10) as f64;
        let mut current_chance: f64 = base
            + (min_count * base) * 0.1
            + if *extra_arr.get(count).unwrap_or(&false) {
                extra_amount
            } else {
                0.0
            };

        if artisan >= 1.0 {
            current_chance = 1.0;
            raw_chances.push(current_chance);
            break;
        }

        raw_chances.push(current_chance);
        count += 1;
        artisan += (46.51_f64 / 100.0) * current_chance * artisan_rate;
        if current_chance == 1.0 {
            break; // for upgrades that have 100% passrate immediately
        }
    }

    // convert raw per-try chances into per-tap probability distribution
    let mut chances = vec![0.0_f64; raw_chances.len()];
    let mut cum_chance = 1.0_f64;
    for (idx, &element) in raw_chances.iter().enumerate() {
        chances[idx] = cum_chance * element;
        cum_chance *= 1.0 - element;
    }
    chances
}

fn dist_to_costs(
    this_dist: &[f64],
    upgrade: &Upgrade,
    extra_arr: &[bool],
    // input_budget_no_gold: &[i64],
    price_arr: &[f64],
) -> Vec<([i64; 9], f64)> {
    let mut list = Vec::with_capacity(this_dist.len());
    let mut juice_count_so_far: i64 = 0;
    for tap in 0..this_dist.len() {
        let prob: f64 = this_dist[tap];
        if prob == 0.0 {
            continue;
        }

        let taps_real = tap as i64 + upgrade.tap_offset;
        let mut this_costs = [0_i64; 9];

        for c in 0..7 {
            this_costs[c] = taps_real * upgrade.costs[c];
        }
        if upgrade.is_normal_honing {
            if tap < extra_arr.len() && extra_arr[tap] {
                juice_count_so_far += 1;
            }
            let j_idx = if upgrade.is_weapon { 7 } else { 8 };
            this_costs[j_idx] = juice_count_so_far * upgrade.one_juice_cost;
        }

        list.push((
            this_costs, // compute_gold_cost_from_raw(&this_costs, input_budget_no_gold, price_arr),
            prob,
        ));
    }
    list
}

fn prob_to_maximize(
    state: &[Vec<bool>],
    upgrade_arr: &[Upgrade],
    cost_so_far: &[i64],
    input_budget_no_gold: &[i64],
    price_arr: &[f64],
    threshold: f64,
    // depth: usize,
    // cache: &mut HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>>,
) -> f64 {
    // let key: (Vec<bool>, usize) = (state[depth].clone(), depth); // need to use an actual bitset here eventually
    // let costs_dist: Vec<([i64; 9], f64)> = if !cache.contains_key(&key) {
    //     let this_dist: Vec<f64> = state_dist(
    //         upgrade_arr[depth].base_chance,
    //         upgrade_arr[depth].artisan_rate,
    //         &state[depth],
    //         upgrade_arr[depth].base_chance,
    //     );
    //     dist_to_costs(
    //         // automatically monotone(sorted)
    //         &this_dist,
    //         &upgrade_arr[depth],
    //         &state[depth],
    //         price_arr,
    //     )
    // } else {
    //     cache[&key].clone()
    // };

    // // let costs_dist: Vec<([i64; 9], f64)> = {
    // //     let this_dist: Vec<f64> = state_dist(
    // //         upgrade_arr[depth].base_chance,
    // //         upgrade_arr[depth].artisan_rate,
    // //         &state[depth],
    // //         upgrade_arr[depth].base_chance,
    // //     );
    // //     dist_to_costs(&this_dist, &upgrade_arr[depth], &state[depth], price_arr)
    // // };

    // if depth == state.len() - 1 {
    //     return costs_dist
    //         .iter()
    //         .take_while(|(costs, _)| {
    //             let new_cost: Vec<i64> = cost_so_far
    //                 .to_vec()
    //                 .iter_mut()
    //                 .enumerate()
    //                 .map(|(index, x)| costs[index] + *x)
    //                 .collect::<Vec<i64>>();
    //             compute_gold_cost_from_raw(&new_cost, input_budget_no_gold, price_arr) <= threshold
    //         })
    //         .fold(0.0, |acc, (_, prob)| acc + prob);
    // } else {
    //     return costs_dist.iter().fold(0.0, |acc, (costs, prob)| {
    //         let new_cost: Vec<i64> = cost_so_far
    //             .to_vec()
    //             .iter_mut()
    //             .enumerate()
    //             .map(|(index, x)| costs[index] + *x)
    //             .collect::<Vec<i64>>();
    //         acc + if threshold
    //             < compute_gold_cost_from_raw(&new_cost, input_budget_no_gold, price_arr)
    //         {
    //             0.0
    //         } else {
    //             prob * prob_to_maximize(
    //                 state,
    //                 upgrade_arr,
    //                 &new_cost,
    //                 input_budget_no_gold,
    //                 price_arr,
    //                 threshold,
    //                 depth + 1,
    //                 cache,
    //             )
    //         }
    //     });
    // }

    // // }
}

fn acceptance<R: Rng>(new: f64, old: f64, temperature: f64, rng: &mut R) -> bool {
    let delta: f64 = old - new;
    let prob: f64 = if delta <= 0.0 {
        1.0
    } else if temperature <= 0.0 {
        0.0
    } else {
        // canonical Boltzmann acceptance with k = 1
        let p = (-delta / temperature).exp();
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
pub fn rotate_prefix_bool_vec(
    v: &mut Vec<bool>,
    end_index: usize,
    rotate_left: bool,
    amount: usize,
) {
    let n = end_index + 1; // prefix length
    if n == 0 {
        return;
    } // (this can only happen if end_index == usize::MAX, but keep for completeness)
    if amount == 0 {
        return;
    }

    let k = amount % n;
    if k == 0 {
        return;
    }

    // Convert a right rotation into an equivalent left rotation.
    let left_k = if rotate_left { k } else { (n - k) % n };

    // Copy prefix into temporary buffer and write rotated values back.
    let mut tmp = Vec::with_capacity(n);
    // initialize with false to set length
    tmp.resize(n, false);

    // new[i] = old[(i + left_k) % n]
    for i in 0..n {
        tmp[i] = v[(i + left_k) % n];
    }

    // write rotated prefix back into v[0..n]
    for i in 0..n {
        v[i] = tmp[i];
    }
}

fn neighbour<R: Rng>(
    state: &[Vec<bool>],
    upgrade_arr: &[Upgrade],
    temp: f64,
    init_temp: f64,
    rng: &mut R,
) -> Vec<Vec<bool>> {
    let mut new_state: Vec<Vec<bool>> = state.to_vec();
    for (u_index, s) in new_state.iter_mut().enumerate() {
        // kinda like noise here ig
        let want_to_flip: usize = WeightedIndex::new(my_pmf(
            (s.len() - 1).max(1),
            (temp / init_temp * s.len() as f64 - 1.0)
                .min(s.len() as f64)
                .max(0.0),
            // .max(),
        ))
        .unwrap()
        .sample(rng)
            + 1;
        // dbg!(want_to_flip);
        let want_to_flip_indices: Vec<usize> = (0..s.len()).choose_multiple(rng, want_to_flip);
        let mut flipped_index: usize = 0;
        let mut true_count: usize = 0;
        for (s_index, bit) in s.iter_mut().enumerate() {
            // dbg!(
            //     s_index,
            //     &bit,
            //     true_count,
            //     &upgrade_arr[u_index].support_lengths
            // );
            if s_index
                > upgrade_arr[u_index].support_lengths
                    [true_count.min(upgrade_arr[u_index].support_lengths.len() - 1)]
            {
                *bit = false;
            } else {
                if flipped_index < want_to_flip_indices.len()
                    && s_index == want_to_flip_indices[flipped_index]
                {
                    flipped_index += 1;
                    *bit = !*bit;
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
    if new < 0.00001 {
        return 0.0;
    }
    return new; // this is very much subject to change
}
fn simulated_annealing<R: Rng>(
    prep_output: &PreparationOutputs,
    rng: &mut R,
) -> (Vec<Vec<bool>>, f64) {
    let init_temp: f64 = -1.0 / 0.969_f64.ln(); // 0.969 = ~32
    let mut cache: HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>> = HashMap::new();
    let mut temp: f64 = init_temp;
    let mut state: Vec<Vec<bool>> = Vec::with_capacity(prep_output.upgrade_arr.len());
    for upgrade in prep_output.upgrade_arr.iter() {
        state.push(vec![false; upgrade.support_lengths[0]]); // random.
    }
    let mut prev_prob: f64 = prob_to_maximize(
        &state,
        &prep_output.upgrade_arr,
        &[0; 9],
        &prep_output.budgets_no_gold,
        &prep_output.mats_value,
        prep_output.budgets[5] as f64,
        0,
        &mut cache,
    );

    let iterations_per_temp = 120;
    let mut temperature_level_k = 0;
    let mut count: i32 = 0;
    let alpha: f64 = 0.9;

    let mut best_state_so_far: Vec<Vec<bool>> = state.clone();
    let mut best_prob_so_far: f64 = prev_prob;
    while temp >= 0.0 {
        let new_state: Vec<Vec<bool>> =
            neighbour(&state, &prep_output.upgrade_arr, temp, init_temp, rng);
        let new_prob: f64 = prob_to_maximize(
            &new_state,
            &prep_output.upgrade_arr,
            &[0; 9],
            &prep_output.budgets_no_gold,
            &prep_output.mats_value,
            prep_output.budgets[5] as f64,
            0,
            &mut cache,
        );
        if new_prob > best_prob_so_far {
            best_prob_so_far = new_prob;
            best_state_so_far = new_state.clone();
        }

        if acceptance(new_prob, prev_prob, temp, rng) {
            state = new_state;
            prev_prob = new_prob;
        }
        count += 1;
        if count > iterations_per_temp {
            count = 0;
            temperature_level_k += 1;
            println!(
                "Temp: {:.6} Prob: {:.6} Best prob: {:.6}",
                temp,
                (prev_prob * 100.0),
                (best_prob_so_far * 100.0) // encode_all(&state)
            );
            temp = new_temp(temp, alpha);
        }
    }
    println!(
        "Temp: {} Prob: {} State: {} (Final)",
        temp.to_string(),
        (best_prob_so_far * 100.0).to_string(),
        encode_all(&best_state_so_far)
    );
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
    strings.join(" ")
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::constants::RNG_SEED;
    use crate::parser::{PreparationOutputs, preparation};
    use crate::test_utils::*;

    use rand::prelude::*;

    use std::time::Instant;
    #[test]
    fn sa_test() {
        let start = Instant::now();
        let test_name: &str = "SA";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25)
                .map(|x| if x == 10 || x == 24 { 1 } else { 0 })
                .collect(),
            (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = false;
        let input_budgets = vec![
            3240, 9240, 46, 17740, 36, 1945670, 108000, 900, 900,
            0,
            // 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        // let data_size: usize = 100000;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value,
            // data_size,
            RNG_SEED,
            PROB_MODE
        );

        let prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );
        let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
        let result = simulated_annealing(&prep_outputs, &mut rng);

        // let result: Vec<Vec<i64>> = out.clone();
        if let Some(_cached_result) =
            read_cached_data::<Vec<Vec<Vec<(f64, String)>>>>(test_name, &hash)
        {
            // my_assert!(*result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result);
        }
        dbg!(start.elapsed());
        // let result: Vec<(Vec<i64>, Vec<i64>)> = brute(&mut upgrade_arr);
        // dbg!(result.len());
        // // let result: Vec<Vec<i64>> = out.clone();
        // if let Some(cached_result) = read_cached_data::<Vec<(Vec<i64>, Vec<i64>)>>(test_name, &hash)
        // {
        //     my_assert!(*result, cached_result);
        // } else {
        //     write_cached_data(test_name, &hash, &result);
        // }
    }
    #[test]
    fn energy_test() {
        let start = Instant::now();
        let test_name: &str = "energy_test";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 10 { 1 } else { 0 }).collect(),
            (0..25).map(|x| if x == 10 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = false;
        let input_budgets = vec![
            3240, 9240, 46, 17740, 36, 0, 108000, 90, 90, 0,
            // 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        // let data_size: usize = 100000;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value,
            // data_size,
            RNG_SEED,
            PROB_MODE
        );

        let prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );
        let mut cache: HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>> = HashMap::new();
        let result = prob_to_maximize(
            &vec![vec![true, true, true], vec![true, true, true]],
            &prep_outputs.upgrade_arr,
            &[0; 9],
            &prep_outputs.budgets,
            &prep_outputs.mats_value,
            66666.6,
            0,
            &mut cache,
        );

        // let result: Vec<Vec<i64>> = out.clone();
        if let Some(_cached_result) =
            read_cached_data::<Vec<Vec<Vec<(f64, String)>>>>(test_name, &hash)
        {
            // my_assert!(*result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result);
        }
        dbg!(start.elapsed());
        // let result: Vec<(Vec<i64>, Vec<i64>)> = brute(&mut upgrade_arr);
        // dbg!(result.len());
        // // let result: Vec<Vec<i64>> = out.clone();
        // if let Some(cached_result) = read_cached_data::<Vec<(Vec<i64>, Vec<i64>)>>(test_name, &hash)
        // {
        //     my_assert!(*result, cached_result);
        // } else {
        //     write_cached_data(test_name, &hash, &result);
        // }
    }
}
