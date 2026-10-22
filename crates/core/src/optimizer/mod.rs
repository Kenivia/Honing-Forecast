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
mod shadow;
mod simulated_annealing;

use crate::chest::ChestStates;
use crate::state::OneState;
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
pub use shadow::shadow_start;
pub use simulated_annealing::{polish_only, scramble, solve};

/// What a run knows of the runs before it: their plans, the best first.
pub struct Previous {
    pub plans: Vec<Plan>,
}

pub struct Solved {
    pub best: StateBundle,
    /// what the optimizer leaves for the next run, at most `Tuning::max_blob_bytes`
    pub blob: Vec<u8>,
}

/// One plan, keyed by upgrade so that it goes onto other upgrades. A state is run-length
/// coded, plans being long runs of the same slot.
#[derive(Serialize, Deserialize, Clone)]
pub struct Plan {
    pub keys: Vec<(usize, usize, bool)>,
    pub states: Vec<Vec<(u32, OneState)>>,
    /// places in `keys`
    pub special_state: Vec<usize>,
    pub chest_states: ChestStates,
}

/// What a run leaves for the next: other plans its chains ended on, best first.
#[derive(Serialize, Deserialize, Default)]
pub struct Blob {
    pub plans: Vec<Plan>,
}

fn key(upgrade: &Upgrade) -> (usize, usize, bool) {
    (
        upgrade.piece_index,
        upgrade.upgrade_index,
        upgrade.is_normal_honing,
    )
}

impl Plan {
    pub fn of(state_bundle: &StateBundle) -> Plan {
        let states = state_bundle
            .upgrade_arr
            .iter()
            .map(|upgrade| {
                let mut runs: Vec<(u32, OneState)> = Vec::new();
                for slot in &upgrade.state.payload {
                    match runs.last_mut() {
                        Some((count, last)) if last == slot => *count += 1,
                        _ => runs.push((1, slot.clone())),
                    }
                }
                runs
            })
            .collect();
        Plan {
            keys: state_bundle.upgrade_arr.iter().map(key).collect(),
            states,
            special_state: state_bundle.special_state.clone(),
            chest_states: state_bundle.chest_states.clone(),
        }
    }

    /// Puts this plan on these upgrades. An upgrade keeps its state, less the taps done since,
    /// and its place in the free tap order; a new one goes last. A chest stack that is as it
    /// was is opened as it was.
    pub fn carry(&self, state_bundle: &mut StateBundle) {
        let was: AHashMap<(usize, usize, bool), usize> = self
            .keys
            .iter()
            .enumerate()
            .map(|(index, key)| (*key, index))
            .collect();
        let mut now: AHashMap<(usize, usize, bool), usize> = AHashMap::new();
        for (index, upgrade) in state_bundle.upgrade_arr.iter_mut().enumerate() {
            now.insert(key(upgrade), index);
            let Some(&old) = was.get(&key(upgrade)) else {
                continue;
            };
            let mut state: Vec<OneState> = self.states[old]
                .iter()
                .flat_map(|(count, slot)| std::iter::repeat_n(slot.clone(), *count as usize))
                .collect();
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
            .special_state
            .iter()
            .filter_map(|&old| now.get(&self.keys[old]).copied())
            .collect();
        let new: Vec<usize> = (0..state_bundle.upgrade_arr.len())
            .filter(|index| !order.contains(index))
            .collect();
        order.extend(new);
        state_bundle.special_state = order;
        let mut chest_states = state_bundle.chest_states.clone();
        for (key, state) in &self.chest_states {
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

impl Blob {
    pub fn of(plans: &[&StateBundle]) -> Vec<u8> {
        let blob = Blob {
            plans: plans.iter().map(|plan| Plan::of(plan)).collect(),
        };
        postcard::to_allocvec(&blob).unwrap()
    }

    /// nothing where it is not one
    pub fn read(bytes: &[u8]) -> Blob {
        postcard::from_bytes(bytes).unwrap_or_default()
    }
}

impl Previous {
    /// the best plan of the run before and the others it left
    pub fn of(best: &StateBundle, blob: &[u8]) -> Previous {
        Previous {
            plans: std::iter::once(Plan::of(best))
                .chain(Blob::read(blob).plans)
                .collect(),
        }
    }

    /// each plan put on these upgrades
    pub fn carried(&self, state_bundle: &StateBundle) -> Vec<StateBundle> {
        self.plans
            .iter()
            .map(|plan| {
                let mut other = state_bundle.clone();
                plan.carry(&mut other);
                other
            })
            .collect()
    }

    pub fn carry(&self, state_bundle: &mut StateBundle) {
        self.plans[0].carry(state_bundle);
    }
}
