//! What one more char-bound unit of a material saves in gross spend, the plan polished for it.

use crate::materials::NUM_BASE_MATS;
use crate::optimizer::polish_only;
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use crate::tuning::Tuning;
use ahash::AHashMap;
use rand::Rng;

// plan 0, scored as the histogram scores it
fn gross_spend(state_bundle: &mut StateBundle, probe: &mut Probe) -> f64 {
    let tuning = std::mem::replace(&mut state_bundle.tuning, Tuning::reference());
    let (metric_per_plan, _, _) = state_bundle.ui_average_gold_metric(probe);
    state_bundle.tuning = tuning;
    metric_per_plan[0]
}

// the most one tap of any upgrade can take of the row
fn step(state_bundle: &StateBundle, row: usize) -> f64 {
    let juice_info = &state_bundle.prep_output.juice_info;
    state_bundle
        .upgrade_arr
        .iter()
        .map(|upgrade| {
            if row < NUM_BASE_MATS {
                return upgrade.costs[row];
            }
            let id = row - NUM_BASE_MATS;
            let ids = if upgrade.is_normal_honing {
                &juice_info.normal_uindex_to_id
            } else {
                &juice_info.adv_uindex_to_id
            };
            if !ids[upgrade.piece_type_usize][upgrade.upgrade_index].contains(&id) {
                return 0.0;
            }
            let juice = juice_info.access(id, upgrade.piece_type_usize, upgrade.upgrade_index);
            if upgrade.is_normal_honing {
                juice.normal_amt_used as f64
            } else {
                juice.adv_amt_used as f64
            }
        })
        .fold(0.0, f64::max)
}

/// Gold saved per unit, by label, for each label some upgrade takes.
pub fn marginal_values<R: Rng>(
    rng: &mut R,
    state_bundle: &mut StateBundle,
    labels: &[String],
) -> AHashMap<String, f64> {
    let mut probe = Probe::default();
    let mut base = polish_only(rng, state_bundle, &mut probe);
    let base_spend = gross_spend(&mut base, &mut probe);
    let block = base.block() as f64;
    labels
        .iter()
        .filter_map(|label| {
            let row = base.prep_output.table.row(label);
            let delta = step(&base, row) * block;
            if delta <= 0.0 {
                return None;
            }
            let mut bumped = base.clone();
            bumped.prep_output.base_materials[row].bound += delta;
            bumped.apply_chest_state(false);
            let mut bumped = polish_only(rng, &mut bumped, &mut probe);
            let saved = (gross_spend(&mut bumped, &mut probe) - base_spend) / delta;
            Some((label.clone(), saved))
        })
        .collect()
}
