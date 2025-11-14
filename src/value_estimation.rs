use crate::helpers::{compress_runs, generate_first_deltas};
use crate::parser::{Upgrade, probability_distribution};
// use crate::success_analysis::compute_all_gold_costs;

#[cfg(debug_assertions)]
use assert_float_eq::assert_float_absolute_eq;

// pub fn explore_one(
//     decision: &(Vec<i64>, Vec<i64>),
//     input_budgets: &[i64],
//     prep_outputs: &PreparationOutputs,
//     data_size: usize,
// ) -> Vec<f64> {
//     let mut combined_prob_dist: Vec<(f64, f64)> = Vec::new();
//     for taps_iter in decision_space_iterator(
//         prep_outputs
//             .upgrade_arr
//             .iter()
//             .map(|x| x.prob_dist_len as i64)
//             .collect(),
//     ) {
//         let taps: Vec<i64> = taps_iter.0;
//         let mut this_cost: Vec<i64> = vec![0; 9];
//         let mut chance: f64 = 1.0;
//         // TODO unlock costs not included right now
//         for (index, tap) in taps.iter().enumerate() {
//             let upgrade: &Upgrade = &prep_outputs.upgrade_arr[index];
//             for cost_type in 0..7 {
//                 this_cost[cost_type] += (tap + upgrade.tap_offset) * upgrade.costs[cost_type];
//             }

//             if upgrade.is_normal_honing {
//                 let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
//                 this_cost[juice_ind] +=
//                     (*tap + upgrade.tap_offset).min(decision.0[index]) * upgrade.one_juice_cost;
//             }
//             let dist: Vec<f64> = probability_distribution(
//                 upgrade.base_chance,
//                 upgrade.artisan_rate,
//                 &generate_first_deltas(
//                     upgrade.base_chance,
//                     upgrade.prob_dist_len,
//                     decision.0[index] as usize,
//                 ),
//             );
//             chance *= if *tap < dist.len() as i64 {
//                 dist[*tap as usize]
//             } else {
//                 0.0
//             };
//             if chance == 0.0 {
//                 break;
//             }
//         }
//         if chance == 0.0 {
//             continue;
//         }
//         let gold_cost: f64 =
//             compute_gold_cost_from_raw(&this_cost, input_budgets, &prep_outputs.mats_value);
//         combined_prob_dist.push((gold_cost, chance));
//     }

//     combined_prob_dist
//         .sort_unstable_by(|(a, _), (c, _)| a.partial_cmp(c).unwrap_or(std::cmp::Ordering::Equal));
//     let mut cumulative: f64 = 0.0;
//     let mut next_thresh: i64 = 0;
//     let mut out: Vec<f64> = Vec::new();
//     let last: usize = combined_prob_dist.len() - 1;
//     let mut index: usize = 0;
//     while next_thresh < 100 {
//         let pair = combined_prob_dist[index];
//         cumulative += pair.1;
//         if cumulative > next_thresh as f64 / 100.0 {
//             out.push(pair.0);
//             next_thresh += 1;
//         } else {
//             index += 1;
//         }
//     }
//     out.push(combined_prob_dist[last].0);
//     out
// }
pub fn average_tap(prob_dist: &[f64], offset: f64) -> f64 {
    let mut out: f64 = 0.0_f64;
    // println!("{:?}", prob_dist[start_index..].iter().sum::<f64>() as f64);
    #[cfg(debug_assertions)]
    assert_float_absolute_eq!(prob_dist.iter().sum::<f64>(), 1.0_f64, 0.0000000001);
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
    assert_float_absolute_eq!(prob_dist.iter().sum::<f64>(), 1.0_f64, 0.0000000001);
    let sum_before_start: f64 = prob_dist.iter().take(truncate - 1).sum();
    // let mut sum_so_far: f64 = 0.0;

    // dbg!(&prob_dist);
    // dbg!(&truncate);
    for (index, item) in prob_dist.iter().enumerate() {
        if index < truncate - 1 {
            out += 0.0;
        } else {
            out += item * (index as f64 + offset) / (1.0 - sum_before_start);
        }
    }
    out + (truncate - 1) as f64
}

fn average_value(upgrade: &Upgrade, mats_value: &[f64], average: f64) -> f64 {
    let mut this_sum = 0.0_f64;
    for cost_type in 0..7 {
        this_sum += mats_value[cost_type] * average * upgrade.costs[cost_type] as f64;
    }
    this_sum
}

