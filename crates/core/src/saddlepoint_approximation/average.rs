use crate::constants::FLOAT_TOL;
use crate::helpers::{find_non_zero_min_vec, sort_by_indices};
use crate::normal_honing_utils::{generate_individual, init_dist};
use crate::performance::Performance;
use crate::saddlepoint_approximation::saddlepoint_approximation::saddlepoint_approximation;
use crate::saddlepoint_approximation::special::special_probs;
use crate::state::StateBundle;
use num::Float;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};
/// Computes average gold using leftover pricing (leftover_value for excess, price for shortage)
pub fn average_gold_metric(state_bundle: &mut StateBundle, performance: &mut Performance) -> f64 {
    init_dist(state_bundle);
    performance.states_evaluated += 1;

    let prep_output = &state_bundle.prep_output;
    let mut total_gold: f64 = 0.0;
    //
    let mut effective_budgets: Vec<f64> = state_bundle
        .prep_output
        .budgets
        .iter()
        .map(|&b| b as f64)
        .collect();
    effective_budgets[3] -= prep_output.unlock_costs[0] as f64;
    effective_budgets[6] -= prep_output.unlock_costs[1] as f64;

    let (mut mats_costs, mut weap_juices_costs, mut armor_juices_costs) =
        generate_individual(state_bundle);

    let (mut log_prob_dist_arr, mut prob_dist_arr) = state_bundle.prep_output.gather_dists();
    sort_by_indices(&mut log_prob_dist_arr, state_bundle.special_state.clone());
    sort_by_indices(&mut prob_dist_arr, state_bundle.special_state.clone());
    for row in mats_costs.iter_mut() {
        sort_by_indices(row, state_bundle.special_state.clone());
    }
    for row in weap_juices_costs.iter_mut() {
        sort_by_indices(row, state_bundle.special_state.clone());
    }
    for row in armor_juices_costs.iter_mut() {
        sort_by_indices(row, state_bundle.special_state.clone());
    }
    for (attempt_index, special_prob) in special_probs(state_bundle).iter().enumerate() {
        if *special_prob < 1e-12 {
            continue;
        }
        // dbg!(special_prob);

        // let upgrade = &prep_output.upgrade_arr[state_bundle.special_state[attempt_index]];
        for (index, support_arr) in mats_costs.iter().enumerate() {
            let this_avg: f64 = saddlepoint_approximation_average_wrapper(
                &log_prob_dist_arr[attempt_index..],
                &prob_dist_arr[attempt_index..],
                &support_arr[attempt_index..],
                find_non_zero_min_vec(
                    &support_arr[attempt_index..],
                    &log_prob_dist_arr[attempt_index..],
                ),
                support_arr[attempt_index..]
                    .iter()
                    .map(|x| x.last().unwrap())
                    .sum(),
                effective_budgets[index] as f64,
                &mut 0.0,
                performance,
                prep_output.price_arr[index],
                prep_output.leftover_values[index],
            );
            // dbg!(this_avg);
            total_gold += *special_prob * this_avg;
        }
        // dbg!(total_gold);
        // dbg!(special_prob, total_gold);

        for (id, support_arr) in weap_juices_costs.iter_mut().enumerate() {
            let this_avg: f64 = saddlepoint_approximation_average_wrapper(
                &log_prob_dist_arr[attempt_index..],
                &prob_dist_arr[attempt_index..],
                &support_arr[attempt_index..],
                find_non_zero_min_vec(
                    &support_arr[attempt_index..],
                    &log_prob_dist_arr[attempt_index..],
                ),
                support_arr[attempt_index..]
                    .iter()
                    .map(|x| x.last().unwrap())
                    .sum(),
                state_bundle.prep_output.juice_books_owned[id].0 as f64,
                &mut 0.0,
                performance,
                prep_output.juice_info.one_gold_cost_id[id].0,
                prep_output.juice_info.one_leftover_value_id[id].0,
            );

            total_gold += *special_prob * this_avg;
        }
        // dbg!(special_prob, total_gold);

        for (id, support_arr) in armor_juices_costs.iter_mut().enumerate() {
            let this_avg: f64 = saddlepoint_approximation_average_wrapper(
                &log_prob_dist_arr[attempt_index..],
                &prob_dist_arr[attempt_index..],
                &support_arr[attempt_index..],
                find_non_zero_min_vec(
                    &support_arr[attempt_index..],
                    &log_prob_dist_arr[attempt_index..],
                ),
                support_arr[attempt_index..]
                    .iter()
                    .map(|x| x.last().unwrap())
                    .sum(),
                state_bundle.prep_output.juice_books_owned[id].0 as f64,
                &mut 0.0,
                performance,
                prep_output.juice_info.one_gold_cost_id[id].1,
                prep_output.juice_info.one_leftover_value_id[id].1,
            );

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

pub fn average_gold_naive_wrapper(
    state_bundle: &mut StateBundle,
    performance: &mut Performance,
) -> f64 {
    init_dist(state_bundle);
    performance.states_evaluated += 1;

    let prep_output = &state_bundle.prep_output;
    let mut total_gold: f64 = 0.0;
    //
    let mut effective_budgets: Vec<f64> = state_bundle
        .prep_output
        .budgets
        .iter()
        .map(|&b| b as f64)
        .collect();
    effective_budgets[3] -= prep_output.unlock_costs[0] as f64;
    effective_budgets[6] -= prep_output.unlock_costs[1] as f64;

    let (mut mats_costs, mut weap_juices_costs, mut armor_juices_costs) =
        generate_individual(state_bundle);

    let (mut log_prob_dist_arr, mut prob_dist_arr) = state_bundle.prep_output.gather_dists();
    sort_by_indices(&mut log_prob_dist_arr, state_bundle.special_state.clone());
    sort_by_indices(&mut prob_dist_arr, state_bundle.special_state.clone());
    for row in mats_costs.iter_mut() {
        sort_by_indices(row, state_bundle.special_state.clone());
    }
    for row in weap_juices_costs.iter_mut() {
        sort_by_indices(row, state_bundle.special_state.clone());
    }
    for row in armor_juices_costs.iter_mut() {
        sort_by_indices(row, state_bundle.special_state.clone());
    }

    for (attempt_index, special_prob) in special_probs(state_bundle).iter().enumerate() {
        // Skip scenarios with negligible probability, but don't break early
        // because probabilities can be non-monotonic (high special budget = high k has most probability)
        if *special_prob < 1e-12 {
            continue;
        }
        // dbg!(&support_arr[index..]);

        for (index, support_arr) in mats_costs.iter().enumerate() {
            let this_avg: f64 = (effective_budgets[index] as f64
                - simple_average(
                    &prob_dist_arr[attempt_index..],
                    &support_arr[attempt_index..],
                ))
                * prep_output.price_arr[index];

            total_gold += *special_prob * this_avg;
        }

        for (id, support_arr) in weap_juices_costs.iter_mut().enumerate() {
            let this_avg: f64 = (prep_output.juice_books_owned[id].0 as f64
                - simple_average(
                    &prob_dist_arr[attempt_index..],
                    &support_arr[attempt_index..],
                ))
                * prep_output.juice_info.one_gold_cost_id[id].0;

            total_gold += *special_prob * this_avg;
        }

        for (id, support_arr) in armor_juices_costs.iter_mut().enumerate() {
            let this_avg: f64 = (prep_output.juice_books_owned[id].1 as f64
                - simple_average(
                    &prob_dist_arr[attempt_index..],
                    &support_arr[attempt_index..],
                ))
                * prep_output.juice_info.one_gold_cost_id[id].1;

            total_gold += *special_prob * this_avg;
        }
    }

    total_gold
}

fn simple_average(prob_dist_arr: &[Vec<f64>], support_arr: &[Vec<f64>]) -> f64 {
    let mut mean: f64 = 0.0;
    for (u_index, support) in support_arr.iter().enumerate() {
        for (index, s) in support.iter().enumerate() {
            mean += s * prob_dist_arr[u_index][index]; // technically if theta is 0.0 we can use the K' or k1 from there but like nah cbb
        }
    }
    mean
}

fn simple_variance(prob_dist_arr: &[Vec<f64>], support_arr: &[Vec<f64>]) -> f64 {
    let mut total_var: f64 = 0.0;
    for (u_index, support) in support_arr.iter().enumerate() {
        let mut mean_u: f64 = 0.0;
        let mut e_x2: f64 = 0.0;
        for (index, s) in support.iter().enumerate() {
            let p = prob_dist_arr[u_index][index];
            mean_u += s * p;
            e_x2 += s * s * p;
        }
        let var_u = e_x2 - mean_u * mean_u;
        total_var += var_u;
    }
    total_var
}

pub fn saddlepoint_approximation_average_wrapper(
    log_prob_dist_arr: &[Vec<f64>],
    prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    min_value: f64,
    max_value: f64,
    budget: f64,
    init_theta: &mut f64,
    performance: &mut Performance,
    price: f64,
    leftover_value: f64,
) -> f64 {
    performance.sa_count += 1;

    let mean: f64 = simple_average(prob_dist_arr, support_arr);
    if price == leftover_value {
        return price * (budget - mean);
    }
    let std: f64 = simple_variance(prob_dist_arr, support_arr).sqrt();
    let mut derivative: f64 = 0.0; // default if it's trivial 
    let prob: f64 = saddlepoint_approximation(
        log_prob_dist_arr,
        support_arr,
        min_value,
        max_value,
        budget,
        init_theta,
        performance,
        false,
        &mut derivative,
        mean,
    );
    let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap(); // TODO can i pre initialize this or is there no point
    let naive_normal_correction =
        if (prob - 1.0).abs() > FLOAT_TOL && prob.abs() > FLOAT_TOL && std.abs() > FLOAT_TOL {
            -std * normal_dist.pdf((budget - mean) / std)
        } else {
            0.0
        };
    let out: f64 = price * (budget - mean)
        + (leftover_value - price) * ((budget - mean) * prob - naive_normal_correction);
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
