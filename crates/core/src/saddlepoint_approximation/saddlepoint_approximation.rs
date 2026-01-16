// use std::f64::consts::PI;

use std::f64::NAN;

use crate::brute::{MAX_BRUTE_SIZE, brute_average_recursive, brute_success_prob};
use crate::constants::FLOAT_TOL;
use crate::helpers::TripletIterator;

use crate::saddlepoint_approximation::average::{DEBUG_AVERAGE, DEBUG_AVG_INDEX};
use crate::saddlepoint_approximation::core::{THETA_LIMIT, THETA_TOL};
use crate::state_bundle::StateBundle;
use crate::upgrade::Support;

use itertools::Itertools;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

pub static DEBUG_SA: bool = false;

pub static MIN_LATTICE_SPAN: f64 = 1.0;

fn support_size_too_big<'a, I>(arr: I, budget: f64, max: usize) -> bool
where
    I: TripletIterator<'a>,
{
    let mut low_side_too_big: bool = false;
    let mut out: usize = 0;
    for a in arr {
        if out == 0 {
            out = a.len();
        } else {
            out *= a
                .iter()
                .take_while(|x| x.0 + FLOAT_TOL <= budget)
                .count()
                .max(1);
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

fn lattice_span<'a, I>(meta_support_arr: I) -> f64
where
    I: Iterator<Item = &'a Support>,
{
    let mut cur_span: f64 = MIN_LATTICE_SPAN;
    let mut found_non_zeros: bool = false;

    for support in meta_support_arr {
        // for (index, (s, _, _)) in support.access_collapsed().iter().enumerate() {
        //     if support.linear && index > 1 {
        //         break;
        //     }
        if support.gap_size < FLOAT_TOL || support.ignore {
            continue;
        }
        if !found_non_zeros {
            cur_span = support.gap_size;
            found_non_zeros = true;
        } else {
            cur_span = float_gcd(support.gap_size, cur_span);
        }
        if cur_span < MIN_LATTICE_SPAN {
            return MIN_LATTICE_SPAN; // always do a little bit of cont correction ig cos why not 
        }
        // }
    }
    // if !cur_span.is_finite() || cur_span < 1.0 {
    //     dbg!(cur_span);
    //     panic!();
    // }
    cur_span
}

impl StateBundle {
    pub fn saddlepoint_approximation_prob_wrapper(
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        init_theta: &mut f64,

        compute_biased: bool,
        mean: f64,
    ) -> f64 {
        let (min_value, max_value) = self.find_min_max(support_index, skip_count, compute_biased);

        let span = lattice_span(self.extract_support_with_meta(support_index, skip_count));
        if budget > max_value + FLOAT_TOL {
            // self.performance.trivial_count += 1;
            return 1.0;
        }

        if budget < min_value - FLOAT_TOL {
            // self.performance.trivial_count += 1;
            return 0.0;
        }
        if support_size_too_big(
            self.extract_triplet(support_index, skip_count),
            budget,
            MAX_BRUTE_SIZE,
        ) && budget > min_value + span / 2.0 + FLOAT_TOL
            && budget < max_value - span / 2.0 - FLOAT_TOL
        {
            return self.saddlepoint_approximation(
                support_index,
                skip_count,
                min_value,
                max_value,
                span,
                budget,
                init_theta,
                compute_biased,
                mean,
            );
        } else {
            // self.performance.brute_count += 1;
            // let prob_dist_arr: Vec<Vec<f64>> =
            //     self.extract_prob(skip_count).into_iter().cloned().collect();
            // dbg!(
            //     probs
            //         .iter()
            //         .map(|x| x.into_iter().sum::<f64>())
            //         .collect::<Vec<f64>>()

            if compute_biased {
                if support_index == DEBUG_AVG_INDEX && DEBUG_AVERAGE {
                    dbg!("brute");
                }

                return brute_average_recursive(
                    &self.gather_collapsed(support_index, skip_count, 1),
                    &self.gather_collapsed(support_index, skip_count, 0),
                    budget,
                    mean,
                );
            } else {
                return brute_success_prob(
                    &self.gather_collapsed(support_index, skip_count, 1),
                    &self.gather_collapsed(support_index, skip_count, 0),
                    budget,
                );
            }
        }
    }

    pub fn saddlepoint_approximation(
        &self,
        support_index: i64,
        skip_count: usize,
        min_value: f64,
        max_value: f64,
        span: f64,
        inp_budget: f64,
        init_theta: &mut f64,
        compute_biased: bool,
        mean: f64,
    ) -> f64 {
        let budget = ((inp_budget / span).floor() * span)
            .min(max_value - span)
            .max(min_value)
            + span / 2.0;
        // self.performance.sa_count += 1;
        let mean_log = if compute_biased { mean.ln() } else { NAN };

        // dbg!(h);

        // f_df(1.0).0.signum() == f_df(-1.0).0.signum()

        let result_opt = self.my_newton(
            *init_theta,
            compute_biased,
            mean_log,
            support_index,
            skip_count,
            budget,
        );

        if DEBUG_SA || result_opt.is_none() {
            dbg!(budget, min_value, max_value,);
            println!(
                "{:?}",
                self.ks(
                    -THETA_LIMIT,
                    &(false, true, true, false, false),
                    compute_biased,
                    mean_log,
                    support_index,
                    skip_count
                )
            );
            println!(
                "{:?}",
                self.ks(
                    -1.0,
                    &(false, true, true, false, false),
                    compute_biased,
                    mean_log,
                    support_index,
                    skip_count
                )
            );
            println!(
                "{:?}",
                self.ks(
                    0.0,
                    &(false, true, true, false, false),
                    compute_biased,
                    mean_log,
                    support_index,
                    skip_count
                )
            );
            println!(
                "{:?}",
                self.ks(
                    1.0,
                    &(false, true, true, false, false),
                    compute_biased,
                    mean_log,
                    support_index,
                    skip_count
                )
            );
            println!(
                "{:?}",
                self.ks(
                    THETA_LIMIT,
                    &(false, true, true, false, false),
                    compute_biased,
                    mean_log,
                    support_index,
                    skip_count
                )
            );
            dbg!(
                self.extract_triplet(support_index, skip_count)
                    .into_iter()
                    .map(|x| x.iter().map(|(_, y)| y).sum::<f64>())
                    .collect::<Vec<f64>>(),
                self.extract_triplet(support_index, skip_count)
                    .into_iter()
                    .map(|x| x.iter().map(|(_, y)| *y).collect())
                    .collect::<Vec<Vec<f64>>>(),
                self.extract_support_with_meta(support_index, skip_count)
                    .into_iter()
                    .collect::<Vec<&Support>>(),
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

        let (theta_hat, last_theta, _) = result;
        // self.performance.newton_iterations += count as i64 - 1;
        // self.performance.ks_count += count as i64;
        *init_theta = theta_hat;
        let theta_error = (theta_hat - last_theta).abs();
        if DEBUG_SA {
            dbg!(theta_hat, theta_error);
        }
        let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap();
        // self.performance.ks_count += 1;
        let ks_tuple = self.ks(
            theta_hat,
            &(true, true, true, true, true),
            compute_biased,
            mean_log,
            support_index,
            skip_count,
        );

        let w = |t: f64, ks_inp: f64| t.signum() * (2.0 * (t * budget - ks_inp)).sqrt();
        let u = |t: f64, ks2_inp: f64| 2.0 / span * (span * t / 2.0).sinh() * ks2_inp.sqrt(); // second continuity correction 
        let w_hat = w(theta_hat, ks_tuple.0);
        let u_hat = u(theta_hat, ks_tuple.2);

        let mut error: f64 = 0.0;

        let sa_out: f64 =
            normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat);
        if theta_hat.abs() < THETA_TOL * 100.0 || theta_error / theta_hat < 0.01 {
            // this theta error / theta hat checkshould only trigger when newton fails after 20 cycles
            // self.performance.ks_count += 1;
            let last_ks_tuple = self.ks(
                theta_hat,
                &(true, true, true, true, true),
                compute_biased,
                mean_log,
                support_index,
                skip_count,
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
            // self.performance.edgeworth_count += 1;
            let std = ks_tuple.2.sqrt();
            let z = (budget - ks_tuple.1) / std;

            let gamma3 = ks_tuple.3 / std.powi(3); // skewness

            let pdf = normal_dist.pdf(z);
            let cdf = normal_dist.cdf(z);

            // Edgeworth (cdf) up to 4th cumulant and k3^2 term:
            let cdf_correction = pdf
                * ((gamma3 / 6.0) * (z.powi(2) - 1.0)
                    + if compute_biased {
                        0.0
                    } else {
                        let gamma4 = ks_tuple.4 / std.powi(4); // excess kurtosis
                        (gamma4 / 24.0) * (z.powi(3) - 3.0 * z)
                            + (gamma3 * gamma3 / 72.0) * (z.powi(5) - 10.0 * z.powi(3) + 15.0 * z)
                    });

            approx = cdf - cdf_correction;
            if DEBUG_SA || approx < 0.0 || approx > 1.0 {
                dbg!(
                    error,
                    budget - ks_tuple.1,
                    z,
                    std,
                    gamma3,
                    cdf,
                    pdf,
                    cdf_correction,
                    approx
                );
            }
            if DEBUG_AVERAGE && support_index == DEBUG_AVG_INDEX {
                dbg!("edge", approx, actual_out);
            }
            actual_out = approx;
        } else {
            // performance.lugganani_count += 1;
            // if compute_biased {
            //     // dbg!(
            //     //     -normal_dist.pdf(w_hat) / theta_hat,
            //     //     -normal_dist.pdf(w_hat) * (budget - mean) / w_hat,
            //     //     -normal_dist.pdf(w_hat) / theta_hat
            //     //         * (1.0 + 1.0 / ((theta_hat * std).powi(2)) - gamma3 / (2.0 * theta_hat * std))
            //     // );
            //     // *derivative_output = -normal_dist.pdf(w_hat) / theta_hat
            //     //     * (1.0 + 1.0 / ((theta_hat * std).powi(2)) - gamma3 / (2.0 * theta_hat * std))
            //     dbg!(
            //         theta_hat,
            //         ks_tuple,
            //         ks_tuple.1,
            //         ks_tuple.2,
            //         ks_tuple.3,
            //         2.0 * (theta_hat * budget - ks_tuple.0),
            //         w_hat,
            //         u_hat,
            //         normal_dist.cdf(w_hat),
            //         normal_dist.pdf(w_hat),
            //         1.0 / w_hat - 1.0 / u_hat,
            //         min_value,
            //         budget,
            //         simple_mean,
            //         max_value,
            //         sa_out,
            //         approx,
            //         actual_out
            //     );
            //     *truncated_mean_output = simple_mean * normal_dist.cdf(w_hat)
            //         + normal_dist.pdf(w_hat) * (simple_mean / w_hat - budget / u_hat);
            // }
        }

        if DEBUG_SA
            || actual_out < -FLOAT_TOL
            || actual_out > 1.0 + FLOAT_TOL
            || !actual_out.is_finite()
        {
            // dbg!(
            //     f_df(THETA_LIMIT),
            //     f_df(1.0),
            //     f_df(0.0),
            //     f_df(-1.0),
            //     f_df(-THETA_LIMIT),
            // );
            dbg!(theta_hat, theta_error);
            dbg!(w_hat, u_hat, error, sa_out);
            dbg!(
                self.extract_triplet(support_index, skip_count)
                    .into_iter()
                    .map(|x| x.iter().map(|y| y.1).sum::<f64>())
                    .collect::<Vec<f64>>(),
                self.extract_triplet(support_index, skip_count)
                    .into_iter()
                    .map(|x| x.iter().map(|y| y.1).collect())
                    .collect::<Vec<Vec<f64>>>(),
                self.extract_triplet(support_index, skip_count)
                    .into_iter()
                    .collect::<Vec<&Vec<(f64, f64)>>>(),
                self.extract_triplet(support_index, skip_count)
                    .try_len()
                    .unwrap(),
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
                self.simple_average(support_index, skip_count),
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

//         for upgrade in upgrade_arr.iter_mut() {
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
//             &upgrade_arr,
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
