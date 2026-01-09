use crate::constants::FLOAT_TOL;
use crate::constants::SPECIAL_TOL;
use crate::helpers::F64_2d;

use itertools::izip;

use crate::performance::Performance;
use crate::saddlepoint_approximation::saddlepoint_approximation::saddlepoint_approximation;
use crate::saddlepoint_approximation::special::special_probs;
use crate::state::StateBundle;

use statrs::distribution::{Continuous, Normal};
/// Computes average gold using leftover pricing (leftover_value for excess, price for shortage)
pub fn average_gold_metric(state_bundle: &mut StateBundle, performance: &mut Performance) -> f64 {
    state_bundle.update_dist();
    state_bundle.update_individual_support();
    performance.states_evaluated += 1;

    let mut total_gold: f64 = 0.0;

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
            total_gold += *special_prob * this_avg;
        }

        // dbg!(special_prob, total_gold);
    }

    // dbg!(
    //     total_gold,
    //     &average_gold_naive_wrapper(state_bundle, performance)
    // );
    total_gold
}

// pub fn average_gold_naive_wrapper(
//     state_bundle: &mut StateBundle,
//     performance: &mut Performance,
// ) -> f64 {
//     init_dist(state_bundle);
//     performance.states_evaluated += 1;

//     let prep_output = &state_bundle.prep_output;
//     let mut total_gold: f64 = 0.0;
//     //
//     let mut effective_budgets: Vec<f64> = state_bundle
//         .prep_output
//         .budgets
//         .iter()
//         .map(|&b| b as f64)
//         .collect();
//     effective_budgets[3] -= prep_output.unlock_costs[0] as f64;
//     effective_budgets[6] -= prep_output.unlock_costs[1] as f64;

//     let (mut mats_costs, mut weap_juices_costs, mut armor_juices_costs) =
//         generate_individual(state_bundle);

//     let (mut log_prob_dist_arr, mut prob_dist_arr) = state_bundle.prep_output.gather_dists();
//     sort_by_indices(&mut log_prob_dist_arr, state_bundle.special_state.clone());
//     sort_by_indices(&mut prob_dist_arr, state_bundle.special_state.clone());
//     for row in mats_costs.iter_mut() {
//         sort_by_indices(row, state_bundle.special_state.clone());
//     }
//     for row in weap_juices_costs.iter_mut() {
//         sort_by_indices(row, state_bundle.special_state.clone());
//     }
//     for row in armor_juices_costs.iter_mut() {
//         sort_by_indices(row, state_bundle.special_state.clone());
//     }

//     for (skip_count, special_prob) in special_probs(state_bundle).iter().enumerate() {
//         // Skip scenarios with negligible probability, but don't break early
//         // because probabilities can be non-monotonic (high special budget = high k has most probability)
//         if *special_prob < SPECIAL_TOL {
//             continue;
//         }
//         // dbg!(&support_arr[index..]);

//         for (index, support_arr) in mats_costs.iter().enumerate() {
//             let this_avg: f64 = (effective_budgets[index] as f64
//                 - simple_average(&prob_dist_arr[skip_count..], &support_arr[skip_count..]))
//                 * prep_output.price_arr[index];

//             total_gold += *special_prob * this_avg;
//         }

//         for (id, support_arr) in weap_juices_costs.iter_mut().enumerate() {
//             let this_avg: f64 = (prep_output.juice_books_owned[id].0 as f64
//                 - simple_average(&prob_dist_arr[skip_count..], &support_arr[skip_count..]))
//                 * prep_output.juice_info.one_gold_cost_id[id].0;

//             total_gold += *special_prob * this_avg;
//         }

//         for (id, support_arr) in armor_juices_costs.iter_mut().enumerate() {
//             let this_avg: f64 = (prep_output.juice_books_owned[id].1 as f64
//                 - simple_average(&prob_dist_arr[skip_count..], &support_arr[skip_count..]))
//                 * prep_output.juice_info.one_gold_cost_id[id].1;

//             total_gold += *special_prob * this_avg;
//         }
//     }

//     total_gold
// }

fn simple_average<'a, I>(prob_dist_arr: I, support_arr: I) -> f64
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
fn simple_variance<'a, I>(prob_dist_arr: I, support_arr: I) -> f64
where
    I: F64_2d<'a>,
{
    let mut total_var = 0.0;
    for (support, prob_dist) in support_arr.into_iter().zip(prob_dist_arr) {
        let mut mean = 0.0;
        let mut ex2 = 0.0;
        for (s, p) in support.iter().zip(prob_dist) {
            mean += s * p;
            ex2 += s * s * p;
        }
        total_var += ex2 - mean * mean;
    }

    total_var
}

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
    let (min_value, max_value) = state_bundle.find_min_max(support_index, skip_count);
    let mean: f64 = simple_average(
        state_bundle.extract_prob(skip_count),
        state_bundle.extract_support(support_index, skip_count),
    );
    if price == leftover_value {
        return price * (effective_budget - mean);
    }
    let std: f64 = simple_variance(
        state_bundle.extract_prob(skip_count),
        state_bundle.extract_support(support_index, skip_count),
    )
    .sqrt();
    let mut derivative: f64 = 0.0; // default if it's trivial 
    let prob: f64 = saddlepoint_approximation(
        state_bundle,
        support_index,
        skip_count,
        min_value,
        max_value,
        effective_budget,
        init_theta,
        performance,
        false,
        &mut derivative,
        mean,
    );
    let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap(); // TODO can i pre initialize this or is there no point
    let naive_normal_correction =
        if (prob - 1.0).abs() > FLOAT_TOL && prob.abs() > FLOAT_TOL && std.abs() > FLOAT_TOL {
            -std * normal_dist.pdf((effective_budget - mean) / std)
        } else {
            0.0
        };
    let out: f64 = price * (effective_budget - mean)
        + (leftover_value - price) * ((effective_budget - mean) * prob - naive_normal_correction);
    // let out_naive: f64 = price * (budget - mean)
    //     + (leftover_value - price) * ((budget - mean) * prob - naive_normal_correction);
    // if std.abs() > FLOAT_TOL {
    //     dbg!(
    //         mean,
    //         budget,
    //         prob,
    //         std,
    //         naive_normal_correction,
    //         derivative,
    //         price,
    //         leftover_value,
    //         out,
    //         out_naive,
    //         "================================"
    //     );
    // }
    return out;
}
