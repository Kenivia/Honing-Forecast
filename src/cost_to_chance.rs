use crate::chance_to_cost::average_juice_cost;
use crate::constants::*;
use crate::helpers::{calc_unlock, compress_runs, sort_by_indices};
use crate::histogram::histograms_for_all_costs;
use crate::monte_carlo::monte_carlo_data;
use crate::parser::{Upgrade, parser};
use crate::value_estimation::{est_juice_value, est_special_honing_value, juice_to_array};
use serde::Serialize;

#[derive(Serialize, Debug)]
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
}

fn extract_upgrade_strings(
    upgrade_arr: &Vec<Upgrade>,
    user_gave_weapon: bool,
    user_gave_armor: bool,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for (_index, upgrade) in upgrade_arr.iter().enumerate() {
        if !upgrade.is_normal_honing {
            continue;
        }
        let level_str: String = format!("+{}", upgrade.upgrade_plus_num + 1);
        let type_str: &'static str = if upgrade.is_weapon { "weapon" } else { "armor" };
        // let value_str: &str = &upgrade.special_value.to_string().as_str();
        let is_valid = if upgrade.is_weapon {
            user_gave_weapon
        } else {
            user_gave_armor
        };
        let value_string: String = if !is_valid {
            "".to_owned()
        } else {
            " ".to_owned() + &upgrade.special_value.round().to_string() + "g"
        };
        result.push(format!("{} {}{}", level_str, type_str, value_string));
    }
    result
}

fn fail_count_to_rates(typed_fail_counter: Vec<f64>, data_size: usize) -> Vec<f64> {
    let mut failure_rates: Vec<f64> = Vec::new();
    for z in 0..typed_fail_counter.len() {
        let failure_rate = 1.0 - typed_fail_counter[z] as f64 / data_size as f64;
        failure_rates.push(failure_rate);
    }
    failure_rates
}

#[derive(Debug)]
struct PreparationOutputs {
    upgrade_arr: Vec<Upgrade>,
    unlock_costs: Vec<i64>,
    budgets: Vec<i64>,
    valid_armor_values: bool,
    valid_weapon_values: bool,
    juice_strings_armor: Vec<String>,
    juice_strings_weapon: Vec<String>,
}

