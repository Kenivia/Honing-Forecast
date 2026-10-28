//! Optimizer approaches written from scratch beside `solve`, picked by `Tuning::algo`.
//! See `AI notes/Calculator core/Optimizer lab.md`.

pub mod cem;
pub mod check;
pub mod lns;
pub mod memetic;
pub mod price;
pub mod surrogate;

use super::{Previous, Solved};
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use rand::Rng;

/// every lab algorithm, by the name `Tuning::algo` takes
pub const ALGOS: &[&str] = &["price", "surrogate", "memetic", "cem", "lns"];

pub fn solve<R: Rng>(
    rng: &mut R,
    state_bundle: StateBundle,
    previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    match state_bundle.tuning.algo.as_str() {
        "" => super::solve(rng, state_bundle, previous, probe),
        "price" => price::solve(rng, state_bundle, previous, probe),
        "surrogate" => surrogate::solve(rng, state_bundle, previous, probe),
        "memetic" => memetic::solve(rng, state_bundle, previous, probe),
        "cem" => cem::solve(rng, state_bundle, previous, probe),
        "lns" => lns::solve(rng, state_bundle, previous, probe),
        other => panic!("no lab algorithm {other}"),
    }
}

/// the metric of the state as it is, timed and counted as an evaluation
pub fn evaluate(state_bundle: &mut StateBundle, probe: &mut Probe) -> f64 {
    let clock = probe.clock();
    let metric = state_bundle.metric_router(probe);
    probe.evaluated(clock);
    metric
}
