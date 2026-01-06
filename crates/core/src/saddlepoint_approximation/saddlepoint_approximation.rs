use statrs::distribution::{Continuous, ContinuousCDF, Normal};

use crate::brute::{MAX_BRUTE_SIZE, brute_naive};
use crate::constants::FLOAT_TOL;

pub static DEBUG: bool = false;
pub static THETA_TOL: f64 = 1e-10;
pub static THETA_LIMIT: f64 = 1e2; // th
pub static MIN_LATTICE_SPAN: f64 = 1.0;

pub fn ks_01234(
    log_prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    theta: f64,
    (total_k, total_k1, total_k2, total_k3, total_k4): &mut (f64, f64, f64, f64, f64),
    toggle: &(bool, bool, bool, bool, bool), // honestly the performance gain here is probably so negligible compared to all the exp and ln calls but whatever
) {
    for (u_index, log_prob_dist) in log_prob_dist_arr.iter().enumerate() {
        let support = &support_arr[u_index];
        let mut ignore: bool = true;
        for i in support {
            if *i > 0.0 {
                ignore = false;
                break;
            }
        }
        if ignore {
            continue;
        }

        let mut alpha_arr: Vec<f64> = Vec::with_capacity(log_prob_dist.len());
        let mut alpha_shift: f64 = f64::NEG_INFINITY;

        let mut sanity_check: f64 = 0.0;
        for (p_index, l) in log_prob_dist.iter().enumerate() {
            let this_alpha: f64 = l + theta * support[p_index];

            alpha_arr.push(this_alpha);
            alpha_shift = this_alpha.max(alpha_shift);
            sanity_check += l.exp();
        }
        if (1.0 - sanity_check).abs() > FLOAT_TOL {
            dbg!(
                sanity_check,
                log_prob_dist,
                log_prob_dist_arr
                    .iter()
                    .map(|x| x.iter().map(|y| y.exp()).sum::<f64>())
                    .collect::<Vec<f64>>(),
                log_prob_dist_arr
                    .iter()
                    .map(|x| x.iter().map(|y| y.exp()).collect())
                    .collect::<Vec<Vec<f64>>>(),
            );
            panic!();
        }

        let mut s: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        let mut third: f64 = 0.0;
        let mut fourth: f64 = 0.0;

        // if theta == 0.0 && DEBUG {
        //     dbg!(&alpha_arr);
        // }

        let mut u_arr: Vec<f64> = Vec::with_capacity(log_prob_dist.len());
        for aj in alpha_arr.iter() {
            if *aj == f64::NEG_INFINITY {
                // just to make it explicit, i think exp does this anyway
                u_arr.push(0.0);
                continue;
            }
            let u: f64 = (aj - alpha_shift).exp(); // i dont think this can be turned into a poly? cos cur_gold_cost is not linear (maybe do this for special?)
            s += u;
            u_arr.push(u);
        }

        for (p_index, &u) in u_arr.iter().enumerate() {
            if u == 0.0 {
                //   l = -inf , p = 0
                continue;
            }
            let w = u / s;
            let x = support[p_index];

            if toggle.1 || toggle.2 || toggle.3 || toggle.4 {
                mean += x * w;
                if toggle.2 || toggle.3 || toggle.4 {
                    let x2: f64 = x * x;
                    second += x2 * w;
                    if toggle.3 || toggle.4 {
                        let x3: f64 = x2 * x;
                        third += x3 * w;
                        if toggle.4 {
                            fourth += x3 * x * w;
                        }
                    }
                }
            }
        }

        if toggle.0 {
            *total_k += alpha_shift + s.ln();
        }
        let mut mu2: f64 = -1.0;
        if toggle.1 {
            *total_k1 += mean;
        }
        if toggle.2 || toggle.4 {
            mu2 = (second - mean * mean).max(0.0);
            *total_k2 += mu2;
        }
        if toggle.3 {
            let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
            *total_k3 += mu3;
        }
        if toggle.4 {
            if !toggle.2 {
                mu2 = (second - mean * mean).max(0.0);
            }
            let mu4 = fourth - 4.0 * third * mean + 6.0 * second * mean * mean
                - 3.0 * mean * mean * mean * mean;
            *total_k4 += mu4 - 3.0 * mu2 * mu2;
        }
    }
}

