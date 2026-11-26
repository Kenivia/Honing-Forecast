//! The calculator's Maxroll override (`optimizer/maxroll.rs`) as a lab algorithm: juice, books
//! and the free tap order by Maxroll's rule from the given state. Switches `lab_on`: `ifreached`
//! (owned stock by gold saved if reached), `nostock` (owned stock ignored). Tradable
//! stock as free (`allstock`) was removed: 70 of 724 worse on `real3na`.

use super::super::maxroll::{Maxroll, Owned, apply_owned};
use super::super::{Previous, Solved};
use crate::honing_utils::state_bundle::StateBundle;
use crate::probe::Probe;
use rand::Rng;

pub const OFF: &[&str] = &[];
pub const ON: &[&str] = &["ifreached", "nostock"];

pub fn solve<R: Rng>(
    _rng: &mut R,
    mut bundle: StateBundle,
    _previous: Option<&Previous>,
    probe: &mut Probe,
) -> Solved {
    let on = |name: &str| bundle.tuning.lab_on.split('+').any(|n| n == name);
    let owned = Owned {
        stock: !on("nostock"),
        if_reached: on("ifreached"),
    };
    let all = Maxroll {
        juice: true,
        book: true,
        special: true,
    };
    apply_owned(&mut bundle, all, owned);
    bundle.metric = bundle.evaluate(probe);
    bundle.set_latest_special_probs();
    Solved {
        best: bundle,
        blob: Vec::new(),
    }
}
