//! Every knob of the evaluator and the optimizer that is worth varying at runtime.
//!
//! The default is production. The arena overrides fields by name, so a new knob only
//! needs a field here and a read where the constant used to be.

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RootFinder {
    Householder,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tuning {
    // evaluator
    pub root_finder: RootFinder,
    pub y_value_tol: f64,
    pub max_root_find_iter: usize, // was 20 before, but crashed 1 time
    pub max_brute_size: usize,
    pub max_lattice_work: usize, // lattice points times outcomes
    /// a last step of the root finder shorter than this, in standard deviations, is taken
    /// on the taylor series instead of another evaluation; 0 for never
    pub taylor_skip: f64,
    /// start the biased root from the plain one
    pub seed_biased: bool,
    /// entries the memo of per material results may hold; 0 for no memo
    pub max_memo_size: usize,

    // optimizer
    pub max_iters: i64,
    pub max_best_size: usize,
    pub non_impact_weight: f64,
    // the expected acceptance rate schedule
    pub magic_number: f64,
    pub warm_up_phase_end: f64,
    pub cooling_phase_start: f64,
    pub special_affinity_decay: f64,
    pub special_affinity_growth: f64,
    /// the share of moves that reopen some chests of a stack, where a stack has a choice
    pub chest_move_rate: f64,
    /// evaluations between two polishes of the best state's chests; 0 for never
    pub chest_polish_every: i64,

    // chained runs
    pub max_blob_bytes: usize,
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            root_finder: RootFinder::Householder,
            y_value_tol: 1e-6,
            max_root_find_iter: 30,
            max_brute_size: 50000,
            max_lattice_work: 200000,
            taylor_skip: 1e-3,
            seed_biased: true,
            max_memo_size: 1 << 16,

            max_iters: 24000,
            max_best_size: 10,
            non_impact_weight: 3.0,
            magic_number: 0.44,
            warm_up_phase_end: 0.15,
            cooling_phase_start: 0.65,
            special_affinity_decay: 0.999,
            special_affinity_growth: 1.02,
            chest_move_rate: 0.15,
            chest_polish_every: 6000,

            max_blob_bytes: 5 * 1024 * 1024,
        }
    }
}

impl Tuning {
    /// every knob a probability depends on, for keys of kept results
    pub fn evaluator_key(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        (self.root_finder as u8, self.max_root_find_iter, self.max_brute_size, self.max_lattice_work, self.seed_biased)
            .hash(&mut hasher);
        (self.y_value_tol.to_bits(), self.taylor_skip.to_bits()).hash(&mut hasher);
        hasher.finish()
    }
}
