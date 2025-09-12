use crate::constants::LABELS;
use crate::helpers::{myformat, unlock};
use crate::monte_carlos::monte_carlos_data;
use crate::parser::parser;

pub fn cost_to_chance(
    hone_counts: Vec<Vec<i64>>,
    actual_budgets: Vec<i64>,
    adv_counts: Vec<Vec<i64>>,
    adv_hone_strategy: String,
) -> (f64, String) {
    let (prob_dist_arr, hone_costs, adv_hone_chances, adv_hone_costs, _tags): (
        Vec<Vec<f32>>,
        Vec<Vec<i64>>,
        Vec<Vec<f32>>,
        Vec<Vec<Vec<i64>>>,
        Vec<String>,
    ) = parser(&hone_counts, &adv_counts, &adv_hone_strategy);

    let cost_data: Vec<Vec<i64>> = monte_carlos_data(
        200000,
        &prob_dist_arr,
        &hone_costs,
        &adv_hone_chances,
        &adv_hone_costs,
        &unlock(&hone_counts, &adv_counts),
        false,
        false, //use_true_rng
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
