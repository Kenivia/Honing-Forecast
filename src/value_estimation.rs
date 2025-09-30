// use crate::constants::*;

use crate::helpers::compress_runs;
use crate::parser::{Upgrade, probability_distribution};

#[cfg(debug_assertions)]
use assert_float_eq::assert_f64_near;

pub fn average_tap(prob_dist: &[f64], offset: f64) -> f64 {
    let mut out: f64 = 0.0_f64;
    // println!("{:?}", prob_dist[start_index..].iter().sum::<f64>() as f64);
    #[cfg(debug_assertions)]
    assert_f64_near!(prob_dist.iter().sum::<f64>() as f64, 1.0 as f64, 40);
    // let sum_before_start: f64 = prob_dist[..start_index].iter().sum();
    for (index, item) in prob_dist.iter().enumerate() {
        out += item * (index as f64 + offset);
    }
    out
}
fn truncated_average_tap(prob_dist: &[f64], offset: f64, truncate: usize) -> f64 {
    let mut out: f64 = 0.0_f64;
    // println!("{:?}", prob_dist[start_index..].iter().sum::<f64>() as f64);
    #[cfg(debug_assertions)]
    assert_f64_near!(prob_dist.iter().sum::<f64>() as f64, 1.0 as f64, 40);
    // let sum_before_start: f64 = prob_dist[..start_index].iter().sum();
    for (index, item) in prob_dist.iter().enumerate() {
        if index < truncate - 1 {
            out += 0.0
        } else {
            out += item * (index as f64 + offset);
        }
    }
    out + (truncate - 1) as f64 + offset
}

// fn average_tap_with_change(prob_dist: &Vec<f64>, change_index: usize, change_value: f64) -> f64 {
//     let mut out: f64 = 0.0_f64;
//     // println!("{:?}", prob_dist[start_index..].iter().sum::<f64>() as f64);
//     #[cfg(debug_assertions)]
//     assert_f64_near!(prob_dist.iter().sum::<f64>() as f64, 1.0 as f64, 10);
//     // let sum_before_start: f64 = prob_dist[..start_index].iter().sum();
//     for (index, item) in prob_dist.iter().enumerate() {
//         out += (item + ((change_index == index) as i64 as f64) * change_value as f64)
//             * (index + 1) as f64;
//     }
//     out
// }
fn average_value(upgrade: &Upgrade, mats_value: &[f64], average: f64) -> f64 {
    let mut this_sum = 0.0_f64;
    for cost_type in 0..7 {
        this_sum += mats_value[cost_type] * average * upgrade.costs[cost_type] as f64;
    }
    this_sum as f64
}

fn est_juice_value_for_prob_dist(
    upgrade: &Upgrade,
    mat_values: &[f64],
    prob_dist: &[f64],
    extra_count: usize,
) -> f64 {
    // if upgrade.failure_raw_delta < 0 {
    average_value(
        upgrade,
        mat_values,
        truncated_average_tap(prob_dist, upgrade.tap_offset as f64, extra_count),
        //     )
        // } else {
        //     upgrade.failure_raw_delta as f64
        //         * upgrade.base_chance
        //         * truncated_average_tap(prob_dist, upgrade.tap_offset as f64, extra_count)
        // })
    ) / upgrade.one_juice_cost as f64
}
pub fn est_special_honing_value(
    upgrade_arr: &mut Vec<Upgrade>,
    mats_values: &[f64],
) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::with_capacity(upgrade_arr.len());
    let mut average: f64;
    let cost_type_count: usize = 7;
    let mut special_value: f64;
    // let mut is_valid: bool;
    debug_assert!(mats_values.len() == cost_type_count);
    for (_, upgrade) in upgrade_arr.iter_mut().enumerate() {
        if upgrade.is_normal_honing {
            average = average_tap(&upgrade.original_prob_dist, upgrade.tap_offset as f64);
            // is_valid = if upgrade.is_weapon {
            //     user_gave_weapon
            // } else {
            //     user_gave_armor
            // };
            // debug_assert!(upgrade.failure_raw_delta < 0);
            special_value = upgrade.base_chance * average_value(upgrade, mats_values, average)
                / upgrade.special_cost as f64;

            out.push(special_value);

            upgrade.special_value = special_value
        } else {
            out.push(0.0_f64);
        }
    }

    out
}

pub fn est_juice_value(upgrade_arr: &mut Vec<Upgrade>, mat_values: &[f64]) {
    let mut this_sum: Vec<f64>;
    // let mut prev_cost: f64;
    // let mut next_cost: f64;
    let mut extra_count: usize;
    // let mut cur_prob_dist: Vec<f64>;

    for (_, upgrade) in upgrade_arr.iter_mut().enumerate() {
        if !upgrade.is_normal_honing || upgrade.upgrade_plus_num <= 2 {
            continue;
        }
        this_sum = Vec::with_capacity(upgrade.prob_dist_len);
        let mut prev_prob_dist: Vec<f64> = probability_distribution(
            upgrade.base_chance,
            upgrade.artisan_rate,
            upgrade.base_chance, // will need to change for books
            0,
        );
        extra_count = 1;

        loop {
            if extra_count >= prev_prob_dist.len() {
                // trying to juice the pity tap
                break;
            }
            let next_prob_dist: Vec<f64> = probability_distribution(
                upgrade.base_chance,
                upgrade.artisan_rate,
                upgrade.base_chance, // will need to change for books
                extra_count,
            );

            let value_with_juice: f64 =
                est_juice_value_for_prob_dist(upgrade, mat_values, &next_prob_dist, extra_count);

            let value_without_juice: f64 =
                est_juice_value_for_prob_dist(upgrade, mat_values, &prev_prob_dist, extra_count);

            this_sum.push(value_without_juice - value_with_juice);
            prev_prob_dist = next_prob_dist;
            extra_count += 1;
        }
        upgrade.juice_values = this_sum;
    }
}

