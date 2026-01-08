use crate::brute::{MAX_BRUTE_SIZE, brute_naive};
use crate::constants::FLOAT_TOL;
use crate::performance::Performance;
use crate::saddlepoint_approximation::core::{THETA_LIMIT, THETA_TOL, ks_01234, my_newton};
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

pub static DEBUG: bool = false;

pub static MIN_LATTICE_SPAN: f64 = 1e-2;

pub fn saddlepoint_approximation_prob_wrapper(
    log_prob_dist_arr: &[Vec<f64>],
    prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    min_value: f64,
    max_value: f64,
    budget: f64,
    init_theta: &mut f64,
    performance: &mut Performance,
) -> f64 {
    let h = lattice_span(support_arr);
    if support_size_too_big(support_arr, budget, MAX_BRUTE_SIZE)
        && budget > min_value + h + FLOAT_TOL
        && budget < max_value - h - FLOAT_TOL
    {
        return saddlepoint_approximation(
            log_prob_dist_arr,
            support_arr,
            min_value,
            max_value, // here for debugging purpose only
            budget,
            init_theta,
            performance,
            false,
            &mut 6.9,
            6.9,
        );
    } else {
        performance.brute_count += 1;
        return brute_naive(prob_dist_arr, support_arr, budget);
    }
}

fn support_size_too_big(arr: &[Vec<f64>], budget: f64, max: usize) -> bool {
    let mut low_side_too_big: bool = false;
    let mut out: usize = 0;
    for a in arr {
        if out == 0 {
            out = a.len();
        } else {
            out *= a.iter().take_while(|x| **x <= budget).count().max(1);
            // the hope is that if budget is very close to min_value then we will use brute, because that will break SA
            if out >= max {
                low_side_too_big = true;
                // return true;
                break;
            }
        }
    }
    low_side_too_big //&& high_side_too_big // hopefully high side wont have any problems idk 
}

fn float_gcd(inp_a: f64, inp_b: f64) -> f64 {
    let mut a = inp_a;
    let mut b = inp_b;
    while b > FLOAT_TOL {
        (a, b) = (b, a % b);
        if (b - a).abs() < FLOAT_TOL {
            b = 0.0;
        }
    }
    a
}

