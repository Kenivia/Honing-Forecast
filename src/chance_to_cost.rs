use crate::constants::*;
use crate::helpers::{calc_unlock, count_failure};
use crate::histogram::{histograms_for_all_costs, transpose_vec_of_vecs};
use crate::monte_carlo::monte_carlo_data;
use crate::parser::{Upgrade, parser};
use crate::value_estimation::average_tap;

use serde::Serialize;
// use web_sys::console;
// use crate::{constants::*, cost_to_chance};

fn find_best_budget_for_this_chance(
    desired_chance: f64,
    cost_size: usize,
    budget_size: usize,
    failure_counts: &Vec<i64>,
    budget_data: &Vec<Vec<i64>>,
) -> (Vec<i64>, f64) {
    let k_i64: i64 = ((1.0f64 - desired_chance / 100f64) * (cost_size as f64)).floor() as i64;
    let k_i64_budget: i64 =
        ((cost_size as f64 - k_i64 as f64) / cost_size as f64 * budget_size as f64).round() as i64;
    let diffs: Vec<i64> = failure_counts
        .iter()
        .map(|&ci| (ci - k_i64).abs())
        .collect();

    let mut sorted_indices: Vec<usize> = (0..budget_data.len()).collect();
    sorted_indices.sort_by_key(|&i| (diffs[i], (k_i64_budget - i as i64).abs())); //TODO MAKE THIS A MIN FUNCTION
    let best_budget: Vec<i64> = budget_data[sorted_indices[0]].clone();
    let best_chance: f64 =
        (1 as f64 - (failure_counts[sorted_indices[0]] as f64 / cost_size as f64)) * 100 as f64;
    (best_budget, best_chance)
}

/// Calculate the total cost for each of the 7 main cost types across all upgrades.
/// Returns a vector of length 7 containing the total cost for each cost type.\
pub fn average_cost(upgrades: &Vec<Upgrade>) -> Vec<f64> {
    let mut total_costs: Vec<f64> = vec![0.0; 7];

    for upgrade in upgrades {
        let avg_taps = average_tap(&upgrade.prob_dist, upgrade.tap_offset as f64);
        for cost_type in 0..7 {
            total_costs[cost_type] += upgrade.costs[cost_type] as f64 * (avg_taps as f64);
        }
    }

    total_costs
}

/// Calculate the average juice cost across all upgrades.
/// Returns a tuple (red_juice_cost, blue_juice_cost) representing the average cost per upgrade.
/// Values are rounded to the nearest integer.
pub fn average_juice_cost(upgrades: &Vec<Upgrade>) -> (i64, i64) {
    let mut total_red_cost = 0.0;
    let mut total_blue_cost = 0.0;
    let mut red_count: i64 = 0;
    let mut blue_count: i64 = 0;

    for upgrade in upgrades {
        if upgrade.is_normal_honing {
            continue;
        }

        // Use the proper juice cost calculation from get_adv_data_juice
        let avg_juice_cost =
            get_adv_data_juice(upgrade.upgrade_plus_num as i64) * upgrade.one_juice_cost as f64;

        if upgrade.is_weapon {
            total_red_cost += avg_juice_cost;
            red_count += 1;
        } else {
            total_blue_cost += avg_juice_cost;
            blue_count += 1;
        }
    }

    (
        if red_count > 0 {
            total_red_cost.round() as i64
        } else {
            0
        },
        if blue_count > 0 {
            total_blue_cost.round() as i64
        } else {
            0
        },
    )
}

#[derive(Serialize)]
pub struct ChanceToCostOut {
    pub hist_counts: Vec<Vec<i64>>, // 7 x num_bins
    pub hist_mins: Vec<i64>,        // 7
    pub hist_maxs: Vec<i64>,        // 7
    pub hundred_budgets: Vec<Vec<i64>>,
    pub hundred_chances: Vec<f64>,
}

