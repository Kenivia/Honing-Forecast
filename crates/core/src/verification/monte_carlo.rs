//! Monte carlo to experimentally verify our results, not used in the website (anymore)

use crate::constants::FLOAT_TOL;
use crate::core::average::DEBUG_AVERAGE;
use crate::my_dbg;
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use crate::verification::utils::{
    add_up_golds, apply_price_generic, apply_price_leftovers, apply_price_naive,
};
use itertools::izip;
use rand::Rng;
use rand::prelude::*;
use statrs::distribution::{ContinuousCDF, Normal};

/// Instead of actually sampling the distribution, we guarantee that every value has exactly the expected number of occurances
fn tap_map_generator<R: Rng>(count_limit: usize, prob_dist: &[f64], rng: &mut R) -> Vec<usize> {
    let mut tap_map: Vec<usize> = Vec::with_capacity(count_limit);

    let mut current_point: f64 = rng.random::<f64>();
    let mut cum_prob: f64 = 0.0;

    for (i, &p) in prob_dist.iter().enumerate() {
        cum_prob += p;
        let target_boundary: f64 = cum_prob * (count_limit as f64);

        // Advance our "comb" of points. Every time a point falls within
        // the current cumulative boundary, we assign a sample to this bucket.
        while current_point < target_boundary && tap_map.len() < count_limit {
            tap_map.push(i);
            current_point += 1.0;
        }
    }

    // Handle floating-point imprecision where the sum of `prob_dist` might be
    // slightly less than 1.0, leaving us a few elements short of `count_limit`.
    let fill_idx: usize = prob_dist.len().saturating_sub(1);
    while tap_map.len() < count_limit {
        tap_map.push(fill_idx);
    }

    tap_map.shuffle(rng);
    tap_map
}

fn sample_truncated_geometric<R: Rng + ?Sized>(p: f64, max_taps: i64, rng: &mut R) -> i64 {
    if max_taps <= 0 {
        panic!();
    }
    if p <= 0.0 {
        return max_taps;
    }
    if p >= 1.0 {
        return 1;
    }
    let q: f64 = 1.0 - p;
    let u: f64 = rng.random_range(0.0..1.0);
    let k: i64 = u.log(q).ceil() as i64;
    let k: i64 = if k <= 0 { 1 } else { k };
    if k > max_taps { max_taps + 1 } else { k }
}

/// this would be significantly easier (at least for the normal honing path)
/// if we just used the support computed and collapsed by honing_utils
/// but that kinda defeats the purpose
///
/// In fact i SHOULD (re)implement a monte carlo for advanced honing also,
/// but cbb rn TODO
fn juice_costs(upgrade: &Upgrade, state_bundle: &StateBundle) -> Vec<Vec<(i64, i64)>> {
    let prep_output = &state_bundle.prep_output;

    let mut juice_used: Vec<Vec<(i64, i64)>> =
        vec![
            vec![(0, 0); prep_output.juice_info.num_juice_avail];
            upgrade.normal_dist.len().max(if upgrade.is_normal_honing {
                0
            } else {
                upgrade.adv_dists[1].len().max(upgrade.adv_dists[2].len())
            })
        ];
    if upgrade.is_normal_honing {
        let mut juice_so_far: Vec<i64> = vec![0; prep_output.juice_info.num_juice_avail];

        for &id in
            state_bundle.prep_output.juice_info.normal_uindex_to_id[upgrade.upgrade_index].iter()
        {
            let dist = &upgrade.normal_dist;
            for (p_index, _) in dist.iter().enumerate() {
                let (weap_used, armor_used) = &mut juice_used[p_index][id];
                let (juice, book_id) = *upgrade.state.get(p_index).unwrap_or(&(false, 0));
                if upgrade.is_weapon {
                    *weap_used = juice_so_far[id];
                } else {
                    *armor_used = juice_so_far[id];
                }
                if p_index >= dist.len() - 2 {
                    continue;
                }

                let juice_amt = prep_output
                    .juice_info
                    .access(id, upgrade.upgrade_index)
                    .normal_amt_used;
                if id == 0 && juice {
                    juice_so_far[id] += juice_amt;
                } else if id > 0 && book_id == id {
                    juice_so_far[id] += juice_amt;
                }
            }
        }
    } else {
        for &id in
            state_bundle.prep_output.juice_info.adv_uindex_to_id[upgrade.upgrade_index].iter()
        {
            let dist = if id == 0 {
                &upgrade.adv_dists[1]
            } else {
                &upgrade.adv_dists[2]
            };
            for (p_index, _) in dist.iter().enumerate() {
                let (weap_used, armor_used) = &mut juice_used[p_index][id];

                if upgrade.is_weapon {
                    *weap_used = p_index as i64
                        * prep_output.juice_info.all_juices[id][&upgrade.upgrade_index]
                            .adv_amt_used;
                } else {
                    *armor_used = p_index as i64
                        * prep_output.juice_info.all_juices[id][&upgrade.upgrade_index]
                            .adv_amt_used;
                }
            }
        }
    }

    juice_used
}

