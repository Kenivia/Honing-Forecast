use crate::parser::PreparationOutput;
use crate::saddlepoint_approximation::normal_sa::{generate_combined, init_dist};
use crate::state::StateBundle;

fn brute_naive_recursive(
    prob_dist_arr: &[Vec<f64>],
    support_arr: &[Vec<f64>],
    cost_so_far: f64,
    budget: f64,
    depth: usize,
) -> f64 {
    let prob_dist: &Vec<f64> = &prob_dist_arr[depth];

    let cost_arr: &Vec<f64> = &support_arr[depth];

    if depth == prob_dist_arr.len() - 1 {
        return cost_arr
            .iter()
            .enumerate()
            .take_while(|(_, cost)| cost_so_far + **cost <= budget)
            .fold(0.0, |acc, (index, _)| acc + prob_dist[index]);
    } else {
        return cost_arr.iter().enumerate().fold(0.0, |acc, (index, cost)| {
            let new_cost: f64 = cost_so_far + *cost;
            acc + if budget < new_cost {
                0.0
            } else {
                prob_dist[index]
                    * brute_naive_recursive(
                        prob_dist_arr,
                        support_arr,
                        new_cost,
                        budget,
                        depth + 1,
                        // cache,
                    )
            }
        });
    }
}

// this is actually just another wrapper
pub fn brute_naive(prob_dist_arr: &[Vec<f64>], support_arr: &[Vec<f64>], budget: f64) -> f64 {
    brute_naive_recursive(prob_dist_arr, support_arr, 0.0, budget, 0)
}

// naive as in it doesn't take into account leftover prices
pub fn brute_naive_wrapper(
    state_bundle: &mut StateBundle,
    prep_output: &mut PreparationOutput,

    states_evaled: &mut i64,
) -> f64 {
    *states_evaled += 1;
    init_dist(state_bundle, prep_output);
    let (_, prob_dist_arr) = prep_output.gather_dists();

    let combined_costs: Vec<Vec<f64>> = generate_combined(prep_output, state_bundle);
    brute_naive(&prob_dist_arr, &combined_costs, prep_output.eqv_gold_budget)
}