pub fn chance_to_cost(
    hone_counts: Vec<Vec<i64>>,
    adv_counts: Vec<Vec<i64>>,
    adv_hone_strategy: String,
    express_event: bool,
    hist_bins: usize,
    data_size: usize,
) -> ChanceToCostOut {
    let budget_size: usize = 1000;
    let aritsan_arr: Vec<f64>;
    if express_event {
        aritsan_arr = EVENT_ARTISAN_MULTIPLIER.to_vec();
    } else {
        aritsan_arr = vec![1.0; 25];
    }
    let upgrade_arr: Vec<Upgrade> = parser(
        &hone_counts,
        &adv_counts,
        &adv_hone_strategy,
        &aritsan_arr,
        &vec![0.0; 25],
        &vec![0; 25],
        express_event,
    );

    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &upgrade_arr,
        &calc_unlock(&hone_counts, &adv_counts, express_event),
        0,
        false, //rigged
        false, // use_true_rng
    );

    let top_bottom: Vec<Vec<i64>> = monte_carlo_data(
        2,
        &upgrade_arr,
        &calc_unlock(&hone_counts, &adv_counts, express_event),
        0,
        true, // rigged
        true, //use_true_rn
    );

    let intermediate_hist: Vec<Vec<i64>> =
        histograms_for_all_costs(&cost_data, budget_size, &top_bottom[1]);
    let mut cum_hist_counts: Vec<Vec<usize>> = vec![vec![0; budget_size]; 7];

    for cost_type in 0..7 {
        cum_hist_counts[cost_type][0] = intermediate_hist[cost_type][0] as usize;
        for j in 1..budget_size {
            cum_hist_counts[cost_type][j] =
                cum_hist_counts[cost_type][j - 1] + intermediate_hist[cost_type][j] as usize;
        }
    }

    let mut transposed_cost_data: Vec<Vec<i64>> = transpose_vec_of_vecs(&cost_data);
    for cost_type in 0..7 {
        transposed_cost_data[cost_type].sort_unstable();
    }
    let mut gap_size: Vec<f64> = vec![0.0; 7];
    for cost_type in 0..7 {
        gap_size[cost_type] = (transposed_cost_data[cost_type][cost_data.len() - 1]
            - transposed_cost_data[cost_type][0]) as f64
            / budget_size as f64;
    }
    // let mut cur_counts: Vec<usize> = vec![0; 7];
    let mut budget_data: Vec<Vec<i64>> = vec![vec![0; 9]; budget_size];

    for cost_type in 0..7 {
        let mut j: usize = 0;
        let mut k: usize = 0;
        let mut cur_count: usize = 0;
        loop {
            // println!("{:?}", k);
            if transposed_cost_data[cost_type][j]
                >= (transposed_cost_data[cost_type][0] as f64 + gap_size[cost_type] * k as f64)
                    .floor() as i64
            {
                budget_data[k][cost_type] += transposed_cost_data[cost_type][cur_count];
                cur_count += (data_size as f64 / budget_size as f64).round() as usize;
                k += 1;
            } else {
                j += 1;
            }

            if k >= budget_size {
                break;
            }
        }
    }

    budget_data.push(top_bottom[1].clone());
    if adv_hone_strategy == "Juice on grace" {
        let (avg_red_juice, avg_blue_juice) = average_juice_cost(&upgrade_arr);

        for i in 0..(budget_size + 1) {
            budget_data[i][7] = avg_red_juice;
            budget_data[i][8] = avg_blue_juice;
        }
    }
    let failure_counts: Vec<i64> = count_failure(&cost_data, &budget_data, true);

    let (hundred_budgets, hundred_chances): (Vec<Vec<i64>>, Vec<f64>) = (0..101)
        .into_iter()
        .map(|x| {
            find_best_budget_for_this_chance(
                x as f64,
                data_size,
                budget_size,
                &failure_counts,
                &budget_data,
            )
        })
        .collect();
    ChanceToCostOut {
        hundred_budgets,
        hundred_chances,
        hist_counts: histograms_for_all_costs(&cost_data, hist_bins, &top_bottom[1]),
        hist_mins: vec![0_i64; 7],
        hist_maxs: top_bottom[1].clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chance_to_cost_stress() {
        let out = chance_to_cost(
            vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()],
            vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()],
            "No juice".to_owned(),
            false,
            1000,
            10000,
        );
        // println!("hundred_budgets = {:?}", out.hundred_budgets);
        println!("hundred_chances = {:?}", out.hundred_chances);
        // println!("hist_mins = {:?}", out.hist_mins);
        // println!("hist_maxs = {:?}", out.hist_maxs);
    }

    #[test]
    fn test_average_cost() {
        use crate::parser::parser;
        let hone_counts = vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
        let adv_counts = vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()];
        let upgrade_arr = parser(
            &hone_counts,
            &adv_counts,
            &"No juice".to_owned(),
            &vec![1.0; 25],
            &vec![0.0; 25],
            &vec![0; 25],
            false,
        );

        let total_costs = average_cost(&upgrade_arr);
        assert_eq!(total_costs.len(), 7);
        // Verify that all costs are non-negative (since we're summing positive values)
        for cost in &total_costs {
            assert!(*cost >= 0.0);
        }
        println!("Total costs: {:?}", total_costs);
    }
}