pub fn monte_carlo_data<R: Rng>(
    data_size: usize,
    state_bundle: &mut StateBundle,
    rng: &mut R,
) -> (Vec<Vec<i64>>, Vec<usize>) {
    let mut special_left: Vec<i64> = vec![state_bundle.prep_output.special_budget; data_size];
    state_bundle.update_dist();
    let mut mats_data: Vec<Vec<i64>> =
        vec![vec![0; state_bundle.prep_output.juice_info.total_num_avail]; data_size];

    let mut juice_data: Vec<Vec<(i64, i64)>> =
        vec![vec![(0, 0); state_bundle.prep_output.juice_info.num_juice_avail]; data_size];

    let mut actually_paid: Vec<i64> = vec![0; state_bundle.upgrade_arr.len() + 1];
    let mut skip_count_data: Vec<usize> = vec![0; data_size];

    let mut highest_upgrade_index_seen: Vec<i64> = vec![-1; 6];
    let mut special_valid: bool;
    // my_dbg!(&state_bundle, &prep_output);
    for (attempt_index, u_index) in state_bundle.special_state.iter().enumerate() {
        let upgrade = &state_bundle.upgrade_arr[*u_index];
        let tap_map_0: Vec<usize> = tap_map_generator(
            data_size,
            if upgrade.is_normal_honing {
                &upgrade.normal_dist
            } else {
                &upgrade.adv_dists[0]
            },
            rng,
        );
        let tap_map_1: Vec<usize> = if upgrade.is_normal_honing {
            vec![0; data_size]
        } else {
            tap_map_generator(data_size, &upgrade.adv_dists[1], rng)
        };
        let tap_map_2: Vec<usize> = if upgrade.is_normal_honing {
            vec![0; data_size]
        } else {
            tap_map_generator(data_size, &upgrade.adv_dists[2], rng)
        };
        // by the way this incorrectly assumes that adv honing mat cost & juice cost & scroll cost are independent, which they are very much not
        // but since we're interested in the average it shouldn't matter
        // as such if we're talking about prob_leftover and stuff like that it wouldn't be accurate but whatever

        let juice_costs: Vec<Vec<(i64, i64)>> = juice_costs(upgrade, state_bundle);

        if highest_upgrade_index_seen[upgrade.piece_type] > upgrade.upgrade_index as i64 {
            special_valid = false;
        } else {
            highest_upgrade_index_seen[upgrade.piece_type] = upgrade.upgrade_index as i64;
            special_valid = true;
        }

        for (
            this_mat,
            this_juice,
            this_special_left,
            this_skip_data,
            rolled_tap_0,
            rolled_tap_1,
            rolled_tap_2,
        ) in izip!(
            mats_data.iter_mut(),
            juice_data.iter_mut(),
            special_left.iter_mut(),
            skip_count_data.iter_mut(),
            tap_map_0,
            tap_map_1,
            tap_map_2,
        ) {
            for cost_type in 0..7 {
                this_mat[cost_type] += upgrade.unlock_costs[cost_type].round() as i64;
            }
            if special_valid {
                let max_affordable_attempts = (*this_special_left / upgrade.special_cost).max(0);
                if max_affordable_attempts > 0 {
                    let special_taps_needed = sample_truncated_geometric(
                        upgrade.base_chance,
                        max_affordable_attempts,
                        rng,
                    );

                    *this_special_left -= special_taps_needed * upgrade.special_cost;

                    if *this_special_left >= 0 {
                        *this_skip_data += 1;
                        continue;
                    }
                }
            }

            actually_paid[attempt_index + 1] += 1;
            let mut rolled_tap = rolled_tap_0;
            // let rolled_tap: usize = tap_map[trial_num];
            assert!(rolled_tap > 0); // we simulate special directly above instead of relying on special_sa(to test if its working), init_dist should've been called with 0 special owned 

            for cost_type in 0..7 {
                this_mat[cost_type] +=
                    upgrade.costs[cost_type].round() as i64 * (rolled_tap as i64);
            }

            for id in 0..state_bundle.prep_output.juice_info.num_juice_avail {
                rolled_tap = if upgrade.is_normal_honing {
                    rolled_tap_0
                } else if id == 0 {
                    rolled_tap_1
                } else {
                    rolled_tap_2
                };
                this_juice[id].0 += juice_costs[rolled_tap][id].0;
                this_juice[id].1 += juice_costs[rolled_tap][id].1;
            }
        }
    }

    // unlock costs

    let mut result = actually_paid
        .iter()
        .map(|&x| 1.0 - x as f64 / data_size as f64)
        .collect::<Vec<f64>>();
    // my_dbg!(&result);
    result[0] = 1.0 - result[1]; // nothing free tapped
    let mut actual_out = Vec::with_capacity(result.len());

    for (index, &i) in result.iter().enumerate() {
        // if index < 1 {
        //     actual_out.push(cumulative * *i);
        // } else {
        if index == result.len() - 1 || index == 0 {
            actual_out.push(i);
        } else {
            actual_out.push(i - result[index + 1]);
        }
    }

    if DEBUG_AVERAGE {
        state_bundle.compute_special_probs(false);
        my_dbg!(actual_out);
        my_dbg!(state_bundle.special_probs());
        my_dbg!(&state_bundle.prep_output.juice_info);
    }

    (mats_data, skip_count_data)
}