pub fn juice_to_array(
    upgrade_arr: &mut Vec<Upgrade>,
    blue_juice: i64,
    red_juice: i64,

    user_gave_armor: bool,
    user_gave_weapon: bool,
) -> (Vec<String>, Vec<String>) {
    // Armor uses blue juice (is_weapon == false), Weapon uses red juice (is_weapon == true)
    let armor_pairs = _juice_to_array(upgrade_arr, false, blue_juice);
    let weapon_pairs = _juice_to_array(upgrade_arr, true, red_juice);

    // Convert pairs of (plus_num, taps) to human-readable strings, sorted by plus_num asc
    let mut armor_sorted = armor_pairs;
    armor_sorted.sort_by_key(|&(plus, _, _, low)| (plus, -low.round() as i64));
    let armor_strings: Vec<String> = compress_runs(
        armor_sorted
            .into_iter()
            .map(|(plus, taps, high, low)| {
                if !user_gave_armor {
                    format!("+{} armor, First {} taps", plus + 1, taps,)
                } else {
                    if high == low {
                        format!("+{} armor, First {} taps, {}g", plus + 1, taps, high,)
                    } else {
                        format!(
                            "+{} armor, first {} taps, {}g to {}g",
                            plus + 1,
                            taps,
                            high,
                            low
                        )
                    }
                }
            })
            .collect(),
        false,
    );

    let mut weapon_sorted = weapon_pairs;
    weapon_sorted.sort_by_key(|&(plus, _, _, low)| (plus, -low.round() as i64));
    let weapon_strings: Vec<String> = compress_runs(
        weapon_sorted
            .into_iter()
            .map(|(plus, taps, high, low)| {
                if !user_gave_weapon {
                    format!("+{} weapon, First {} taps", plus + 1, taps,)
                } else {
                    if high == low {
                        format!("+{} weapon, First {} taps, {}g", plus + 1, taps, high,)
                    } else {
                        format!(
                            "+{} weapon, First {} taps, {}g to {}g",
                            plus + 1,
                            taps,
                            high,
                            low
                        )
                    }
                }
            })
            .collect(),
        false,
    );

    (armor_strings, weapon_strings)
}

fn _juice_to_array(
    upgrade_arr: &mut Vec<Upgrade>,
    is_weapon: bool,
    mut juice: i64,
) -> Vec<(usize, usize, f64, f64)> {
    let mut cur_upgrade: &mut Upgrade;
    let mut idxs: Vec<usize>;
    let mut max_value_index: usize;
    let mut cur_extras: Vec<usize> = vec![0; upgrade_arr.len()];
    // let mut _max_extra_index: usize;
    loop {
        // max_extra = *cur_extras.iter().max().unwrap();
        // _max_extra_index = cur_extras
        //     .iter()
        //     .enumerate()
        //     .max_by_key(|&(_, val)| val)
        //     .map(|(i, _)| i)
        //     .unwrap_or(0);
        idxs = (0..upgrade_arr.len())
            .filter(|&x| {
                upgrade_arr[x].is_normal_honing
                    && upgrade_arr[x].is_weapon == is_weapon
                    && upgrade_arr[x].one_juice_cost <= juice
                    && cur_extras[x] < upgrade_arr[x].juice_values.len()
            })
            .collect();
        if idxs.is_empty() {
            break;
        }
        max_value_index = idxs
            .into_iter()
            .max_by(|&a, &b| {
                upgrade_arr[a].juice_values[cur_extras[a]]
                    .total_cmp(&upgrade_arr[b].juice_values[cur_extras[b]])
            })
            .unwrap();

        cur_extras[max_value_index] += 1;
        cur_upgrade = &mut upgrade_arr[max_value_index];
        cur_upgrade.prob_dist = probability_distribution(
            cur_upgrade.base_chance,
            cur_upgrade.artisan_rate,
            cur_upgrade.base_chance, // will need to change for books
            cur_extras[max_value_index],
        );
        juice -= cur_upgrade.one_juice_cost;
    }
    // Extract (plus_num, taps_used) only for selected type and where taps_used > 0
    let mut out: Vec<(usize, usize, f64, f64)> = Vec::new();
    for (i, upgrade) in upgrade_arr.iter().enumerate() {
        if upgrade.is_normal_honing && upgrade.is_weapon == is_weapon {
            let taps_used = cur_extras[i];
            if taps_used > 0 {
                out.push((
                    upgrade.upgrade_plus_num,
                    taps_used,
                    upgrade
                        .juice_values
                        .clone()
                        .into_iter()
                        .enumerate()
                        .filter(|(i, _)| *i < taps_used)
                        .max_by(|(_, a), (_, b)| a.total_cmp(b))
                        .unwrap_or((0, 0.0_f64))
                        .1
                        .round(),
                    upgrade
                        .juice_values
                        .clone()
                        .into_iter()
                        .enumerate()
                        .filter(|(i, _)| *i < taps_used)
                        .min_by(|(_, a), (_, b)| a.total_cmp(b))
                        .unwrap_or((0, 0.0_f64))
                        .1
                        .round(),
                ));
            }
        }
    }
    out
}