pub fn extract_special_strings(upgrade_arr: &[Upgrade]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for upgrade in upgrade_arr {
        if !upgrade.is_normal_honing {
            continue;
        }
        let level_str: String = format!("+{}", upgrade.upgrade_plus_num + 1);
        let type_str: &'static str = if upgrade.is_weapon { "weapon" } else { "armor" };

        let value_string: String =
            " ".to_owned() + &upgrade.special_value.round().to_string() + "g";

        result.push(format!("{level_str} {type_str}{value_string}"));
    }
    result
}
fn est_juice_value_for_prob_dist(
    upgrade: &Upgrade,
    mat_values: &[f64],
    prob_dist: &[f64],
    extra_count: usize,
) -> f64 {
    average_value(
        upgrade,
        mat_values,
        truncated_average_tap(prob_dist, upgrade.tap_offset as f64, extra_count),
    )
}
pub fn est_special_honing_value(upgrade_arr: &mut Vec<Upgrade>, mats_values: &[f64]) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::with_capacity(upgrade_arr.len());
    let mut average: f64;
    // let cost_type_count: usize = 7;
    let mut special_value: f64;
    // let mut is_valid: bool;
    // debug_assert!(mats_values.len() == cost_type_count);
    for upgrade in upgrade_arr.iter_mut() {
        if upgrade.is_normal_honing {
            average = average_tap(&upgrade.original_prob_dist, upgrade.tap_offset as f64);
            special_value = upgrade.base_chance * average_value(upgrade, mats_values, average)
                / upgrade.special_cost as f64;

            out.push(special_value);

            upgrade.special_value = special_value;
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

    for upgrade in upgrade_arr.iter_mut() {
        if !upgrade.is_normal_honing || upgrade.upgrade_plus_num <= 2 {
            continue;
            //TODO add adv honing juice value estimation
        }
        this_sum = Vec::with_capacity(upgrade.prob_dist_len);
        let mut prev_prob_dist: Vec<f64> = probability_distribution(
            upgrade.base_chance,
            upgrade.artisan_rate,
            &generate_first_deltas(upgrade.base_chance, upgrade.prob_dist_len, 0),
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
                &generate_first_deltas(upgrade.base_chance, upgrade.prob_dist_len, extra_count),
            );

            let value_with_juice: f64 =
                est_juice_value_for_prob_dist(upgrade, mat_values, &next_prob_dist, extra_count);

            let value_without_juice: f64 =
                est_juice_value_for_prob_dist(upgrade, mat_values, &prev_prob_dist, extra_count);

            this_sum.push((value_without_juice - value_with_juice) / upgrade.one_juice_cost as f64);
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
) -> (Vec<String>, Vec<String>) {
    // Armor uses blue juice (is_weapon == false), Weapon uses red juice (is_weapon == true)
    let (mut armor_sorted, next_armor_value): (Vec<(usize, usize, f64)>, f64) =
        _juice_to_array(upgrade_arr, false, blue_juice);
    let (mut weapon_sorted, next_weapon_value): (Vec<(usize, usize, f64)>, f64) =
        _juice_to_array(upgrade_arr, true, red_juice);

    armor_sorted.sort_by_key(|&(plus, _, min)| (plus, -min.round() as i64));
    weapon_sorted.sort_by_key(|&(plus, _, min)| (plus, -min.round() as i64));
    let armor_strings: Vec<String> = compress_runs(
        armor_sorted
            .into_iter()
            .map(|(plus, taps, _)| format!("+{} armor, First {} taps", plus + 1, taps,))
            .collect(),
        false,
        vec![format!(
            "Next value: {next_armor_value}. You should buy more if the price is lower than {next_armor_value}"
        )],
    );

    let weapon_strings: Vec<String> = compress_runs(
        weapon_sorted
            .into_iter()
            .map(|(plus, taps, _)| format!("+{} weapon, First {} taps", plus + 1, taps,))
            .collect(),
        false,
        vec![format!(
            "Next value: {next_weapon_value}. You should buy more if the price is lower than {next_weapon_value}."
        )],
    );

    (armor_strings, weapon_strings)
}

fn _juice_to_array(
    upgrade_arr: &mut Vec<Upgrade>,
    is_weapon: bool,
    mut juice: i64,
) -> (Vec<(usize, usize, f64)>, f64) {
    let mut cur_upgrade: &mut Upgrade;
    let mut idxs: Vec<usize>;
    let mut max_value_index: usize;
    let mut cur_extras: Vec<usize> = vec![0; upgrade_arr.len()];
    let mut next_value: f64;
    // let mut _max_extra_index: usize;
    loop {
        next_value = upgrade_arr
            .iter_mut()
            .enumerate()
            .filter(|(index, x)| {
                x.is_normal_honing
                    && x.is_weapon == is_weapon
                    && cur_extras[*index] + 1 < x.juice_values.len()
            })
            .map(|(index, x)| {
                if cur_extras[index] + 1 == x.juice_values.len() - 1 {
                    x.juice_values[cur_extras[index] + 1]
                } else {
                    (x.juice_values[cur_extras[index] + 1] + x.juice_values[cur_extras[index] + 2])
                        / 2.0
                }
            })
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0)
            .ceil();

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
            &generate_first_deltas(
                cur_upgrade.base_chance,
                cur_upgrade.prob_dist_len,
                cur_extras[max_value_index],
            ),
        );
        juice -= cur_upgrade.one_juice_cost;
    }
    // Extract (plus_num, taps_used) only for selected type and where taps_used > 0
    let mut out: Vec<(usize, usize, f64)> = Vec::new();
    for (i, upgrade) in upgrade_arr.iter().enumerate() {
        if upgrade.is_normal_honing && upgrade.is_weapon == is_weapon {
            let taps_used: usize = cur_extras[i];
            if taps_used > 0 {
                out.push((
                    upgrade.upgrade_plus_num,
                    taps_used,
                    upgrade
                        .juice_values
                        .clone()
                        .into_iter()
                        .take(taps_used)
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap()
                        .round(),
                ));
            }
        }
    }
    (out, next_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser;
    use crate::test_utils::*;
    use crate::{calculate_hash, my_assert};
    // #[test]
    // fn decision_space_test() {
    //     let test_name: &str = "decision_space_test";
    //     let hone_counts: Vec<Vec<i64>> = vec![
    //         (0..25).map(|_| 0).collect(),
    //         (0..25)
    //             .map(|x| if x == 10 || x == 11 { 1 } else { 0 })
    //             .collect(),
    //     ];
    //     let adv_counts: Vec<Vec<i64>> =
    //         vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

    //     let adv_hone_strategy: &str = "No juice";
    //     let express_event: bool = true;

    //     let hash: String =
    //         calculate_hash!(&hone_counts, &adv_counts, adv_hone_strategy, express_event);

    //     let mut upgrade_arr = parser(
    //         &hone_counts,
    //         &adv_counts,
    //         &adv_hone_strategy.to_string(),
    //         express_event,
    //     );

    //     let result: Vec<(Vec<i64>, Vec<i64>)> = decision_space_iterator(&mut upgrade_arr).collect();
    //     dbg!(result.len());
    //     // let result: Vec<Vec<i64>> = out.clone();
    //     if let Some(cached_result) = read_cached_data::<Vec<(Vec<i64>, Vec<i64>)>>(test_name, &hash)
    //     {
    //         my_assert!(*result, cached_result);
    //     } else {
    //         write_cached_data(test_name, &hash, &result);
    //     }
    // }
    #[test]
    fn est_juice_value_25_wep() {
        let test_name: &str = "est_juice_value_25_wep";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|_| 0).collect(),
            (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;

        let hash: String =
            calculate_hash!(&hone_counts, &adv_counts, adv_hone_strategy, express_event);

        let mut upgrade_arr = parser(
            &hone_counts,
            &adv_counts,
            &adv_hone_strategy.to_string(),
            express_event,
        );

        est_juice_value(&mut upgrade_arr, &DEFAULT_GOLD_VALUES);
        let result: Vec<f64> = upgrade_arr[0].juice_values.clone();
        if let Some(cached_result) = read_cached_data::<Vec<f64>>(test_name, &hash) {
            for (index, i) in result.iter().enumerate() {
                my_assert!(*i, cached_result[index]);
            }
        } else {
            write_cached_data(test_name, &hash, &result);
        }
    }
    #[test]
    fn est_juice_value_23_wep() {
        let test_name: &str = "est_juice_value_23_wep";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|_| 0).collect(),
            (0..25).map(|x| if x == 22 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;

        let hash: String =
            calculate_hash!(&hone_counts, &adv_counts, adv_hone_strategy, express_event);

        let mut upgrade_arr = parser(
            &hone_counts,
            &adv_counts,
            &adv_hone_strategy.to_string(),
            express_event,
        );

        est_juice_value(&mut upgrade_arr, &DEFAULT_GOLD_VALUES);
        let result: Vec<f64> = upgrade_arr[0].juice_values.clone();
        if let Some(cached_result) = read_cached_data::<Vec<f64>>(test_name, &hash) {
            my_assert!(*result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result);
        }
    }

    #[test]
    fn est_juice_value_5_wep() {
        let test_name: &str = "est_juice_value_5_wep";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|_| 0).collect(),
            (0..25).map(|x| if x == 4 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;

        let hash: String =
            calculate_hash!(&hone_counts, &adv_counts, adv_hone_strategy, express_event);

        let mut upgrade_arr = parser(
            &hone_counts,
            &adv_counts,
            &adv_hone_strategy.to_string(),
            express_event,
        );

        est_juice_value(&mut upgrade_arr, &DEFAULT_GOLD_VALUES);
        let result: Vec<f64> = upgrade_arr[0].juice_values.clone();

        if let Some(cached_result) = read_cached_data::<Vec<f64>>(test_name, &hash) {
            for (index, i) in result.iter().enumerate() {
                my_assert!(*i, cached_result[index]);
            }
        } else {
            write_cached_data(test_name, &hash, &result);
        }
    }
}
