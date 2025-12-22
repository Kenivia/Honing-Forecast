use crate::parser::Upgrade;

use core::f64;

use crate::helpers::compute_eqv_gold_values;

#[cfg(test)]
use crate::helpers::eqv_gold_per_tap;
#[cfg(test)]
use crate::saddlepoint_approximation::StateBundle;
#[cfg(test)]
use crate::saddlepoint_approximation::saddlepoint_approximation;
// use num::Float;
// use num::complex::Complex64;
// use quad_rs::{EvaluationError, Integrable, Integrator};
// use rootfinder::{Interval, SolverSettings, root_bisection};
// use statrs::distribution::{Continuous, ContinuousCDF, Normal};
// use std::f64::consts::PI;
// use std::ops::Range;

fn dist_to_costs(
    this_dist: &[f64],
    upgrade: &Upgrade,
    extra_arr: &[bool],
    // input_budget_no_gold: &[i64],
    price_arr: &[f64],
) -> Vec<(f64, f64)> {
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
            compute_eqv_gold_values(&this_costs, price_arr), // compute_gold_cost_from_raw(&this_costs, input_budget_no_gold, price_arr),
            prob,
        ));
    }
    list
}

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

pub fn prob_to_maximize_exact(
    state: &[Vec<bool>],
    upgrade_arr: &mut [Upgrade],
    cost_so_far: f64,
    price_arr: &[f64],
    budget: f64,
    depth: usize,
    // cache: &mut HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>>,
) -> f64 {
    // let key: (Vec<bool>, usize) = (state[depth].clone(), depth); // need to use an actual bitset here eventually
    //if !cache.contains_key(&key) {
    let this_dist: Vec<f64> = state_dist(
        upgrade_arr[depth].base_chance,
        upgrade_arr[depth].artisan_rate,
        &state[depth],
        upgrade_arr[depth].base_chance,
    );
    let costs_dist: Vec<(f64, f64)> = dist_to_costs(
        // automatically monotone(sorted)
        &this_dist,
        &upgrade_arr[depth],
        &state[depth],
        price_arr,
    );
    // dbg!(&costs_dist);
    // }
    // else {
    //     cache[&key].clone()
    // };

    // let costs_dist: Vec<([i64; 9], f64)> = {
    //     let this_dist: Vec<f64> = state_dist(
    //         upgrade_arr[depth].base_chance,
    //         upgrade_arr[depth].artisan_rate,
    //         &state[depth],
    //         upgrade_arr[depth].base_chance,
    //     );
    //     dist_to_costs(&this_dist, &upgrade_arr[depth], &state[depth], price_arr)
    // };

    if depth == state.len() - 1 {
        return costs_dist
            .iter()
            .take_while(|(eqv_gold_cost, _)| cost_so_far + eqv_gold_cost <= budget)
            .fold(0.0, |acc, (_, prob)| acc + prob);
    } else {
        return costs_dist.iter().fold(0.0, |acc, (eqv_gold_cost, prob)| {
            let new_cost: f64 = cost_so_far + eqv_gold_cost;
            acc + if budget < new_cost {
                0.0
            } else {
                prob * prob_to_maximize_exact(
                    state,
                    upgrade_arr,
                    new_cost,
                    price_arr,
                    budget,
                    depth + 1,
                    // cache,
                )
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::constants::RNG_SEED;
    use crate::helpers::eqv_gold_unlock;
    use crate::parser::PreparationOutputs;
    use crate::parser::preparation;
    use crate::saddlepoint_approximation::prob_to_maximize;
    use crate::test_utils::*;
    use std::time::Instant;
    #[test]
    fn saddlepoint_approximation_test() {
        let start = Instant::now();
        let test_name = format!("saddlepoint_approximation_test");
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
            (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
        ];
        // let hone_counts: Vec<Vec<i64>> =
        //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let input_budgets = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value,
            RNG_SEED,
            PROB_MODE
        );

        let mut prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );

        for upgrade in prep_outputs.upgrade_arr.iter_mut() {
            let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
            for i in upgrade.prob_dist.iter() {
                log_prob_dist.push(i.ln());
            }
            upgrade.log_prob_dist = log_prob_dist;
            upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, &prep_outputs.mats_value);
            let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
            upgrade.eqv_gold_per_juice =
                &prep_outputs.mats_value[juice_ind] * upgrade.one_juice_cost as f64;
            upgrade.juiced_arr = vec![0.0];
        }
        let result: f64 = saddlepoint_approximation(
            &prep_outputs.upgrade_arr,
            // 38591813.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
            // 25916.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
            62010.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
            0.0,
        );
        dbg!(result);
        if let Some(_cached_result) = read_cached_data::<f64>(test_name.as_str(), &hash) {
        } else {
            write_cached_data(test_name.as_str(), &hash, &result);
        }
        dbg!(start.elapsed());
    }

    #[test]
    fn energy_test() {
        let start = Instant::now();
        let test_name: &str = "energy_test";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25)
                .map(|x| if x == 23 || x == 24 { 1 } else { 0 })
                .collect(),
            (0..25)
                .map(|x| if x == 23 || x == 24 { 1 } else { 0 })
                .collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = false;
        let input_budgets = vec![
            3240, 9240, 46, 17740, 36, 7945670, 108000, 0, 0,
            0,
            // 0, 0, 0, 0, 0, 16000, 0, 0, 0, 0,
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

        let mut prep_output: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );
        // let mut cache: HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>> = HashMap::new();
        // dbg!(prep_output.upgrade_arr);
        // panic!();
        let mut states_evaled: i64 = 0;
        let approx_result = prob_to_maximize(
            &StateBundle {
                state: vec![
                    // vec![true; 16],
                    // vec![true; 2],
                    // vec![true; 18],
                    vec![0],
                    vec![0],
                    vec![0],
                    vec![0],
                ],
                names: vec![],
                state_index: vec![],
            },
            &mut prep_output,
            &mut states_evaled,
        );

        let exact_result = prob_to_maximize_exact(
            &vec![
                // vec![true; 16],
                // vec![true; 2],
                // vec![true; 18],
                vec![false],
                vec![false],
                vec![false],
                vec![false],
            ],
            &mut prep_output.upgrade_arr,
            0.0,
            &prep_output.mats_value,
            compute_eqv_gold_values(&prep_output.budgets, &prep_output.mats_value)
                - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
            0,
        );
        dbg!(approx_result);

        dbg!(exact_result);
        // let result: Vec<Vec<i64>> = out.clone();
        if let Some(_cached_result) =
            read_cached_data::<Vec<Vec<Vec<(f64, String)>>>>(test_name, &hash)
        {
            // my_assert!(*result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &approx_result);
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
