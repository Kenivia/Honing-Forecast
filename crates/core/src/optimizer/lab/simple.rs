//! Maxroll's rule, no search: each normal upgrade takes its best plan with every material at its
//! buy price (a juice or book goes on a tap where it saves more than it costs), then the greedy
//! free tap order. Advanced upgrades and chests stay as given.

use super::super::response::Response;
use super::super::simulated_annealing::greedy_order;
use super::super::{Previous, Solved};
use crate::honing_utils::state_bundle::StateBundle;
use crate::probe::Probe;
use rand::Rng;

pub const OFF: &[&str] = &[];
pub const ON: &[&str] = &[];

pub fn solve<R: Rng>(
    _rng: &mut R,
    mut bundle: StateBundle,
    _previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    let prep = &bundle.prep_output;
    let prices: Vec<f64> = prep.bands[prep.optimizer_plan]
        .iter()
        .map(|bands| bands.last().unwrap().1)
        .collect();
    for index in 0..bundle.upgrade_arr.len() {
        if !bundle.upgrade_arr[index].is_normal_honing {
            continue;
        }
        let response = Response::linear(&bundle, index, &prices);
        let now = response
            .model
            .indices(&bundle.upgrade_arr[index].state.payload);
        let (_, found) = response.better(&now, usize::MAX);
        let plan = response.model.plan(&found);
        bundle.upgrade_arr[index].state.update_payload(plan);
    }
    if !bundle.upgrade_arr.is_empty() {
        greedy_order(&mut bundle, probe);
    }
    bundle.metric = bundle.evaluate(probe);
    bundle.set_latest_special_probs();
    Solved {
        best: bundle,
        blob: Vec::new(),
    }
}
