use hf_core::constants::juice_info::JuiceInfo;
use hf_core::constants::*;
use hf_core::performance::Performance;
use hf_core::state_bundle::StateBundle;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HistogramOutputs {
    cum_percentiles: Vec<Vec<(f64, f64)>>,

    chances_arr: Vec<Vec<f64>>, //  [treatment plan][material type] for all 3 of these
    gold_breakdown_arr: Vec<Vec<f64>>,
    metrics_arr: Vec<f64>,

    avg_breakdown: Vec<f64>,
    juice_info: JuiceInfo,
    state_bundle: StateBundle,
}

pub fn histogram(state_bundle: &mut StateBundle) -> HistogramOutputs {
    state_bundle.update_prob_dist();
    state_bundle.update_cost_dist();
    state_bundle.compute_special_probs(false);
    state_bundle.set_latest_special_probs(); // needed by luckiest_mf in addition to the usual 3 above

    let mut dummy_performance = Performance::new();
    let num_sup = state_bundle.prep_output.raw_material_info.len();

    let mut cum_percentiles: Vec<Vec<(f64, f64)>> = vec![Vec::with_capacity(BUCKET_COUNT); num_sup];

    // let mut average: Vec<f64> = Vec::with_capacity(num_sup);
    let mut chances_arr: Vec<Vec<f64>> =
        vec![Vec::with_capacity(num_sup); state_bundle.prep_output.raw_num_breakpoints];

    for (support_index, item) in cum_percentiles.iter_mut().enumerate().take(num_sup) {
        let this_pity = state_bundle.pity()[support_index] as f64;
        let this_one_tap = state_bundle.luckiest_mf()[support_index] as f64;

        for index in 0..(BUCKET_COUNT + 1) {
            let this_budget = (this_one_tap
                + index as f64 * (this_pity - this_one_tap) / (BUCKET_COUNT) as f64)
                .round();
            item.push((
                this_budget,
                state_bundle.one_dimension_prob(
                    support_index as i64,
                    this_budget,
                    &mut dummy_performance,
                ),
            ));
        }

        let mut cumulative: f64 = 0.0;
        for treatment_plan in 0..state_bundle.prep_output.raw_num_breakpoints {
            cumulative +=
                state_bundle.prep_output.raw_material_info[support_index][treatment_plan].0;
            chances_arr[treatment_plan].push(state_bundle.one_dimension_prob(
                support_index as i64,
                cumulative,
                &mut dummy_performance,
            ));
        }
    }

    let (metrics_arr, avg_breakdown, gold_breakdown_arr) =
        state_bundle.ui_average_gold_metric(Some(&UI_TREATMENTS.to_vec()), &mut dummy_performance);
    // state_bundle.average_gold_metric(true, &mut Performance::new());
    HistogramOutputs {
        cum_percentiles,
        chances_arr,
        avg_breakdown,
        gold_breakdown_arr,
        metrics_arr,
        juice_info: state_bundle.prep_output.juice_info.clone(),
        state_bundle: state_bundle.clone(),
    }
}
