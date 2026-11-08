//! Pieces of `solve` with their evaluation counts, for the lab's `cem`, and the lab `lns`'s
//! rebuild as a kick (`kick_types` `rebuild`), which only the kick loop calls.

use super::{polish, refine, respond_one, start_solver};
use crate::optimizer::response::Response;
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use rand::Rng;
use rand::seq::SliceRandom;
use smallvec::smallvec;

/// lns's rebuild: up to 10 normal upgrades of one piece type emptied, or each given its best
/// plan at the shadow prices times lognormal noise (0.35), half and half (`rebuildempty` and
/// `rebuildprice` take one half only); then responses in turn over them, kept where better,
/// until none moves (3 rounds at most).
pub(crate) fn rebuild_kick<R: Rng>(
    rng: &mut R,
    sb: &mut StateBundle,
    normal: &[usize],
    probe: &mut Probe,
    kind: &str,
) {
    let piece = sb.upgrade_arr[normal[rng.random_range(0..normal.len())]].piece_type_usize;
    let mut set: Vec<usize> = normal
        .iter()
        .copied()
        .filter(|&i| sb.upgrade_arr[i].piece_type_usize == piece)
        .collect();
    set.shuffle(rng);
    set.truncate(10);
    sb.metric = sb.metric_router(probe);
    let empty = match kind {
        "rebuildempty" => true,
        "rebuildprice" => false,
        _ => rng.random_bool(0.5),
    };
    if empty {
        for &i in &set {
            let len = sb.upgrade_arr[i].state.payload.len();
            sb.upgrade_arr[i]
                .state
                .update_payload(vec![smallvec![]; len]);
        }
    } else {
        let mut prices = sb.shadow_prices(probe);
        for price in prices.iter_mut() {
            let u: f64 = rng.random::<f64>().max(1e-12);
            let v: f64 = rng.random();
            let z = (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos();
            *price *= (0.35 * z).exp();
        }
        for &i in &set {
            let response = Response::linear(sb, i, &prices);
            let now = response.model.indices(&sb.upgrade_arr[i].state.payload);
            let (_, found) = response.better(&now, sb.tuning.response_joint);
            sb.upgrade_arr[i]
                .state
                .update_payload(response.model.plan(&found));
        }
    }
    sb.metric = sb.metric_router(probe);
    for _ in 0..3 {
        set.shuffle(rng);
        let before = sb.metric;
        for &i in &set {
            respond_one(sb, i, probe);
        }
        if sb.metric <= before {
            break;
        }
    }
}

/// The polishes alone, and what they took in evaluations.
pub(crate) fn polish_counted<R: Rng>(
    rng: &mut R,
    state_bundle: &mut StateBundle,
    probe: &mut Probe,
) -> (StateBundle, i64) {
    state_bundle.tuning.max_iters = 0;
    let mut solver_bundle = start_solver(rng, state_bundle, probe);
    let cost = polish(&mut solver_bundle, probe) + 1;
    solver_bundle.state_bundle.clone_from_essence(
        &solver_bundle.best_n_states.peek_max().unwrap().0,
        solver_bundle.best_n_states.peek_max().unwrap().1,
    );
    (solver_bundle.state_bundle, cost)
}

/// `refine` from the given state, and the evaluations it took by the probe's count, responses
/// left out.
pub(crate) fn refine_counted<R: Rng>(
    rng: &mut R,
    state_bundle: &mut StateBundle,
    probe: &mut Probe,
) -> i64 {
    #[cfg(feature = "trace")]
    let before = probe.data.counts[0];
    state_bundle.metric = state_bundle.metric_router(probe);
    refine(rng, state_bundle, true, probe);
    #[cfg(feature = "trace")]
    return probe.data.counts[0] - before + 1;
    #[cfg(not(feature = "trace"))]
    1
}
