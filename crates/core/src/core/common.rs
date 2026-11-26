//! What the evaluator's parts share: the gold memo's hash and a sum's mean

use crate::state_bundle::StateBundle;

const MIX: ahash::RandomState = ahash::RandomState::with_seeds(1, 2, 3, 4);

pub fn mix(a: u64, b: u64) -> u64 {
    MIX.hash_one((a, b))
}

impl StateBundle {
    /// notably this is the average of the unbiased distribution(it's not the mean of the biased one)
    pub fn simple_avg(&self, support_index: i64, skip_count: usize) -> f64 {
        let mut mean: f64 = 0.0;
        for (index, &u_index) in self.special_state.iter().enumerate() {
            let support = self.support_from_index(u_index, support_index);
            mean += if index < skip_count {
                support.access_min(true)
            } else {
                support.mean
            };
        }
        mean
    }
}
