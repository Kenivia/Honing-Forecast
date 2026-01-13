// use std::f64::NAN;

// use crate::constants::FLOAT_TOL;
use crate::constants::SPECIAL_TOL;
use crate::helpers::F64_2d;

use itertools::izip;

use crate::performance::Performance;

use crate::saddlepoint_approximation::saddlepoint_approximation::saddlepoint_approximation_prob_wrapper;
use crate::saddlepoint_approximation::special::special_probs;
use crate::state::StateBundle;

pub static DEBUG_AVERAGE: bool = true;
// use statrs::distribution::{Continuous, Normal};

pub fn average_gold_metric(state_bundle: &mut StateBundle, performance: &mut Performance) -> f64 {
    state_bundle.update_dist();
    state_bundle.update_individual_support();
    performance.states_evaluated += 1;

    let mut total_gold: f64 = 0.0;

    let mut dbg_sa_avg = vec![0.0; 15];
    for (skip_count, special_prob) in special_probs(state_bundle).iter().enumerate() {
        if *special_prob < SPECIAL_TOL {
            continue;
        }
        // dbg!(special_prob);

        for (support_index, (effective_budget, price, leftover)) in izip!(
            state_bundle.flattened_effective_budgets(),
            state_bundle.flattened_price(),
            state_bundle.flattened_leftover()
        )
        .enumerate()
        {
            let this_avg: f64 = saddlepoint_approximation_average_wrapper(
                state_bundle,
                support_index as i64,
                skip_count,
                effective_budget,
                &mut 0.0,
                performance,
                price,
                leftover,
            );
            // dbg!(this_avg);

            if DEBUG_AVERAGE {
                dbg!(
                    skip_count,
                    support_index,
                    effective_budget,
                    price,
                    leftover,
                    this_avg,
                    *special_prob,
                    simple_average(
                        state_bundle.extract_prob(skip_count),
                        state_bundle.extract_support(support_index as i64, skip_count)
                    ),
                    *special_prob * this_avg,
                    "================================"
                );
            }
            total_gold += *special_prob * this_avg;
            dbg_sa_avg[support_index] += *special_prob * this_avg;
        }
    }

    if DEBUG_AVERAGE {
        dbg!(dbg_sa_avg);
    }

    total_gold
}

pub fn simple_average<'a, I>(prob_dist_arr: I, support_arr: I) -> f64
where
    I: F64_2d<'a>,
{
    let mut mean: f64 = 0.0;
    for (support, prob_dist) in support_arr.into_iter().zip(prob_dist_arr) {
        for (s, p) in support.iter().zip(prob_dist) {
            mean += s * p; // technically if theta is 0.0 we can use the K' or k1 from there but like nah cbb
        }
    }
    mean
}
// fn simple_variance<'a, I>(prob_dist_arr: I, support_arr: I) -> f64
// where
//     I: F64_2d<'a>,
// {
//     let mut total_var = 0.0;
//     for (support, prob_dist) in support_arr.into_iter().zip(prob_dist_arr) {
//         let mut mean = 0.0;
//         let mut ex2 = 0.0;
//         for (s, p) in support.iter().zip(prob_dist) {
//             mean += s * p;
//             ex2 += s * s * p;
//         }
//         total_var += ex2 - mean * mean;
//     }

//     total_var
// }

pub fn saddlepoint_approximation_average_wrapper(
    state_bundle: &StateBundle,
    support_index: i64,
    skip_count: usize,
    effective_budget: f64,
    init_theta: &mut f64,
    performance: &mut Performance,
    price: f64,
    leftover_value: f64,
) -> f64 {
    performance.sa_count += 1;
    let simple_mean: f64 = simple_average(
        state_bundle.extract_prob(skip_count),
        state_bundle.extract_support(support_index, skip_count),
    );

    if price == leftover_value {
        return price * (effective_budget - simple_mean);
    }

    // let mut truncated_mean: f64 = NAN; // default if it's trivial
    let biased_prob: f64 = saddlepoint_approximation_prob_wrapper(
        state_bundle,
        support_index,
        skip_count,
        effective_budget,
        init_theta,
        performance,
        true,
    );
    let prob = saddlepoint_approximation_prob_wrapper(
        state_bundle,
        support_index,
        skip_count,
        effective_budget,
        init_theta,
        performance,
        false,
    );

    let out: f64 = price * (effective_budget - simple_mean)
        + (leftover_value - price) * (effective_budget * prob - biased_prob * simple_mean);

    let left = effective_budget - simple_mean;
    let right = effective_budget * prob - biased_prob * (simple_mean);
    if !out.is_finite() || DEBUG_AVERAGE {
        dbg!(
            simple_mean,
            effective_budget,
            biased_prob,
            left,
            right,
            price,
            leftover_value,
            out,
        );
    }

    // }
    return out;
}