pub fn monte_carlo_wrapper<R: Rng>(
    data_size: usize,
    state_bundle: &mut StateBundle,
    rng: &mut R,
) -> (Vec<f64>, f64, f64, f64, f64) {
    let (cost_data, skip_count_data) = monte_carlo_data(data_size, state_bundle, rng);
    let mut success_count: i64 = 0;
    let mut sum: f64 = 0.0;

    let mut average_sq: f64 = 0.0;
    let mut leftover_counts: Vec<i64> =
        vec![0; state_bundle.prep_output.juice_info.total_num_avail];

    let mut debug_avg_gold_by_mats: Vec<f64> =
        vec![0.0; state_bundle.prep_output.juice_info.total_num_avail];
    let mut debug_avg_gold_by_mats_by_skip: Vec<Vec<f64>> =
        vec![
            vec![0.0; state_bundle.prep_output.juice_info.total_num_avail];
            state_bundle.upgrade_arr.len() + 1
        ];
    let mut debug_truncated_mean_by_skip: Vec<Vec<f64>> =
        vec![
            vec![0.0; state_bundle.prep_output.juice_info.total_num_avail];
            state_bundle.upgrade_arr.len() + 1
        ];
    for (r_index, row) in cost_data.iter().enumerate() {
        let float_row: Vec<f64> = row.iter().map(|x| *x as f64).collect();

        for (index, d) in debug_avg_gold_by_mats.iter_mut().enumerate() {
            *d += apply_price_generic(float_row[index], &state_bundle.prep_output, index)
        }
        for (index, d) in debug_avg_gold_by_mats_by_skip[skip_count_data[r_index]]
            .iter_mut()
            .enumerate()
        {
            *d += apply_price_generic(float_row[index], &state_bundle.prep_output, index)
        }

        let this: f64 = float_row
            .iter()
            .enumerate()
            .map(|(index, used)| apply_price_generic(*used, &state_bundle.prep_output, index))
            .sum();
        sum += this;
        average_sq += this * this;

        if this > -FLOAT_TOL {
            success_count += 1;
        }

        let mut leftover_index: usize = 0;
        for (index, mat) in row.iter().enumerate() {
            if *mat as f64 <= state_bundle.prep_output.bound_budgets[index] {
                leftover_counts[leftover_index] += 1;
            }
            leftover_index += 1;
        }
    }
    for d in debug_avg_gold_by_mats.iter_mut() {
        *d /= data_size as f64;
    }
    for (index, row) in debug_truncated_mean_by_skip.iter_mut().enumerate() {
        for d in row.iter_mut() {
            *d /= skip_count_data.iter().filter(|x| **x == index).count() as f64;
        }
    }
    for row in debug_avg_gold_by_mats_by_skip.iter_mut() {
        for d in row.iter_mut() {
            *d /= data_size as f64;
        }
    }

    if DEBUG_AVERAGE {
        my_dbg!(
            // &debug_avg_gold_by_mats,
            &debug_avg_gold_by_mats_by_skip,
            &debug_truncated_mean_by_skip,
            &state_bundle.prep_output.leftover_price,
            &state_bundle.prep_output.tradable_price,
            &state_bundle.prep_output.market_price,
            sum / data_size as f64
        );
    }
    let prob_leftover: Vec<f64> = leftover_counts
        .into_iter()
        .map(|x| x as f64 / data_size as f64)
        .collect();
    let success = success_count as f64 / data_size as f64;
    let average = sum / data_size as f64;
    (
        prob_leftover,
        success,
        average,
        success * (1.0 - success) * (data_size as f64) / (data_size - 1) as f64,
        (average_sq - sum * average) / (data_size - 1) as f64,
    )
}