fn preparation(
    hone_counts: &Vec<Vec<i64>>,
    input_budgets: &Vec<i64>,
    adv_counts: &Vec<Vec<i64>>,
    express_event: bool,
    user_mats_value: &Vec<f64>,
    adv_hone_strategy: &str,
) -> PreparationOutputs {
    let mut mats_value: Vec<f64> = user_mats_value.clone();
    let unlock_costs: Vec<i64> = calc_unlock(hone_counts, adv_counts, express_event);

    let aritsan_arr: Vec<f64>;
    if express_event {
        aritsan_arr = EVENT_ARTISAN_MULTIPLIER.to_vec();
    } else {
        aritsan_arr = vec![1.0; 25];
    }

    let mut upgrade_arr: Vec<Upgrade> = parser(
        hone_counts,
        adv_counts,
        &adv_hone_strategy.to_string(),
        &aritsan_arr,
        &vec![0.0; 25],
        &vec![0; 25],
        express_event,
    );
    let mut budgets: Vec<i64> = input_budgets.clone();

    // Add average juice costs to budgets for all upgrades
    if adv_hone_strategy == "Juice on grace" {
        let (avg_red_juice, avg_blue_juice) = average_juice_cost(&upgrade_arr);
        budgets[7] -= avg_red_juice;
        budgets[8] -= avg_blue_juice;
    }

    let valid_armor_values: bool =
        mats_value.iter().skip(1).any(|&x| x != 0.0) || upgrade_arr.iter().all(|x| x.is_weapon);

    let valid_weapon_values: bool = mats_value
        .iter()
        .enumerate()
        .any(|(index, &x)| index != 1 && x != 0.0)
        || upgrade_arr.iter().all(|x| !x.is_weapon);
    let both_valid: bool = valid_armor_values && valid_weapon_values;

    if !both_valid {
        mats_value = vec![1.0, 0.1, 13.0, 0.2, 90.0, 1.0, 0.0];
    };

    est_juice_value(&mut upgrade_arr, &mats_value);
    let (juice_strings_armor, juice_strings_weapon) = juice_to_array(
        &mut upgrade_arr,
        budgets[8],
        budgets[7],
        valid_armor_values,
        valid_weapon_values,
    );
    let value_per_special_leap: Vec<f64> = est_special_honing_value(&mut upgrade_arr, &mats_value);
    let mut special_indices: Vec<usize> = (0..value_per_special_leap.len()).collect();
    special_indices
        .sort_by(|&a, &b| value_per_special_leap[b].total_cmp(&value_per_special_leap[a]));
    sort_by_indices(&mut upgrade_arr, special_indices.clone());

    PreparationOutputs {
        upgrade_arr,
        unlock_costs,
        budgets,
        valid_armor_values,
        valid_weapon_values,
        juice_strings_armor,
        juice_strings_weapon,
    }
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

fn count_failure_typed(
    cost_data: &Vec<Vec<i64>>,
    input_budgets: &Vec<i64>,
) -> FailureAnalysisOutputs {
    let mut typed_fail_counter_final: Vec<f64> = vec![0.0_f64; 7];
    let mut overall_fail_counter: i64 = 0;
    let mut failed;
    for (_trail_num, data) in cost_data.iter().enumerate() {
        failed = false;
        for cost_type in 0..7 {
            // Cost to chance does take silver into account
            if input_budgets[cost_type as usize] < data[cost_type] {
                failed = true;
                typed_fail_counter_final[cost_type] += 1.0_f64;
            }
        }
        if failed {
            overall_fail_counter += 1;
        }
    }

    let final_chance = 1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64;

    FailureAnalysisOutputs {
        typed_fail_counter_final,
        final_chance,
    }
}

fn count_failure_typed_arr(
    cost_data: &Vec<Vec<i64>>,
    input_budgets_arr: &Vec<Vec<i64>>,
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
    hone_counts: &Vec<Vec<i64>>,
    adv_counts: &Vec<Vec<i64>>,
    express_event: bool,
    cost_data: &Vec<Vec<i64>>,
    hist_bins: usize,
) -> HistogramOutputs {
    let upgrade_strings: Vec<String> = compress_runs(
        extract_upgrade_strings(upgrade_arr, valid_weapon_values, valid_armor_values),
        true,
    );

    let top_bottom: Vec<Vec<i64>> = monte_carlo_data(
        2,
        upgrade_arr,
        &calc_unlock(hone_counts, adv_counts, express_event),
        0,
        true, // rigged
        true, //use_true_rn
    );

    let bins = hist_bins.min(BUCKET_COUNT).max(1);
    let hist_maxs = top_bottom[1].clone();
    let hist_counts: Vec<Vec<i64>> = histograms_for_all_costs(cost_data, bins, &hist_maxs);

    HistogramOutputs {
        upgrade_strings,
        // top_bottom,
        hist_counts,
        hist_mins: vec![0_i64; 7],
        hist_maxs,
    }
}

pub fn cost_to_chance(
    hone_counts: &Vec<Vec<i64>>,
    input_budgets: &Vec<i64>,
    adv_counts: &Vec<Vec<i64>>,
    express_event: bool,
    hist_bins: usize,
    user_mats_value: &Vec<f64>,
    adv_hone_strategy: String,
    data_size: usize,
) -> CostToChanceOut {
    // Section 1: Preparation - setup and parsing
    let mut prep_outputs = preparation(
        hone_counts,
        input_budgets,
        adv_counts,
        express_event,
        user_mats_value,
        &adv_hone_strategy,
    );

    // Section 2: Monte Carlo simulation
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &mut prep_outputs.upgrade_arr,
        &prep_outputs.unlock_costs,
        input_budgets[9],
        false,
        false, //use_true_rng
    );

    // Section 3: Failure analysis
    let failure_outputs = count_failure_typed(&cost_data, input_budgets);

    // Section 4: Histogram preparation
    let histogram_outputs = prep_histogram(
        &mut prep_outputs.upgrade_arr,
        prep_outputs.valid_weapon_values,
        prep_outputs.valid_armor_values,
        hone_counts,
        adv_counts,
        express_event,
        &cost_data,
        hist_bins,
    );

    CostToChanceOut {
        chance: failure_outputs.final_chance,
        reasons: fail_count_to_rates(failure_outputs.typed_fail_counter_final, data_size),
        hist_counts: histogram_outputs.hist_counts,
        hist_mins: histogram_outputs.hist_mins,
        hist_maxs: histogram_outputs.hist_maxs,
        upgrade_strings: histogram_outputs.upgrade_strings,
        juice_strings_armor: prep_outputs.juice_strings_armor,
        juice_strings_weapon: prep_outputs.juice_strings_weapon,
        budgets_red_remaining: prep_outputs.budgets[7],
        budgets_blue_remaining: prep_outputs.budgets[8],
    }
}

