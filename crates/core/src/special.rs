//! This evaluates the probability that we succeed n free taps
//!
//! We make the restriction that we must attempt one upgrade until we run out or succeed
//! This drastically reduces the number of metric evaluations we need to do (average or success prob)
//! If we allowed everything (attempt this, then that, then back to this), then not only would this special prob be much harder to comupte,
//! we'd have to do 2^n metric evaluations, with this restriction it's n
//!
//! Admittedly dynamic programming is one of these things that I never really understood so this is mostly vibe coded
//! but this matches experimental result so at least it's working
use crate::constants::SPECIAL_TOL;
use crate::state_bundle::StateBundle;
use num::Integer;
use std::f64;
use std::mem::swap;
use std::ops::Range;

impl StateBundle {
    pub fn special_probs(&self) -> &Vec<f64> {
        // we don't automatically call compute_special here cos that'd need &mut self
        &self.special_cache[&self.special_state]
    }
    fn gcd_special(&self) -> i64 {
        let mut out: i64 = 1;
        for (index, upgrade) in self.upgrade_arr.iter().enumerate() {
            if !upgrade.is_normal_honing || upgrade.succeeded {
                continue;
            }
            if index == 0 {
                out = upgrade.special_cost;
            } else {
                out = out.gcd(&upgrade.special_cost);
            }
        }
        out
    }

    pub fn clean_special_state(&mut self) {
        let mut highest_upgrade_index_seen: Vec<i64> = vec![-1; 6];
        let mut valid_uindex: Vec<usize> = Vec::with_capacity(self.upgrade_arr.len());
        let mut invalid_uindex: Vec<usize> = Vec::with_capacity(self.upgrade_arr.len());
        for u_index in self.special_state.iter() {
            let upgrade = &self.upgrade_arr[*u_index];
            if !upgrade.is_normal_honing
                || upgrade.succeeded
                || highest_upgrade_index_seen[upgrade.piece_type] > upgrade.upgrade_index as i64
            {
                invalid_uindex.push(*u_index);
                continue;
            }
            valid_uindex.push(*u_index);
            highest_upgrade_index_seen[upgrade.piece_type] = upgrade.upgrade_index as i64;
        }
        invalid_uindex.sort();
        self.special_invalid_index = Some(valid_uindex.len());
        valid_uindex.extend_from_slice(&invalid_uindex);
        self.special_state = valid_uindex;
    }

