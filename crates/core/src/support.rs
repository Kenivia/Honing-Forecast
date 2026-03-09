use serde::{Deserialize, Serialize};

use std::f64::{INFINITY, NEG_INFINITY};
use std::ops::Deref;

use crate::constants::FLOAT_TOL;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ProbDist {
    pub payload: Vec<f64>,
    #[serde(skip)]
    pub prob_state_hash: u64,
}

impl ProbDist {
    pub fn new(new_payload: Vec<f64>) -> ProbDist {
        ProbDist {
            payload: new_payload,
            prob_state_hash: 0,
            // logged_payload: vec![],
            // log_state_hash: 0,
        }
    }
    pub fn update_payload(&mut self, new_payload: Vec<f64>, new_state_hash: u64) {
        self.payload = new_payload;
        self.prob_state_hash = new_state_hash;
    }
}

impl Deref for ProbDist {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Support {
    pub support: Vec<f64>,
    #[serde(skip)]
    pub support_state_hash: u64,
    pub state_invariant: bool, // only talking about the support, prob can always change
    pub linear: bool,          // gap between 0 and 1 = gap between n and n+1

    collapsed_pair: Vec<(f64, f64)>, // (support, prob, logged_prob)
    #[serde(skip)]
    pub collapsed_state_hash: u64,
    pub ignore: bool,
    pub gap_size: f64,
    max_value: f64,
    min_value: f64,
    pub first_non_zero_prob_index: usize,
    skipped_pair: Vec<(f64, f64)>,
}

impl Support {
    pub fn access_collapsed(&self, skipped: bool) -> &Vec<(f64, f64)> {
        assert!(self.collapsed_state_hash == self.support_state_hash);
        if skipped {
            &self.skipped_pair
        } else {
            &self.collapsed_pair
        }
    }
    pub fn access_max(&self, skipped: bool) -> f64 {
        if skipped {
            self.access_collapsed(true)[0].0
        } else {
            self.max_value
        }
    }

    pub fn access_min(&self, skipped: bool) -> f64 {
        if skipped {
            self.access_collapsed(true)[0].0
        } else {
            self.min_value
        }
    }
    /// Incredibly crucial pre-processing, collapses identical values into 1 thing, and removes values with p = 0.
    /// cumulant.rs makes the assumption that nothing has p = 0
    ///
    /// we assume that the support is increasing, but p may be 0 in-between (probably exceptionally rare but possible in advanced honing?)
    ///
    /// also right now everything is linear so some logic is a bit redundant, but keeping them just in case
    pub fn collapse_support(&mut self, prob_dist: &ProbDist, alr_failed: usize) {
        // these hash checks are mostly just for preventing me from doing stupid stuff and saving me from debugging
        // it has the added benefit that we don't update if the state didn't change but is like negligible
        assert!(prob_dist.payload.len() == self.support.len());
        assert!(prob_dist.prob_state_hash == self.support_state_hash);

        if self.collapsed_state_hash != self.support_state_hash {
            let mut result: Vec<(f64, f64)> = Vec::with_capacity(self.support.len());
            let mut max_value: f64 = NEG_INFINITY;
            let mut min_value: f64 = INFINITY;

            let mut cur_p = 0.0;
            let front = prob_dist.iter().take_while(|p| **p == 0.0).count();
            let back = prob_dist.iter().rev().take_while(|p| **p == 0.0).count();

            assert!(front != prob_dist.len());
            let mut iter = self
                .support
                .iter()
                .zip(prob_dist.iter())
                .skip(front)
                .take(prob_dist.len() - back - front)
                .peekable();
            while let Some((&s, &p)) = iter.next() {
                cur_p += p;
                if iter.peek().is_none() || s != *iter.peek().unwrap().0 {
                    if cur_p > FLOAT_TOL {
                        max_value = max_value.max(s);
                        min_value = min_value.min(s);
                        result.push((s, cur_p));
                    }

                    cur_p = 0.0;
                }
            }
            self.skipped_pair[0].0 = self.support[alr_failed];
            self.ignore = result.len() == 1 && result[0].0.abs() < FLOAT_TOL;
            self.max_value = max_value;
            self.min_value = min_value;
            self.first_non_zero_prob_index = 0;
            self.collapsed_pair = result;
            self.collapsed_state_hash = self.support_state_hash;
        }
    }

    pub fn update_payload(
        &mut self,
        new_payload: Vec<f64>,
        new_state_hash: u64,
        prob_dist: &ProbDist,
        gap_size: f64,
        linear: bool, // max: f64,
        alr_failed: usize,
    ) {
        self.support = new_payload;
        self.support_state_hash = new_state_hash;
        self.gap_size = gap_size;
        self.linear = linear;
        self.collapse_support(prob_dist, alr_failed);
    }
}

impl Default for Support {
    fn default() -> Self {
        Self {
            support: vec![0.0],
            support_state_hash: 0,
            state_invariant: false,
            linear: false,
            collapsed_state_hash: 0,
            collapsed_pair: vec![(0.0, 1.0)],
            ignore: true,
            gap_size: 0.0,
            max_value: 0.0,
            min_value: 0.0,
            first_non_zero_prob_index: 0,
            skipped_pair: vec![(0.0, 1.0)],
        }
    }
}