pub fn find_root<F>(
    mut func: F,
    init_theta: f64,
    min: f64,
    max: f64,
    tol: f64,
    max_iter: usize,
) -> Option<(f64, f64)>
where
    F: FnMut(f64) -> (f64, f64),
{
    let mut theta: f64 = init_theta.max(min).min(max);

    let mut count = 0;
    let (mut y, mut dy) = func(theta);
    loop {
        // dbg!(count, theta, y, dy);
        // if y.abs() < 1e-12 {
        //     // this is largely irrelevant because we're interested in theta
        //     return Some(theta);
        // }

        if dy == 0.0 {
            // let (y_min, _) = func(min);
            // let (y_max, _) = func(max);
            dbg!(y, dy, theta);
            // this shouldn't happen no more
            return None;
            // return Some((
            //     if y_min.abs() < y_max.abs() { min } else { max },
            //     if y_min.abs() < y_max.abs() {
            //         min + THETA_TOL
            //     } else {
            //         max - THETA_TOL
            //     },
            // ));
        }

        let proposed_delta: f64 = -y / dy;
        let mut cur_delta: f64 = proposed_delta;
        let mut damping_factor = 1.0;
        let mut new_theta: f64 = theta + cur_delta;
        let (mut new_y, mut new_dy);
        loop {
            (new_y, new_dy) = func(new_theta);

            // If the error (magnitude of y) got worse, reduce step size
            if new_y.abs() > y.abs() || new_dy == 0.0 {
                damping_factor *= 0.5;
                cur_delta = proposed_delta * damping_factor;
                new_theta = theta + cur_delta;

                if damping_factor < 1e-3 {
                    break;
                }
            } else {
                // The step is good, it reduced the error
                break;
            }
        }
        if !new_theta.is_finite() {
            dbg!(y, dy, proposed_delta, theta);
            return None;
        }

        // assert!(new_theta < max && new_theta > min);

        count += 1;
        if (new_theta - theta).abs() < tol || count >= max_iter {
            return Some((new_theta, theta));
        }

        theta = new_theta;
        (y, dy) = (new_y, new_dy);
    }
}

pub fn my_newton<F>(f_df: F, init_theta: f64) -> Option<(f64, f64)>
where
    F: FnMut(f64) -> (f64, f64),
{
    let root = find_root(f_df, init_theta, -THETA_LIMIT, THETA_LIMIT, THETA_TOL, 20); // i mean its usually like 3 iters but idk 
    root
}

pub fn saddlepoint_approximation_wrapper(
    log_prob_dist_arr: &[Vec<f64>],
    prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    min_value: f64,
    max_value: f64,
    budget: f64,
    init_theta: &mut f64,
) -> f64 {
    if budget < min_value - FLOAT_TOL {
        return 0.0;
    }

    if budget > max_value + FLOAT_TOL {
        return 1.0;
    }
    if min_value == max_value {
        return if min_value <= 0.0 { 1.0 } else { 0.0 };
    }
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
            h,
        );
    } else {
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
    low_side_too_big //&& high_side_too_big
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
    span: f64,
) -> f64 {
    let budget = (inp_budget / span).floor() * span + span / 2.0;
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
    let result_opt = my_newton(&f_df, *init_theta);

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
    if DEBUG
        || !result.0.is_finite()
        || !result.1.is_finite()
        || f_df(THETA_LIMIT).0.signum() == f_df(-THETA_LIMIT).0.signum()
    {
        //   (this means budget is outside of range)
        dbg!(
            f_df(THETA_LIMIT),
            f_df(1.0),
            f_df(0.0),
            f_df(-1.0),
            f_df(-THETA_LIMIT),
            result,
        );
    }

    let (theta_hat, last_theta) = result;

    *init_theta = theta_hat;
    let theta_error = (theta_hat - last_theta).abs();
    if DEBUG {
        dbg!(theta_hat, theta_error);
    }
    let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap(); // TODO can i pre initialize this or is there no point

    let mut ks_tuple = (0.0, 0.0, 0.0, 0.0, 0.0);

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
    let sa_out: f64 = normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat);
    if theta_hat.abs() < THETA_TOL * 100.0 || theta_error / theta_hat < 0.01 {
        // this theta error / theta hat checkshould only trigger when newton fails after 20 cycles
        let mut last_ks_tuple = (0.0, 0.0, 0.0, 0.0, 0.0);

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

    // if !error.is_finite() || !out.is_finite() {
    //     dbg!(
    //         theta_hat,
    //         theta_error,
    //         w_hat,
    //         u_hat,
    //         w_last,
    //         u_last,
    //         error,
    //         out,
    //         old_out
    //     );
    // }
    // //  > 1% raw error in the end, hopefully edgeworth can do better
    // // this is supposedly caused by a very small theta but instead of checking for small theta we check for large error,
    // // maybe change this later to only check error when theta is small TODO
    // else {
    let mut approx: f64 = -6.9;
    let mut actual_out = sa_out;
    if error > 1e-2 || !error.is_finite() {
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
        actual_out = approx;
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
