use std::f64;
use std::mem::swap;

// use crate::constants::FLOAT_TOL;

use crate::state_bundle::StateBundle;
use num::Integer;

impl StateBundle {
    pub fn special_probs(&self) -> &Vec<f64> {
        // this isn't like automatically with the special because it takes in a &mut and messes with loops
        return &self.special_cache[&self.special_state];
    }
    fn gcd_special(&self) -> i64 {
        let mut out: i64 = 1;
        for (index, upgrade) in self.upgrade_arr.iter().enumerate() {
            if !upgrade.is_normal_honing {
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
    pub fn compute_special_probs(&mut self) {
        self.clean_special_state();
        if self.special_cache.contains_key(&self.special_state) {
            return;
        }

        let prep_output = &self.prep_output;

        let m = self
            .upgrade_arr
            .iter()
            .filter(|x| x.is_normal_honing && !x.succeeded)
            .count();
        if m == 0 {
            self.special_cache
                .insert(self.special_state.clone(), vec![1.0]);
            return;
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

            // dbg!(upgrade.upgrade_index, upgrade.is_weapon, upgrade.piece_type);
            if !upgrade.is_normal_honing
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

                let max_attempts = b / cost;
                // fail_all = (1-p) ^ max_attempts
                // We read from our cache instead of calling powi
                let fail_all = fail_probs[max_attempts];

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

        // dbg!(&result, invalid_index);
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

        // Tolerance check
        let sum: f64 = actual_out.iter().sum();
        let length = actual_out.len();
        actual_out[length - 1] += 1.0 - sum;
        // assert!((sum - 1.0).abs() < FLOAT_TOL);

        self.special_cache
            .insert(self.special_state.clone(), actual_out);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::calculate_hash;
//     use crate::constants::RNG_SEED;

//     use crate::helpers::naive_count_to_ticks;
//     use crate::parser::PreparationOutput;

//     use crate::state_bundle::default_state_arr;
//     use crate::test_utils::*;

//     use std::collections::HashMap;
//     use std::time::Instant;
//     static DEBUG: bool = true;
//     // #[test]
//     // fn special_sa_test() {
//     //     let start = Instant::now();
//     //     let test_name = format!("special_sa_test");
//     //     let hone_counts: Vec<Vec<i64>> = vec![
//     //         (0..25)
//     //             .map(|x| if x == 24 || x == 23 { 1 } else { 0 })
//     //             .collect(),
//     //         (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
//     //     ];

//     //     let alr_fail_arr: Vec<Vec<usize>> = vec![(0..25).map(|x| 0).collect(); 6];

//     //     let state_grid: Vec<Vec<usize>> = vec![default_state_arr(upgrade_arr)];
//     //     // let hone_counts: Vec<Vec<i64>> =
//     //     //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
//     //     let adv_counts: Vec<Vec<i64>> =
//     //         vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

//     //     let adv_hone_strategy: &str = "x2 balls";
//     //     let express_event: bool = true;
//     //     let budget = vec![0, 0, 0, 0, 0, 3333333, 0, 6767];
//     //     let juice_books_owned: Vec<(i64, i64)> = vec![(0, 0), (0, 0), (0, 0), (0, 0)];
//     //     let juice_prices: Vec<(f64, f64)> = vec![
//     //         (123.0, 123.0),
//     //         (123.0, 123.0),
//     //         (123.0, 123.0),
//     //         (123.0, 123.0),
//     //     ];
//     //     let prices = DEFAULT_GOLD_VALUES;
//     //     let hash: String = calculate_hash!(
//     //         &hone_counts,
//     //         &adv_counts,
//     //         adv_hone_strategy,
//     //         express_event,
//     //         &budget,
//     //         &prices,
//     //         RNG_SEED,
//     //         PROB_MODE
//     //     );

//     //     let (prep_output, upgrade_arr) = PreparationOutput::initialize(
//     //         &naive_count_to_ticks(&hone_counts),
//     //         &budget,
//     //         &naive_count_to_ticks(&adv_counts),
//     //         express_event,
//     //         &prices,
//     //         &adv_hone_strategy,
//     //         &juice_books_owned,
//     //         &juice_prices,
//     //         &prices,
//     //         &juice_prices,
//     //         &alr_fail_arr,
//     //         &state_grid,
//     //     );

//     //     // let mut starting_special: Vec<usize> = Vec::with_capacity(upgrade_arr.len() * 2);
//     //     // for (index, _upgrade) in upgrade_arr.iter().enumerate() {
//     //     //     starting_special.push(index); //, (1.0 / upgrade.base_chance).round() as usize));
//     //     // }
//     //     let starting_special: Vec<usize> = vec![0, 1, 2];

//     //     let mut state_bundle: StateBundle = StateBundle {
//     //         // state_index: vec![],
//     //         metric: -1.0,
//     //         special_state: starting_special,
//     //         prep_output,
//     //         special_cache: HashMap::new(),
//     //         upgrade_arr,

//     //         metric_type: 0,
//     //     };

//     //     // init_dist(&mut state_bundle, &mut prep_output);

//     //     // dbg!(&state_bundle, &upgrade_arr);
//     //     state_bundle.update_dist();
//     //     state_bundle.compute_special_probs();
//     //     let result: Vec<f64> = state_bundle.special_probs().clone();

//     //     if DEBUG {
//     //         dbg!(&result);
//     //     }
//     //     if let Some(_cached_result) = read_cached_data::<Vec<f64>>(test_name.as_str(), &hash) {
//     //     } else {
//     //         write_cached_data(test_name.as_str(), &hash, &result);
//     //     }
//     //     if DEBUG {
//     //         dbg!(start.elapsed());
//     //     }
//     // }
// }
