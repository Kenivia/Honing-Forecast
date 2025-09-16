use crate::constants::LABELS;
use crate::helpers::{calc_unlock, myformat, sort_by_indices};
use crate::monte_carlos::monte_carlos_data;
use crate::parser::{parser, Upgrade};
use crate::value_estimation::{est_juice_value, est_special_honing_value, juice_to_array};

fn fail_count_to_string(typed_fail_counter: Vec<f32>, data_size: usize) -> String {
    let failed_labels: String;
    let mut failed_indices: Vec<usize> = (0..typed_fail_counter.len()).collect();
    failed_indices.sort_by(|&a, &b| typed_fail_counter[b].total_cmp(&typed_fail_counter[a]));
    let mut this_failed: Vec<String> = Vec::new();
    let mut displayed: bool = false;
    let mut spread_str: String;
    let mut spread_num: f64;
    for z in failed_indices {
        spread_num = typed_fail_counter[z] as f64 / data_size as f64;
        spread_str = myformat(spread_num);
        if spread_num >= 0.001 || !displayed {
            this_failed.push(LABELS[z].to_owned() + "(" + &spread_str + "%)");
        }
        displayed = true
    }
    if typed_fail_counter
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max)
        == 0.0_f32
    {
        failed_labels = "None".to_string();
    } else {
        failed_labels = this_failed.join("\n");
    }
    return failed_labels;
}

fn _cost_to_chance(
    upgrade_arr: &mut Vec<Upgrade>,
    actual_budgets: &Vec<i64>,
    unlock: &Vec<i64>,
    data_size: usize,
    mats_value_weight: &Vec<f32>,
) -> (f64, Vec<f32>) {
    // TODO implement tickbox & value in Ui just like maxroll
    // let mats_value_weight: Vec<f32> =;
    let value_per_special_leap: Vec<f32> =
        est_special_honing_value(upgrade_arr, &mats_value_weight);
    let mut special_indices: Vec<usize> = (0..value_per_special_leap.len()).collect();
    special_indices
        .sort_by(|&a, &b| value_per_special_leap[b].total_cmp(&value_per_special_leap[a]));
    // dbg!(&upgrade_arr[0]);
    sort_by_indices(upgrade_arr, special_indices.clone());

    let cost_data: Vec<Vec<i64>> = monte_carlos_data(
        data_size,
        upgrade_arr,
        unlock,
        actual_budgets[9],
        false,
        true, //use_true_rng
    );
    let mut typed_fail_counter: Vec<f32> = vec![0.0_f32; 7];
    let mut overall_fail_counter: i64 = 0;
    let mut failed;
    for (_trail_num, data) in cost_data.iter().enumerate() {
        failed = false;
        for cost_type in 0..7 {
            // Cost to chance does take silver into account
            if actual_budgets[cost_type as usize] < data[cost_type] {
                failed = true;
                typed_fail_counter[cost_type] += 1.0_f32;
            }
        }
        if failed {
            overall_fail_counter += 1;
        }
    }

    return (
        1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64,
        typed_fail_counter,
    );
}

pub fn cost_to_chance(
    hone_counts: &Vec<Vec<i64>>,
    actual_budgets: &Vec<i64>,
    adv_counts: &Vec<Vec<i64>>,
) -> (f64, String) {
    let data_size: usize = 100000;
    let adv_hone_strategy: String = String::from("No juice");
    let unlock_costs: Vec<i64> = calc_unlock(hone_counts, adv_counts);
    let mut override_special: Vec<i64> = actual_budgets.clone();
    override_special[9] = 0;
    let mut upgrade_arr: Vec<Upgrade> = parser(
        &hone_counts,
        &adv_counts,
        &adv_hone_strategy,
        &vec![1.0; 25],
        &vec![0.0; 25],
        &vec![0; 25],
    );
    let (_chance_1, typed_fail_counter_1): (f64, Vec<f32>) = _cost_to_chance(
        &mut upgrade_arr,
        &override_special,
        &unlock_costs,
        data_size,
        &vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    );
    est_juice_value(&mut upgrade_arr, &typed_fail_counter_1);
    juice_to_array(&mut upgrade_arr, actual_budgets[7], actual_budgets[8]);
    let (chance_2, typed_fail_counter_2): (f64, Vec<f32>) = _cost_to_chance(
        &mut upgrade_arr,
        actual_budgets,
        &unlock_costs,
        data_size,
        &typed_fail_counter_1,
    );

    return (
        chance_2,
        fail_count_to_string(typed_fail_counter_2, data_size),
    );
}