    /// The preserve tail option is exclusively for UI purpose and does not write to special_cache
    pub fn compute_special_probs(&mut self, preserve_tail: bool) -> Option<Vec<f64>> {
        self.clean_special_state();
        if self.special_cache.contains_key(&self.special_state) && !preserve_tail {
            return None;
        }

        let prep_output = &self.prep_output;

        let m = self
            .upgrade_arr
            .iter()
            .take(self.special_invalid_index.unwrap())
            .count();
        if m == 0 {
            let mut out = vec![0.0; self.special_state.len()];
            if !out.is_empty() {
                out[0] = 1.0;
            }

            if preserve_tail {
                return Some(out);
            } else {
                self.special_cache.insert(self.special_state.clone(), out);
                return None;
            }
        }
        // 1. GCD Optimization: Scale the world down
        let gcd = self.gcd_special() as usize; // Ensure this returns 1 if no upgrades
        let raw_budget: usize = prep_output.budgets[7] as usize;
        let budget = raw_budget / gcd;

        // active[b] = probability we are running with 'b' SCALED budget left
        let mut active: Vec<f64> = vec![0.0; budget + 1];
        active[budget] = 1.0;

        // result[k] will store the cumulative probability of passing stage k
        let mut result = vec![0.0; m + 1];

        // We reuse this vector to avoid allocation in the loop
        let mut next_active = vec![0.0; budget + 1];
        // Cache for powers of (1-p). Size is budget + 1 to cover max possible attempts.
        let mut fail_probs = vec![0.0; budget + 1];

        let mut highest_upgrade_index_seen: Vec<i64> = vec![-1; 6];
        let mut invalid_index: usize = m + 1;
        for (attempt_index, u_index) in self.special_state.iter().enumerate() {
            let upgrade = &self.upgrade_arr[*u_index];

            // my_dbg!(upgrade.upgrade_index, upgrade.is_weapon, upgrade.piece_type);
            if !upgrade.is_normal_honing
                || upgrade.succeeded
                || highest_upgrade_index_seen[upgrade.piece_type] > upgrade.upgrade_index as i64
            {
                invalid_index = attempt_index + 1;
                break;
            }

            highest_upgrade_index_seen[upgrade.piece_type] = upgrade.upgrade_index as i64;

            let p = if upgrade.succeeded {
                1.0
            } else {
                upgrade.base_chance
            };
            let one_minus_p = 1.0 - p;

            // Scale cost
            let cost = if upgrade.succeeded {
                0
            } else {
                (upgrade.special_cost as usize) / gcd
            };

            // If cost is higher than total budget, we can't possibly succeed.
            // (Probability mass stays in 'active' and doesn't move to 'result')
            if cost > budget {
                active.fill(0.0); // No more moves possible
                break;
            }

            // 2. Precompute Failure Probabilities
            // fail_probs[k] = (1-p)^k
            fail_probs[0] = 1.0;
            for t in 1..=budget {
                fail_probs[t] = fail_probs[t - 1] * one_minus_p; //TODO cache this 
            }

            // 3. Calculate "Success" (passing this stage)
            // We iterate to calculate how much mass moves to the next STAGE (result),
            // but we don't move the mass in the 'active' vector yet.
            for b in 0..=budget {
                let mass = active[b];
                if mass == 0.0 {
                    continue;
                }

                if b < cost {
                    // Cannot afford even one attempt.
                    // Mass stays here (implicit failure to proceed).
                    continue;
                }

                // fail_all = (1-p) ^ max_attempts
                // We read from our cache instead of calling powi
                let fail_all = if cost > 0 {
                    let max_attempts = b / cost;
                    fail_probs[max_attempts]
                } else {
                    0.0
                };

                // Add the mass that successfully moved to the next stage
                result[attempt_index + 1] += mass * (1.0 - fail_all);
            }

            // 4. Linear Recurrence for Next Budget State
            // We calculate where the mass lands for the NEXT upgrade iteration.
            // Recurrence: Next[b] = p*Active[b+C] + (1-p)*Next[b+C]
            // We must iterate downwards from (budget - cost) to avoid reading dirty data,
            // or simply strictly follow the dependency chain.
            // Since Next[b] depends on Next[b+C], we go High -> Low.

            // Clear next_active (or just overwrite if we are careful, but fill(0) is safer)
            next_active.fill(0.0);

            let start_idx = budget - cost;

            // We use a 'sliding window' logic based on modulo classes implicitly.
            // By iterating backwards, next_active[b+cost] is already computed.
            for b in (0..=start_idx).rev() {
                let source_idx = b + cost;

                // The probability flowing into 'b' comes from 'source_idx' trying once (p)
                // OR flowing through 'source_idx' from even higher budgets (1-p)
                let flow_from_above = next_active[source_idx];
                let source_mass = active[source_idx];

                // Note: The recurrence formula is technically:
                // next[b] = p * active[b+cost] + (1-p) * next[b+cost]
                next_active[b] = (p * source_mass) + (one_minus_p * flow_from_above);
            }

            // Swap vectors to prepare for next upgrade
            swap(&mut active, &mut next_active);
        }

        // cdf to pdf here
        result[0] = 1.0 - result[1];
        let mut actual_out = Vec::with_capacity(result.len());
        for (index, &val) in result.iter().enumerate() {
            if index >= invalid_index {
                break;
            }
            if index == result.len() - 1 || index == 0 {
                actual_out.push(val);
            } else {
                actual_out.push(val - result[index + 1]);
            }
        }

        let sum: f64 = actual_out.iter().sum();
        let length = actual_out.len();
        actual_out[length - 1] += 1.0 - sum; // the prob that we fail everything is not included, we add it to the last entry
        // my_dbg!(&actual_out);
        if !preserve_tail {
            eliminate_tail(0..actual_out.len(), &mut actual_out);
            eliminate_tail(actual_out.len()..0, &mut actual_out);
        }

        // my_dbg!(&actual_out);

        if preserve_tail {
            return Some(actual_out);
        } else {
            self.special_cache
                .insert(self.special_state.clone(), actual_out);
            return None;
        }
    }
}

/// Incredibly crude thingy to try and mitigate the effect of ignoring small chances
/// idk how mathematically sound this is but let's just hope its fine (at least it should be better than just plain ignoring them)
///
/// empirically this allows us to drop special tol to 1e-4
/// BUT that's evaluated on optimized states which often don't have small tails anyway
/// so all we know is that a tol of 1e-4 is good enough to make the optimizer find the same non-small tailed optima
///
/// the optimizer is often incentivized to NOT have trailing small tails because higher upgrades are often more efficient
/// so this is kinda completely useless (hence why special tol is left at 1e-7) but whatever
fn eliminate_tail(range: Range<usize>, actual_out: &mut Vec<f64>) {
    let mut cur_weighted_index: f64 = 0.0;
    let mut cur_sum: f64 = 0.0;
    for index in range {
        let this = actual_out[index];
        if this > SPECIAL_TOL {
            if cur_sum > 0.0 {
                let actual_index = (cur_weighted_index / cur_sum as f64).round() as usize;
                actual_out[actual_index] = cur_sum;
            }

            break;
        }
        cur_weighted_index += this * index as f64;
        cur_sum += this;

        if cur_sum > SPECIAL_TOL {
            let actual_index = (cur_weighted_index / cur_sum as f64).round() as usize;
            actual_out[actual_index] = cur_sum;
            break; // i mean we don't HAVE to break here but this guarantees that we're never fucking with more than SPECIAL_TOL of the distribution
        }
        actual_out[index] = 0.0;
    }
}
