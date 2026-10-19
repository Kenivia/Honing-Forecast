use serde::{Deserialize, Serialize};

use std::f64::{INFINITY, NEG_INFINITY};
use std::ops::Deref;

use crate::constants::{FLOAT_TOL, IGNORE_PROB_TOL};
use crate::core::probability::cumulants::KsTuple;

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

    collapsed_pair: Vec<(f64, f64)>, // (support, prob)
    #[serde(skip)]
    pub collapsed_state_hash: u64,
    pub ignore: bool,
    pub gap_size: f64,
    max_value: f64,
    min_value: f64,
    pub first_non_zero_prob_index: usize,
    skipped_pair: Vec<(f64, f64)>,
    /// of the collapsed pairs, kept because they are asked for far more often than they change
    pub mean: f64,
    pub zero: KsTuple,
    /// more than one outcome, each a gap from the last
    pub arithmetic: bool,
}

/// What the supports of one distribution share when each is a start and a gap an outcome:
/// where the probabilities start and stop, and the cumulants of how many outcomes in.
pub struct Outline {
    front: usize,
    len: usize,
    holes: bool,
    log_sum: f64,
    index: [f64; 4],
}

impl Outline {
    pub fn of(prob_dist: &ProbDist) -> Outline {
        let front = prob_dist
            .iter()
            .take_while(|p| p.abs() < IGNORE_PROB_TOL)
            .count();
        let back = prob_dist
            .iter()
            .rev()
            .take_while(|p| p.abs() < IGNORE_PROB_TOL)
            .count();
        assert!(front != prob_dist.len());
        let live = &prob_dist[front..prob_dist.len() - back];
        let (mut sum, mut first, mut second, mut third, mut fourth) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for (index, p) in live.iter().enumerate() {
            let x = index as f64;
            sum += p;
            first += x * p;
            second += x * x * p;
            third += x * x * x * p;
            fourth += x * x * x * x * p;
        }
        let (mean, second, third, fourth) = (first / sum, second / sum, third / sum, fourth / sum);
        let mu2 = (second - mean * mean).max(0.0);
        let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
        let mu4 = fourth - 4.0 * third * mean + 6.0 * second * mean * mean
            - 3.0 * mean * mean * mean * mean;
        Outline {
            front,
            len: live.len(),
            holes: live.iter().any(|p| p.abs() <= IGNORE_PROB_TOL),
            log_sum: sum.ln(),
            index: [mean, mu2, mu3, mu4 - 3.0 * mu2 * mu2],
        }
    }
}

impl Support {
    /// The support that starts here and rises by the gap an outcome, which is every base
    /// material's. Nothing to collapse where the gap is real and no probability is missing,
    /// and what is kept of it comes from the outline.
    pub fn update_even(
        &mut self,
        start: f64,
        gap: f64,
        state_hash: u64,
        prob_dist: &ProbDist,
        outline: &Outline,
    ) {
        self.support.clear();
        let mut value = start;
        for index in 0..prob_dist.len() {
            self.support.push(value);
            if index + 1 < prob_dist.len() {
                value += gap;
            }
        }
        self.support_state_hash = state_hash;
        self.gap_size = gap;
        self.linear = true;
        if outline.holes || outline.len < 2 || gap <= 0.0 {
            self.collapse_support(prob_dist);
            return;
        }
        let live = outline.front..outline.front + outline.len;
        self.collapsed_pair.clear();
        self.collapsed_pair.extend(
            self.support[live.clone()]
                .iter()
                .copied()
                .zip(prob_dist[live.clone()].iter().copied()),
        );
        self.min_value = self.support[live.start];
        self.max_value = self.support[live.end - 1];
        self.skipped_pair[0].0 = self.support[0];
        self.ignore = false;
        self.first_non_zero_prob_index = 0;
        self.collapsed_state_hash = state_hash;
        self.arithmetic = true;
        self.mean = self.min_value + gap * outline.index[0];
        let gap2 = gap * gap;
        self.zero = KsTuple(
            outline.log_sum,
            self.mean,
            gap2 * outline.index[1],
            gap2 * gap * outline.index[2],
            gap2 * gap2 * outline.index[3],
        );
    }

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
    /// One certain outcome, for splitting a sum on this upgrade's outcomes
    pub fn pin(&mut self, value: f64) {
        self.collapsed_pair = vec![(value, 1.0)];
        self.max_value = value;
        self.min_value = value;
        self.ignore = value.abs() < FLOAT_TOL;
        self.summarise();
    }

    fn summarise(&mut self) {
        self.mean = self
            .collapsed_pair
            .iter()
            .fold(0.0, |prev, (s, p)| prev + s * p);
        let low = self.collapsed_pair[0].0;
        self.arithmetic = self.collapsed_pair.len() > 1
            && self.collapsed_pair.iter().enumerate().all(|(index, pair)| {
                (pair.0 - (low + index as f64 * self.gap_size)).abs()
                    <= 1e-9 * pair.0.abs().max(1.0)
            });
        self.zero = self.one_cumulant(0.0, false);
    }

    /// Incredibly crucial pre-processing, collapses identical values into 1 thing, and removes values with p = 0.
    /// cumulant.rs makes the assumption that nothing has p = 0
    ///
    /// we assume that the support is increasing, but p may be 0 in-between (probably exceptionally rare but possible in advanced honing?)
    ///
    /// also right now everything is linear so some logic is a bit redundant, but keeping them just in case
    pub fn collapse_support(&mut self, prob_dist: &ProbDist) {
        // these hash checks are mostly just for preventing me from doing stupid stuff and saving me from debugging
        // it has the added benefit that we don't update if the state didn't change but is like negligible
        assert!(prob_dist.payload.len() == self.support.len());
        assert!(prob_dist.prob_state_hash == self.support_state_hash);

        if self.collapsed_state_hash != self.support_state_hash {
            let mut result: Vec<(f64, f64)> = Vec::with_capacity(self.support.len());
            let mut max_value: f64 = NEG_INFINITY;
            let mut min_value: f64 = INFINITY;

            let mut cur_p = 0.0;
            let front = prob_dist
                .iter()
                .take_while(|p| p.abs() < IGNORE_PROB_TOL)
                .count();
            let back = prob_dist
                .iter()
                .rev()
                .take_while(|p| p.abs() < IGNORE_PROB_TOL)
                .count();

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
                    if cur_p.abs() > IGNORE_PROB_TOL {
                        max_value = max_value.max(s);
                        min_value = min_value.min(s);
                        result.push((s, cur_p));
                    }

                    cur_p = 0.0;
                }
            }
            self.skipped_pair[0].0 = self.support[0];
            self.ignore = result.len() == 1 && result[0].0.abs() < FLOAT_TOL;
            self.max_value = max_value;
            self.min_value = min_value;
            self.first_non_zero_prob_index = 0;
            self.collapsed_pair = result;
            self.collapsed_state_hash = self.support_state_hash;
            self.summarise();
        }
    }

    pub fn update_payload(
        &mut self,
        new_payload: Vec<f64>,
        new_state_hash: u64,
        prob_dist: &ProbDist,
        gap_size: f64,
        linear: bool, // max: f64,
    ) {
        self.support = new_payload;
        self.support_state_hash = new_state_hash;
        self.gap_size = gap_size;
        self.linear = linear;
        self.collapse_support(prob_dist);
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
            mean: 0.0,
            zero: KsTuple::default(),
            arithmetic: false,
        }
    }
}
