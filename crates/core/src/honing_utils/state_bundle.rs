use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::eval::lattice::CoarseMemo;
use crate::honing_utils::chest::ChestStates;
use crate::optimizer::state::OneState;
use crate::parser::PreparationOutput;
use crate::probe::Probe;
use crate::tuning::Tuning;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateBundle {
    pub upgrade_arr: Vec<Upgrade>,
    pub special_state: Vec<usize>,
    /// one per chest of prep_output.chests, under its key
    pub chest_states: ChestStates,
    pub special_invalid_index: Option<usize>,
    pub latest_special_probs: Option<Vec<f64>>,
    pub metric: f64,
    pub min_resolution: usize,
    pub prep_output: PreparationOutput,

    pub num_threads: usize,

    /// by free tap budget, which the chest states move, then by free tap order
    #[serde(skip)]
    pub special_cache: AHashMap<i64, AHashMap<Vec<usize>, Vec<f64>>>,

    pub adv_cache: AHashMap<AdvConfig, AdvDistTriplet>,

    /// the free tap budget and order the cache was last filled for
    #[serde(skip)]
    pub special_done: Option<(i64, Vec<usize>)>,

    /// one material's average gold under one free tap outcome, by what it was made from
    #[serde(skip)]
    pub gold_memo: AHashMap<u64, f64>,

    #[serde(skip)]
    pub coarse_memo: CoarseMemo,

    #[serde(skip)]
    pub tuning: Tuning,
}

impl StateBundle {
    pub fn set_latest_special_probs(&mut self) {
        let mut out = Vec::with_capacity(self.upgrade_arr.len());
        self.clean_special_state();
        let dist = self.compute_special_probs(true).unwrap();
        for (index, _) in dist.iter().enumerate() {
            out.push(if index < self.special_invalid_index.unwrap() {
                dist.iter().skip(index + 1).sum::<f64>()
            } else {
                0.0
            });
        }
        self.latest_special_probs = Some(out);
    }

    /// The taps that go together: from an upgrade's next tap on, each run of this many takes
    /// the same, so that every streak but the last before the pity is a multiple of it.
    pub fn block(&self) -> usize {
        if self.tuning.streak_length > 0 {
            self.tuning.streak_length
        } else {
            self.min_resolution.max(1)
        }
    }

    /// whether the budget pays for one free tap of any upgrade; where not, the order is nothing
    pub fn affords_a_free_tap(&self) -> bool {
        let budget = self.prep_output.special_budget;
        self.upgrade_arr
            .iter()
            .any(|upgrade| upgrade.is_normal_honing && upgrade.special_cost <= budget)
    }

    /// the metric of the state as it is, timed as an evaluation (`Probe::evaluated`)
    pub fn evaluate(&mut self, probe: &mut Probe) -> f64 {
        let clock = probe.clock();
        let metric = self.optimizer_average_gold_metric(probe);
        probe.evaluated(clock);
        metric
    }

    /// the chest stacks with a choice, and their option counts
    pub fn choice_stacks(&self) -> Vec<(String, usize)> {
        self.prep_output
            .chests
            .iter()
            .filter(|(_, chest)| chest.options.len() > 1 && chest.count > 0)
            .map(|(key, chest)| (key.clone(), chest.options.len()))
            .collect()
    }

    /// indices of the normal honing upgrades, or of the advanced ones
    pub fn upgrades_of(&self, normal: bool) -> Vec<usize> {
        (0..self.upgrade_arr.len())
            .filter(|&index| self.upgrade_arr[index].is_normal_honing == normal)
            .collect()
    }

    /// for sending to JS, which keeps its own
    pub fn without_adv_cache(&self) -> StateBundle {
        let mut out = self.clone();
        out.adv_cache = AHashMap::new();
        out
    }

    pub fn my_clone_from(&mut self, source: &StateBundle) {
        for (s, upgrade) in source.upgrade_arr.iter().zip(self.upgrade_arr.iter_mut()) {
            upgrade.state.copy_from(&s.state);
        }
        self.special_state.clone_from(&source.special_state);
        self.set_chest_state(&source.chest_states);
        self.metric = source.metric;
    }

    pub fn to_essence(&self) -> StateEssence {
        StateEssence {
            state_arr: self
                .upgrade_arr
                .iter()
                .map(|x| x.state.payload.clone())
                .collect(),
            special_state: self.special_state.clone(),
            chest_states: self.chest_states.clone(),
        }
    }

    pub fn set_chest_state(&mut self, source: &ChestStates) {
        if self.chest_states != *source {
            self.chest_states.clone_from(source);
            self.apply_chest_state(false);
        }
    }

    /// a state stored elsewhere put on these upgrades; its chest states only where its stacks
    /// are these (a snapshot from before chests sends none)
    pub fn apply_essence(&mut self, essence: &StateEssence) {
        for (state, upgrade) in essence.state_arr.iter().zip(self.upgrade_arr.iter_mut()) {
            upgrade.state.update_payload(state.clone());
        }
        self.special_state = essence.special_state.clone();
        if essence.chest_states.keys().eq(self.chest_states.keys()) {
            self.set_chest_state(&essence.chest_states);
        }
    }

    pub fn clone_from_essence(&mut self, source: &StateEssence, input_metric: &OrderedFloat<f64>) {
        for (s, upgrade) in source.state_arr.iter().zip(self.upgrade_arr.iter_mut()) {
            if upgrade.state.payload != *s {
                upgrade.state.payload.clone_from(s);
                upgrade.state.update_hash();
            }
        }
        self.special_state.clone_from(&source.special_state);
        self.set_chest_state(&source.chest_states);
        self.metric = f64::from(*input_metric);
    }
}

/// The decisions of a state alone; the arena stores it as JSON.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateEssence {
    pub state_arr: Vec<Vec<OneState>>,
    pub special_state: Vec<usize>,
    #[serde(default)]
    pub chest_states: ChestStates,
}
