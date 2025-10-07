use crate::helpers::transpose_vec_of_vecs;
use crate::parser::Upgrade;
use rand::Rng;
use rand::prelude::*;
use std::cmp::min;
// use vose_alias::VoseAlias;

// generates budget from 0th to 99th percentile(or 99.9th more specifically), intentionally leaving out
// pity because there is a distinction between pity and the unluckiest draw in 100k.
// (where 0th percentile is the luckiest draws of each dimension)
pub fn generate_budget_data(
    cost_data: &[Vec<i64>],
    budget_size: usize,
    data_size: usize,
) -> Vec<Vec<i64>> {
    let mut transposed_cost_data: Vec<Vec<i64>> = transpose_vec_of_vecs(&cost_data);
    for transposed_row in &mut transposed_cost_data {
        transposed_row.sort_unstable();
    }
    let gap_size: Vec<f64> = transposed_cost_data
        .iter()
        .map(|row| (row[row.len() - 1] - row[0]) as f64 / budget_size as f64)
        .collect();
    let mut budget_data: Vec<Vec<i64>> = vec![vec![0; 9]; budget_size];

    for (cost_type, transposed_row) in transposed_cost_data.iter().enumerate() {
        let mut j: usize = 0;
        let mut k: usize = 0;
        let mut cur_count: usize = 0;
        loop {
            if transposed_row[j]
                >= (transposed_row[0] as f64 + gap_size[cost_type] * k as f64).floor() as i64
            {
                budget_data[k][cost_type] = transposed_row[cur_count];
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

    budget_data
}

#[inline]
fn calc_failure_lim(avail_special: i64, cost: i64) -> i64 {
    if cost <= 0 {
        return 0;
    }
    (avail_special / cost).max(0)
}

fn tap_map_generator<R: Rng>(count_limit: usize, prob_dist: &[f64], rng: &mut R) -> Vec<usize> {
    let mut tap_map: Vec<usize> = vec![0usize; count_limit];

    let mut assigned: usize = 0;
    let mut cum: f64 = 0.0;
    for (i, &p) in prob_dist.iter().enumerate() {
        cum += p;
        if cum > 1.0 {
            cum = 1.0;
        }
        let exact_target: f64 = cum * (count_limit as f64);
        let target: f64 = exact_target.max(assigned as f64);
        let frac: f64 = target - target.floor();
        let mut cur_samples = target.floor() as usize;
        if frac > 0.0 && rng.random_bool(frac) {
            cur_samples += 1;
        }
        if cur_samples > assigned {
            let to_assign: usize = min(cur_samples - assigned, count_limit - assigned);
            let end: usize = assigned + to_assign;
            for dest in assigned..end {
                tap_map[dest] = i;
            }
            assigned = end;
            if assigned >= count_limit {
                break;
            }
        }
    }
    if assigned < count_limit {
        let fill_idx: usize = prob_dist.len().saturating_sub(1);
        for dest in assigned..count_limit {
            tap_map[dest] = fill_idx;
        }
    }

    tap_map.shuffle(rng);
    tap_map
}

/// stochastic rounding helper (juice)
#[inline]
fn round_juice<R: Rng>(this_juice_cost: f64, rng: &mut R) -> i64 {
    let base: i64 = this_juice_cost.floor() as i64;
    let frac: f64 = this_juice_cost.fract();
    base + if frac > 0.0 && rng.random_bool(frac) {
        1
    } else {
        0
    }
}

/// Sample from a geometric distribution with parameter `p = base_chance`
/// producing k in 0..=max_taps such that k == max_taps represents the tail.
/// Uses the identity k = floor( ln(U) / ln(q) ) (q = 1-p) and truncates to max_taps.
/// Handles edge cases p <= 0 and p >= 1.
#[inline]
fn sample_truncated_geometric<R: Rng + ?Sized>(p: f64, max_taps: i64, rng: &mut R) -> usize {
    if max_taps < 0 {
        return 0usize;
    }
    if p <= 0.0 {
        return max_taps as usize; // degenerate -> always tail
    }
    if p >= 1.0 {
        return 0usize; // succeed immediately
    }
    let q: f64 = 1.0 - p;
    let u: f64 = rng.random_range(0.0..1.0);
    // ln(u)/ln(q) >= 0 (both negative logs) gives k (0-based)
    let k: i64 = (u.ln() / q.ln()).floor() as i64;
    let k: i64 = if k < 0 { 0 } else { k };
    if k > max_taps {
        max_taps as usize
    } else {
        k as usize
    }
}

pub fn monte_carlo_data<R: Rng>(
    data_size: usize,
    upgrade_arr: &[Upgrade],
    unlock_costs: &[i64],
    avail_special: i64,
    mut rng: &mut R,
) -> Vec<Vec<i64>> {
    debug_assert!(unlock_costs.len() == 2);

    let mut cost_data: Vec<Vec<i64>> = vec![vec![0i64; 9]; data_size];
    // let mut rng = rand::rng();

    // special budgets & pass counts
    let mut special_budgets: Vec<i64> = vec![avail_special; data_size];
    let mut special_pass_arr: Vec<usize> = vec![0usize; data_size];

    if avail_special > 0 {
        for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
            if upgrade.is_normal_honing {
                // compute limit (max_taps)
                let limit: i64 = calc_failure_lim(avail_special, upgrade.special_cost);
                // We no longer build a WalkerTable; instead sample directly from truncated geometric
                for trial in 0..data_size {
                    if special_budgets[trial] <= 0 {
                        continue;
                    }
                    let k: usize = sample_truncated_geometric(upgrade.base_chance, limit, &mut rng);
                    let rolled_special_cost: i64 = (k as i64 + 1) * upgrade.special_cost;
                    special_budgets[trial] -= rolled_special_cost;
                    if special_budgets[trial] > 0 {
                        special_pass_arr[trial] += 1;
                        debug_assert!(special_pass_arr[trial] == upgrade_index + 1); // this breaks when there's an advanced honing upgrade sorted before normal honing upgrade in upgrade_arr
                    }
                }
            }
        }
    }

    // Latin-hypercube sampling path unchanged
    for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
        let tap_map: Vec<usize> = tap_map_generator(data_size, &upgrade.prob_dist, rng);
        for trial_num in 0..data_size {
            if upgrade_index < special_pass_arr[trial_num] {
                continue;
            }
            let rolled_tap: usize = tap_map[trial_num];
            for cost_type in 0..7 {
                cost_data[trial_num][cost_type] +=
                    upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
            }
            if !upgrade.is_normal_honing {
                let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
                cost_data[trial_num][juice_ind] +=
                    round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
            }
        }
    }

    // apply unlock costs
    for row in cost_data.iter_mut() {
        row[3] += unlock_costs[0];
        row[6] += unlock_costs[1];
    }

    cost_data
}

pub fn get_top_bottom(upgrade_arr: &[Upgrade], unlock_costs: &[i64]) -> Vec<Vec<i64>> {
    debug_assert!(unlock_costs.len() == 2);
    const DATA_SIZE: usize = 2;
    let mut cost_data: Vec<Vec<i64>> = vec![vec![0i64; 9]; DATA_SIZE];
    let mut rng = rand::rng();
    for upgrade in upgrade_arr.iter() {
        let pd_len: f64 = upgrade.prob_dist_len.saturating_sub(1) as f64;
        for trial_num in 0..DATA_SIZE {
            let rolled_tap =
                ((pd_len * (trial_num) as f64) / (DATA_SIZE as f64 - 1.0)).floor() as usize;
            for cost_type in 0..7 {
                cost_data[trial_num][cost_type] +=
                    upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
            }
            if !upgrade.is_normal_honing {
                cost_data[trial_num][if upgrade.is_weapon { 7 } else { 8 }] +=
                    round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
            }
        }
    }
    for row in cost_data.iter_mut() {
        row[3] += unlock_costs[0];
        row[6] += unlock_costs[1];
    }
    cost_data
}
