//! Serves in place of saddlepoint_approximation when the complexity isn't too bad

use crate::constants::FLOAT_TOL;
use crate::state_bundle::StateBundle;
use ahash::AHashMap;
use either::Either;
use std::hash::{Hash, Hasher};
pub const MAX_BRUTE_SIZE: usize = 50000;

#[derive(Clone, Copy, Debug)]
pub struct FloatKey(f64, u64);

impl From<f64> for FloatKey {
    fn from(x: f64) -> Self {
        let rounded = (x * 1.0e6) as u64; // cost should be positive anyway
        FloatKey(x, rounded)
    }
}
impl PartialEq for FloatKey {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for FloatKey {}
impl Hash for FloatKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

impl StateBundle {
    pub fn brute_success_prob(
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        mean: f64,
        biased: bool,
    ) -> f64 {
        let n = self.upgrade_arr.len();
        let inv_mean = if mean.abs() < FLOAT_TOL {
            0.0
        } else {
            mean.recip()
        };

        let prune_flipped: bool = budget > mean;

        // min_suffix[i] = sum of minimum costs from layer i to end
        // max_suffix[i] = sum of maximum costs from layer i to end
        let mut min_suffix = vec![0.0; n + 1];
        let mut max_suffix = vec![0.0; n + 1];
        let mut future_avg = vec![0.0; n + 1];
        let mut u_index = n - 1; // cant use enumerate for whatever reason and i cant be bothered to find out why 

        for support in self.extract_all_support_with_meta(support_index).rev() {
            min_suffix[u_index] =
                min_suffix[u_index + 1] + support.access_min(skip_count > u_index);
            max_suffix[u_index] =
                max_suffix[u_index + 1] + support.access_max(skip_count > u_index);

            future_avg[u_index] = future_avg[u_index + 1]
                + support
                    .access_collapsed(skip_count > u_index)
                    .iter()
                    .fold(0.0, |prev, (s, p)| prev + s * p);
            u_index = u_index.saturating_sub(1);
        }

        // Stores currently active uncertain states
        let mut current_states: AHashMap<FloatKey, f64> = AHashMap::with_capacity(1);
        let mut next_states: AHashMap<FloatKey, f64> = AHashMap::with_capacity(256);
        current_states.insert(FloatKey::from(0.0), 1.0);

        let mut total_guaranteed_prob = 0.0;

        // my_dbg!(&future_avg);
        let mut index = 0;
        for pairs in self.extract_collapsed_pair(support_index, skip_count) {
            let next_min_rem = min_suffix[index + 1];
            let next_max_rem = max_suffix[index + 1];
            let cur_cap = next_states.capacity();
            next_states.reserve(
                (current_states.len() * pairs.len())
                    .min(MAX_BRUTE_SIZE)
                    .saturating_sub(cur_cap),
            );

            for (current_cost, current_prob) in current_states.drain() {
                let iter = if prune_flipped {
                    Either::Left(pairs.into_iter().rev())
                } else {
                    Either::Right(pairs.into_iter())
                };
                for (s, p) in iter {
                    let new_cost = current_cost.0 + s;
                    let step_prob = current_prob * p;

                    if prune_flipped {
                        if new_cost + next_max_rem < budget - FLOAT_TOL {
                            break;
                        }

                        if new_cost + next_min_rem > budget + FLOAT_TOL {
                            if biased {
                                total_guaranteed_prob +=
                                    step_prob * (new_cost + future_avg[index + 1]) * inv_mean;
                            } else {
                                total_guaranteed_prob += step_prob;
                            }

                            continue;
                        }
                    } else {
                        if new_cost + next_min_rem > budget + FLOAT_TOL {
                            break;
                        }
                        if new_cost + next_max_rem <= budget - FLOAT_TOL {
                            if biased {
                                total_guaranteed_prob +=
                                    step_prob * (new_cost + future_avg[index + 1]) * inv_mean;
                            } else {
                                total_guaranteed_prob += step_prob;
                            }
                            continue;
                        }
                    }

                    *next_states.entry(FloatKey::from(new_cost)).or_insert(0.0) += step_prob;
                }
            }
            std::mem::swap(&mut current_states, &mut next_states);
            if current_states.is_empty() {
                break;
            }
            index += 1;
        }

        let sum: f64 = total_guaranteed_prob
            + if biased {
                current_states
                    .iter()
                    .fold(0.0, |prev, (cost, p)| prev + cost.0 * p * inv_mean)
            } else {
                current_states.iter().fold(0.0, |prev, (_, p)| prev + p)
            };
        if prune_flipped { 1.0 - sum } else { sum }
    }
}