pub struct MCResult {
    pub is_match: bool,
    pub mean: f64,
    pub lower: f64,
    pub upper: f64,
    pub samples: usize,
    pub prob_leftover: Vec<f64>,
}

pub fn verify_result_with_monte_carlo<R: Rng>(
    sa_result: f64,        // The value we are testing
    confidence: f64,       // e.g., 0.95 for 95% confidence
    target_precision: f64, // The tightest interval width we demand before accepting it
    batch_size: usize,
    state_bundle: &mut StateBundle,
    rng: &mut R,
) -> MCResult {
    let alpha = 1.0 - confidence;
    let normal = Normal::new(0.0, 1.0).unwrap();
    let z = normal.inverse_cdf(1.0 - alpha / 2.0);

    let mut total_n: usize = 0;
    let mut mean = 0.0;
    let mut m2 = 0.0;

    loop {
        let (prob_leftover, _, batch_mean, _, batch_var) =
            monte_carlo_wrapper(batch_size, state_bundle, rng);

        let batch_n = batch_size;
        let batch_m2 = batch_var * (batch_n as f64 - 1.0);

        if total_n == 0 {
            mean = batch_mean;
            m2 = batch_m2;
            total_n = batch_n;
        } else {
            let delta = batch_mean - mean;
            let new_n = total_n + batch_n;

            mean += delta * (batch_n as f64 / new_n as f64);
            m2 += batch_m2 + delta * delta * (total_n as f64 * batch_n as f64 / new_n as f64);

            total_n = new_n;
        }

        if total_n > 1 {
            let variance = m2 / (total_n as f64 - 1.0);
            let std_err = variance.sqrt() / (total_n as f64).sqrt();

            // This is the radius of our confidence interval
            let half_width = z * std_err;
            let distance_from_mean = (mean - sa_result).abs();

            // CONDITION 1: REJECTION
            // The sa_result is outside our confidence interval.
            // We are confident it is WRONG.
            if distance_from_mean > half_width {
                return MCResult {
                    is_match: false,
                    mean,
                    lower: mean - half_width,
                    upper: mean + half_width,
                    samples: total_n,
                    prob_leftover,
                };
            }

            // CONDITION 2: ACCEPTANCE
            // The sa_result is inside the interval, AND the interval
            // is finally narrow enough to satisfy our precision requirements.
            // We are confident it is RIGHT.
            if half_width <= target_precision {
                return MCResult {
                    is_match: true,
                    mean,
                    lower: mean - half_width,
                    upper: mean + half_width,
                    samples: total_n,
                    prob_leftover,
                };
            }
        }
    }
}
