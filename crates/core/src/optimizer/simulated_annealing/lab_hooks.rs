//! Pieces of `solve` with their evaluation counts, for the lab's `cem`. Nothing in `solve` calls these.

use super::{polish, refine, start_solver};
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use rand::Rng;

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
