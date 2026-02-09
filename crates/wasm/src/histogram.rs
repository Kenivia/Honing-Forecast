use hf_core::constants::*;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct HistogramOutputs {
    cum_percentiles: Vec<Vec<(f64, f64)>>,
    average: Vec<f64>,
    budgets: Vec<f64>, // just for convenience
}

pub fn histogram(state_bundle: &mut StateBundle) -> HistogramOutputs {
    state_bundle.update_dist();
    state_bundle.update_individual_support();
    state_bundle.compute_special_probs();
    let special_probs = state_bundle.special_cache[&state_bundle.special_state].clone();
    let mut dummy_performance = Performance::new();
    let num_sup = state_bundle.flattened_effective_budgets().count();
    let mut cum_percentiles: Vec<Vec<(f64, f64)>> =
        vec![vec![(0.0, 0.0); BUCKET_COUNT + 1]; num_sup];

    let mut average: Vec<f64> = Vec::with_capacity(num_sup);

    for (support_index, item) in cum_percentiles.iter_mut().enumerate().take(num_sup) {
        for (index, this_prob) in item.iter_mut().enumerate() {
            let this_budget =
                index as f64 * state_bundle.pity()[support_index] as f64 / BUCKET_COUNT as f64;
            *this_prob = (
                this_budget,
                state_bundle.one_dimension_prob(
                    support_index as i64,
                    this_budget,
                    &mut dummy_performance,
                ),
            );
        }
    }
    for support_index in 0..num_sup {
        let mut out: f64 = 0.0;
        for (skip_count, &special_prob) in special_probs.iter().enumerate() {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            out += special_prob * state_bundle.simple_avg(support_index as i64, skip_count);
        }
        average.push(out.ceil())
    }

    HistogramOutputs {
        cum_percentiles,
        average,
        budgets: state_bundle
            .prep_output
            .effective_budgets
            .iter()
            .enumerate()
            .take(7)
            .map(|(_, x)| *x)
            .chain(
                state_bundle
                    .prep_output
                    .juice_books_owned
                    .iter()
                    .map(|x| x.0),
            )
            .chain(
                state_bundle
                    .prep_output
                    .juice_books_owned
                    .iter()
                    .map(|x| x.1),
            )
            .collect(),
    }
}