pub fn cost_to_chance_arr(
    hone_counts: &Vec<Vec<i64>>,
    input_budgets_arr: &Vec<Vec<i64>>,
    adv_counts: &Vec<Vec<i64>>,
    express_event: bool,
    user_mats_value: &Vec<f64>,
    adv_hone_strategy: String,
    data_size: usize,
) -> (Vec<f64>, Vec<Vec<f64>>, i64, i64) {
    // Section 1: Preparation - setup and parsing (only run once with first budget)
    let first_budget = &input_budgets_arr[0];
    let mut prep_outputs = preparation(
        hone_counts,
        first_budget,
        adv_counts,
        express_event,
        user_mats_value,
        &adv_hone_strategy,
    );

    // Section 2: Monte Carlo simulation (only run once with first budget's special)
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &mut prep_outputs.upgrade_arr,
        &prep_outputs.unlock_costs,
        first_budget[9], // Use first budget's special leap count
        false,
        false, //use_true_rng
    );

    // Section 3: Failure analysis for all budgets
    let (final_chances, typed_fail_counters) =
        count_failure_typed_arr(&cost_data, input_budgets_arr);

    // Return only the required data: chances, failure counters, and remaining budgets
    (
        final_chances,
        typed_fail_counters,
        prep_outputs.budgets[7], // budgets_red_remaining
        prep_outputs.budgets[8], // budgets_blue_remaining
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cost_to_chance_stress() {
        let out = cost_to_chance(
            &vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()],
            &[
                431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
            ]
            .to_vec(),
            &vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()],
            false,
            1000,
            &vec![0.0; 7],
            "No juice".to_owned(),
            100000,
        );
        let _chance = out.chance;
        let _reasons = out.reasons;
        println!("{:?}", _chance);
        assert!(_chance < 0.001);
    }
    #[test]
    fn cost_to_chance_18_demo() {
        let out = cost_to_chance(
            &vec![
                (0..25)
                    .map(|i| if i == 19 || i == 20 || i == 21 { 5 } else { 0 })
                    .collect(),
                (0..25).map(|i| if i >= 19 { 1 } else { 0 }).collect(),
            ],
            &[
                631777, 1064398, 33748, 12010948, 25125, 3803792, 999999999, 1420, 690, 6767,
            ]
            .to_vec(),
            &vec![
                (0..4).map(|i| if i == 3 { 3 } else { 0 }).collect(),
                (0..4).map(|i| if i == 2 { 0 } else { 0 }).collect(),
            ],
            false,
            1000,
            // &vec![1.2, 0.1, 13.0, 0.3, 90.0, 1.0, 0.0],
            &vec![0.0; 7],
            "No juice".to_owned(),
            100000,
        );
        println!("{:?}", out.chance);
        println!("{:?}", out.reasons);
        println!("{:?}", out.upgrade_strings);
        println!("{:?}", out.juice_strings_armor);
        println!("{:?}", out.juice_strings_weapon);

        // println!("{:?}", out);
        assert!(0.07 < out.chance && out.chance < 0.08);
    }
    #[test]
    fn cost_to_chance_50_normal_weapon_25() {
        let out = cost_to_chance(
            &vec![
                (0..25).map(|_| 0).collect(),
                (0..25).map(|i| if i == 24 { 1 } else { 0 }).collect(),
            ],
            &[324000, 0, 4680, 1774000, 3600, 406800, 10800000, 0, 0, 0].to_vec(),
            &vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()],
            false,
            1000,
            &vec![0.0; 7],
            "No juice".to_owned(),
            100000,
        );
        println!("{:?}", out.chance);
        println!("{:?}", out.reasons);
        assert!(0.31 < out.chance && out.chance < 0.314);
    }
    #[test]
    fn cost_to_chance_53_adv_armor_40() {
        let out = cost_to_chance(
            &vec![(0..25).map(|_| 0).collect(), (0..25).map(|_| 0).collect()],
            &[0, 63600, 1219, 564000, 1007, 127200, 5003000, 0, 0, 0].to_vec(),
            &vec![
                (0..4).map(|x| if x == 3 { 1 } else { 0 }).collect(),
                (0..4).map(|_| 0).collect(),
            ],
            false,
            1000,
            &vec![0.0; 7],
            "No juice".to_owned(),
            100000,
        );
        println!("{:?}", out.chance);
        println!("{:?}", out.reasons);
        assert!(0.52 < out.chance && out.chance < 0.54);
    }

    #[test]
    fn cost_to_chance_arr_test() {
        let budget_arr = vec![
            vec![
                431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
            ],
            vec![
                431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
            ],
        ];

        let (final_chances, typed_fail_counters, budgets_red_remaining, budgets_blue_remaining) =
            cost_to_chance_arr(
                &vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()],
                &budget_arr,
                &vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()],
                false,
                &vec![0.0; 7],
                "No juice".to_owned(),
                100000,
            );

        println!("Final chances: {:?}", final_chances);
        println!("Typed fail counters: {:?}", typed_fail_counters);
        println!("Budgets red remaining: {}", budgets_red_remaining);
        println!("Budgets blue remaining: {}", budgets_blue_remaining);

        // Should have 2 results (one for each budget)
        assert_eq!(final_chances.len(), 2);
        assert_eq!(typed_fail_counters.len(), 2);
        assert_eq!(typed_fail_counters[0].len(), 7);
        assert_eq!(typed_fail_counters[1].len(), 7);

        // Both budgets are identical, so results should be the same
        assert!((final_chances[0] - final_chances[1]).abs() < 0.001);
    }
}
