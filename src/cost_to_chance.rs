use crate::chance_to_cost::average_juice_cost;
use crate::constants::*;
use crate::helpers::{calc_unlock, compress_runs, myformat, sort_by_indices};
use crate::histogram::histograms_for_all_costs;
use crate::monte_carlo::monte_carlo_data;
use crate::parser::{Upgrade, parser};
use crate::value_estimation::{est_juice_value, est_special_honing_value, juice_to_array};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct CostToChanceOut {
    pub chance: f64,
    pub reasons: Vec<String>,
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

fn fail_count_to_string(typed_fail_counter: Vec<f64>, data_size: usize) -> Vec<String> {
    let failed_indices: Vec<usize> = (0..typed_fail_counter.len()).collect();
    // failed_indices.sort_by(|&a, &b| typed_fail_counter[b].total_cmp(&typed_fail_counter[a]));
    let mut this_failed: Vec<String> = Vec::new();
    // let mut displayed: bool = false;
    let mut spread_str: String;
    let mut spread_num: f64;
    for z in failed_indices {
        spread_num = 1.0 - typed_fail_counter[z] as f64 / data_size as f64;
        spread_str = myformat(spread_num);
        // if spread_num >= 0.001 || !displayed {
        this_failed.push(spread_str.to_owned() + "% chance to have enough " + LABELS[z]);
        // }
        // displayed = true
    }
    if typed_fail_counter
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        == 0.0_f64
    {
        return vec!["None".to_string()];
    } else {
        return this_failed;
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
    let mut mats_value: Vec<f64> = user_mats_value.clone();
    let data_size: usize = data_size.max(1000);
    // let adv_hone_strategy: String = String::from("No juice");
    let unlock_costs: Vec<i64> = calc_unlock(hone_counts, adv_counts, express_event);

    let aritsan_arr: Vec<f64>;
    if express_event {
        aritsan_arr = EVENT_ARTISAN_MULTIPLIER.to_vec();
    } else {
        aritsan_arr = vec![1.0; 25];
    }

    let mut upgrade_arr: Vec<Upgrade> = parser(
        &hone_counts,
        &adv_counts,
        &adv_hone_strategy,
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
    let mut override_special: Vec<i64> = budgets.clone();
    override_special[9] = 0;

    // let user_armor_values: Vec<f64> = zero_this_index(user_forced_mats_value, 0);

    let valid_armor_values: bool =
        mats_value.iter().skip(1).any(|&x| x != 0.0) || upgrade_arr.iter().all(|x| x.is_weapon);

    // let user_weapon_values: Vec<f64> = zero_this_index(user_forced_mats_value, 1);
    // let user_weapon_values: Vec<f64> =
    let valid_weapon_values: bool = mats_value
        .iter()
        .enumerate()
        .any(|(index, &x)| index != 1 && x != 0.0)
        || upgrade_arr.iter().all(|x| !x.is_weapon);
    let both_valid: bool = valid_armor_values && valid_weapon_values;

    // let auto_optimize: bool = user_gave_values
    // &&(actual_budgets[9] > 0 || actual_budgets[8] > 0 || actual_budgets[7] > 0);

    // Use original calibration approach

    if !both_valid {
        mats_value = vec![0.6, 0.1, 13.0, 0.2, 90.0, 1.0, 0.0];
        // valid_weapon_values = vec![1.2, 0.1, 13.0, 0.3, 90.0, 1.0, 0.0];

        // let mut prelim_cost_data_arr: Vec<Vec<Vec<i64>>> = Vec::new();

        // for z in 0..upgrade_arr.len() {
        //     let mut this_data: Vec<Vec<i64>> = monte_carlo_data(
        //         data_size / 10,
        //         &mut upgrade_arr[z..z + 1],
        //         &unlock_costs,
        //         0,
        //         false,
        //         false,
        //     );
        //     for row in this_data.iter_mut() {
        //         row[3] -= unlock_costs[0];
        //         row[6] -= unlock_costs[1];
        //     }
        //     prelim_cost_data_arr.push(this_data);
        // }
        // let toal_cost_data: Vec<Vec<i64>> = calc_failure_raw_delta(
        //     input_budgets,
        //     &mut upgrade_arr,
        //     &prelim_cost_data_arr,
        //     &unlock_costs,
        // );
        // calc_failure_delta_order(input_budgets, &mut upgrade_arr, &toal_cost_data)
        // // calc_failure_delta_order(&input_budgets, &mut upgrade_arr, &prelim_cost_data);
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
    // if both_valid {
    let mut special_indices: Vec<usize> = (0..value_per_special_leap.len()).collect();
    special_indices
        .sort_by(|&a, &b| value_per_special_leap[b].total_cmp(&value_per_special_leap[a]));
    sort_by_indices(&mut upgrade_arr, special_indices.clone());
    // } else {
    //     let upgrade_len: i64 = upgrade_arr.len() as i64;
    //     upgrade_arr.sort_by(|a, b| {
    //         (if a.is_normal_honing {
    //             a.failure_delta_order as f64 // a.special_cost as f64 * a.base_chance
    //         } else {
    //             upgrade_len as f64
    //         })
    //         .total_cmp(
    //             &(if b.is_normal_honing {
    //                 b.failure_delta_order as f64 // b.special_cost as f64 * a.base_chance
    //             } else {
    //                 upgrade_len as f64
    //             }),
    //         )
    //     });
    // }
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &mut upgrade_arr,
        &unlock_costs,
        input_budgets[9],
        false,
        false, //use_true_rng
    );
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

    let upgrade_strings: Vec<String> = compress_runs(
        extract_upgrade_strings(&mut upgrade_arr, valid_weapon_values, valid_armor_values),
        true,
    );
    let final_chance = 1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64;

    // Generate histogram data from simulated cost data
    let cost_data_for_hist: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &upgrade_arr,
        &unlock_costs,
        budgets[9],
        false,
        false,
    );
    let bins = hist_bins.min(BUCKET_COUNT).max(1);

    let budget_data: Vec<Vec<i64>> = monte_carlo_data(
        2,
        &upgrade_arr,
        &calc_unlock(&hone_counts, &adv_counts, express_event),
        0,
        true, // rigged
        true, //use_true_rn
    );
    let hist_counts: Vec<Vec<i64>> =
        histograms_for_all_costs(&cost_data_for_hist, bins, &budget_data[1]);
    CostToChanceOut {
        chance: final_chance,
        reasons: fail_count_to_string(typed_fail_counter_final, data_size),
        hist_counts,
        hist_mins: vec![0_i64; 7],
        hist_maxs: budget_data[1].clone(),
        upgrade_strings,
        juice_strings_armor,
        juice_strings_weapon,
        budgets_red_remaining: budgets[7],
        budgets_blue_remaining: budgets[8],
    }
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
        assert!(0.183 < _chance && _chance < 0.189);
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
        // assert!(0.172 < out.chance && out.chance < 0.178);
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
}
