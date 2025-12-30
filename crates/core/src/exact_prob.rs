use core::f64;

use crate::helpers::compute_eqv_gold_values;
use crate::parser::Upgrade;
#[cfg(test)]
use crate::state::StateBundle;

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
) -> f64 {
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

    use crate::parser::PreparationOutput;
    use crate::saddlepoint_approximation::normal_sa::normal_honing_sa_wrapper;

    use crate::test_utils::*;
    use std::time::Instant;

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
            3240, 9240, 46, 17740, 36, 7545670, 108000, 0, 0,
            0,
            // 0, 0, 0, 0, 0, 16000, 0, 0, 0, 0,
        ];
        let user_price_arr = DEFAULT_GOLD_VALUES;
        // let data_size: usize = 100000;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_price_arr,
            // data_size,
            RNG_SEED,
            PROB_MODE
        );

        let mut prep_output: PreparationOutput = PreparationOutput::initialize(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_price_arr,
            adv_hone_strategy,
            &vec![(0, 0), (0, 0), (0, 0), (0, 0)],
            &vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
        );
        // let mut cache: HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>> = HashMap::new();
        // dbg!(prep_output.upgrade_arr);
        // panic!();
        let mut states_evaled: i64 = 0;
        let approx_result = normal_honing_sa_wrapper(
            &mut StateBundle {
                state: vec![
                    // vec![true; 16],
                    // vec![true; 2],
                    // vec![true; 18],
                    vec![(false, 0); 300],
                    vec![(false, 0); 300],
                    vec![(false, 0); 300],
                    vec![(false, 0); 300],
                ],
                names: vec![],
                state_index: vec![],
                prob: -1.0,
                special_state: vec![],
                log_prob_dist_arr: vec![],
                gold_costs_arr: vec![],
            },
            &mut prep_output,
            &mut states_evaled,
        );
        let budget: f64 = prep_output.budget_eqv_gold;
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
            &prep_output.price_arr,
            budget,
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
    //     #[test]
    // fn saddlepoint_approximation_test() {
    //     let start = Instant::now();
    //     let test_name = format!("saddlepoint_approximation_test");
    //     let hone_counts: Vec<Vec<i64>> = vec![
    //         (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
    //         (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
    //     ];
    //     // let hone_counts: Vec<Vec<i64>> =
    //     //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
    //     let adv_counts: Vec<Vec<i64>> =
    //         vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

    //     let adv_hone_strategy: &str = "No juice";
    //     let express_event: bool = true;
    //     let input_budgets = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    //     let user_price_arr = DEFAULT_GOLD_VALUES;
    //     let hash: String = calculate_hash!(
    //         &hone_counts,
    //         &adv_counts,
    //         adv_hone_strategy,
    //         express_event,
    //         &input_budgets,
    //         &user_price_arr,
    //         RNG_SEED,
    //         PROB_MODE
    //     );

    //     let mut prep_output: PreparationOutput = preparation(
    //         &hone_counts,
    //         &input_budgets,
    //         &adv_counts,
    //         express_event,
    //         &user_price_arr,
    //         adv_hone_strategy,
    //         &vec![],
    //     );

    //     for upgrade in prep_output.upgrade_arr.iter_mut() {
    //         let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
    //         for i in upgrade.prob_dist.iter() {
    //             log_prob_dist.push(i.ln());
    //         }
    //         upgrade.log_prob_dist = log_prob_dist;
    //         upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, &prep_output.price_arr);
    //     }
    //     let result: f64 = saddlepoint_approximation(
    //         &prep_output,
    //         &StateBundle {
    //             state: vec![vec![vec![false]]; 2],
    //             names: vec![],
    //             state_index: vec![],
    //             prob: -1.0,
    //         },
    //         // 38591813.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
    //         // 25916.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
    //         62010.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
    //         0.0,
    //     );
    //     dbg!(result);
    //     if let Some(_cached_result) = read_cached_data::<f64>(test_name.as_str(), &hash) {
    //     } else {
    //         write_cached_data(test_name.as_str(), &hash, &result);
    //     }
    //     dbg!(start.elapsed());
    // }
}
