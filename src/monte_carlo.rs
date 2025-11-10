use crate::helpers::round_juice;
use crate::parser::Upgrade;
use crate::value_estimation::juice_value_now;
use rand::Rng;
use rand::prelude::*;
use std::cmp::min;

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

/// Sample from a geometric distribution with parameter `p = base_chance`
/// producing k in `0..=max_taps` such that k == `max_taps` represents the tail.
/// Uses the identity k = floor( ln(U) / ln(q) ) (q = 1-p) and truncates to `max_taps`.
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
    let k: i64 = u.log(q).floor() as i64;
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
    input_budgets: &[i64],
    mut rng: &mut R,
) -> Vec<[i64; 9]> {
    debug_assert!(unlock_costs.len() == 2);
    let avail_special: i64 = input_budgets[9];
    let mut cost_data: Vec<[i64; 9]> = vec![[0i64; 9]; data_size];
    let mut special_budgets: Vec<i64> = vec![avail_special; data_size];
    let mut special_pass_arr: Vec<usize> = vec![0usize; data_size];

    // pre-generate an array of how many pieces were free tapped, mostly because it just makes things easier
    // i doubt it's actually faster
    if avail_special > 0 {
        for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
            if upgrade.is_normal_honing {
                let limit: i64 = calc_failure_lim(avail_special, upgrade.special_cost);
                for trial in 0..data_size {
                    if special_budgets[trial] <= 0 {
                        continue;
                    }
                    let taps_used: usize =
                        sample_truncated_geometric(upgrade.base_chance, limit, &mut rng);
                    let rolled_special_cost: i64 = (taps_used as i64 + 1) * upgrade.special_cost;
                    special_budgets[trial] -= rolled_special_cost;
                    if special_budgets[trial] > 0 {
                        special_pass_arr[trial] += 1;
                        debug_assert!(special_pass_arr[trial] == upgrade_index + 1);
                        // this assertion is triggered when there's an advanced honing upgrade sorted before normal honing upgrade in upgrade_arr
                        // which shouldnt happen
                    }
                }
            }
        }
    }

    // juicy part, we use tap_map_generator to get the excpected distribution and just multiply it by data_size
    // e.g. if the distribution is 0.2,0.1,0.7 and data size is 100, we generate [20 zeros, 10 ones, 70 twos] etc
    // it is then shuffled before being added to cost_data
    // This supposedly reduces variance by quite a big amount
    // we COULD also do this for free taps but um WIP ig
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
            // This is completely useless right now I believe but in the future juice optimization will need this i think
            if !upgrade.is_normal_honing {
                let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
                cost_data[trial_num][juice_ind] +=
                    round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
            }
        }
    }

    // unlock costs
    for row in &mut cost_data {
        row[3] += unlock_costs[0];
        row[6] += unlock_costs[1];
    }

    cost_data
}
