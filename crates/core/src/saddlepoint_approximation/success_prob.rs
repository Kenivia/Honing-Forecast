use super::saddlepoint_approximation::saddlepoint_approximation_prob_wrapper;

use crate::constants::SPECIAL_TOL;

use crate::performance::Performance;
use crate::saddlepoint_approximation::special::special_probs;
use crate::state::StateBundle;

pub fn honing_sa_wrapper(
    state_bundle: &StateBundle,
    support_index: i64,
    budget: f64,
    performance: &mut Performance,
) -> f64 {
    let mut out: f64 = 0.0;

    for (skip_count, special_prob) in special_probs(state_bundle).iter().enumerate() {
        if *special_prob < SPECIAL_TOL {
            continue;
        }
        // dbg!(&support_arr[index..]);
        let this_prob: f64 = saddlepoint_approximation_prob_wrapper(
            state_bundle,
            support_index,
            skip_count,
            budget,
            &mut 0.0,
            performance,
            false,
        );

        out += *special_prob * this_prob;
    }

    out
}
pub fn success_prob_metric(state_bundle: &mut StateBundle, performance: &mut Performance) -> f64 {
    performance.states_evaluated += 1;

    state_bundle.update_dist();
    state_bundle.update_combined();
    let budget = state_bundle.prep_output.eqv_gold_budget;
    let out: f64 = honing_sa_wrapper(state_bundle, -1, budget, performance);

    out
}
