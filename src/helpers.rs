use crate::constants::{
    get_adv_data_juice, get_event_modified_adv_unlock_cost, get_event_modified_armor_unlock_cost,
    get_event_modified_weapon_unlock_cost,
};
use crate::parser::Upgrade;
use crate::value_estimation::average_tap;

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

fn count_failure_naive(cost_data: &[Vec<i64>], budget_data: &[Vec<i64>]) -> Vec<i64> {
    let mut count: Vec<i64> = vec![0; budget_data.len()];
    for cost in cost_data.iter() {
        for (i, budget) in budget_data.iter().enumerate() {
            if !cost_passes_budget(cost, budget) {
                count[i] += 1;
            }
        }
    }
    count
}
/// Count, for each budget, how many costs fail it.
/// `cost_data` is a slice of N cost vectors; `budget_data` is a slice of M budget vectors
/// that are sorted element-wise ascending. Returns a Vec<i64> length M.
fn count_failure_ascending(cost_data: &[Vec<i64>], budget_data: &[Vec<i64>]) -> Vec<i64> {
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

pub fn count_failure(cost_data: &[Vec<i64>], budget_data: &[Vec<i64>], asc: bool) -> Vec<i64> {
    if asc {
        return count_failure_ascending(cost_data, budget_data);
    } else {
        return count_failure_naive(cost_data, budget_data);
    }
}

pub fn sort_by_indices<T>(upgrade_arr: &mut Vec<T>, mut indices: Vec<usize>) {
    for idx in 0..upgrade_arr.len() {
        if indices[idx] != idx {
            let mut current_idx: usize = idx;
            loop {
                let target_idx: usize = indices[current_idx];
                indices[current_idx] = current_idx;
                if indices[target_idx] == target_idx {
                    break;
                }
                upgrade_arr.swap(current_idx, target_idx);
                current_idx = target_idx;
            }
        }
    }
}
pub fn ticks_to_counts(ticks: Vec<Vec<bool>>) -> Vec<Vec<i64>> {
    // assume ticks is always 6 rows
    let cols: usize = ticks[0].len();
    let mut out: Vec<Vec<i64>> = vec![vec![0i64; cols]; 2];

    for i in 0..cols {
        // sum ticks[0..4][i]
        out[0][i] = (0..5)
            .map(|row: usize| if ticks[row][i] { 1 } else { 0 })
            .sum();

        // ticks[5][i] as 0/1
        out[1][i] = if ticks[5][i] { 1 } else { 0 };
    }

    out
}

/// Compute shard and silver unlock costs.
///
/// Parameters:
/// - `hone_counts`: &[Vec<i64>] (expected shape: [armor/weapon][index])
/// - `adv_counts`: &[Vec<i64>] (advanced counts)
/// - `express_event`: bool (whether express event is active)
///
/// Returns: (shard_unlock, silver_unlock)
pub fn calc_unlock(
    hone_counts: &[Vec<i64>],
    adv_counts: &[Vec<i64>],
    express_event: bool,
) -> Vec<i64> {
    let mut shard_unlock: i64 = 0;
    let mut silver_unlock: i64 = 0;

    // Get event-modified unlock costs
    let weapon_unlock_costs: [[i64; 25]; 2] = get_event_modified_weapon_unlock_cost(express_event);
    let armor_unlock_costs: [[i64; 25]; 2] = get_event_modified_armor_unlock_cost(express_event);
    let adv_unlock_costs: [[i64; 8]; 2] = get_event_modified_adv_unlock_cost(express_event);

    // Weapon unlocks: hone_counts[1][index]
    for (cost_type, element) in weapon_unlock_costs.iter().enumerate() {
        for (index, &cost) in element.iter().enumerate() {
            match cost_type {
                0 => shard_unlock += hone_counts[1][index] * cost,
                1 => silver_unlock += hone_counts[1][index] * cost,
                _ => {}
            }
        }
    }

    // Armor unlocks: hone_counts[0][index]
    for (cost_type, element) in armor_unlock_costs.iter().enumerate() {
        for (index, &cost) in element.iter().enumerate() {
            match cost_type {
                0 => shard_unlock += hone_counts[0][index] * cost,
                1 => silver_unlock += hone_counts[0][index] * cost,
                _ => {}
            }
        }
    }

    // Advanced unlocks: indexing alternates between adv_counts[0] and adv_counts[1]
    for (cost_type, element) in adv_unlock_costs.iter().enumerate() {
        for (index, &cost) in element.iter().enumerate() {
            if index % 2 == 1 {
                // odd index -> use adv_counts[0][(index-1)/2]
                let idx: usize = (index - 1) / 2;
                match cost_type {
                    0 => shard_unlock += adv_counts[0][idx] * cost,
                    1 => silver_unlock += adv_counts[0][idx] * cost,
                    _ => {}
                }
            } else {
                // even index -> use adv_counts[1][index/2]
                let idx: usize = index / 2;
                match cost_type {
                    0 => shard_unlock += adv_counts[1][idx] * cost,
                    1 => silver_unlock += adv_counts[1][idx] * cost,
                    _ => {}
                }
            }
        }
    }

    Vec::from([shard_unlock, silver_unlock])
}

// (maxroll) average, without the unlock costs
pub fn average_cost(upgrades: &[Upgrade]) -> Vec<f64> {
    let mut total_costs: Vec<f64> = vec![0.0; 7];

    for upgrade in upgrades {
        let avg_taps: f64 = average_tap(&upgrade.prob_dist, upgrade.tap_offset as f64);
        for cost_type in 0..7 {
            total_costs[cost_type] += upgrade.costs[cost_type] as f64 * (avg_taps as f64);
        }
    }

    total_costs
}

pub fn average_juice_cost(upgrades: &[Upgrade]) -> (i64, i64) {
    let mut total_red_cost: f64 = 0.0;
    let mut total_blue_cost: f64 = 0.0;
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

// pub fn myformat(mut f: f64) -> String {
//     f *= 100.0;
//     if f == 1.0_f64 {
//         return "100".to_owned();
//     }
//     let mut place: i32 = 1;

//     loop {
//         if (f - 1.0_f64).abs() >= 1.0 / 10f64.powi(place) {
//             return format!("{:.*}", place as usize, f);
//         }
//         if place >= 4 {
//             return "0".to_string();
//         }
//         place += 1;
//     }
// }

/// Compress consecutive duplicate strings into one with suffix ` xN`.
/// Example: ["A", "A", "A", "B", "C", "C"] -> ["A x3", "B", "C x2"].
pub fn compress_runs(strings: Vec<String>, no_x: bool) -> Vec<String> {
    if strings.is_empty() {
        return strings;
    }
    let mut out: Vec<String> = Vec::new();
    let mut prev: &str = &strings[0];
    let mut count: usize = 1;
    for s in strings.iter().skip(1) {
        if s == prev {
            count += 1;
        } else {
            if count > 1 {
                if no_x {
                    out.push(format!("{}", prev));
                } else {
                    out.push(format!("{} ({} such pieces)", prev, count));
                }
            } else {
                out.push(prev.to_string());
            }
            prev = s;
            count = 1;
        }
    }
    if count > 1 && !no_x {
        out.push(format!("{} ({} such pieces)", prev, count));
    } else {
        out.push(prev.to_string());
    }
    out
}
