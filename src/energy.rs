use crate::parser::Upgrade;

use core::f64;

use crate::helpers::eqv_gold_per_tap;
use num::Float;
use num::complex::Complex64;
use quad_rs::{EvaluationError, Integrable, Integrator};
use rootfinder::{Interval, SolverSettings, root_bisection};
use statrs::distribution::{Continuous, ContinuousCDF, Normal};
use std::f64::consts::PI;
use std::ops::Range;
static TOL: f64 = 1e-6;
fn newton(upgrade_arr: &[Upgrade], budget: f64) -> (f64, f64, f64, f64, f64) {
    let mut theta: f64 = 0.0;
    let (mut ks, mut ks1, mut ks2, mut ks3) = ks_0123(upgrade_arr, theta);
    let mut converged = false;
    for i in 0..20 {
        // dbg!(ks1, budget);

        let f = ks1 - budget;

        if f.abs() < 1e-6 {
            converged = true;
            break;
        }
        if ks2 == 0.0 {
            // dbg!("newton failed", budget, ks1,);
            // break;
            ks2 = 1.0;
        }
        theta = theta - f / ks2;
        (ks, ks1, ks2, ks3) = ks_0123(upgrade_arr, theta);
    }
    // if (ks1 - budget).abs() > 1e-6 {
    //     dbg!(theta, ks, ks1, ks2, budget, "failed");
    // }
    if theta.abs() < 1e-8 {
        // dbg!(theta, ks, ks1, ks2, budget, f, i);
        theta = 0.0;
        (ks, ks1, ks2, ks3) = ks_0123(upgrade_arr, theta);
    }
    (theta, ks, ks1, ks2, ks3)
}

pub fn ks_0123(upgrade_arr: &[Upgrade], theta: f64) -> (f64, f64, f64, f64) {
    let mut total_k: f64 = 0.0;
    let mut total_k1: f64 = 0.0;
    let mut total_k2: f64 = 0.0;
    let mut total_k3: f64 = 0.0;

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
        let mut cur_juice_count = 0;
        // dbg!(&alpha_arr);
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
            // dbg!(x, w, u, s);
        }

        total_k += shift + s.ln();
        total_k1 += mean;
        total_k2 += (second - mean * mean).max(0.0);
        total_k3 += third - 3.0 * second * mean + 2.0 * mean * mean * mean;
    }

    (total_k, total_k1, total_k2, total_k3)
}

