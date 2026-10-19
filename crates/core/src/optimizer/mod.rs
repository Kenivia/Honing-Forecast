//! A Simulated annealing implementation based a heuristic from <http://dx.doi.org/10.1609/socs.v8i1.18424>
//! I'm SURE there's a lot of things we can do better like i'm in no way an authority
//!
//! Some specific things off the top of my head:
//! - States are restricted to a streak that start from the beginning and a streak that starts from the end. This (empirically from algorithms that didn't have this assumption) appear to always be the optimal
//! - Our energy function is not normalized (average gold cost could be literally anything), so the normal temperature formula didn't work. This uses a crude (and probably too sensitve but whatever) adaptive temperature function to keep acceptance rate at around the wanted rate
//! - Restarts are triggered VERY often to the point I feel like we're just doing hill climbing but seems to work well enough
//! - Whenever we restart, we (sometimes) do a self-crossover where we take a state from an identical piece and copy it to another state. This is to reflect the fact that sometimes the best strategy is the same among identical upgrades.
//! - A solve is several short chains of this, each polished, the best kept. See `AI notes/Calculator core/Optimizer.md`.
//!
//! This module is under constant iterative improvements so I'll probably forget to update this text when I do that
//! Which is also why I'm commenting here rather than the actual algorithm (and not cleaning up) cos they're very much subject to change
//!
//! Variants are compared with the arena (crates/arena): constants live in `Tuning`, and a
//! code change is a snapshot of the working tree.

mod neighbour;
mod one_batch;
pub mod response;
mod scaler;
mod simulated_annealing;

use crate::payload::Payload;
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
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

impl Previous {
    /// Puts the plan of the run before on this run's upgrades. An upgrade keeps its state, less
    /// the taps done since, and its place in the free tap order; a new one goes last. A chest
    /// stack that is as it was is opened as it was.
    pub fn carry(&self, state_bundle: &mut StateBundle) {
        let key = |upgrade: &Upgrade| {
            (
                upgrade.piece_index,
                upgrade.upgrade_index,
                upgrade.is_normal_honing,
            )
        };
        let was: AHashMap<(usize, usize, bool), usize> = self
            .best
            .upgrade_arr
            .iter()
            .enumerate()
            .map(|(index, upgrade)| (key(upgrade), index))
            .collect();
        let mut now: AHashMap<(usize, usize, bool), usize> = AHashMap::new();
        for (index, upgrade) in state_bundle.upgrade_arr.iter_mut().enumerate() {
            now.insert(key(upgrade), index);
            let Some(&old) = was.get(&key(upgrade)) else {
                continue;
            };
            let mut state = self.best.upgrade_arr[old].state.payload.clone();
            let len = upgrade.state.len();
            if upgrade.is_normal_honing {
                state.drain(..state.len().saturating_sub(len));
                state.resize(len, Default::default());
            }
            if state.len() == len {
                upgrade.state.update_payload(state);
            }
        }
        let mut order: Vec<usize> = self
            .best
            .special_state
            .iter()
            .filter_map(|&old| now.get(&key(&self.best.upgrade_arr[old])).copied())
            .collect();
        let new: Vec<usize> = (0..state_bundle.upgrade_arr.len())
            .filter(|index| !order.contains(index))
            .collect();
        order.extend(new);
        state_bundle.special_state = order;
        let mut chest_states = state_bundle.chest_states.clone();
        for (key, state) in &self.best.chest_states {
            if state_bundle
                .prep_output
                .chests
                .get(key)
                .is_some_and(|chest| chest.is_valid(state))
            {
                chest_states.insert(key.clone(), state.clone());
            }
        }
        state_bundle.set_chest_state(&chest_states);
    }
}
