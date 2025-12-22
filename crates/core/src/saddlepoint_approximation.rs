use crate::parser::PreparationOutputs;
use crate::parser::Upgrade;
use crate::parser::probability_distribution;

use rootfinder::{Interval, SolverSettings, root_bisection};
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

static DEBUG: bool = false;
static TOL: f64 = 1e-10;

pub struct StateBundle {
    pub state: Vec<Vec<i64>>,
    pub names: Vec<String>,
    pub state_index: Vec<Vec<Vec<i64>>>, // state_index[which_upgrade][what_number(0,1,2] = [indices that number]
}

/// Returns all unique subset sums of `values`.
/// Includes the empty subset (sum = 0).

fn ks_01234(upgrade_arr: &[Upgrade], theta: f64) -> (f64, f64, f64, f64, f64) {
    let mut total_k: f64 = 0.0;
    let mut total_k1: f64 = 0.0;
    let mut total_k2: f64 = 0.0;
    let mut total_k3: f64 = 0.0;
    let mut total_k4: f64 = 0.0;

    for upgrade in upgrade_arr {
        let mut alpha_arr: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
        let mut shift: f64 = f64::NEG_INFINITY;
        let mut cur_juice_count = 0;

        for (index, l) in upgrade.log_prob_dist.iter().enumerate() {
            if index < upgrade.juiced_arr.len()
                && index < upgrade.log_prob_dist.len() - 1
                && upgrade.juiced_arr[index] > 0.0
            {
                cur_juice_count += 1;
            }
            let this_alpha: f64 = l + theta
                * ((index as f64 + upgrade.tap_offset as f64) * upgrade.eqv_gold_per_tap
                    + cur_juice_count as f64 * upgrade.eqv_gold_per_juice);
            alpha_arr.push(this_alpha);
            shift = this_alpha.max(shift);
        }

        let mut s: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        let mut third: f64 = 0.0;
        let mut fourth: f64 = 0.0;
        let mut cur_juice_count = 0;
        if theta == 0.0 {
            if DEBUG {
                dbg!(&alpha_arr);
            }
        }
        let mut u_arr: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
        for aj in alpha_arr.iter() {
            let u: f64 = (aj - shift).exp(); // this can for sure be optimized into a polynomial TODO
            s += u;
            u_arr.push(u);
        }

        for (index, &u) in u_arr.iter().enumerate() {
            if index < upgrade.juiced_arr.len()
                && index < upgrade.log_prob_dist.len() - 1
                && upgrade.juiced_arr[index] > 0.0
            {
                cur_juice_count += 1;
            }
            if u == 0.0 {
                continue;
            }
            let w = u / s;
            let x = (index as f64 + upgrade.tap_offset as f64) * upgrade.eqv_gold_per_tap
                + cur_juice_count as f64 * upgrade.eqv_gold_per_juice; // can reuse the calculatoin from alpha array
            mean += x * w;
            second += (x * x) * w;
            third += (x * x * x) * w;
            fourth += x * x * x * x * w;
        }

        let mu2 = second - mean * mean;
        let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
        let mu4 = fourth - 4.0 * third * mean + 6.0 * second * mean * mean
            - 3.0 * mean * mean * mean * mean;

        total_k += shift + s.ln();
        total_k1 += mean;
        total_k2 += mu2.max(0.0);
        total_k3 += mu3;
        total_k4 += mu4 - 3.0 * mu2 * mu2;
    }

    (total_k, total_k1, total_k2, total_k3, total_k4)
}

