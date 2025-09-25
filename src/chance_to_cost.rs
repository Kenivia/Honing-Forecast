use crate::constants::*;
use crate::helpers::calc_unlock;
use crate::histogram::{histograms_for_all_costs, transpose_vec_of_vecs};
use crate::monte_carlo::monte_carlo_data;
use crate::parser::{Upgrade, parser};
use crate::value_estimation::average_tap;

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
    sorted_indices.sort_by_key(|&i| (diffs[i], (k_i64_budget - i as i64).abs()));
    let best_budget: Vec<i64> = budget_data[sorted_indices[0]].clone();
    let best_chance: f64 =
        (1 as f64 - (failure_counts[sorted_indices[0]] as f64 / cost_size as f64)) * 100 as f64;
    (best_budget, best_chance)
}

/// Given two non-increasing vectors `a` and `b` (both `&Vec<i64>`),
/// for each element a[i] find an index j into `b` such that
/// b[j] is the closest-in-value element to a[i].
///
/// Tie-breaker:
///  - if two b-values are equally close in value, choose the j with smaller |j - i|.
///  - if still tied, choose the smaller j.
///
/// Returns a Vec<usize> of length a.len() with the chosen indices into `b`.
/// Given two non-increasing vectors `a` and `b` (both `&Vec<i64>`),
/// for each element a[i] find an index j into `b` such that
/// b[j] is the closest-in-value element to a[i].
///
/// Tie-breaker:
///  - if two b-values are equally close in value, choose the j with smaller |j - i|.
///  - if still tied, choose the smaller j.
///
/// Returns a Vec<usize> of length a.len() with the chosen indices into `b`.
pub fn argmin_indices_closest(a: &Vec<i64>, b: &Vec<i64>) -> Vec<usize> {
    let n = a.len();
    let m = b.len();
    let mut out = Vec::with_capacity(n);

    if m == 0 {
        // No candidates in b; nothing we can do — return zeros (or panic depending on desired behaviour)
        // Here we choose to panic because an index into an empty b is not well-defined.
        panic!("vec_2 (b) must not be empty");
    }

    // Helper: find smallest index `k` in [0, m) such that b[k] <= x.
    // If all b[k] > x, returns m.
    let first_leq = |x: i64| -> usize {
        let mut lo: usize = 0;
        let mut hi: usize = m;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if b[mid] <= x {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    };

    // Helper: find smallest index `k` in [0, m) such that b[k] < x.
    // If no such index, returns m.
    let first_less = |x: i64| -> usize {
        let mut lo: usize = 0;
        let mut hi: usize = m;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if b[mid] < x {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    };

    for (i, &val) in a.iter().enumerate() {
        // idx is first index where b[idx] <= val
        let idx = first_leq(val);

        // If there is an exact match b[k] == val, find leftmost and rightmost occurrences
        if idx < m && b[idx] == val {
            // leftmost is idx (first <= val, and equals val -> leftmost equal)
            let left = idx;
            // rightmost: first index where b < val (if any) minus one
            let right_exclusive = first_less(val);
            let right = if right_exclusive == 0 {
                0
            } else {
                right_exclusive - 1
            };

            // choose nearest index in [left..=right] to i
            let chosen = if i <= left {
                left
            } else if i >= right {
                right
            } else {
                // i is inside [left, right], the exact i may be outside range of b length;
                // but i is usize comparing with right which is usize
                // choose i clamped into [left..=right]
                // i is index into `a`, but might be >= m; clamp to nearest in run
                let i_clamped = std::cmp::min(i, right);
                std::cmp::max(left, i_clamped)
            };

            out.push(chosen);
            continue;
        }

        // No exact match. Candidate indices are:
        // - idx (first <= val), if idx < m
        // - idx - 1 (last > val), if idx > 0
        let mut candidates = Vec::with_capacity(2);
        if idx < m {
            candidates.push(idx);
        }
        if idx > 0 {
            candidates.push(idx - 1);
        }

        // Safety: candidates must be non-empty because m > 0. But check nonetheless.
        if candidates.is_empty() {
            out.push(0);
            continue;
        }

        // Choose candidate minimizing value distance; tie-break on index distance to i; final tie-breaker smaller index
        let mut best = candidates[0];
        let mut best_val_diff = (b[best] - val).abs();
        let mut best_idx_diff = if best > i { best - i } else { i - best };

        for &c in &candidates[1..] {
            let val_diff = (b[c] - val).abs();
            if val_diff < best_val_diff {
                best = c;
                best_val_diff = val_diff;
                best_idx_diff = if c > i { c - i } else { i - c };
            } else if val_diff == best_val_diff {
                let idx_diff = if c > i { c - i } else { i - c };
                if idx_diff < best_idx_diff {
                    best = c;
                    best_idx_diff = idx_diff;
                } else if idx_diff == best_idx_diff {
                    if c < best {
                        best = c;
                    }
                }
            }
        }

        out.push(best);
    }

    out
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
        && cost[6] <= budget[6]
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

    let budget_data_for_juice: Vec<Vec<i64>> = monte_carlo_data(
        budget_size,
        &upgrade_arr,
        &calc_unlock(&hone_counts, &adv_counts, express_event),
        0,
        true, // rigged
        true, //use_true_rng
    );
    let failure_counts_1: Vec<i64> = count_failure(&cost_data, &budget_data, true);
    let failure_counts_2: Vec<i64> = count_failure(&cost_data, &budget_data_for_juice, true);
    let closest_indices: Vec<usize> = argmin_indices_closest(&failure_counts_1, &failure_counts_2);

    for i in 0..budget_size {
        budget_data[i][7] = budget_data_for_juice[closest_indices[i]][7];
        budget_data[i][8] = budget_data_for_juice[closest_indices[i]][8];
    }
    budget_data.push(top_bottom[1].clone());
    let (hundred_budgets, hundred_chances): (Vec<Vec<i64>>, Vec<f64>) = (0..101)
        .into_iter()
        .map(|x| {
            find_best_budget_for_this_chance(
                x as f64,
                data_size,
                budget_size,
                &failure_counts_1,
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
    #[test]
    fn boundaries() {
        let a = vec![200, 10];
        let b = vec![100, 50, 0];
        // For 200: all b < 200 -> closest is b[0]
        // For 10: candidates 50 (idx1) and 0 (idx2): 0 is closer (10 vs 0 diff10, vs 40)
        let res = argmin_indices_closest(&a, &b);
        assert_eq!(res, vec![0usize, 2usize]);
    }
    #[test]
    fn basic_examples() {
        let a = vec![95, 85, 70];
        let b = vec![100, 90, 90, 80, 60];
        // For a[0]=95 closest in b is 100 (idx0) vs 90 (idx1): 100 is closer.
        // For a[1]=85 two candidates 90 (idx1 or idx2) and 80 (idx3). 90 and 80 both dist 5 -> choose
        // the 90 with index closer to i=1 (idx1 is distance 0, idx2 distance 1) -> idx1.
        // For a[2]=70 closest is 60 (idx4).
        let res = argmin_indices_closest(&a, &b);
        assert_eq!(res, vec![0usize, 1usize, 4usize]);
    }

    #[test]
    fn equal_values_choose_nearest_index() {
        let a = vec![90];
        let b = vec![100, 90, 90, 90, 80];
        // exact matches at indices 1..3. a index i=0, nearest index among 1..3 is 1.
        let res = argmin_indices_closest(&a, &b);
        assert_eq!(res, vec![1usize]);
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
    //     println!("best_chance = {:?}", out.best_chance);
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
    //     println!("best_chance = {:?}", out.best_chance);
    //     println!("hist_mins = {:?}", out.hist_mins);
    //     println!("hist_maxs = {:?}", out.hist_maxs);
    // }
}
