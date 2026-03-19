use hf_core::constants::*;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct HistogramOutputs {
    cum_percentiles: Vec<Vec<(f64, f64)>>,
    state_bundle: StateBundle,
    bound_chance: Vec<f64>,
    // tradable_chance: Vec<f64>,
}

pub fn histogram(state_bundle: &mut StateBundle) -> HistogramOutputs {
    state_bundle.update_prob_dist();
    state_bundle.update_cost_dist();
    state_bundle.compute_special_probs(false);
    state_bundle.set_latest_special_probs(); // needed by luckiest_mf in addition to the usual 3 above

    let mut dummy_performance = Performance::new();
    let num_sup = state_bundle.prep_output.bound_budgets.len();

    let mut cum_percentiles: Vec<Vec<(f64, f64)>> = vec![Vec::with_capacity(BUCKET_COUNT); num_sup];

    // let mut average: Vec<f64> = Vec::with_capacity(num_sup);
    let mut bound_chance: Vec<f64> = Vec::with_capacity(num_sup);

    for (support_index, item) in cum_percentiles.iter_mut().enumerate().take(num_sup) {
        let this_pity = state_bundle.pity()[support_index] as f64;
        let this_one_tap = state_bundle.luckiest_mf()[support_index] as f64;

        let bound_budget = state_bundle.prep_output.bound_budgets[support_index];
        for index in 0..(BUCKET_COUNT + 1) {
            let this_budget =
                this_one_tap + index as f64 * (this_pity - this_one_tap) / (BUCKET_COUNT) as f64;
            item.push((
                this_budget,
                state_bundle.one_dimension_prob(
                    support_index as i64,
                    this_budget,
                    &mut dummy_performance,
                ),
            ));
        }

        bound_chance.push(state_bundle.one_dimension_prob(
            support_index as i64,
            bound_budget,
            &mut dummy_performance,
        ));
    }

    // state_bundle.average_gold_metric(true, &mut Performance::new());
    HistogramOutputs {
        cum_percentiles,
        state_bundle: state_bundle.clone(),
        bound_chance,
    }
}