pub fn saddlepoint_approximation(upgrade_arr: &[Upgrade], budget: f64, leftover: f64) -> f64 {
    // let (theta_hat, ks, ks1, ks2, ks3) = newton(upgrade_arr, budget);
    let f = |theta: f64| ks_01234(upgrade_arr, theta).1 - budget;

    let settings = SolverSettings {
        vtol: Some(TOL),
        rebracket: Some(true),
        ..Default::default() // fill other fields with None
    };
    let mut min_value: f64 = 0.0;
    let mut max_value: f64 = 0.0; // pre-calculate this  TODO
    for upgrade in upgrade_arr {
        for (index, _) in upgrade.prob_dist.iter().enumerate() {
            let mut this_value = upgrade.eqv_gold_per_tap;

            if index < upgrade.juiced_arr.len() - 1 && upgrade.juiced_arr[index] > 0.0 {
                this_value += upgrade.eqv_gold_per_juice;
            }
            if index < upgrade.prob_dist.len() {
                max_value += this_value;
            }
            if index <= 0 {
                min_value += this_value;
            }
        }
    }

    let result = root_bisection(&f, Interval::new(-1.0, 1.0), Some(&settings), None);
    if budget < min_value - TOL {
        return 0.0;
    }
    if budget < min_value + TOL {
        let mut prob: f64 = 1.0;
        for upgrade in upgrade_arr {
            prob *= upgrade.prob_dist[0];
        }
        return prob;
    }
    if budget + leftover > max_value - TOL {
        return 1.0;
    }

    if DEBUG || result.is_err() {
        dbg!(
            f(10000.0),
            f(1.0),
            f(0.0000001),
            f(-1.0),
            f(-10000.0),
            budget,
            min_value,
            max_value,
        );
    }
    let (ks, ks1, ks2, ks3, ks4);

    let theta_hat: f64 = result.unwrap();

    let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap(); // TODO can i pre initialize this or is there no point

    #[allow(unused_assignments)]
    if theta_hat.abs() < TOL {
        // pre-calculate K(0) and stuff TODO
        (ks, ks1, ks2, ks3, ks4) = ks_01234(upgrade_arr, 0.0);

        let std = ks2.sqrt();
        let z = (budget - ks1) / std;

        let gamma3 = ks3 / std.powi(3); // skewness
        let gamma4 = ks4 / std.powi(4); // excess kurtosis

        let pdf = normal_dist.pdf(z);
        let cdf = normal_dist.cdf(z);

        // Edgeworth (cdf) up to 4th cumulant and k3^2 term:
        let cdf_correction = pdf
            * ((gamma3 / 6.0) * (z * z - 1.0)
                + (gamma4 / 24.0) * (z * z * z - 3.0 * z)
                + (gamma3 * gamma3 / 72.0) * (z * z * z * z * z - 10.0 * z * z * z + 15.0 * z));

        let approx = cdf + cdf_correction;
        if DEBUG {
            dbg!(
                theta_hat,
                ks,
                ks1,
                ks2,
                ks3,
                budget - ks1,
                z,
                cdf,
                cdf_correction,
                approx
            );
        }
        approx
    } else {
        (ks, ks1, ks2, ks3, ks4) = ks_01234(upgrade_arr, theta_hat);
        let w_hat: f64 = theta_hat.signum() * (2.0 * (theta_hat * budget - ks)).sqrt();
        let u_hat: f64 = theta_hat * ks2.sqrt();

        let out = normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat);
        // if out < 0.0 || out > 1.0 || ks2.abs() < 1e-12 {
        if DEBUG {
            dbg!(
                theta_hat,
                ks,
                ks1,
                ks2,
                ks3,
                2.0 * (theta_hat * budget - ks),
                w_hat,
                u_hat,
                normal_dist.cdf(w_hat),
                normal_dist.pdf(w_hat),
                1.0 / w_hat - 1.0 / u_hat,
                budget,
                min_value,
                max_value,
                out
            );
        }
        // }
        out
    }
    //P(S<B)≈Φ(w^)+ϕ(w^)(w^1​−u^1​)
}

pub fn prob_to_maximize(
    state: &StateBundle,
    prep_outputs: &mut PreparationOutputs,

    states_evaled: &mut i64,
    // depth: usize,
    // cache: &mut HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>>,
) -> f64 {
    for (index, upgrade) in prep_outputs.upgrade_arr.iter_mut().enumerate() {
        let new_extra: Vec<f64> = state.state[index]
            .iter()
            .map(|x| if *x > 0 { upgrade.base_chance } else { 0.0 }) //
            .collect();

        upgrade.prob_dist =
            probability_distribution(upgrade.base_chance, upgrade.artisan_rate, &new_extra);
        upgrade.log_prob_dist = upgrade.prob_dist.iter().map(|x| x.ln()).collect();
        upgrade.juiced_arr = new_extra;
    }

    *states_evaled += 1;

    saddlepoint_approximation(
        &prep_outputs.upgrade_arr,
        prep_outputs.base_gold_budget - expected_juice_leftover(prep_outputs),
        expected_juice_leftover(prep_outputs),
    )
}

