use crate::parser::PreparationOutput;
use crate::parser::Upgrade;
use crate::parser::probability_distribution;

use rootfinder::{Interval, SolverSettings, root_bisection};
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

static DEBUG: bool = false;
static TOL: f64 = 1e-10;
#[derive(Clone)]
pub struct StateBundle {
    pub state: Vec<Vec<Vec<bool>>>,
    pub names: Vec<String>,
    pub state_index: Vec<Vec<Vec<i64>>>, // state_index[which_upgrade][what_number(0,1,2] = [indices that number]
    pub prob: f64,
}

fn increment_gold_cost(
    cur_gold_cost: &mut f64,
    upgrade: &Upgrade,
    state_bundle: &StateBundle,
    prep_output: &PreparationOutput,
    u_index: usize,
    p_index: usize,
) {
    *cur_gold_cost += upgrade.eqv_gold_per_tap;
    for (bit_index, bit) in state_bundle.state[u_index][p_index].iter().enumerate() {
        if *bit {
            let this_price = prep_output.juice_info.gold_costs[upgrade.upgrade_index][bit_index];
            if upgrade.is_weapon {
                *cur_gold_cost += this_price.0;
            } else {
                *cur_gold_cost += this_price.1;
            }
        }
    }
}
fn ks_01234(
    state_bundle: &StateBundle,
    prep_output: &PreparationOutput,
    theta: f64,
) -> (f64, f64, f64, f64, f64) {
    let mut total_k: f64 = 0.0;
    let mut total_k1: f64 = 0.0;
    let mut total_k2: f64 = 0.0;
    let mut total_k3: f64 = 0.0;
    let mut total_k4: f64 = 0.0;

    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        let mut alpha_arr: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
        let mut shift: f64 = f64::NEG_INFINITY;

        let mut cur_gold_cost = 0.0;
        let mut gold_cost_record: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
        for (p_index, l) in upgrade.log_prob_dist.iter().enumerate() {
            increment_gold_cost(
                &mut cur_gold_cost,
                upgrade,
                state_bundle,
                prep_output,
                u_index,
                p_index,
            );
            gold_cost_record.push(cur_gold_cost);
            let this_alpha: f64 = l + theta * cur_gold_cost;

            alpha_arr.push(this_alpha);
            shift = this_alpha.max(shift);
        }

        let mut s: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        let mut third: f64 = 0.0;
        let mut fourth: f64 = 0.0;

        if theta == 0.0 && DEBUG {
            dbg!(&alpha_arr);
        }

        let mut u_arr: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
        for aj in alpha_arr.iter() {
            let u: f64 = (aj - shift).exp(); // this can for sure be optimized into a polynomial TODO
            s += u;
            u_arr.push(u);
        }

        for (p_index, &u) in u_arr.iter().enumerate() {
            if u == 0.0 {
                continue;
            }
            let w = u / s;
            let x = gold_cost_record[p_index];
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

pub fn saddlepoint_approximation(
    prep_output: &PreparationOutput,
    state_bundle: &StateBundle,
    budget: f64,
    leftover: f64,
) -> f64 {
    // let (theta_hat, ks, ks1, ks2, ks3) = newton(upgrade_arr, budget);
    let f = |theta: f64| ks_01234(state_bundle, prep_output, theta).1 - budget;

    let settings = SolverSettings {
        vtol: Some(TOL),
        rebracket: Some(true),
        ..Default::default() // fill other fields with None
    };
    let mut min_value: f64 = 0.0;
    let mut max_value: f64 = 0.0; // pre-calculate this  TODO
    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        for (p_index, _) in upgrade.prob_dist.iter().enumerate() {
            let mut this_value = upgrade.eqv_gold_per_tap;
            for (bit_index, bit) in state_bundle.state[u_index][p_index].iter().enumerate() {
                if *bit {
                    let this_price =
                        prep_output.juice_info.gold_costs[upgrade.upgrade_index][bit_index];
                    if upgrade.is_weapon {
                        this_value += this_price.0;
                    } else {
                        this_value += this_price.1;
                    }
                }
            }

            max_value += this_value;

            if p_index <= 0 {
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
        for upgrade in prep_output.upgrade_arr.iter() {
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
        (ks, ks1, ks2, ks3, ks4) = ks_01234(state_bundle, prep_output, 0.0);

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
        (ks, ks1, ks2, ks3, ks4) = ks_01234(state_bundle, prep_output, theta_hat);
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
    state_bundle: &StateBundle,
    prep_output: &mut PreparationOutput,
    states_evaled: &mut i64,
    // depth: usize,
    // cache: &mut HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>>,
) -> f64 {
    for (index, upgrade) in prep_output.upgrade_arr.iter_mut().enumerate() {
        let new_extra: Vec<f64> = state_bundle.state[index]
            .iter()
            .map(|x| {
                x.iter().enumerate().fold(0.0, |last, (index, y)| {
                    if *y {
                        last + prep_output.juice_info.chances[upgrade.upgrade_index][index]
                    } else {
                        last
                    }
                })
            }) //if *x > 0 { upgrade.base_chance } else { 0.0 }) //
            .collect();

        upgrade.prob_dist =
            probability_distribution(upgrade.base_chance, upgrade.artisan_rate, &new_extra);
        upgrade.log_prob_dist = upgrade.prob_dist.iter().map(|x| x.ln()).collect();
    }

    *states_evaled += 1;
    let expected_leftover: f64 = expected_juice_leftover(prep_output, state_bundle);
    saddlepoint_approximation(
        prep_output,
        state_bundle,
        prep_output.base_gold_budget - expected_leftover,
        expected_leftover,
    )
}

fn expected_juice_leftover(prep_output: &PreparationOutput, state_bundle: &StateBundle) -> f64 {
    let mut avg_used: Vec<(f64, f64)> =
        vec![(0.0, 0.0); prep_output.juice_info.one_gold_cost_id.len()];
    let mut full_avg: Vec<(f64, f64)> =
        vec![(0.0, 0.0); prep_output.juice_info.one_gold_cost_id.len()];
    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        let mut used_so_far: Vec<(i64, i64)> = vec![(0, 0); prep_output.juice_info.ids.len()];
        let mut max_used: Vec<(i64, i64)> = vec![(0, 0); prep_output.juice_info.ids.len()];
        for (p_index, p) in upgrade.prob_dist.iter().enumerate() {
            for (bit_index, bit) in state_bundle.state[u_index][p_index].iter().enumerate() {
                // dbg!(&prep_output.juice_info);
                let id = prep_output.juice_info.ids[upgrade.upgrade_index][bit_index];

                if upgrade.is_weapon {
                    if *bit {
                        used_so_far[id].0 +=
                            prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].0;
                    }
                    max_used[id].0 +=
                        prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].0;
                } else {
                    if *bit {
                        used_so_far[id].1 +=
                            prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].1;
                    }
                    max_used[id].1 +=
                        prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].1;
                }

                avg_used[id].0 += p * used_so_far[id].0 as f64;
                avg_used[id].1 += p * used_so_far[id].1 as f64;

                full_avg[id].0 += p * max_used[id].0 as f64;
                full_avg[id].1 += p * max_used[id].1 as f64;
            }
        }
    }
    // dbg!(
    //     &prep_output.juice_info,
    //     &prep_output.juice_books_owned,
    //     &avg_used
    // );
    let mut total_gold: f64 = 0.0;
    for (id, a) in avg_used.iter().enumerate() {
        total_gold += ((prep_output.juice_books_owned[id].0 as f64).min(full_avg[id].0) - a.0)
            .max(0.0) as f64
            * prep_output.juice_info.one_gold_cost_id[id].0;
        total_gold += ((prep_output.juice_books_owned[id].1 as f64).min(full_avg[id].1) - a.1)
            .max(0.0) as f64
            * prep_output.juice_info.one_gold_cost_id[id].1;
    }
    total_gold
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::calculate_hash;
//     use crate::constants::RNG_SEED;
//     use crate::helpers::eqv_gold_unlock;
//     use crate::parser::PreparationOutput;
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

//         let mut prep_output: PreparationOutput = preparation(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             &user_mats_value,
//             adv_hone_strategy,
//         );

//         for upgrade in prep_output.upgrade_arr.iter_mut() {
//             let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
//             for i in upgrade.prob_dist.iter() {
//                 log_prob_dist.push(i.ln());
//             }
//             upgrade.log_prob_dist = log_prob_dist;
//             upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, &prep_output.mats_value);
//             let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
//             upgrade.eqv_gold_per_juice =
//                 &prep_output.mats_value[juice_ind] * upgrade.one_juice_cost as f64;
//             upgrade.juiced_arr = vec![0.0];
//         }
//         let result: f64 = saddlepoint_approximation(
//             &prep_output.upgrade_arr,
//             // 38591813.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
//             // 25916.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
//             62010.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.mats_value),
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
