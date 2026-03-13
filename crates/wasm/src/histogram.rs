use hf_core::constants::*;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct HistogramOutputs {
    cum_percentiles: Vec<Vec<(f64, f64)>>,
    average: Vec<f64>,
}

pub fn histogram(state_bundle: &mut StateBundle) -> HistogramOutputs {
    state_bundle.update_dist();
    state_bundle.update_individual_support();
    state_bundle.compute_special_probs(false);
    let special_probs = state_bundle.special_cache[&state_bundle.special_state].clone();
    let mut dummy_performance = Performance::new();
    let num_sup = state_bundle.prep_output.bound_budgets.len();

    let mut cum_percentiles: Vec<Vec<(f64, f64)>> = vec![Vec::with_capacity(BUCKET_COUNT); num_sup];

    let mut average: Vec<f64> = Vec::with_capacity(num_sup);

    for (support_index, item) in cum_percentiles.iter_mut().enumerate().take(num_sup) {
        let this_pity = state_bundle.pity()[support_index] as f64;
        let this_one_tap = state_bundle.one_tap()[support_index] as f64;

        let bound_budget = state_bundle.prep_output.bound_budgets[support_index];
        let trade_budget = state_bundle.prep_output.trade_budgets[support_index];
        let mut bound_done: bool = false;
        let mut trade_done: bool = (bound_budget - trade_budget).abs() < FLOAT_TOL; // dont bother if they're the same

        for index in 0..(BUCKET_COUNT + 1) {
            let this_budget =
                this_one_tap + index as f64 * (this_pity - this_one_tap) / (BUCKET_COUNT) as f64;
            if this_budget > bound_budget && !bound_done {
                if (this_budget - bound_budget).abs() > FLOAT_TOL {
                    // dont bother if budget happends to land on the a percentile already
                    item.push((
                        bound_budget,
                        state_bundle.one_dimension_prob(
                            support_index as i64,
                            bound_budget,
                            &mut dummy_performance,
                        ),
                    ));
                }

                bound_done = true;
            }
            if this_budget > bound_budget + trade_budget && !trade_done {
                if (this_budget - bound_budget - trade_budget).abs() > FLOAT_TOL {
                    item.push((
                        bound_budget + trade_budget,
                        state_bundle.one_dimension_prob(
                            support_index as i64,
                            bound_budget + trade_budget,
                            &mut dummy_performance,
                        ),
                    ));
                }

                trade_done = true;
            }
            item.push((
                this_budget,
                state_bundle.one_dimension_prob(
                    support_index as i64,
                    this_budget,
                    &mut dummy_performance,
                ),
            ));
        }
    }
    for support_index in 0..num_sup {
        let mut out: f64 = 0.0;
        for (skip_count, &special_prob) in special_probs.iter().enumerate() {
            if special_prob == 0.0 {
                continue;
            }
            out += special_prob * state_bundle.simple_avg(support_index as i64, skip_count);
        }
        average.push(out.ceil())
    }

    HistogramOutputs {
        cum_percentiles,
        average,
    }
}
