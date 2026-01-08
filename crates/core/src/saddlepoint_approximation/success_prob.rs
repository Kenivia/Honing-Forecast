use super::saddlepoint_approximation::saddlepoint_approximation_prob_wrapper;

use crate::helpers::{find_non_zero_min_vec, sort_by_indices};
use crate::normal_honing_utils::{generate_combined, init_dist};

use crate::performance::Performance;
use crate::saddlepoint_approximation::special::special_probs;
use crate::state::StateBundle;

pub fn honing_sa_wrapper(
    state_bundle: &mut StateBundle,
    mut support_arr: &mut [Vec<f64>],
    budget: f64,
    performance: &mut Performance,
) -> f64 {
    let mut out: f64 = 0.0;

    let (mut log_prob_dist_arr, mut prob_dist_arr) = state_bundle.prep_output.gather_dists();
    sort_by_indices(&mut log_prob_dist_arr, state_bundle.special_state.clone());
    sort_by_indices(&mut prob_dist_arr, state_bundle.special_state.clone());
    sort_by_indices(&mut support_arr, state_bundle.special_state.clone());
    for (index, prob) in special_probs(state_bundle).iter().enumerate() {
        // Skip scenarios with negligible probability, but don't break early
        // because probabilities can be non-monotonic (high special budget = high k has most probability)
        if *prob < 1e-12 {
            continue;
        }
        // dbg!(&support_arr[index..]);
        let this_prob: f64 = saddlepoint_approximation_prob_wrapper(
            &log_prob_dist_arr[index..],
            &prob_dist_arr[index..],
            &support_arr[index..],
            find_non_zero_min_vec(&support_arr[index..], &log_prob_dist_arr[index..]),
            support_arr[index..].iter().map(|x| x.last().unwrap()).sum(),
            budget,
            &mut 0.0,
            performance,
        );

        out += *prob * this_prob;
    }

    out
}
pub fn success_prob_metric(state_bundle: &mut StateBundle, performance: &mut Performance) -> f64 {
    performance.states_evaluated += 1;

    init_dist(state_bundle);
    let mut combined_costs: Vec<Vec<f64>> = generate_combined(state_bundle);
    let budget = state_bundle.prep_output.eqv_gold_budget;
    let out: f64 = honing_sa_wrapper(state_bundle, &mut combined_costs, budget, performance);

    out
}
