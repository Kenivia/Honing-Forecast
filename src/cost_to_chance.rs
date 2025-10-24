use core::f64;

use crate::constants::BUCKET_COUNT;
use crate::helpers::{compress_runs, compute_gold_cost_from_raw};
use crate::histogram::histograms_for_all_costs;
use crate::monte_carlo::get_top_bottom;
use crate::parser::{PreparationOutputs, Upgrade, preparation};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CostToChanceOut {
    pub chance: f64,
    pub reasons: Vec<f64>,                 // 7 failure rates for each cost type
    pub hist_counts: Vec<Vec<i64>>,        // 7 x num_bins
    pub hist_mins: Vec<i64>,               // 7
    pub hist_maxs: Vec<i64>,               // 7
    pub upgrade_strings: Vec<String>,      // ordered upgrade descriptions
    pub juice_strings_armor: Vec<String>,  // e.g., ["+14 armor first 10 taps", ...]
    pub juice_strings_weapon: Vec<String>, // e.g., ["+15 weapon first 6 taps", ...]
    pub budgets_red_remaining: i64,        // budgets[7]
    pub budgets_blue_remaining: i64,       // budgets[8]
    pub hundred_gold_costs: Vec<i64>,
    pub chance_if_buy: f64,
    pub typical_costs: Vec<[i64; 9]>,
    // pub optimized_budgets: Vec<Vec<i64>>,
    // pub optimized_chances: Vec<f64>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CostToChanceArrOut {
    pub final_chances: Vec<f64>,
    pub typed_fail_counters: Vec<Vec<f64>>,
    pub budgets_red_remaining: i64,  // budgets_red_remaining
    pub budgets_blue_remaining: i64, // budgets_blue_remaining
    pub buy_chances: Vec<f64>,
    // buy_gold_costs: Vec<i64>,
}

#[derive(Debug)]
struct HistogramOutputs {
    upgrade_strings: Vec<String>,
    // top_bottom: Vec<Vec<i64>>,
    hist_counts: Vec<Vec<i64>>,
    hist_mins: Vec<i64>,
    hist_maxs: Vec<i64>,
}

#[derive(Debug)]
struct FailureAnalysisOutputs {
    typed_fail_counter_final: Vec<f64>,
    final_chance: f64,
}

fn extract_upgrade_strings(
    upgrade_arr: &[Upgrade],
    user_gave_weapon: bool,
    user_gave_armor: bool,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for upgrade in upgrade_arr {
        if !upgrade.is_normal_honing {
            continue;
        }
        let level_str: String = format!("+{}", upgrade.upgrade_plus_num + 1);
        let type_str: &'static str = if upgrade.is_weapon { "weapon" } else { "armor" };
        // let value_str: &str = &upgrade.special_value.to_string().as_str();
        let is_valid: bool = if upgrade.is_weapon {
            user_gave_weapon
        } else {
            user_gave_armor
        };
        let value_string: String = if is_valid {
            " ".to_owned() + &upgrade.special_value.round().to_string() + "g"
        } else {
            String::new()
        };
        result.push(format!("{level_str} {type_str}{value_string}"));
    }
    result
}

fn count_failure_typed(cost_data: &[[i64; 9]], input_budgets: &[i64]) -> FailureAnalysisOutputs {
    let mut typed_fail_counter_final: Vec<f64> = vec![0.0_f64; 7];
    let mut overall_fail_counter: i64 = 0;
    let mut failed: bool;
    for data in cost_data {
        failed = false;
        for cost_type in 0..7 {
            // Cost to chance does take silver into account
            if input_budgets[cost_type] < data[cost_type] {
                failed = true;
                typed_fail_counter_final[cost_type] += 1.0_f64;
            }
        }
        if failed {
            overall_fail_counter += 1;
        }
    }

    let final_chance: f64 = 1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64;
    for cost_type in 0..7 {
        typed_fail_counter_final[cost_type] =
            1.0 - typed_fail_counter_final[cost_type] / cost_data.len() as f64;
    }
    FailureAnalysisOutputs {
        typed_fail_counter_final,
        final_chance,
    }
}

