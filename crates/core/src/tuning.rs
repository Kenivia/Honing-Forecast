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
    /// entries the memo of per material results may hold; 0 for no memo
    pub max_memo_size: usize,
    /// grid points a standard deviation of the coarse lattice, which answers the metric where the
    /// exact one is too big; 0 for none, which leaves it to the Edgeworth series
    pub coarse_cells: f64,
    /// multiplications a coarse lattice may take before the Edgeworth series answers instead
    pub max_coarse_work: usize,

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
    /// the share of normal honing moves that give an upgrade its best plan against the rest
    /// (`optimizer/response.rs`) in place of a random change of its streaks; 0 for none
    pub response_rate: f64,
    /// the share of those that first change another upgrade at random
    pub response_kick: f64,
    /// steps of a response's recursion that count as one evaluation
    pub response_steps: f64,
    /// steps a response may take over every plan of its upgrade at once; past that each pool
    /// takes its best taps in turn
    pub response_joint: usize,
    /// amounts of a material a response evaluates where the lattice does not answer
    pub response_points: usize,
    /// evaluations between two passes of responses over the best state, from which the
    /// chain goes on; 0 for never
    pub response_every: i64,
    /// passes of responses over the best state at the end; 0 for none
    pub response_polish: usize,
    /// a response may take two ids of one pool, on different taps
    pub response_mixed: bool,

    /// taps a streak length is a multiple of, in place of the payload's `min_resolution`; 0 for the payload's
    pub streak_length: usize,

    /// going back to the best state reads its streaks again, in place of going on from the chain's
    pub fresh_streaks: bool,
    /// rounds of the polishes at the end (free tap order, responses, chests) while any moves; 0 for the responses alone
    pub polish_rounds: usize,

    /// chains that share `max_iters` between them, each from the start and polished, the best kept;
    /// 0 for as many as fit at `chain_iters_per_upgrade` evaluations an upgrade each, no less than
    /// `chain_iters_min` a chain and no fewer than 4 chains
    pub chains: usize,
    pub chain_iters_per_upgrade: f64,
    pub chain_iters_min: i64,
    /// stop once this many chains have ended on the same best; 0 for never
    pub agree_exit: usize,
    /// stop once this many results have ended on the same best and none elsewhere; 0 for never
    pub unanimous: usize,
    /// the best chain's result takes what the others ended on, piece by piece, where that is better
    pub merge: bool,
    /// the best result is polished by moving uses of a juice or book between two upgrades
    pub exchange: bool,
    /// and so under a streak length, a block at a time
    pub exchange_blocks: bool,
    /// the least share of moves that change the free tap order, while there is a budget
    pub special_floor: f64,
    /// a move of the free tap order puts an upgrade where a free tap can get to
    pub order_reach: bool,
    /// start from a free tap order made by what a free tap saves for what it takes
    pub greedy_order: bool,
    /// rounds of the shadow price start (`optimizer/shadow.rs`) every fresh chain goes on from; 0 for none
    pub shadow_start: usize,

    // chained runs
    pub max_blob_bytes: usize,
    /// start from the plan of the run before, where it is better than the one given
    pub carry: bool,
    /// the share of `max_iters` a run that starts from a carried plan gets
    pub carry_iters: f64,
    /// a run that starts from a carried plan polishes it first
    pub carry_polish: bool,
    /// such a run stops once this many chains in a row have not bettered the best; 0 for never
    pub carry_stall: usize,
    /// the run gone on from was of nearly this payload (the payload's `quick`): one stalled chain
    /// is enough, unless polishing the carried plan gained more than `quick_settle` of what the
    /// upgrades cost
    pub quick: bool,
    pub quick_settle: f64,
    /// other plans the chains ended on that a run leaves in its blob for the next
    pub elites: usize,
    /// the carried plans that are polished, best first; the best of them is gone on from
    pub warm_polish: usize,
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
            max_memo_size: 1 << 16,
            coarse_cells: 8.0,
            max_coarse_work: 20_000_000,

            max_iters: 48000,
            max_best_size: 10,
            non_impact_weight: 3.0,
            magic_number: 0.44,
            warm_up_phase_end: 0.15,
            cooling_phase_start: 0.65,
            special_affinity_decay: 0.999,
            special_affinity_growth: 1.02,
            chest_move_rate: 0.15,
            chest_polish_every: 6000,
            response_rate: 0.0,
            response_kick: 0.0,
            response_steps: 700.0,
            response_joint: 20000,
            response_points: 16,
            response_every: 0,
            response_polish: 3,
            response_mixed: true,
            streak_length: 0,
            fresh_streaks: true,
            polish_rounds: 3,

            chains: 0,
            chain_iters_per_upgrade: 250.0,
            chain_iters_min: 4000,
            agree_exit: 5,
            unanimous: 3,
            merge: true,
            exchange: true,
            exchange_blocks: true,
            special_floor: 0.2,
            order_reach: true,
            greedy_order: true,
            shadow_start: 4,
            max_blob_bytes: 5 * 1024 * 1024,
            carry: true,
            carry_iters: 1.0,
            carry_polish: true,
            carry_stall: 4,
            quick: false,
            quick_settle: 1e-3,
            elites: 0,
            warm_polish: 1,
        }
    }
}

impl Tuning {
    /// What reported metrics are scored with: the coarse lattice at 64 points, where the optimizer has 8
    pub fn reference() -> Tuning {
        Tuning {
            coarse_cells: 64.0,
            ..Tuning::default()
        }
    }

    /// every knob a probability depends on, for keys of kept results
    pub fn evaluator_key(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        (
            self.root_finder as u8,
            self.max_root_find_iter,
            self.max_brute_size,
            self.max_lattice_work,
        )
            .hash(&mut hasher);
        (
            self.y_value_tol.to_bits(),
            self.taylor_skip.to_bits(),
            self.coarse_cells.to_bits(),
        )
            .hash(&mut hasher);
        self.max_coarse_work.hash(&mut hasher);
        hasher.finish()
    }
}
