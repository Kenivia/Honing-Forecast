use crate::constants::juice_info::JuiceInfo;
use crate::constants::*;
use crate::honing_utils::materials::{GOLD_LABEL, NUM_BASE_MATS};
use crate::honing_utils::state_bundle::StateBundle;
use crate::probe::Probe;
use crate::tuning::Tuning;
use crate::ui::gold_dist::{GoldDist, gold_distribution};
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
    /// tradable gold owned after chests, which every metric and the gold distribution are net of
    tradable_gold: f64,

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
    let pity = state_bundle.pity();
    let luckiest = state_bundle.luckiest_mf();

    for (support_index, item) in cum_percentiles.iter_mut().enumerate() {
        let this_pity = pity[support_index] as f64;
        let this_one_tap = luckiest[support_index] as f64;

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
    // gold owed (negative) is spent, not owned
    let tradable_gold = table.get_row(GOLD_LABEL).map_or(0.0, |row| {
        state_bundle.prep_output.materials[row].tradable.max(0.0)
    });
    HistogramOutputs {
        cum_percentiles: table.keyed(&cum_percentiles),
        chance_within: table.keyed(&chance_within),
        avg_used: table.keyed(&avg_used),
        no_juice_pity: table.keyed(&no_juice_pity),
        gold_per_plan: gold_breakdown.iter().map(|x| table.keyed(x)).collect(),
        metric_per_plan,
        gold_dist,
        tradable_gold,
        juice_info: state_bundle.prep_output.juice_info.clone(),
        state_bundle: state_bundle.without_adv_cache(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::honing_utils::materials::gold;
    use crate::payload::Payload;

    /// with nothing to hone, every plan is worth exactly what is owned
    #[test]
    fn no_upgrades_is_what_is_owned() {
        let mut payload = Payload::load_step(
            "../../arena_data/scenarios/real3/r3_04b85565_09120f_f0_s1.json",
            0,
        );
        payload.upgrade_info = vec![];
        payload.special_state = None;
        let red = payload.material_labels[0].clone();
        payload.materials.get_mut("Gold").unwrap().tradable = 1000.0;
        payload.materials.get_mut(&red).unwrap().tradable = 100.0;
        let mut state_bundle = StateBundle::from_payload(payload);
        let out = histogram(&mut state_bundle, Some(0));
        let mut any = false;
        for (plan, metric) in out.metric_per_plan.iter().enumerate() {
            let owned: f64 = state_bundle.prep_output.bands[plan]
                .iter()
                .map(|bands| gold(0.0, bands))
                .sum();
            any |= owned != 0.0;
            assert!(
                (metric - owned).abs() <= 1e-6 * owned.abs(),
                "{metric} {owned}"
            );
        }
        assert!(any);
    }
}