fn lattice_span(support_arr: &[Vec<f64>]) -> f64 {
    let mut cur_span: f64 = MIN_LATTICE_SPAN;
    let mut found_non_zeros: bool = false;

    for s in support_arr.iter().flatten() {
        if *s < FLOAT_TOL {
            continue;
        }
        if !found_non_zeros {
            cur_span = *s;
            found_non_zeros = true;
        } else {
            cur_span = float_gcd(*s, cur_span);
        }
        if cur_span < MIN_LATTICE_SPAN {
            return MIN_LATTICE_SPAN; // always do a little bit of cont correction ig cos why not 
        }
    }
    // if !cur_span.is_finite() || cur_span < 1.0 {
    //     dbg!(cur_span);
    //     panic!();
    // }
    cur_span
}
pub fn saddlepoint_approximation(
    log_prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    min_value: f64,
    max_value: f64,
    inp_budget: f64,
    init_theta: &mut f64,

    performance: &mut Performance,
    compute_derivative: bool,
    derivative_output: &mut f64,
    mean: f64,
) -> f64 {
    let span = lattice_span(support_arr);

    if inp_budget < min_value - FLOAT_TOL + span {
        performance.trivial_count += 1;
        return 0.0;
    }

    if inp_budget > max_value + FLOAT_TOL - span {
        performance.trivial_count += 1;
        return 1.0;
    }
    if min_value + FLOAT_TOL + span > max_value - FLOAT_TOL - span {
        // this is just for when support is all 0s, idek what the condition should be
        performance.trivial_count += 1;
        return if min_value <= inp_budget { 1.0 } else { 0.0 };
    }

    let budget = (inp_budget / span).floor() * span + span / 2.0;
    performance.sa_count += 1;

    // dbg!(h);
    let f_df = |theta| {
        let mut ks_tuple = (0.0, 0.0, 0.0, 0.0, 0.0);
        ks_01234(
            &log_prob_dist_arr,
            &support_arr,
            theta,
            &mut ks_tuple,
            &(false, true, true, false, false),
        );
        (ks_tuple.1 - budget, ks_tuple.2)
    };
    // f_df(1.0).0.signum() == f_df(-1.0).0.signum()

    let result_opt = my_newton(&f_df, *init_theta, performance);

    if DEBUG || result_opt.is_none() {
        dbg!(
            budget,
            min_value,
            max_value,
            log_prob_dist_arr
                .iter()
                .map(|x| x.iter().map(|y| y.exp()).sum::<f64>())
                .collect::<Vec<f64>>(),
            log_prob_dist_arr
                .iter()
                .map(|x| x.iter().map(|y| y.exp()).collect())
                .collect::<Vec<Vec<f64>>>(),
            support_arr
        );
    }
    let result = result_opt.unwrap();
    // if DEBUG
    //     || !result.0.is_finite()
    //     || !result.1.is_finite()
    //     || f_df(THETA_LIMIT).0.signum() == f_df(-THETA_LIMIT).0.signum()
    // {
    //     //   (this means budget is outside of range)
    //     dbg!(
    //         f_df(THETA_LIMIT),
    //         f_df(1.0),
    //         f_df(0.0),
    //         f_df(-1.0),
    //         f_df(-THETA_LIMIT),
    //         result,
    //     );
    // }

    let (theta_hat, last_theta) = result;

    *init_theta = theta_hat;
    let theta_error = (theta_hat - last_theta).abs();
    if DEBUG {
        dbg!(theta_hat, theta_error);
    }
    let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap(); // TODO can i pre initialize this or is there no point

    let mut ks_tuple = (0.0, 0.0, 0.0, 0.0, 0.0);
    performance.ks_count += 1;
    ks_01234(
        &log_prob_dist_arr,
        &support_arr,
        theta_hat,
        &mut ks_tuple,
        &(true, true, true, true, true),
    );

    let w = |t: f64, ks_inp: f64| t.signum() * (2.0 * (t * budget - ks_inp)).sqrt();
    let u = |t: f64, ks2_inp: f64| 2.0 / span * (span * t / 2.0).sinh() * ks2_inp.sqrt(); // second continuity correction 
    let w_hat = w(theta_hat, ks_tuple.0);
    let u_hat = u(theta_hat, ks_tuple.2);

    let mut error: f64 = 0.0;
    let correction_multiplier = 1.0 / w_hat - 1.0 / u_hat;
    let sa_out: f64 = normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * correction_multiplier;
    if theta_hat.abs() < THETA_TOL * 100.0 || theta_error / theta_hat < 0.01 {
        // this theta error / theta hat checkshould only trigger when newton fails after 20 cycles
        let mut last_ks_tuple = (0.0, 0.0, 0.0, 0.0, 0.0);
        performance.ks_count += 1;
        ks_01234(
            &log_prob_dist_arr,
            &support_arr,
            last_theta,
            &mut last_ks_tuple,
            &(true, false, true, false, false),
        );

        let w_last = w(last_theta, last_ks_tuple.0);
        let u_last = u(last_theta, last_ks_tuple.2);

        let old_out =
            normal_dist.cdf(w_last) + normal_dist.pdf(w_last) * (1.0 / w_last - 1.0 / u_last);
        error = (sa_out - old_out).abs();
    }

    let mut approx: f64 = -6.9;
    let mut actual_out = sa_out;
    if error > 1e-2 || !error.is_finite() {
        performance.edgeworth_count += 1;
        let std = ks_tuple.2.sqrt();
        let z = (budget - ks_tuple.1) / std;

        let gamma3 = ks_tuple.3 / std.powi(3); // skewness
        let gamma4 = ks_tuple.4 / std.powi(4); // excess kurtosis

        let pdf = normal_dist.pdf(z);
        let cdf = normal_dist.cdf(z);

        // Edgeworth (cdf) up to 4th cumulant and k3^2 term:
        let cdf_correction = pdf
            * ((gamma3 / 6.0) * (z * z - 1.0)
                + (gamma4 / 24.0) * (z * z * z - 3.0 * z)
                + (gamma3 * gamma3 / 72.0) * (z * z * z * z * z - 10.0 * z * z * z + 15.0 * z));

        approx = cdf - cdf_correction;
        if DEBUG || approx < 0.0 || approx > 1.0 {
            dbg!(
                error,
                budget - ks_tuple.1,
                z,
                std,
                gamma3,
                gamma4,
                cdf,
                pdf,
                cdf_correction,
                approx
            );
        }
        if compute_derivative {
            dbg!("edge");
            *derivative_output = -normal_dist.pdf(w_hat) * (budget - mean) / w_hat;
        }
        actual_out = approx;
    } else {
        let std = ks_tuple.2.sqrt();
        let gamma3 = ks_tuple.3 / std.powi(3); // skewness
        performance.lugganani_count += 1;
        if compute_derivative {
            // dbg!(
            //     -normal_dist.pdf(w_hat) / theta_hat,
            //     -normal_dist.pdf(w_hat) * (budget - mean) / w_hat,
            //     -normal_dist.pdf(w_hat) / theta_hat
            //         * (1.0 + 1.0 / ((theta_hat * std).powi(2)) - gamma3 / (2.0 * theta_hat * std))
            // );
            // *derivative_output = -normal_dist.pdf(w_hat) / theta_hat
            //     * (1.0 + 1.0 / ((theta_hat * std).powi(2)) - gamma3 / (2.0 * theta_hat * std))

            *derivative_output = normal_dist.pdf(w_hat)
                * ((budget - mean) / u_hat
                    - 1.0 / (u_hat * theta_hat)
                    - (budget - mean) / w_hat.powi(3) * (1.0 - w_hat / u_hat))
        }
    }

    if DEBUG || actual_out < 0.0 || actual_out > 1.0 || !actual_out.is_finite() {
        dbg!(
            f_df(THETA_LIMIT),
            f_df(1.0),
            f_df(0.0),
            f_df(-1.0),
            f_df(-THETA_LIMIT),
        );
        dbg!(theta_hat, theta_error);
        dbg!(w_hat, u_hat, error, sa_out);
        dbg!(
            log_prob_dist_arr
                .iter()
                .map(|x| x.iter().map(|y| y.exp()).sum::<f64>())
                .collect::<Vec<f64>>(),
            log_prob_dist_arr
                .iter()
                .map(|x| x.iter().map(|y| y.exp()).collect())
                .collect::<Vec<Vec<f64>>>(),
            support_arr
        );
        dbg!(
            theta_hat,
            ks_tuple,
            ks_tuple.1,
            ks_tuple.2,
            ks_tuple.3,
            2.0 * (theta_hat * budget - ks_tuple.0),
            w_hat,
            u_hat,
            normal_dist.cdf(w_hat),
            normal_dist.pdf(w_hat),
            1.0 / w_hat - 1.0 / u_hat,
            min_value,
            budget,
            max_value,
            sa_out,
            approx,
            actual_out
        );
        println!("==============================");
        panic!();
    }

    actual_out.max(0.0).min(1.0) // head in the sand
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
//         let user_price_arr = DEFAULT_GOLD_VALUES;
//         let hash: String = calculate_hash!(
//             &hone_counts,
//             &adv_counts,
//             adv_hone_strategy,
//             express_event,
//             &input_budgets,
//             &user_price_arr,
//             RNG_SEED,
//             PROB_MODE
//         );

//         let mut prep_output: PreparationOutput = preparation(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             &user_price_arr,
//             adv_hone_strategy,
//         );

//         for upgrade in prep_output.upgrade_arr.iter_mut() {
//             let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
//             for i in upgrade.prob_dist.iter() {
//                 log_prob_dist.push(i.ln());
//             }
//             upgrade.log_prob_dist = log_prob_dist;
//             upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, &prep_output.price_arr);
//             let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
//             upgrade.eqv_gold_per_juice =
//                 &prep_output.price_arr[juice_ind] * upgrade.one_juice_cost as f64;
//             upgrade.juiced_arr = vec![0.0];
//         }
//         let result: f64 = saddlepoint_approximation(
//             &prep_output.upgrade_arr,
//             // 38591813.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
//             // 25916.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
//             62010.0 - eqv_gold_unlock(&prep_output.unlock_costs, &prep_output.price_arr),
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
