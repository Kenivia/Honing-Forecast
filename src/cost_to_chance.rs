use crate::constants::*;
use crate::helpers::{argmax_with_priority, calc_unlock, compress_runs, myformat, sort_by_indices};
use crate::histogram::histograms_for_all_costs;
use crate::monte_carlo::monte_carlo_data;
use crate::parser::{Upgrade, parser};
use crate::value_estimation::{est_juice_value, est_special_honing_value, juice_to_array};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct CostToChanceOut {
    pub chance: f64,
    pub reasons: Vec<String>,
    pub hist_counts: Vec<Vec<i64>>,      // 7 x num_bins
    pub hist_mins: Vec<i64>,             // 7
    pub hist_maxs: Vec<i64>,             // 7
    pub upgrade_strings: Vec<String>,    // ordered upgrade descriptions
    pub juice_order_armor: Vec<String>,  // e.g., ["+14 armor first 10 taps", ...]
    pub juice_order_weapon: Vec<String>, // e.g., ["+15 weapon first 6 taps", ...]
    pub budgets_red_remaining: i64,      // budgets[7]
    pub budgets_blue_remaining: i64,     // budgets[8]
}

#[inline]
fn zero_this_index(arr: &Vec<f64>, ind: usize) -> Vec<f64> {
    arr.iter()
        .enumerate()
        .map(|(i, &x)| if i == ind { 0.0_f64 } else { x })
        .collect()
}
#[inline]
fn to_onehot(arr: &Vec<f64>, ind: usize) -> Vec<f64> {
    arr.iter()
        .enumerate()
        .map(|(index, _)| if index == ind { 1.0_f64 } else { 0.0_f64 })
        .collect()
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

fn _cost_to_chance(
    upgrade_arr: &mut Vec<Upgrade>,
    actual_budgets: &Vec<i64>,
    unlock: &Vec<i64>,
    data_size: usize,
    weapon_value_weight: &Vec<f64>,
    armor_value_weight: &Vec<f64>,
    calibrating: bool,
    user_gave_weapon: bool,
    user_gave_armor: bool,
) -> (f64, Vec<f64>, Vec<String>) {
    // TODO implement tickbox & value in Ui just like maxroll
    // let mats_value_weight: Vec<f64> =;
    let value_per_special_leap: Vec<f64> = est_special_honing_value(
        upgrade_arr,
        &weapon_value_weight,
        &armor_value_weight,
        calibrating,
    );
    let mut special_indices: Vec<usize> = (0..value_per_special_leap.len()).collect();
    special_indices
        .sort_by(|&a, &b| value_per_special_leap[b].total_cmp(&value_per_special_leap[a]));
    sort_by_indices(upgrade_arr, special_indices.clone());
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        upgrade_arr,
        unlock,
        actual_budgets[9],
        false,
        false, //use_true_rng
    );
    let mut typed_fail_counter: Vec<f64> = vec![0.0_f64; 7];
    let mut overall_fail_counter: i64 = 0;
    let mut failed;
    for (_trail_num, data) in cost_data.iter().enumerate() {
        failed = false;
        for cost_type in 0..7 {
            // Cost to chance does take silver into account
            if actual_budgets[cost_type as usize] < data[cost_type] {
                failed = true;
                typed_fail_counter[cost_type] += 1.0_f64;
            }
        }
        if failed {
            overall_fail_counter += 1;
        }
    }
    let upgrade_strings = if calibrating {
        Vec::new()
    } else {
        compress_runs(
            extract_upgrade_strings(upgrade_arr, user_gave_weapon, user_gave_armor),
            true,
        )
    };

    return (
        1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64,
        typed_fail_counter,
        upgrade_strings,
    );
}

