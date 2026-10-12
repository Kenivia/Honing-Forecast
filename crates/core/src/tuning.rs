//! Every knob of the evaluator and the optimizer that is worth varying at runtime.
//!
//! The default is production. The arena overrides fields by name, so a new knob only
//! needs a field here and a read where the constant used to be.

use serde::{Deserialize, Serialize};

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

            max_iters: 24000,
            max_best_size: 10,
            non_impact_weight: 3.0,
            magic_number: 0.44,
            warm_up_phase_end: 0.15,
            cooling_phase_start: 0.65,
            special_affinity_decay: 0.999,
            special_affinity_growth: 1.02,

            max_blob_bytes: 5 * 1024 * 1024,
        }
    }
}
