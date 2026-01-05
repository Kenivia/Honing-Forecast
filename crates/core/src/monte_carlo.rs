use crate::average::{add_up_golds, apply_price_leftovers, apply_price_naive};
use crate::parser::PreparationOutput;
use crate::parser::Upgrade;
use crate::saddlepoint_approximation::normal_sa::init_dist;
use crate::state::StateBundle;
use rand::Rng;
use rand::prelude::*;
use std::cmp::min;

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
fn sample_truncated_geometric<R: Rng + ?Sized>(p: f64, max_taps: i64, rng: &mut R) -> i64 {
    if max_taps < 0 {
        panic!();
    }
    if p <= 0.0 {
        return max_taps; // degenerate -> always tail
    }
    if p >= 1.0 {
        return 0; // succeed immediately
    }
    let q: f64 = 1.0 - p;
    let u: f64 = rng.random_range(0.0..1.0);
    // ln(u)/ln(q) >= 0 (both negative logs) gives k (0-based)
    let k: i64 = u.log(q).floor() as i64;
    let k: i64 = if k < 0 { 0 } else { k };
    if k > max_taps { max_taps } else { k }
}

fn juice_map(
    upgrade: &Upgrade,
    state_bundle: &StateBundle,
    prep_output: &PreparationOutput,
    u_index: usize,
) -> Vec<Vec<(i64, i64)>> {
    let mut juice_so_far: Vec<i64> = vec![0; prep_output.juice_info.amt_used_id.len()];

    let mut juice_used: Vec<Vec<(i64, i64)>> =
        vec![vec![(0, 0); prep_output.juice_info.amt_used_id.len()]; upgrade.prob_dist.len()];
    for (p_index, _) in upgrade.prob_dist.iter().enumerate() {
        if u_index >= state_bundle.state.len() || p_index >= state_bundle.state[u_index].len() {
            dbg!(
                state_bundle.state.len(),
                u_index,
                p_index,
                state_bundle.state[u_index].len(),
                &state_bundle.state[u_index],
                &upgrade.prob_dist
            );
        }
        let (juice, book_index) = state_bundle.state[u_index][p_index];
        for (id, (weap_used, armor_used)) in juice_used[p_index].iter_mut().enumerate() {
            if upgrade.is_weapon {
                *weap_used += juice_so_far[id];
            } else {
                *armor_used += juice_so_far[id];
            }
            let juice_amt = prep_output.juice_info.amt_used_id[id][upgrade.upgrade_index];
            if id == 0 && juice {
                juice_so_far[id] += juice_amt;
            } else if id > 0 && prep_output.juice_info.ids[upgrade.upgrade_index][book_index] == id
            {
                juice_so_far[id] += juice_amt;
            }
        }
    }
    juice_used
}

pub fn monte_carlo_data<R: Rng>(
    data_size: usize,
    state_bundle: &mut StateBundle,
    prep_output: &mut PreparationOutput,

    rng: &mut R,
) -> (Vec<[i64; 7]>, Vec<Vec<(i64, i64)>>) {
    let mut special_left: Vec<i64> = vec![prep_output.budgets[7]; data_size];
    prep_output.budgets[7] = 0;
    init_dist(state_bundle, prep_output);
    let mut mats_data: Vec<[i64; 7]> = vec![[0i64; 7]; data_size];

    let mut juice_data: Vec<Vec<(i64, i64)>> =
        vec![vec![(0, 0); prep_output.juice_info.amt_used_id.len()]; data_size];

    for (u_index, juice) in state_bundle.special_state.iter() {
        let upgrade = &prep_output.upgrade_arr[*u_index];
        let tap_map: Vec<usize> = tap_map_generator(data_size, &upgrade.prob_dist, rng);
        let juice_map: Vec<Vec<(i64, i64)>> =
            juice_map(upgrade, state_bundle, prep_output, *u_index);
        for trial_num in 0..data_size {
            let this_special_left: &mut i64 = &mut special_left[trial_num];

            let attempts_specified = *juice as i64;
            let max_affordable_attempts = (*this_special_left / upgrade.special_cost).max(0);
            if max_affordable_attempts > 0 && attempts_specified > 0 {
                let taps_rolled =
                    sample_truncated_geometric(upgrade.base_chance, max_affordable_attempts, rng)
                        + 1;
                *this_special_left -= taps_rolled.min(attempts_specified) * upgrade.special_cost;
                if taps_rolled <= max_affordable_attempts.min(attempts_specified) {
                    continue;
                }
            }

            let rolled_tap: usize = tap_map[trial_num];
            assert!(rolled_tap > 0); // we simulate special directly above instead of relying on special_sa(to test if its working), init_dist should've been called with 0 special owned 
            for cost_type in 0..7 {
                mats_data[trial_num][cost_type] +=
                    upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
            }
            for id in 0..prep_output.juice_info.amt_used_id.len() {
                juice_data[trial_num][id].0 += juice_map[rolled_tap][id].0;
                juice_data[trial_num][id].1 += juice_map[rolled_tap][id].1;
            }
        }
    }

    // unlock costs
    for row in &mut mats_data {
        row[3] += prep_output.unlock_costs[0];
        row[6] += prep_output.unlock_costs[1];
    }

    (mats_data, juice_data)
}

