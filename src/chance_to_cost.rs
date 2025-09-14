use crate::helpers::unlock;
use crate::monte_carlos::monte_carlos_data;
use crate::parser::parser;
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
    let n = cost_data.len();
    let m = budget_data.len();
    if n == 0 || m == 0 {
        return vec![0i64; m];
    }

    // Difference array approach (length m+1 to mark ranges)
    let mut diffs: Vec<i64> = vec![0i64; m + 1];

    for cost in cost_data.iter() {
        let cs = cost.as_slice();

        // Binary search for the first budget index that the cost *passes*.
        // If none pass, first_pass_index stays m (meaning it fails all budgets).
        let mut low: usize = 0;
        let mut high: usize = (m as usize) - 1;
        let mut first_pass_index: usize = m;

        while low <= high {
            let mid = ((low + high) >> 1) as usize;
            let mid_usize = mid as usize;

            if cost_passes_budget(cs, budget_data[mid_usize].as_slice()) {
                first_pass_index = mid_usize;
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
    let mut counts = vec![0i64; m];
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
pub fn chance_to_cost(
    hone_counts: Vec<Vec<i64>>,
    adv_counts: Vec<Vec<i64>>,
    desired_chance: f32,
    adv_hone_strategy: String,
) -> (Vec<i64>, f32) {
    let cost_size: usize = 200000;
    let budget_size: usize = 1000;
    let (prob_dist_arr, hone_costs, adv_hone_chances, adv_hone_costs, special_costs): (
        Vec<Vec<f32>>,
        Vec<Vec<i64>>,
        Vec<Vec<f32>>,
        Vec<Vec<Vec<i64>>>,
        Vec<i64>,
    ) = parser(&hone_counts, &adv_counts, &adv_hone_strategy);
    let cost_data: Vec<Vec<i64>> = monte_carlos_data(
        cost_size,
        &prob_dist_arr,
        &hone_costs,
        &adv_hone_chances,
        &adv_hone_costs,
        &unlock(&hone_counts, &adv_counts),
        0,
        &special_costs,
        false, //use_true_rng
        false, // rigged
    );
    let budget_data: Vec<Vec<i64>> = monte_carlos_data(
        budget_size,
        &prob_dist_arr,
        &hone_costs,
        &adv_hone_chances,
        &adv_hone_costs,
        &unlock(&hone_counts, &adv_counts),
        0,
        &special_costs,
        true,  // rigged
        false, //use_true_rn
    );
    let failure_counts: Vec<i64> = count_failure(&cost_data, &budget_data, true);

    let k_i64: i64 = ((1.0f32 - desired_chance / 100f32) * (cost_size as f32)).floor() as i64;
    let diffs: Vec<i64> = failure_counts
        .iter()
        .map(|&ci| (ci - k_i64).abs())
        .collect();

    let mut sorted_indices: Vec<usize> = (0..budget_size as usize).collect();
    sorted_indices.sort_by_key(|&i| (diffs[i], i));
    let best_budget: Vec<i64> = budget_data[sorted_indices[0]].clone();
    (
        best_budget,
        (1 as f32 - (failure_counts[sorted_indices[0]] as f32 / cost_data.len() as f32))
            * 100 as f32,
    )
}
