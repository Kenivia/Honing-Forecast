use crate::parser::Upgrade;
use rand::Rng;
use rand::prelude::*;
use std::cmp::min;
// use vose_alias::VoseAlias;

/// floor(avail_special / cost)
#[inline]
fn calc_failure_lim(avail_special: i64, cost: i64) -> i64 {
    if cost <= 0 {
        return 0;
    }
    (avail_special / cost).max(0)
}

/// tap_map_generator (latin-hypercube path) — unchanged behaviour but generic over Rng.
fn tap_map_generator<R: Rng + ?Sized>(
    count_limit: usize,
    prob_dist: &[f64],
    rng: &mut R,
) -> Vec<usize> {
    let mut tap_map = vec![0usize; count_limit];

    let mut assigned: usize = 0;
    let mut cum: f64 = 0.0;
    for (i, &p) in prob_dist.iter().enumerate() {
        cum += p;
        if cum > 1.0 {
            cum = 1.0;
        }
        let exact_target = cum * (count_limit as f64);
        let target = exact_target.max(assigned as f64);
        let frac = target - target.floor();
        let mut cur_samples = target.floor() as usize;
        if frac > 0.0 && rng.random_bool(frac) {
            cur_samples += 1;
        }
        if cur_samples > assigned {
            let to_assign = min(cur_samples - assigned, count_limit - assigned);
            let end = assigned + to_assign;
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
        let fill_idx = prob_dist.len().saturating_sub(1);
        for dest in assigned..count_limit {
            tap_map[dest] = fill_idx;
        }
    }

    tap_map.shuffle(rng);
    tap_map
}

/// stochastic rounding helper (juice)
#[inline]
fn round_juice<R: Rng + ?Sized>(this_juice_cost: f64, rng: &mut R) -> i64 {
    let base = this_juice_cost.floor() as i64;
    let frac = this_juice_cost.fract();
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
    let q = 1.0 - p;
    let u: f64 = rng.random_range(0.0..1.0);
    // ln(u)/ln(q) >= 0 (both negative logs) gives k (0-based)
    let k = (u.ln() / q.ln()).floor() as i64;
    let k = if k < 0 { 0 } else { k };
    if k > max_taps {
        max_taps as usize
    } else {
        k as usize
    }
}

/// Main monte carlo function — drop in replacement. Aliasing is used if available.
/// WalkerTable usage removed; special sampling uses the truncated geometric sampler above.
pub fn monte_carlo_data(
    data_size: usize,
    upgrade_arr: &[Upgrade],
    unlock_costs: &[i64],
    avail_special: i64,
    rigged: bool,
    use_true_rng: bool,
) -> Vec<Vec<i64>> {
    let unlock_costs = if unlock_costs.len() >= 2 {
        unlock_costs
    } else {
        &[0, 0]
    };

    let mut cost_data: Vec<Vec<i64>> = vec![vec![0i64; 9]; data_size];
    let mut rng = rand::rng();

    if rigged {
        for upgrade in upgrade_arr.iter() {
            let pd_len = upgrade.prob_dist_len.saturating_sub(1) as f64;
            for trial_num in 0..data_size {
                let rolled_tap =
                    ((pd_len * (trial_num + 1) as f64) / (data_size as f64)).floor() as usize;
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
    } else {
        // special budgets & pass counts
        let mut special_budgets: Vec<i64> = vec![avail_special; data_size];
        let mut special_pass_arr: Vec<usize> = vec![0usize; data_size];

        if avail_special > 0 {
            for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
                if upgrade.is_normal_honing {
                    // compute limit (max_taps)
                    let limit = calc_failure_lim(avail_special, upgrade.special_cost);
                    // We no longer build a WalkerTable; instead sample directly from truncated geometric
                    for trial in 0..data_size {
                        if special_budgets[trial] <= 0 {
                            continue;
                        }
                        let k = sample_truncated_geometric(upgrade.base_chance, limit, &mut rng);
                        // original code used (index + 1) * special_cost when using WalkerTable
                        let rolled_special_cost = (k as i64 + 1) * upgrade.special_cost;
                        special_budgets[trial] -= rolled_special_cost;
                        if special_budgets[trial] > 0 {
                            special_pass_arr[trial] += 1;
                            debug_assert!(special_pass_arr[trial] == upgrade_index + 1);
                        }
                    }
                }
            }
        }

        if use_true_rng {
            for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
                // Check if alias table is built
                let has_alias = upgrade.alias_table.is_some();

                for trial in 0..data_size {
                    if upgrade_index < special_pass_arr[trial] {
                        continue;
                    }
                    let rolled_tap: usize = if has_alias {
                        upgrade.alias_table.as_ref().unwrap().sample()
                    } else {
                        // fallback: do manual inverse-CDF using prob_dist
                        // This fallback is rare if you precomputed alias_table in Upgrade::new_*
                        let u = rng.random_range(0.0..1.0);
                        let mut cum = 0.0;
                        let mut idx = upgrade.prob_dist.len().saturating_sub(1);
                        for (i, &p) in upgrade.prob_dist.iter().enumerate() {
                            cum += p;
                            if u < cum {
                                idx = i;
                                break;
                            }
                        }
                        idx
                    };

                    for cost_type in 0..7 {
                        cost_data[trial][cost_type] +=
                            upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
                    }
                    if !upgrade.is_normal_honing {
                        let juice_ind = if upgrade.is_weapon { 7 } else { 8 };
                        cost_data[trial][juice_ind] +=
                            round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
                    }
                }
            }
        } else {
            // Latin-hypercube sampling path unchanged
            for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
                let tap_map = tap_map_generator(data_size, &upgrade.prob_dist, &mut rng);
                for trial_num in 0..data_size {
                    if upgrade_index < special_pass_arr[trial_num] {
                        continue;
                    }
                    let rolled_tap = tap_map[trial_num];
                    for cost_type in 0..7 {
                        cost_data[trial_num][cost_type] +=
                            upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
                    }
                    if !upgrade.is_normal_honing {
                        let juice_ind = if upgrade.is_weapon { 7 } else { 8 };
                        cost_data[trial_num][juice_ind] +=
                            round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
                    }
                }
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
