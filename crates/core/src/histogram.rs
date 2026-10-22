use crate::constants::juice_info::JuiceInfo;
use crate::constants::*;
use crate::gold_dist::{GoldDist, gold_distribution};
use crate::js_interface::remove_adv_cache;
use crate::materials::NUM_BASE_MATS;
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use crate::tuning::Tuning;
use ahash::AHashMap;
use serde::Serialize;
use smallvec::smallvec;

/// Everything keyed by material label is a map on the JS side.
#[derive(Debug, Serialize)]
pub struct HistogramOutputs {
    cum_percentiles: AHashMap<String, Vec<(f64, f64)>>,

    /// chance the first k ownership bands cover every upgrade, as [bound, +roster, +tradable]
    chance_within: AHashMap<String, [f64; 3]>,
    /// average amount of the material used, regardless of plan
    avg_used: AHashMap<String, f64>,
    /// base materials used at pity with nothing juiced, the most any plan can use
    no_juice_pity: AHashMap<String, f64>,

    /// one entry per plan in the payload
    gold_per_plan: Vec<AHashMap<String, f64>>,
    metric_per_plan: Vec<f64>,
    /// of the payload's dist_plan, when it has one
    gold_dist: Option<GoldDist>,

    juice_info: JuiceInfo,
    state_bundle: StateBundle,
}

pub fn histogram(state_bundle: &mut StateBundle, dist_plan: Option<usize>) -> HistogramOutputs {
    state_bundle.apply_chest_state(true);
    state_bundle.update_prob_dist();
    state_bundle.update_cost_dist();
    state_bundle.compute_special_probs(false);
    state_bundle.set_latest_special_probs(); // needed by luckiest_mf in addition to the usual 3 above

    let mut dummy_probe = Probe::default();
    let num_sup = state_bundle.prep_output.table.len();

    let mut cum_percentiles: Vec<Vec<(f64, f64)>> = vec![Vec::with_capacity(BUCKET_COUNT); num_sup];
    let mut chance_within: Vec<[f64; 3]> = vec![[0.0; 3]; num_sup];

    for (support_index, item) in cum_percentiles.iter_mut().enumerate() {
        let this_pity = state_bundle.pity()[support_index] as f64;
        let this_one_tap = state_bundle.luckiest_mf()[support_index] as f64;

        for index in 0..(BUCKET_COUNT + 1) {
            let this_budget =
                this_one_tap + index as f64 * (this_pity - this_one_tap) / (BUCKET_COUNT) as f64;
            item.push((
                this_budget,
                state_bundle.one_dimension_prob(
                    support_index as i64,
                    this_budget,
                    &mut dummy_probe,
                ),
            ));
        }

        let mat = state_bundle.prep_output.materials[support_index];
        let mut cumulative: f64 = 0.0;
        for (band, owned) in [mat.bound, mat.roster, mat.tradable].iter().enumerate() {
            cumulative += owned;
            chance_within[support_index][band] =
                state_bundle.one_dimension_prob(support_index as i64, cumulative, &mut dummy_probe);
        }
    }

    // the metric shown is the reference's, not what the optimizer worked with
    let tuning = std::mem::replace(&mut state_bundle.tuning, Tuning::reference());
    let (metric_per_plan, avg_used, gold_breakdown) =
        state_bundle.ui_average_gold_metric(&mut dummy_probe);
    state_bundle.tuning = tuning;

    let gold_dist = dist_plan
        .filter(|_| !state_bundle.special_state.is_empty())
        .map(|plan| gold_distribution(state_bundle, plan));

    let no_juice_pity = no_juice_pity(state_bundle);
    let table = &state_bundle.prep_output.table;
    HistogramOutputs {
        cum_percentiles: table.keyed(&cum_percentiles),
        chance_within: table.keyed(&chance_within),
        avg_used: table.keyed(&avg_used),
        no_juice_pity: table.keyed(&no_juice_pity),
        gold_per_plan: gold_breakdown.iter().map(|x| table.keyed(x)).collect(),
        metric_per_plan,
        gold_dist,
        juice_info: state_bundle.prep_output.juice_info.clone(),
        state_bundle: remove_adv_cache(state_bundle),
    }
}

fn no_juice_pity(sb: &mut StateBundle) -> Vec<f64> {
    let mut out = vec![0.0; sb.prep_output.table.len()];
    for &u_index in &sb.special_state {
        let mut upgrade = sb.upgrade_arr[u_index].clone();
        let empty = if upgrade.is_normal_honing {
            vec![smallvec![]; upgrade.state.payload.len()]
        } else {
            vec![smallvec![0]; 2]
        };
        upgrade.state.update_payload(empty);
        upgrade.update_this_prob_dist(&mut sb.adv_cache, &sb.prep_output.juice_info);
        upgrade.update_this_support(&sb.prep_output.juice_info);
        for (row, used) in out.iter_mut().take(NUM_BASE_MATS).enumerate() {
            *used += upgrade.cost_dist[row].access_max(false);
        }
    }
    out
}