fn count_failure_typed_arr(
    cost_data: &[[i64; 9]],
    input_budgets_arr: &[Vec<i64>],
) -> (Vec<f64>, Vec<Vec<f64>>) {
    let mut final_chances: Vec<f64> = Vec::new();
    let mut typed_fail_counters: Vec<Vec<f64>> = Vec::new();

    for input_budgets in input_budgets_arr {
        let failure_outputs: FailureAnalysisOutputs = count_failure_typed(cost_data, input_budgets);
        final_chances.push(failure_outputs.final_chance);
        typed_fail_counters.push(failure_outputs.typed_fail_counter_final);
    }

    (final_chances, typed_fail_counters)
}

fn prep_histogram(
    upgrade_arr: &mut Vec<Upgrade>,
    valid_weapon_values: bool,
    valid_armor_values: bool,
    cost_data: &[[i64; 9]],
    hist_bins: usize,
    unlock_costs: &[i64],
) -> HistogramOutputs {
    let upgrade_strings: Vec<String> = compress_runs(
        extract_upgrade_strings(upgrade_arr, valid_weapon_values, valid_armor_values),
        true,
        vec![],
    );

    let top_bottom: Vec<Vec<i64>> = get_top_bottom(upgrade_arr, unlock_costs);

    let bins: usize = hist_bins.min(BUCKET_COUNT).max(1);
    let hist_maxs: Vec<i64> = top_bottom[1].clone();
    let hist_counts: Vec<Vec<i64>> = histograms_for_all_costs(cost_data, bins, &hist_maxs);

    HistogramOutputs {
        upgrade_strings,
        // top_bottom,
        hist_counts,
        hist_mins: vec![0_i64; 7],
        hist_maxs,
    }
}

