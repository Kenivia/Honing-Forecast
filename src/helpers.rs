use crate::constants::{
    get_event_modified_adv_unlock_cost, get_event_modified_armor_unlock_cost,
    get_event_modified_weapon_unlock_cost,
};
use std::cmp::Ordering;
fn cmp_f64_ignore_nan(a: f64, b: f64) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => {
            // Normalize zeros so +0.0 == -0.0
            if a == 0.0 && b == 0.0 {
                Ordering::Equal
            } else {
                a.total_cmp(&b) // deterministic, total ordering
            }
        }
    }
}
pub fn argmax_with_priority(scores: &Vec<f64>, tie_priority: &[usize]) -> Option<usize> {
    let n = scores.len();
    if n == 0 {
        return None;
    }

    // Build priority_rank[index] = rank (lower = higher priority).
    // If some indices are missing from tie_priority, they get lower priority after the listed ones.
    let mut priority_rank = vec![usize::MAX; n];
    for (rank, &idx) in tie_priority.iter().enumerate() {
        if idx < n && priority_rank[idx] == usize::MAX {
            priority_rank[idx] = rank;
        }
    }
    for i in 0..n {
        if priority_rank[i] == usize::MAX {
            priority_rank[i] = tie_priority.len() + i;
        }
    }

    // Single pass to pick the best index
    let mut best = 0usize;
    for i in 1..n {
        match cmp_f64_ignore_nan(scores[i], scores[best]) {
            Ordering::Greater => best = i,
            Ordering::Equal => {
                if priority_rank[i] < priority_rank[best] {
                    best = i;
                }
            }
            Ordering::Less => {}
        }
    }
    Some(best)
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
    hone_counts: &Vec<Vec<i64>>,
    adv_counts: &Vec<Vec<i64>>,
    express_event: bool,
) -> Vec<i64> {
    let mut shard_unlock: i64 = 0;
    let mut silver_unlock: i64 = 0;

    // Get event-modified unlock costs
    let weapon_unlock_costs = get_event_modified_weapon_unlock_cost(express_event);
    let armor_unlock_costs = get_event_modified_armor_unlock_cost(express_event);
    let adv_unlock_costs = get_event_modified_adv_unlock_cost(express_event);

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
                let idx = (index - 1) / 2;
                match cost_type {
                    0 => shard_unlock += adv_counts[0][idx] * cost,
                    1 => silver_unlock += adv_counts[0][idx] * cost,
                    _ => {}
                }
            } else {
                // even index -> use adv_counts[1][index/2]
                let idx = index / 2;
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

pub fn myformat(mut f: f64) -> String {
    f *= 100.0;
    if f == 1.0_f64 {
        return "100".to_owned();
    }
    let mut place: i32 = 1;

    loop {
        if (f - 1.0_f64).abs() >= 1.0 / 10f64.powi(place) {
            return format!("{:.*}", place as usize, f);
        }
        if place >= 4 {
            return "0".to_string();
        }
        place += 1;
    }
}

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
                    out.push(format!("{} x{} times", prev, count));
                }
            } else {
                out.push(prev.to_string());
            }
            prev = s;
            count = 1;
        }
    }
    if count > 1 {
        out.push(format!("{} x{} times", prev, count));
    } else {
        out.push(prev.to_string());
    }
    out
}