fn expected_juice_leftover(prep_outputs: &PreparationOutputs) -> f64 {
    let mut avg_used_blue: f64 = 0.0;
    let mut avg_used_red: f64 = 0.0;
    for (_, upgrade) in prep_outputs.upgrade_arr.iter().enumerate() {
        let mut cur_juice_count = 0.0;
        for (index, p) in upgrade.prob_dist.iter().enumerate() {
            if upgrade.juiced_arr[index] > 0.0 {
                cur_juice_count += 1.0;
            }
            let amt: f64 = cur_juice_count * p * upgrade.one_juice_cost as f64;
            if upgrade.is_weapon {
                avg_used_red += amt;
            } else {
                avg_used_blue += amt;
            }
        }
    }
    (prep_outputs.budgets[8] as f64 - avg_used_blue).max(0.0) * prep_outputs.mats_value[8]
        + (prep_outputs.budgets[7] as f64 - avg_used_red).max(0.0) * prep_outputs.mats_value[7]
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::calculate_hash;
//     use crate::constants::RNG_SEED;
//     use crate::helpers::eqv_gold_unlock;
//     use crate::parser::PreparationOutputs;
//     use crate::parser::preparation;
//     use crate::test_utils::*;
//     use std::time::Instant;
//     #[test]
//     fn saddlepoint_approximation_test() {
//         let start = Instant::now();
//         let test_name = format!("saddlepoint_approximation_test");
//         let hone_counts: Vec<Vec<i64>> = vec![
//             (0..25).map(|x| if x == 25 { 2 } else { 0 }).collect(),
//             (0..25).map(|x| if x == 25 { 1 } else { 0 }).collect(),
//         ];
//         // let hone_counts: Vec<Vec<i64>> =
//         //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
//         let adv_counts: Vec<Vec<i64>> =
//             vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

//         let adv_hone_strategy: &str = "No juice";
//         let express_event: bool = true;
//         let input_budgets = vec![0, 0, 0, 0, 0, 3333333, 0, 0, 0, 0];
//         let user_mats_value = DEFAULT_GOLD_VALUES;
//         let hash: String = calculate_hash!(
//             &hone_counts,
//             &adv_counts,
//             adv_hone_strategy,
//             express_event,
//             &input_budgets,
//             &user_mats_value,
//             RNG_SEED,
//             PROB_MODE
//         );

//         let mut prep_outputs: PreparationOutputs = preparation(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             &user_mats_value,
//             adv_hone_strategy,
//         );

//         for upgrade in prep_outputs.upgrade_arr.iter_mut() {
//             let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
//             for i in upgrade.prob_dist.iter() {
//                 log_prob_dist.push(i.ln());
//             }
//             upgrade.log_prob_dist = log_prob_dist;
//             upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, &prep_outputs.mats_value);
//             let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
//             upgrade.eqv_gold_per_juice =
//                 &prep_outputs.mats_value[juice_ind] * upgrade.one_juice_cost as f64;
//             upgrade.juiced_arr = vec![0.0];
//         }
//         let result: f64 = saddlepoint_approximation(
//             &prep_outputs.upgrade_arr,
//             // 38591813.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
//             // 25916.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
//             62010.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
//         );
//         if DEBUG {
//             dbg!(result);
//         }
//         if let Some(_cached_result) = read_cached_data::<f64>(test_name.as_str(), &hash) {
//         } else {
//             write_cached_data(test_name.as_str(), &hash, &result);
//         }
//         if DEBUG {
//             dbg!(start.elapsed());
//         }
//     }
// }