pub fn saddlepoint_approximation(upgrade_arr: &[Upgrade], budget: f64) -> f64 {
    // let (theta_hat, ks, ks1, ks2, ks3) = newton(upgrade_arr, budget);
    let f = |theta: f64| ks_0123(upgrade_arr, theta).1 - budget;

    let settings = SolverSettings {
        vtol: Some(1e-6),
        rebracket: Some(true),
        ..Default::default() // fill other fields with None
    };
    let mut min_value: f64 = 0.0;
    let mut max_value: f64 = 0.0; // pre-calculate this  TODO
    for upgrade in upgrade_arr {
        for (index, i) in upgrade.prob_dist.iter().enumerate() {
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
        // if !result.is_err() {
        // dbg!(
        //     f(10000.0),
        //     f(1.0),
        //     f(0.0),
        //     f(-1.0),
        //     f(-10000.0),
        //     budget,
        //     min_value,
        //     max_value,
        //     &upgrade_arr[0].juiced_arr,
        //     &upgrade_arr[1].juiced_arr,
        //     &upgrade_arr[0].prob_dist,
        //     &upgrade_arr[1].prob_dist,
        //     // result.unwrap(),
        // );
        // }
        // panic!();
        return 0.0;
    }
    if budget < min_value + TOL {
        let mut prob: f64 = 1.0;
        for upgrade in upgrade_arr {
            prob *= upgrade.prob_dist[0];
        }
        return prob;
    }
    if budget > max_value - TOL {
        // if !result.is_err() {
        // dbg!(
        //     f(10000.0),
        //     f(1.0),
        //     f(0.0),
        //     f(-1.0),
        //     f(-10000.0),
        //     budget,
        //     min_value,
        //     max_value,
        //     // result.unwrap(),
        // );
        // }
        // // panic!();
        return 1.0;
    }
    if result.is_err() {
        // dbg!(
        //     f(10000.0),
        //     f(1.0),
        //     f(0.0),
        //     f(-1.0),
        //     f(-10000.0),
        //     budget,
        //     min_value,
        //     max_value,
        //     &upgrade_arr[0].eqv_gold_per_tap,
        //     &upgrade_arr[1].eqv_gold_per_tap,
        //     &upgrade_arr[0].eqv_gold_per_juice,
        //     &upgrade_arr[1].eqv_gold_per_juice,
        //     &upgrade_arr[0].juiced_arr,
        //     &upgrade_arr[1].juiced_arr,
        //     &upgrade_arr[0].prob_dist,
        //     &upgrade_arr[1].prob_dist,
        // );
    }
    // dbg!(
    //     f(10000.0),
    //     f(1.0),
    //     f(0.0),
    //     f(-1.0),
    //     f(-10000.0),
    //     budget,
    //     min_value,
    //     max_value,
    //     &upgrade_arr[0].eqv_gold_per_tap,
    //     &upgrade_arr[1].eqv_gold_per_tap,
    //     &upgrade_arr[0].eqv_gold_per_juice,
    //     &upgrade_arr[1].eqv_gold_per_juice,
    //     &upgrade_arr[0].juiced_arr,
    //     &upgrade_arr[1].juiced_arr,
    //     &upgrade_arr[0].prob_dist,
    //     &upgrade_arr[1].prob_dist,
    // );
    let (ks, ks1, ks2, ks3);

    let mut theta_hat;

    theta_hat = result.unwrap();
    (ks, ks1, ks2, ks3) = ks_0123(upgrade_arr, theta_hat);
    if theta_hat.abs() < 1e-6 {
        theta_hat = 0.0;
    }

    let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap(); // TODO can i pre initialize this or is there no point
    // dbg!(theta_hat, budget);
    if theta_hat == 0.0 {
        // pre-calculate K(0) and stuff TODO
        let std = ks2.sqrt();
        let z = (budget - ks1) / std;
        // dbg!(1);
        // dbg!(
        //     theta_hat,
        //     ks,
        //     ks1,
        //     ks2,
        //     ks3,
        //     budget - ks1,
        //     std,
        //     normal_dist.cdf(z) + normal_dist.pdf(z) * ks3 / (6.0 * std * std * std) * (1.0 - z * z)
        // );
        normal_dist.cdf(z) + normal_dist.pdf(z) * ks3 / (6.0 * std * std * std) * (1.0 - z * z)
    } else {
        let w_hat: f64 = theta_hat.signum() * (2.0 * (theta_hat * budget - ks)).sqrt();
        let u_hat: f64 = theta_hat * ks2.sqrt();
        // dbg!(2);
        let out = normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat);
        if out < 0.0 || out > 1.0 || ks2.abs() < 1e-12 {
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
        out
    }
    //P(S<B)≈Φ(w^)+ϕ(w^)(w^1​−u^1​)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::constants::RNG_SEED;
    use crate::helpers::eqv_gold_unlock;
    use crate::parser::preparation;
    use crate::parser::{PreparationOutputs, Upgrade, probability_distribution};
    use crate::test_utils::*;
    use std::time::Instant;
    #[test]
    fn saddlepoint_approximation_test() {
        let start = Instant::now();
        let test_name = format!("saddlepoint_approximation_test");
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 9 { 5 } else { 0 }).collect(),
            (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
        ];
        // let hone_counts: Vec<Vec<i64>> =
        //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let input_budgets = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value,
            RNG_SEED,
            PROB_MODE
        );

        let mut prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );

        for upgrade in prep_outputs.upgrade_arr.iter_mut() {
            let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
            for i in upgrade.prob_dist.iter() {
                log_prob_dist.push(i.ln());
            }
            upgrade.log_prob_dist = log_prob_dist;
            upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, &prep_outputs.mats_value);
            upgrade.juiced_arr = vec![0.0];
        }
        let result: f64 = saddlepoint_approximation(
            &prep_outputs.upgrade_arr,
            // 38591813.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
            // 25916.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
            62010.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
        );
        dbg!(result);
        if let Some(_cached_result) = read_cached_data::<f64>(test_name.as_str(), &hash) {
        } else {
            write_cached_data(test_name.as_str(), &hash, &result);
        }
        dbg!(start.elapsed());
    }
}