pub fn monte_carlo_wrapper<R: Rng>(
    data_size: usize,
    state_bundle: &mut StateBundle,
    prep_output: &mut PreparationOutput,
    rng: &mut R,
) -> (Vec<f64>, f64, f64) {
    let (cost_data, juice_data) = monte_carlo_data(data_size, state_bundle, prep_output, rng);
    let mut success_count: i64 = 0;
    let mut average: f64 = 0.0;
    let mut leftover_counts: Vec<i64> = vec![0; 7 + prep_output.juice_info.one_gold_cost.len() * 2];
    for (r_index, row) in cost_data.iter().enumerate() {
        let float_row: Vec<f64> = row.iter().map(|x| *x as f64).collect();
        let float_juice: Vec<(f64, f64)> = juice_data[r_index]
            .iter()
            .map(|x| (x.0 as f64, x.1 as f64))
            .collect();
        let (mats_gold_leftover, juice_gold_leftover) =
            apply_price_leftovers(&float_row, &float_juice, prep_output);

        let (mats_gold_naive, juice_gold_naive) =
            apply_price_naive(&float_row, &float_juice, prep_output);
        let gold_eqv_naive: f64 = add_up_golds(&mats_gold_naive, &juice_gold_naive);
        if gold_eqv_naive >= 0.0 {
            success_count += 1;
        }
        average += add_up_golds(&mats_gold_leftover, &juice_gold_leftover);

        let mut leftover_index: usize = 0;
        for (index, mat) in row.iter().enumerate() {
            if *mat < prep_output.budgets[index] {
                leftover_counts[leftover_index] += 1;
            }
            leftover_index += 1;
        }
        for (index, juice) in juice_data[r_index].iter().enumerate() {
            if juice.0 < prep_output.juice_books_owned[index].0 {
                leftover_counts[leftover_index] += 1;
            }
            leftover_index += 1;
        }
        for (index, juice) in juice_data[r_index].iter().enumerate() {
            if juice.1 < prep_output.juice_books_owned[index].1 {
                leftover_counts[leftover_index] += 1;
            }
            leftover_index += 1;
        }
    }

    let prob_leftover: Vec<f64> = leftover_counts
        .into_iter()
        .map(|x| x as f64 / data_size as f64)
        .collect();

    (
        prob_leftover,
        success_count as f64 / data_size as f64,
        average / data_size as f64,
    )
}

// pub fn monte_carlo_one<R: Rng>(
//     data_size: usize,
//     upgrade: &Upgrade,
//     avail_special: i64,
//     juice_count: i64,
//     mut rng: &mut R,
// ) -> Vec<[i64; 10]> {
//     let mut cost_data: Vec<[i64; 10]> = vec![[0i64; 10]; data_size];
//     // let mut special_budgets: Vec<i64> = vec![avail_special; data_size];
//     // let mut special_pass_arr: Vec<usize> = vec![0usize; data_size];

//     // TODO
//     // if avail_special > 0 {
//     //     // TODO ignore when later passed ->
//     //     for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
//     //         if upgrade.is_normal_honing {
//     //             let limit: i64 = calc_failure_lim(avail_special, upgrade.special_cost);
//     //             for trial in 0..data_size {
//     //                 if special_budgets[trial] <= 0 {
//     //                     continue;
//     //                 }
//     //                 let taps_used: usize =
//     //                     sample_truncated_geometric(upgrade.base_chance, limit, &mut rng);
//     //                 let rolled_special_cost: i64 = (taps_used as i64 + 1) * upgrade.special_cost;
//     //                 special_budgets[trial] -= rolled_special_cost;
//     //                 if special_budgets[trial] > 0 {
//     //                     special_pass_arr[trial] += 1;
//     //                     debug_assert!(special_pass_arr[trial] == upgrade_index + 1);
//     //                     // this assertion is triggered when there's an advanced honing upgrade sorted before normal honing upgrade in upgrade_arr
//     //                     // which shouldnt happen
//     //                 }
//     //             }
//     //         }
//     //     }
//     // }

//     let tap_map: Vec<usize> = tap_map_generator(data_size, &upgrade.prob_dist, rng);
//     for trial_num in 0..data_size {
//         let rolled_tap: usize = tap_map[trial_num];
//         for cost_type in 0..7 {
//             cost_data[trial_num][cost_type] +=
//                 upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
//         }

//         if upgrade.is_normal_honing {
//             let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
//             cost_data[trial_num][juice_ind] +=
//                 (rolled_tap as i64).min(juice_count) * upgrade.one_juice_cost;
//         }
//     }

//     cost_data
// }

// let mut special_budgets: Vec<i64> = vec![avail_special; data_size];
// let mut special_pass_arr: Vec<usize> = vec![0usize; data_size];

// // pre-generate an array of how many pieces were free tapped, mostly because it just makes things easier
// // i doubt it's actually faster
// if avail_special > 0 {
//     // TODO ignore when later passed ->
//     for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
//         if upgrade.is_normal_honing {
//             let limit: i64 = calc_failure_lim(avail_special, upgrade.special_cost);
//             for trial in 0..data_size {
//                 if special_budgets[trial] <= 0 {
//                     continue;
//                 }
//                 let taps_used: usize =
//                     sample_truncated_geometric(upgrade.base_chance, limit, &mut rng);
//                 let rolled_special_cost: i64 = (taps_used as i64 + 1) * upgrade.special_cost;
//                 special_budgets[trial] -= rolled_special_cost;
//                 if special_budgets[trial] > 0 {
//                     special_pass_arr[trial] += 1;
//                     debug_assert!(special_pass_arr[trial] == upgrade_index + 1);
//                     // this assertion is triggered when there's an advanced honing upgrade sorted before normal honing upgrade in upgrade_arr
//                     // which shouldnt happen
//                 }
//             }
//         }
//     }
// }

// juicy part, we use tap_map_generator to get the excpected distribution and just multiply it by data_size
// e.g. if the distribution is 0.2,0.1,0.7 and data size is 100, we generate [20 zeros, 10 ones, 70 twos] etc
// it is then shuffled before being added to cost_data
// This supposedly reduces variance by quite a big amount
// we COULD also do this for free taps but um WIP ig
