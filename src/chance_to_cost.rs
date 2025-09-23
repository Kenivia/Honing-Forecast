use crate::constants::*;
use crate::helpers::calc_unlock;
use crate::histogram::{histograms_for_all_costs, transpose_vec_of_vecs};
use crate::monte_carlo::monte_carlo_data;
use crate::parser::{Upgrade, parser};
// use web_sys::console;
// use crate::{constants::*, cost_to_chance};

fn count_failure_naive(cost_data: &Vec<Vec<i64>>, budget_data: &Vec<Vec<i64>>) -> Vec<i64> {
    let mut count: Vec<i64> = vec![0; budget_data.len()];
    for cost in cost_data.iter() {
        for (i, budget) in budget_data.iter().enumerate() {
            if !cost_passes_budget(cost, budget)
            // no silver comparison
            {
                count[i] += 1;
            }
        }
    }
    count
}

#[inline]
fn cost_passes_budget(cost: &[i64], budget: &[i64]) -> bool {
    cost[0] <= budget[0]
        && cost[1] <= budget[1]
        && cost[2] <= budget[2]
        && cost[3] <= budget[3]
        && cost[4] <= budget[4]
        && cost[5] <= budget[5]
}

/// Count, for each budget, how many costs fail it.
/// `cost_data` is a slice of N cost vectors; `budget_data` is a slice of M budget vectors
/// that are sorted element-wise ascending. Returns a Vec<i64> length M.
fn count_failure_ascending(cost_data: &Vec<Vec<i64>>, budget_data: &Vec<Vec<i64>>) -> Vec<i64> {
    let n: usize = cost_data.len();
    let m: usize = budget_data.len();
    if n == 0 || m == 0 {
        return vec![0i64; m];
    }

    // Difference array approach (length m+1 to mark ranges)
    let mut diffs: Vec<i64> = vec![0i64; m + 1];

    for cost in cost_data.iter() {
        let cs: &[i64] = cost.as_slice();

        // Binary search for the first budget index that the cost *passes*.
        // If none pass, first_pass_index stays m (meaning it fails all budgets).
        let mut low: usize = 0;
        let mut high: usize = (m as usize) - 1;
        let mut first_pass_index: usize = m;

        while low <= high {
            let mid: usize = ((low + high) >> 1) as usize;

            if cost_passes_budget(cs, budget_data[mid].as_slice()) || mid == 0 {
                first_pass_index = mid;
                if mid == 0 {
                    break;
                }
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }

        // If first_pass_index > 0 then this cost fails budgets [0 .. first_pass_index-1].
        if first_pass_index > 0 {
            diffs[0] += 1;
            diffs[first_pass_index] -= 1;
        }
    }

    // Build prefix-sum to obtain counts per budget index.
    let mut counts: Vec<i64> = vec![0i64; m];
    counts[0] = diffs[0];
    for i in 1..m {
        counts[i] = counts[i - 1] + diffs[i];
    }

    counts
}

fn count_failure(cost_data: &Vec<Vec<i64>>, budget_data: &Vec<Vec<i64>>, asc: bool) -> Vec<i64> {
    if asc {
        return count_failure_ascending(cost_data, budget_data);
    } else {
        return count_failure_naive(cost_data, budget_data);
    }
}
use serde::Serialize;

#[derive(Serialize)]
pub struct ChanceToCostOut {
    pub best_budget: Vec<i64>,
    pub actual_prob: f64,
    pub hist_counts: Vec<Vec<i64>>, // 7 x num_bins
    pub hist_mins: Vec<i64>,        // 7
    pub hist_maxs: Vec<i64>,        // 7
}

pub fn chance_to_cost(
    hone_counts: Vec<Vec<i64>>,
    adv_counts: Vec<Vec<i64>>,
    desired_chance: f64,
    adv_hone_strategy: String,
    express_event: bool,
    hist_bins: usize,
    data_size: usize,
) -> ChanceToCostOut {
    let cost_size: usize = data_size;
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
        cost_size,
        &upgrade_arr,
        &calc_unlock(&hone_counts, &adv_counts),
        0,
        false, //rigged
        false, // use_true_rng
    );

    let top_bottom: Vec<Vec<i64>> = monte_carlo_data(
        2,
        &upgrade_arr,
        &calc_unlock(&hone_counts, &adv_counts),
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
    let mut budget_data: Vec<Vec<i64>> = vec![vec![0; 7]; budget_size];

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

    let failure_counts: Vec<i64> = count_failure(&cost_data, &budget_data, false); // not sure if can use asc? just to be safe keeping it naive for now

    let k_i64: i64 = ((1.0f64 - desired_chance / 100f64) * (cost_size as f64)).floor() as i64;
    let k_i64_budget: i64 =
        ((cost_size as f64 - k_i64 as f64) / cost_size as f64 * budget_size as f64).round() as i64;
    let diffs: Vec<i64> = failure_counts
        .iter()
        .map(|&ci| (ci - k_i64).abs())
        .collect();

    let mut sorted_indices: Vec<usize> = (0..budget_data.len()).collect();
    sorted_indices.sort_by_key(|&i| (diffs[i], (k_i64_budget - i as i64).abs()));
    let best_budget: Vec<i64> = budget_data[sorted_indices[0]].clone();
    ChanceToCostOut {
        best_budget,
        actual_prob: (1 as f64
            - (failure_counts[sorted_indices[0]] as f64 / cost_data.len() as f64))
            * 100 as f64,
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
            100.0,
            "No juice".to_owned(),
            false,
            1000,
            10000,
        );
        println!("best_budget = {:?}", out.best_budget);
        println!("actual_prob = {:?}", out.actual_prob);
        println!("hist_mins = {:?}", out.hist_mins);
        println!("hist_maxs = {:?}", out.hist_maxs);
    }

    // #[test]
    // fn chance_to_cost_18_demo() {
    //     let hone_counts = vec![
    //         (0..25)
    //             .map(|i| if i == 19 || i == 20 || i == 21 { 5 } else { 0 })
    //             .collect(),
    //         (0..25)
    //             .map(|i| if i == 19 || i == 20 || i == 21 { 1 } else { 0 })
    //             .collect(),
    //     ];
    //     let adv_counts = vec![
    //         (0..4).map(|i| if i == 2 { 5 } else { 0 }).collect(),
    //         (0..4).map(|i| if i == 2 { 1 } else { 0 }).collect(),
    //     ];
    //     let out = chance_to_cost(
    //         hone_counts,
    //         adv_counts,
    //         69.0,
    //         "No juice".to_owned(),
    //         false,
    //         1000,
    //         100000,
    //     );
    //     println!("best_budget = {:?}", out.best_budget);
    //     println!("actual_prob = {:?}", out.actual_prob);
    //     println!("hist_mins = {:?}", out.hist_mins);
    //     println!("hist_maxs = {:?}", out.hist_maxs);
    // }

    // #[test]
    // fn chance_to_cost_53_adv_armor_40() {
    //     let hone_counts = vec![(0..25).map(|_| 0).collect(), (0..25).map(|_| 0).collect()];
    //     let adv_counts = vec![
    //         (0..4).map(|x| if x == 3 { 5 } else { 0 }).collect(),
    //         (0..4).map(|x| if x == 3 { 1 } else { 0 }).collect(),
    //     ];
    //     let out = chance_to_cost(
    //         hone_counts,
    //         adv_counts,
    //         53.0,
    //         "No juice".to_owned(),
    //         false,
    //         1000,
    //         100000,
    //     );
    //     println!("best_budget = {:?}", out.best_budget);
    //     println!("actual_prob = {:?}", out.actual_prob);
    //     println!("hist_mins = {:?}", out.hist_mins);
    //     println!("hist_maxs = {:?}", out.hist_maxs);
    // }
}
