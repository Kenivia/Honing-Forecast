use crate::constants::{
    FLOAT_TOL, get_event_modified_adv_unlock_cost, get_event_modified_armor_unlock_cost,
    get_event_modified_weapon_unlock_cost,
};

use crate::upgrade::Upgrade;

// use crate::value_estimation::average_tap;

use rand::Rng;

pub trait TripletIterator<'a>: Iterator<Item = &'a Vec<(f64, f64)>> {}
impl<'a, T> TripletIterator<'a> for T where T: Iterator<Item = &'a Vec<(f64, f64)>> {}

pub fn find_non_zero_min_vec(support_arr: &[Vec<f64>], prob_dist_arr: &[Vec<f64>]) -> f64 {
    support_arr
        .iter()
        .enumerate()
        .map(|(u_index, x)| {
            x.iter()
                .enumerate()
                .find(|(index, _)| prob_dist_arr[u_index][*index] > FLOAT_TOL)
                .unwrap_or((0, &0.0))
                .1
        })
        .sum()
}

pub fn eqv_gold_per_tap(upgrade: &Upgrade, price_arr: &[f64]) -> f64 {
    // a bit redundent but whatever
    let mut c: f64 = 0.0;
    for i in 0..7 {
        c += price_arr[i] * upgrade.costs[i] as f64;
    }
    c
}

#[inline]
pub fn round_juice<R: Rng>(this_juice_cost: f64, rng: &mut R) -> i64 {
    let base: i64 = this_juice_cost.floor() as i64;
    let frac: f64 = this_juice_cost.fract();
    base + i64::from(frac > 0.0 && rng.random_bool(frac))
}

pub fn get_percentile_window(p: f64, cost_data: &[[i64; 9]]) -> &[[i64; 9]] {
    let n = cost_data.len();

    // Calculate the lower bound: (p - 0.005) * n, floored
    let lower_p = p - 0.005;
    let mut lower_idx = if lower_p <= 0.0 {
        0
    } else {
        let idx = (lower_p * n as f64).floor() as usize;
        idx.min(n - 1)
    };

    // Calculate the upper bound: (p + 0.005) * n, ceiled
    let upper_p = p + 0.005;
    let mut upper_idx = if upper_p >= 1.0 {
        n - 1
    } else {
        let idx = (upper_p * n as f64).ceil() as usize;
        idx.min(n - 1)
    };

    if lower_idx == 0 {
        upper_idx = 0;
    }
    if upper_idx == n - 1 {
        lower_idx = n - 1;
    }

    // Return the slice (upper_idx is inclusive, so we add 1)
    &cost_data[lower_idx..=upper_idx]
}

pub fn get_count(counts: Option<Vec<Vec<i64>>>, ticks: Option<Vec<Vec<bool>>>) -> Vec<Vec<i64>> {
    if counts.is_some() {
        counts.unwrap()
    } else if ticks.is_some() {
        ticks_to_counts(ticks.unwrap())
    } else {
        panic!("Either normal_counts or normal_hone_ticks must be provided");
    }
}

pub fn compute_gold_cost_from_raw(
    needed: &[i64],
    input_budget_no_gold: &[i64],
    price_arr: &[f64],
) -> f64 {
    let mut c: f64 = 0f64;
    for i in 0..7 {
        let val = (needed[i] - input_budget_no_gold[i]).max(0) as f64
            - (input_budget_no_gold[i] - needed[i]).max(0) as f64 * 1.0; // this constant will be customizable 
        c += price_arr[i] * val;
    }
    c
}

pub fn compute_eqv_gold_values(input_budget: &[i64], price_arr: &[f64]) -> f64 {
    let mut c: f64 = 0f64;
    for i in 0..7 {
        c += price_arr[i] * input_budget[i] as f64;
    }
    c
}

pub fn transpose_vec_of_vecs(matrix: &[[i64; 9]]) -> Vec<Vec<i64>> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return Vec::new();
    }

    let rows: usize = matrix.len();
    let cols: usize = matrix[0].len();

    let mut transposed: Vec<Vec<i64>> = vec![vec![0_i64; rows]; cols]; // Initialize with default values

    for r in 0..rows {
        for c in 0..cols {
            transposed[c][r] = matrix[r][c];
        }
    }
    transposed
}

pub fn sort_by_indices<T>(arr: &mut [T], mut indices: Vec<usize>) {
    for idx in 0..arr.len() {
        if indices[idx] != idx {
            let mut current_idx: usize = idx;
            loop {
                let target_idx: usize = indices[current_idx];
                indices[current_idx] = current_idx;
                if indices[target_idx] == target_idx {
                    break;
                }
                arr.swap(current_idx, target_idx);
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
        out[0][i] = (0..5).map(|row: usize| i64::from(ticks[row][i])).sum();

        // ticks[5][i] as 0/1
        out[1][i] = i64::from(ticks[5][i]);
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
/// Returns: (`shard_unlock`, `silver_unlock`)
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

    vec![shard_unlock, silver_unlock]
}

// // (maxroll) average, without the unlock costs
// pub fn average_cost(upgrades: &[Upgrade]) -> Vec<f64> {
//     let mut total_costs: Vec<f64> = vec![0.0; 7];

//     for upgrade in upgrades {
//         let avg_taps: f64 = average_tap(&upgrade.prob_dist, upgrade.tap_offset as f64);
//         for cost_type in 0..7 {
//             total_costs[cost_type] += upgrade.costs[cost_type] as f64 * avg_taps;
//         }
//     }

//     total_costs
// }

// pub fn average_juice_cost(upgrades: &[Upgrade]) -> (i64, i64) {
//     let mut total_red_cost: f64 = 0.0;
//     let mut total_blue_cost: f64 = 0.0;
//     let mut red_count: i64 = 0;
//     let mut blue_count: i64 = 0;

//     for upgrade in upgrades {
//         if upgrade.is_normal_honing {
//             continue;
//         }

//         // Use the proper juice cost calculation from get_adv_data_juice
//         let avg_juice_cost =
//             get_adv_data_juice(upgrade.upgrade_index as i64) * upgrade.one_juice_cost as f64;

//         if upgrade.is_weapon {
//             total_red_cost += avg_juice_cost;
//             red_count += 1;
//         } else {
//             total_blue_cost += avg_juice_cost;
//             blue_count += 1;
//         }
//     }

//     (
//         if red_count > 0 {
//             total_red_cost.round() as i64
//         } else {
//             0
//         },
//         if blue_count > 0 {
//             total_blue_cost.round() as i64
//         } else {
//             0
//         },
//     )
// }

/// Compress consecutive duplicate strings into one with suffix ` xN`.
/// Example: ["A", "A", "A", "B", "C", "C"] -> ["A x3", "B", "C x2"].
pub fn compress_runs(strings: Vec<String>, no_x: bool, mut out: Vec<String>) -> Vec<String> {
    if strings.is_empty() {
        return out;
    }
    // let mut out: Vec<String> = out;
    let mut prev: &str = &strings[0];
    let mut count: usize = 1;
    for s in strings.iter().skip(1) {
        if s == prev {
            count += 1;
        } else {
            if count > 1 {
                if no_x {
                    out.push(prev.to_string());
                } else {
                    out.push(format!("{prev} ({count} such pieces)"));
                }
            } else {
                out.push(prev.to_string());
            }
            prev = s;
            count = 1;
        }
    }
    if count > 1 && !no_x {
        out.push(format!("{prev} ({count} such pieces)"));
    } else {
        out.push(prev.to_string());
    }
    out
}
