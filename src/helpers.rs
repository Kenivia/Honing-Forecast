use crate::constants::{ADV_HONE_UNLOCK, NORMAL_HONE_ARMOR_UNLOCK, NORMAL_HONE_WEAPON_UNLOCK};

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
/// - `weap_unlock`, `armor_unlock`: &[Vec<i64>] (outer index = cost_type)
/// - `adv_counts`: &[Vec<i64>] (advanced counts)
/// - `adv_unlock`: Option<&[Vec<i64>]> (if Some, same structure as other unlock arrays)
///
/// Returns: (shard_unlock, silver_unlock)
pub fn calc_unlock(hone_counts: &Vec<Vec<i64>>, adv_counts: &Vec<Vec<i64>>) -> Vec<i64> {
    let mut shard_unlock: i64 = 0;
    let mut silver_unlock: i64 = 0;

    // Weapon unlocks: hone_counts[1][index]
    for (cost_type, element) in NORMAL_HONE_WEAPON_UNLOCK.iter().enumerate() {
        for (index, &cost) in element.iter().enumerate() {
            match cost_type {
                0 => shard_unlock += hone_counts[1][index] * cost,
                1 => silver_unlock += hone_counts[1][index] * cost,
                _ => {}
            }
        }
    }

    // Armor unlocks: hone_counts[0][index]
    for (cost_type, element) in NORMAL_HONE_ARMOR_UNLOCK.iter().enumerate() {
        for (index, &cost) in element.iter().enumerate() {
            match cost_type {
                0 => shard_unlock += hone_counts[0][index] * cost,
                1 => silver_unlock += hone_counts[0][index] * cost,
                _ => {}
            }
        }
    }

    // Advanced unlocks: indexing alternates between adv_counts[0] and adv_counts[1]

    for (cost_type, element) in ADV_HONE_UNLOCK.iter().enumerate() {
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
    let mut place: i32 = 0;

    loop {
        if f >= 1.0 / 10f64.powi(place) {
            return format!("{:.*}", place as usize, f);
        }
        if place >= 4 {
            return "0".to_string();
        }
        place += 1;
    }
}
