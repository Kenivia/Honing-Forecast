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

/// Panics on a switch or kick kind nothing reads, so a typo cannot run as the plain algorithm.
/// `solve` reads the surrogate's switches in its kick loop and memetic's in `mem_polish`, as
/// does a lab algorithm that runs `solve` (`fromsolve`).
pub fn check_switches(tuning: &crate::tuning::Tuning) {
    let (mut off, mut on): (Vec<&str>, Vec<&str>) = match tuning.algo.as_str() {
        "" => (vec![], vec![]),
        "price" => (price::OFF.to_vec(), price::ON.to_vec()),
        "surrogate" => (surrogate::OFF.to_vec(), surrogate::ON.to_vec()),
        "memetic" => (memetic::OFF.to_vec(), memetic::ON.to_vec()),
        "cem" => (cem::OFF.to_vec(), cem::ON.to_vec()),
        "lns" => (lns::OFF.to_vec(), lns::ON.to_vec()),
        other => panic!("no lab algorithm {other}"),
    };
    let solves = tuning.algo.is_empty() || tuning.lab_on.split('+').any(|n| n == "fromsolve");
    if solves && tuning.sur_stall > 0 {
        off.extend(surrogate::OFF);
        on.extend(surrogate::ON);
        for kind in tuning.sur_kinds.split('+') {
            assert!(surrogate::KINDS.contains(&kind), "no sur_kinds kind {kind}");
        }
    }
    for kind in tuning.kick_types.split('+') {
        let kinds = [
            "window",
            "chest",
            "adv",
            "piece",
            "rebuild",
            "rebuildempty",
            "rebuildprice",
        ];
        assert!(kinds.contains(&kind), "no kick_types kind {kind}");
    }
    assert!(
        ["", "screen", "none"].contains(&tuning.chain_polish.as_str()),
        "no chain_polish {}",
        tuning.chain_polish
    );
    assert!(
        ["", "all", "final"].contains(&tuning.chest_flip.as_str()),
        "no chest_flip {}",
        tuning.chest_flip
    );
    if solves && tuning.mem_polish {
        off.extend(memetic::OFF);
        on.extend(memetic::ON);
    }
    let algo = if tuning.algo.is_empty() {
        "solve"
    } else {
        &tuning.algo
    };
    for (field, list, known) in [
        ("lab_off", &tuning.lab_off, off),
        ("lab_on", &tuning.lab_on, on),
    ] {
        for name in list.split('+').filter(|n| !n.is_empty()) {
            assert!(
                known.contains(&name),
                "{algo} reads no {field} switch {name} here"
            );
        }
    }
}

/// the metric of the state as it is, timed and counted as an evaluation
pub fn evaluate(state_bundle: &mut StateBundle, probe: &mut Probe) -> f64 {
    let clock = probe.clock();
    let metric = state_bundle.metric_router(probe);
    probe.evaluated(clock);
    metric
}