pub fn cost_to_chance(
    hone_counts: &Vec<Vec<i64>>,
    actual_budgets: &Vec<i64>,
    adv_counts: &Vec<Vec<i64>>,
    express_event: bool,
    hist_bins: usize,
    user_forced_mats_value: &Vec<f64>,
    adv_hone_strategy: String,
    data_size: usize,
) -> CostToChanceOut {
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
    let mut budgets: Vec<i64> = actual_budgets.clone();
    if adv_hone_strategy == "Juice on grace" {
        for upgrade in upgrade_arr.iter() {
            if upgrade.is_normal_honing {
                continue;
            }
            if upgrade.is_weapon {
                budgets[7] -= (get_adv_data_juice(upgrade.upgrade_plus_num as i64)
                    * upgrade.one_juice_cost as f64)
                    .round() as i64;
            } else {
                budgets[8] -= (get_adv_data_juice(upgrade.upgrade_plus_num as i64)
                    * upgrade.one_juice_cost as f64)
                    .round() as i64;
            }
        }
    }
    let mut override_special: Vec<i64> = budgets.clone();
    override_special[9] = 0;

    let user_armor_values: Vec<f64> = zero_this_index(user_forced_mats_value, 0);

    let valid_armor_values: bool =
        user_armor_values.iter().any(|&x| x != 0.0) || upgrade_arr.iter().all(|x| x.is_weapon);

    let user_weapon_values: Vec<f64> = zero_this_index(user_forced_mats_value, 1);
    // let user_weapon_values: Vec<f64> =
    let valid_weapon_values: bool =
        user_weapon_values.iter().any(|&x| x != 0.0) || upgrade_arr.iter().all(|x| !x.is_weapon);
    // let both_valid: bool = valid_armor_values && valid_weapon_values;

    // let auto_optimize: bool = user_gave_values
    // &&(actual_budgets[9] > 0 || actual_budgets[8] > 0 || actual_budgets[7] > 0);

    let (
        final_chance,
        typed_fail_counter_final,
        upgrade_strings,
        juice_order_armor,
        juice_order_weapon,
    ) = {
        // Use original calibration approach
        let (weapon_values, armor_values): (Vec<f64>, Vec<f64>) = if valid_weapon_values
            && valid_armor_values
        {
            (user_weapon_values.clone(), user_armor_values.clone())
        } else {
            let typed_fail_counter_1: Vec<f64> = _cost_to_chance(
                &mut upgrade_arr,
                &override_special,
                &unlock_costs,
                data_size / 5,
                &vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                &vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                true,
                valid_weapon_values,
                valid_armor_values,
            )
            .1;
            let weapon_bottleneck: usize =
                argmax_with_priority(&typed_fail_counter_1, &[6usize, 0, 4, 3, 7, 2, 1]).unwrap();
            let armor_bottleneck: usize =
                argmax_with_priority(&typed_fail_counter_1, &[6usize, 0, 4, 3, 7, 2, 1]).unwrap();
            (
                if valid_weapon_values {
                    user_weapon_values
                } else {
                    to_onehot(&typed_fail_counter_1, weapon_bottleneck)
                },
                if valid_armor_values {
                    user_armor_values
                } else {
                    to_onehot(&typed_fail_counter_1, armor_bottleneck)
                },
            )
        };
        // dbg!(&mats_value);
        est_juice_value(&mut upgrade_arr, &weapon_values, &armor_values);
        let (armor_strings, weapon_strings) = juice_to_array(
            &mut upgrade_arr,
            budgets[8],
            budgets[7],
            valid_weapon_values,
            valid_armor_values,
        );

        let (success_chance, typed_fail_counter, upgrade_string) = _cost_to_chance(
            &mut upgrade_arr,
            &budgets,
            &unlock_costs,
            data_size,
            &weapon_values,
            &armor_values,
            false,
            valid_weapon_values,
            valid_armor_values,
        );
        (
            success_chance,
            typed_fail_counter,
            upgrade_string,
            armor_strings,
            weapon_strings,
        )
    };

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
        juice_order_armor,
        juice_order_weapon,
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
        // assert!(0.183 < chance && chance < 0.189);
    }
    #[test]
    fn cost_to_chance_18_demo() {
        let out = cost_to_chance(
            &vec![
                (0..25)
                    .map(|i| if i == 19 || i == 20 || i == 21 { 5 } else { 0 })
                    .collect(),
                (0..25)
                    .map(|i| if i == 19 || i == 20 || i == 21 { 1 } else { 0 })
                    .collect(),
            ],
            &[
                431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 690, 420, 6767,
            ]
            .to_vec(),
            &vec![
                (0..4).map(|i| if i == 3 { 3 } else { 0 }).collect(),
                (0..4).map(|i| if i == 2 { 0 } else { 0 }).collect(),
            ],
            false,
            1000,
            &vec![0.0; 7],
            "No juice".to_owned(),
            100000,
        );
        println!("{:?}", out.chance);
        println!("{:?}", out.reasons);
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
