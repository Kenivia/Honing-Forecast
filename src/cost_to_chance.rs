use crate::constants::LABELS;
use crate::helpers::{myformat, unlock};
use crate::monte_carlos::monte_carlos_data;
use crate::parser::parser;
use crate::value_estimation::est_special_honing_value;

fn sort_by_indices(
    prob_dist_arr: &mut Vec<Vec<f32>>,
    hone_costs: &mut Vec<Vec<i64>>,
    special_costs: &mut Vec<i64>,
    mut indices: Vec<usize>,
) {
    for idx in 0..prob_dist_arr.len() {
        if indices[idx] != idx {
            let mut current_idx = idx;
            loop {
                let target_idx = indices[current_idx];
                indices[current_idx] = current_idx;
                if indices[target_idx] == target_idx {
                    break;
                }
                prob_dist_arr.swap(current_idx, target_idx);
                for i in 0..hone_costs.len() {
                    hone_costs[i].swap(current_idx, target_idx);
                }
                special_costs.swap(current_idx, target_idx);
                current_idx = target_idx;
            }
        }
    }
}
pub fn cost_to_chance(
    hone_counts: Vec<Vec<i64>>,
    actual_budgets: Vec<i64>,
    adv_counts: Vec<Vec<i64>>,
    adv_hone_strategy: String,
) -> (f64, String) {
    let (mut prob_dist_arr, mut hone_costs, adv_hone_chances, adv_hone_costs, mut special_costs): (
        Vec<Vec<f32>>,
        Vec<Vec<i64>>,
        Vec<Vec<f32>>,
        Vec<Vec<Vec<i64>>>,
        Vec<i64>,
    ) = parser(&hone_counts, &adv_counts, &adv_hone_strategy); // TODO implement tickbox & value in Ui just like maxroll
    let mats_value_in_gold: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 70.0, 1.0, 0.0];
    let value_per_special_leap: Vec<f32> = est_special_honing_value(
        &prob_dist_arr,
        &hone_costs,
        &special_costs,
        &mats_value_in_gold,
    );
    let mut special_indices: Vec<usize> = (0..value_per_special_leap.len()).collect();
    special_indices
        .sort_by(|&a, &b| value_per_special_leap[b].total_cmp(&value_per_special_leap[a]));
    sort_by_indices(
        &mut prob_dist_arr,
        &mut hone_costs,
        &mut special_costs,
        special_indices,
    );

    let cost_data: Vec<Vec<i64>> = monte_carlos_data(
        200000,
        &prob_dist_arr,
        &hone_costs,
        &adv_hone_chances,
        &adv_hone_costs,
        &unlock(&hone_counts, &adv_counts),
        actual_budgets[9],
        &special_costs,
        false,
        true, //use_true_rng
    );
    let mut typed_fail_counter: Vec<i64> = vec![0; 9];
    let mut overall_fail_counter: i64 = 0;
    let mut failed;
    for (_trail_num, data) in cost_data.iter().enumerate() {
        failed = false;
        for cost_type in 0..7 {
            // Cost to chance does take silver into account
            if actual_budgets[cost_type as usize] < data[cost_type] {
                failed = true;
                typed_fail_counter[cost_type] += 1;
            }
        }
        if failed {
            overall_fail_counter += 1;
        }
    }
    let failed_labels: String;
    let mut failed_indices: Vec<usize> = (0..typed_fail_counter.len()).collect();
    failed_indices.sort_by(|&a, &b| typed_fail_counter[b].cmp(&typed_fail_counter[a]));

    let mut this_failed: Vec<String> = Vec::new();
    let mut displayed: bool = false;
    let mut spread_str: String;
    let mut spread_num: f64;
    for z in failed_indices {
        spread_num = typed_fail_counter[z] as f64 / cost_data.len() as f64;
        spread_str = myformat(spread_num);
        if spread_num >= 0.1 || !displayed {
            this_failed.push(LABELS[z].to_owned() + "(" + &spread_str + "%)");
        }
        displayed = true
    }
    if *typed_fail_counter.iter().max().unwrap() == 0 {
        failed_labels = "None".to_string();
    } else {
        failed_labels = this_failed.join("\n");
    }
    return (
        1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64,
        failed_labels,
    );
}
