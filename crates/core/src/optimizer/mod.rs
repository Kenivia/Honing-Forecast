//! A Simulated annealing implementation based a heuristic from <http://dx.doi.org/10.1609/socs.v8i1.18424>
//! I'm SURE there's a lot of things we can do better like i'm in no way an authority
//!
//! Some specific things off the top of my head:
//! - States are restricted to a streak that start from the beginning and a streak that starts from the end. This (empirically from algorithms that didn't have this assumption) appear to always be the optimal
//! - Our energy function is not normalized (average gold cost could be literally anything), so the normal temperature formula didn't work. This uses a crude (and probably too sensitve but whatever) adaptive temperature function to keep acceptance rate at around the wanted rate
//! - Restarts are triggered VERY often to the point I feel like we're just doing hill climbing but seems to work well enough
//! - Whenever we restart, we (sometimes) do a self-crossover where we take a state from an identical piece and copy it to another state. This is to reflect the fact that sometimes the best strategy is the same among identical upgrades.
//!
//! This module is under constant iterative improvements so I'll probably forget to update this text when I do that
//! Which is also why I'm commenting here rather than the actual algorithm (and not cleaning up) cos they're very much subject to change
//!
//! Variants are compared with the arena (crates/arena): constants live in `Tuning`, and a
//! code change is a snapshot of the working tree.

mod neighbour;
mod one_batch;
mod scaler;
mod simulated_annealing;

use crate::payload::Payload;
use crate::state_bundle::StateBundle;
pub use simulated_annealing::{scramble, solve};

/// What a run in a chain knows about the run before it.
pub struct Previous {
    pub payload: Payload,
    pub best: StateBundle,
    /// whatever the optimizer left for itself, at most `Tuning::max_blob_bytes`
    pub blob: Vec<u8>,
}

pub struct Solved {
    pub best: StateBundle,
    pub blob: Vec<u8>,
}
