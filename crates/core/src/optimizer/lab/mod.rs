//! Optimizer approaches written from scratch beside `solve`, picked by `Tuning::algo`.
//! See `AI notes/Calculator core/Optimizer lab.md`.

pub mod cem;
pub mod check;
pub mod lns;
pub mod maxroll;
pub mod memetic;
pub mod price;
pub mod simple;
pub mod surrogate;

use super::{Previous, Solved};
use crate::honing_utils::state_bundle::StateBundle;
use crate::probe::Probe;
use rand::Rng;

/// every lab algorithm, by the name `Tuning::algo` takes
pub const ALGOS: &[&str] = &[
    "price",
    "surrogate",
    "memetic",
    "cem",
    "lns",
    "simple",
    "maxroll",
];

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
        "simple" => simple::solve(rng, state_bundle, previous, probe),
        "maxroll" => maxroll::solve(rng, state_bundle, previous, probe),
        other => panic!("no lab algorithm {other}"),
    }
}

/// the kinds `Tuning::kick_types` takes; the `rebuild` ones are lab kicks
pub const KICK_TYPES: &[&str] = &[
    "window",
    "chest",
    "adv",
    "piece",
    "rebuild",
    "rebuildempty",
    "rebuildprice",
];

/// the `lab_off` and `lab_on` switches an algorithm reads
fn switches(algo: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match algo {
        "" => (&[], &[]),
        "price" => (price::OFF, price::ON),
        "surrogate" => (surrogate::OFF, surrogate::ON),
        "memetic" => (memetic::OFF, memetic::ON),
        "cem" => (cem::OFF, cem::ON),
        "lns" => (lns::OFF, lns::ON),
        "simple" => (simple::OFF, simple::ON),
        "maxroll" => (maxroll::OFF, maxroll::ON),
        other => panic!("no lab algorithm {other}"),
    }
}

/// Panics on a switch or kick kind nothing reads, so a typo cannot run as the plain algorithm.
/// `solve` reads the surrogate's switches in its kick loop and memetic's in `mem_polish`, as
/// does a lab algorithm that runs `solve` (`fromsolve`).
pub fn check_switches(tuning: &crate::tuning::Tuning) {
    let (off, on) = switches(&tuning.algo);
    let (mut off, mut on) = (off.to_vec(), on.to_vec());
    let solves = tuning.algo.is_empty() || tuning.lab_on.split('+').any(|n| n == "fromsolve");
    if solves && tuning.sur_stall > 0 {
        off.extend(surrogate::OFF);
        on.extend(surrogate::ON);
        for kind in tuning.sur_kinds.split('+') {
            assert!(surrogate::KINDS.contains(&kind), "no sur_kinds kind {kind}");
        }
    }
    for kind in tuning.kick_types.split('+') {
        assert!(KICK_TYPES.contains(&kind), "no kick_types kind {kind}");
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