pub fn compute_all_gold_costs(
    input_budgets: &[i64],
    cost_data: &[[i64; 9]],
    prep_outputs: &PreparationOutputs,
) -> Vec<f64> {
    let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
    input_budget_no_gold[5] = 0;
    let mut all_gold_costs: Vec<f64> = Vec::with_capacity(cost_data.len());
    for cost in cost_data.iter() {
        all_gold_costs.push(compute_gold_cost_from_raw(
            cost,
            &input_budget_no_gold,
            &prep_outputs.mats_value,
        ));
    }
    all_gold_costs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_gold_costs
}
pub fn compute_all_gold_costs_and_sort_data(
    input_budgets: &[i64],
    cost_data: &mut [[i64; 9]],
    prep_outputs: &PreparationOutputs,
) -> Vec<f64> {
    let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
    input_budget_no_gold[5] = 0;
    let mut all_gold_costs: Vec<f64> = Vec::with_capacity(cost_data.len());

    for cost in cost_data.iter() {
        all_gold_costs.push(compute_gold_cost_from_raw(
            cost,
            &input_budget_no_gold,
            &prep_outputs.mats_value,
        ));
    }

    // Create indices paired with gold costs
    let mut indices: Vec<usize> = (0..cost_data.len()).collect();
    indices.sort_unstable_by(|&a, &b| {
        all_gold_costs[a]
            .partial_cmp(&all_gold_costs[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply permutation in-place using cycle-following algorithm
    for i in 0..indices.len() {
        if indices[i] != i {
            let mut current = i;
            loop {
                let next = indices[current];
                indices[current] = current; // Mark as visited
                if next == i {
                    break;
                }
                cost_data.swap(current, next);
                current = next;
            }
        }
    }

    // Sort all_gold_costs to match
    all_gold_costs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_gold_costs
}

fn get_hundred_gold_costs(
    all_gold_costs: &[f64],
    cost_data: &[[i64; 9]],
    prep_outputs: &PreparationOutputs,
    input_budgets: &[i64],
) -> Vec<i64> {
    let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
    input_budget_no_gold[5] = 0;
    let mut hundred_gold_costs: Vec<i64> = Vec::with_capacity(101);
    for i in 0..100_usize {
        hundred_gold_costs.push(
            all_gold_costs[(i as f64 * cost_data.len() as f64 / 100_f64).ceil() as usize].ceil()
                as i64,
        );
    }
    hundred_gold_costs.push(
        compute_gold_cost_from_raw(
            &get_top_bottom(&prep_outputs.upgrade_arr, &prep_outputs.unlock_costs)[1],
            &input_budget_no_gold,
            &prep_outputs.mats_value,
        )
        .ceil() as i64,
    );
    hundred_gold_costs
}

fn get_percentile_window(p: f64, cost_data: &[[i64; 9]]) -> &[[i64; 9]] {
    let n = cost_data.len();

    // Calculate the lower bound: (p - 0.005) * n, floored
    let lower_p = p - 0.005;
    let mut lower_idx = if lower_p <= 0.0 {
        0
    } else {
        let idx = (lower_p * n as f64).floor() as usize;
        idx.min(n - 1)
    };

    // Calculate the upper bound: (p + 0.005) * n, ceiled
    let upper_p = p + 0.005;
    let mut upper_idx = if upper_p >= 1.0 {
        n - 1
    } else {
        let idx = (upper_p * n as f64).ceil() as usize;
        idx.min(n - 1)
    };

    if lower_idx == 0 {
        upper_idx = 0;
    }
    if upper_idx == n - 1 {
        lower_idx = n - 1;
    }

    // Return the slice (upper_idx is inclusive, so we add 1)
    &cost_data[lower_idx..=upper_idx]
}
fn typical_cost(
    cost_data_sorted: &[[i64; 9]],
    desired_chance: f64,
    price_arr: &[f64],
    input_budget_no_gold: &[i64],
    target_gold: i64,
) -> [i64; 9] {
    let relevant_data: &[[i64; 9]] = get_percentile_window(desired_chance, cost_data_sorted);

    // let mut median: Vec<f64> = Vec::with_capacity(9);
    let mut average: Vec<f64> = vec![0.0; 9];
    for data in relevant_data {
        for i in 0..7 {
            if i != 5 {
                average[i] += data[i] as f64;
            }
        }
    }
    for i in 0..7 {
        average[i] /= relevant_data.len() as f64;
    }

    let mut out: [i64; 9] = average
        .iter()
        .map(|x| x.round() as i64)
        .collect::<Vec<i64>>()
        .try_into()
        .unwrap();
    // web_sys::console::log_1(&desired_chance.into());
    // for _iteration in 0..1 {
    // honestly idk why it doesn't just take 1 iteration but here we are
    let gold_cost_of_average: f64 =
        compute_gold_cost_from_raw(&out, input_budget_no_gold, price_arr);
    let mut modified_gold_costs: Vec<f64> = Vec::with_capacity(cost_data_sorted.len());
    for cost in cost_data_sorted {
        modified_gold_costs.push(compute_gold_cost_from_raw(cost, &out, price_arr));
    }
    modified_gold_costs
        .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut current_success_chance: f64 = 1.0;
    for (index, g) in modified_gold_costs.iter().enumerate() {
        if *g > gold_cost_of_average - target_gold as f64 {
            // web_sys::console::log_1(&(*g).into());
            current_success_chance =
                index.saturating_sub(1) as f64 / modified_gold_costs.len() as f64;

            break;
        }
    }

    if current_success_chance < (desired_chance - 0.005).max(0.0) {
        let needed_gold_for_modified: f64 = modified_gold_costs
            [((desired_chance) * modified_gold_costs.len() as f64).ceil() as usize];

        out[5] += needed_gold_for_modified.round() as i64;
    }

    out
}
pub fn cost_to_chance(
    hone_counts: &[Vec<i64>],
    input_budgets: &[i64],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    hist_bins: usize,
    user_mats_value: &[f64],
    adv_hone_strategy: String,
    cost_data_to_sort: &mut [[i64; 9]],
) -> CostToChanceOut {
    // Section 1: Preparation - setup and parsing
    let mut prep_outputs: PreparationOutputs = preparation(
        hone_counts,
        input_budgets,
        adv_counts,
        express_event,
        user_mats_value,
        &adv_hone_strategy,
    );

    // Section 3: Failure analysis
    let failure_outputs: FailureAnalysisOutputs =
        count_failure_typed(cost_data_to_sort, input_budgets);

    let all_gold_costs: Vec<f64> =
        compute_all_gold_costs_and_sort_data(input_budgets, cost_data_to_sort, &prep_outputs);
    let hundred_gold_costs: Vec<i64> = get_hundred_gold_costs(
        &all_gold_costs,
        cost_data_to_sort,
        &prep_outputs,
        input_budgets,
    );
    let mut chance_if_buy: f64 = 1.0;
    for (index, gold) in all_gold_costs.iter().enumerate() {
        if *gold > input_budgets[5] as f64 {
            chance_if_buy = index as f64 / cost_data_to_sort.len() as f64; // intentionally not subtracting by 1 because index starts from 0
            break;
        }
    }
    let mut typical_costs: Vec<[i64; 9]> = Vec::with_capacity(101);
    let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
    input_budget_no_gold[5] = 0;
    for i in 0..100 {
        typical_costs.push(typical_cost(
            cost_data_to_sort,
            i as f64 / 100.0,
            &prep_outputs.mats_value,
            &input_budget_no_gold,
            hundred_gold_costs[i],
        ));
    }
    typical_costs.push(
        get_top_bottom(&prep_outputs.upgrade_arr, &prep_outputs.unlock_costs)[1]
            .clone()
            .try_into()
            .unwrap(),
    );

    // Section 4: Histogram preparation
    let histogram_outputs: HistogramOutputs = prep_histogram(
        &mut prep_outputs.upgrade_arr,
        prep_outputs.valid_weapon_values,
        prep_outputs.valid_armor_values,
        cost_data_to_sort,
        hist_bins,
        &prep_outputs.unlock_costs,
    );

    CostToChanceOut {
        chance: failure_outputs.final_chance,
        reasons: failure_outputs.typed_fail_counter_final,
        hist_counts: histogram_outputs.hist_counts,
        hist_mins: histogram_outputs.hist_mins,
        hist_maxs: histogram_outputs.hist_maxs,
        upgrade_strings: histogram_outputs.upgrade_strings,
        juice_strings_armor: prep_outputs.juice_strings_armor,
        juice_strings_weapon: prep_outputs.juice_strings_weapon,
        budgets_red_remaining: prep_outputs.budgets[7],
        budgets_blue_remaining: prep_outputs.budgets[8],
        hundred_gold_costs,
        chance_if_buy,
        typical_costs,
        // optimized_budgets: optimization_out.optimized_budgets,
        // optimized_chances: optimization_out.optimized_chances,
    }
}

pub fn cost_to_chance_arr(
    hone_counts: &[Vec<i64>],
    input_budgets_arr: &[Vec<i64>],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    user_mats_value: &[f64],
    adv_hone_strategy: String,
    cost_data: &[[i64; 9]],
) -> CostToChanceArrOut {
    // Section 1: Preparation - setup and parsing (only run once with first budget)
    let first_budget: &Vec<i64> = &input_budgets_arr[0];
    let prep_outputs: PreparationOutputs = preparation(
        hone_counts,
        first_budget,
        adv_counts,
        express_event,
        user_mats_value,
        &adv_hone_strategy,
    );

    // Section 2: Monte Carlo simulation (only run once with first budget's special)

    // Section 3: Failure analysis for all budgets
    let (final_chances, typed_fail_counters): (Vec<f64>, Vec<Vec<f64>>) =
        count_failure_typed_arr(cost_data, input_budgets_arr);

    let mut buy_chances: Vec<f64> = Vec::with_capacity(input_budgets_arr.len());
    // let mut buy_gold_costs: Vec<i64> = Vec::with_capacity(input_budgets_arr.len());
    let mut all_gold_costs: Vec<f64>;
    for budget in input_budgets_arr {
        all_gold_costs = compute_all_gold_costs(budget, cost_data, &prep_outputs);
        let mut count: f64 = 0.0;
        for gold in all_gold_costs.iter() {
            if *gold <= budget[5] as f64 {
                count += 1.0;
            }
        }
        // buy_gold_costs.push((*gold - budget[5] as f64).ceil() as i64);
        buy_chances.push(count / all_gold_costs.len() as f64);
    }
    CostToChanceArrOut {
        final_chances,
        typed_fail_counters,
        budgets_red_remaining: prep_outputs.budgets[7], // budgets_red_remaining
        budgets_blue_remaining: prep_outputs.budgets[8], // budgets_blue_remaining
        buy_chances,
        // buy_gold_costs,
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::constants::RNG_SEED;
//     use crate::test_utils::{read_cached_data, write_cached_data};
//     use crate::{calculate_hash, my_assert};
//     use rand::prelude::*;

//     #[test]
//     fn cost_to_chance_stress() {
//         let test_name: &str = "cost_to_chance_stress";
//         let hone_counts: Vec<Vec<i64>> =
//             vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];

//         let adv_counts: Vec<Vec<i64>> =
//             vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()];
//         let input_budgets: Vec<i64> = vec![
//             431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
//         ];
//         let express_event: bool = false;
//         let hist_bins: usize = 1000;
//         let user_mats_value: Vec<f64> = vec![0.0; 7];
//         let adv_hone_strategy: &'static str = "No juice";
//         let data_size: usize = 10000;

//         let hash: String = calculate_hash!(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &adv_hone_strategy,
//             data_size
//         );
//         // Run the function to get the full output
//         let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
//         let result: CostToChanceOut = cost_to_chance(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &user_mats_value,
//             adv_hone_strategy.to_owned(),
//             data_size,
//             &mut rng,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }
//     #[test]
//     fn cost_to_chance_18_demo() {
//         let test_name: &str = "cost_to_chance_18_demo";
//         let hone_counts = vec![
//             (0..25)
//                 .map(|i| if i == 19 || i == 20 || i == 21 { 5 } else { 0 })
//                 .collect(),
//             (0..25).map(|i| if i >= 19 { 1 } else { 0 }).collect(),
//         ];
//         let input_budgets = vec![
//             631777, 1064398, 33748, 12010948, 25125, 3803792, 999999999, 1420, 690, 6767,
//         ];
//         let adv_counts = vec![
//             (0..4).map(|i| if i == 3 { 3 } else { 0 }).collect(),
//             (0..4).map(|i| if i == 2 { 0 } else { 0 }).collect(),
//         ];
//         let express_event = false;
//         let hist_bins: usize = 1000;
//         let user_mats_value = vec![0.0; 7];
//         let adv_hone_strategy = "No juice";
//         let data_size: usize = 100000;

//         let hash = calculate_hash!(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &adv_hone_strategy,
//             data_size
//         );
//         // Run the function to get the full output
//         let mut rng = StdRng::seed_from_u64(RNG_SEED);
//         let result: CostToChanceOut = cost_to_chance(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &user_mats_value,
//             adv_hone_strategy.to_owned(),
//             data_size,
//             &mut rng,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }
//     // #[test]
//     // fn cost_to_chance_50_normal_weapon_25() {
//     //     let test_name: &str = "cost_to_chance_50_normal_weapon_25";
//     //     let hone_counts = vec![
//     //         (0..25).map(|_| 0).collect(),
//     //         (0..25).map(|i| if i == 24 { 1 } else { 0 }).collect(),
//     //     ];
//     //     let input_budgets = vec![324000, 0, 4680, 1774000, 3600, 406800, 10800000, 0, 0, 0];
//     //     let adv_counts = vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];
//     //     let express_event = false;
//     //     let hist_bins: usize = 1000;
//     //     let user_mats_value = vec![0.0; 7];
//     //     let adv_hone_strategy = "No juice";
//     //     let data_size: usize = 100000;

//     //     let hash = calculate_hash!(
//     //         &hone_counts,
//     //         &input_budgets,
//     //         &adv_counts,
//     //         express_event,
//     //         hist_bins,
//     //         &adv_hone_strategy,
//     //         data_size
//     //     );
//     //     // Run the function to get the full output
//     //     let mut rng = StdRng::seed_from_u64(RNG_SEED);
//     //     let result: CostToChanceOut = cost_to_chance(
//     //         &hone_counts,
//     //         &input_budgets,
//     //         &adv_counts,
//     //         express_event,
//     //         hist_bins,
//     //         &user_mats_value,
//     //         adv_hone_strategy.to_owned(),
//     //         data_size,
//     //         &mut rng,
//     //     );

//     //     // let result_of_interst: Vec<f64> = result.optimized_chances.clone();
//     //     // dbg!(&result_of_interst);
//     //     // if let Some(cached_result) = read_cached_data::<Vec<f64>>(test_name, &hash) {
//     //     //     for (index, i) in result.optimized_chances.iter().enumerate() {
//     //     //         my_assert!(
//     //     //             *i,
//     //     //             cached_result[index],
//     //     //             0.000000000001
//     //     //         );
//     //     //     }
//     //     // } else {
//     //     //     write_cached_data(test_name, &hash, &result_of_interst);
//     //     // }
//     // }
//     #[test]
//     fn cost_to_chance_53_adv_armor_40() {
//         let test_name: &str = "cost_to_chance_53_adv_armor_40";
//         let hone_counts = vec![(0..25).map(|_| 0).collect(), (0..25).map(|_| 0).collect()];
//         let input_budgets = vec![0, 63600, 1219, 564000, 1007, 127200, 5003000, 0, 0, 0];
//         let adv_counts = vec![
//             (0..4).map(|x| if x == 3 { 1 } else { 0 }).collect(),
//             (0..4).map(|_| 0).collect(),
//         ];
//         let express_event = false;
//         let hist_bins: usize = 1000;
//         let user_mats_value = vec![0.0; 7];
//         let adv_hone_strategy = "No juice";
//         let data_size: usize = 100000;

//         let hash = calculate_hash!(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &adv_hone_strategy,
//             data_size
//         );
//         // Run the function to get the full output
//         let mut rng = StdRng::seed_from_u64(RNG_SEED);
//         let result: CostToChanceOut = cost_to_chance(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &user_mats_value,
//             adv_hone_strategy.to_owned(),
//             data_size,
//             &mut rng,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }

//     #[test]
//     fn cost_to_chance_arr_test() {
//         let test_name: &str = "cost_to_chance_arr_test";
//         let budget_arr = vec![
//             vec![
//                 431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
//             ],
//             vec![
//                 431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
//             ],
//         ];

//         let hone_counts = vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
//         let adv_counts = vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()];
//         let express_event = false;
//         let user_mats_value = vec![0.0; 7];
//         let adv_hone_strategy = "No juice";
//         let data_size: usize = 100000;

//         let hash = calculate_hash!(
//             &hone_counts,
//             &budget_arr,
//             &adv_counts,
//             express_event,
//             &adv_hone_strategy,
//             data_size
//         );
//         // Run the function to get the full output
//         let mut rng = StdRng::seed_from_u64(RNG_SEED);
//         let result: CostToChanceArrOut = cost_to_chance_arr(
//             &hone_counts,
//             &budget_arr,
//             &adv_counts,
//             express_event,
//             &user_mats_value,
//             adv_hone_strategy.to_owned(),
//             data_size,
//             &mut rng,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceArrOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }
// }
