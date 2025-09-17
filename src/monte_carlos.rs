use crate::parser::Upgrade;
use rand::Rng;
use rand::prelude::*;

use weighted_rand::builder::*;

use std::collections::{HashMap, VecDeque};

// vibe coding final boss
/// Compute the final PMF with tail aggregation.
///
/// - `budget` : total budget (i64)
/// - `costs`  : slice of costs C_k, must have the same length as `ps` (or at least
///              up to the maximum k you expect to reach). Each C_k must be > 0.
/// - `ps`     : slice of success probabilities p_k (0.0 < p_k < 1.0)
/// - `k_cap`  : cap for aggregation. Returned Vec has length `k_cap + 1`. Elements
///              0..k_cap-1 are exact probabilities Pr(K = k). Index `k_cap` stores
///              Pr(K >= k_cap) (all mass of outcomes with final K at least k_cap).
///
/// Returns `Ok(pmf)` on success, or `Err(String)` with an explanatory message on bad input.
pub fn final_pmf_tail_aggregate(budget: i64, upgrade_arr: &Vec<Upgrade>) -> Vec<f64> {
    // Basic validation

    // A small overflow-safe cumulative check to compute the budget-limited Kmax.
    // Not strictly necessary for the algorithm, but useful to reason about
    // the problem. We keep it to match the theory and for potential logging.
    let mut cum: i64 = 0;
    let mut kmax_budget: usize = 0;
    for upgrade in upgrade_arr {
        if cum + upgrade.special_cost <= budget {
            kmax_budget += 1usize;
            cum += upgrade.special_cost;
        } else {
            break;
        }
    }
    let k_cap: usize = upgrade_arr
        .iter()
        .filter(|x| x.is_normal_honing)
        .count()
        .min(kmax_budget);
    // State probability map: key is (k, spent) -> prob mass (f64).
    // We only store states that have nonzero probability. Use HashMap for sparsity.
    let mut state_prob: HashMap<(usize, i64), f64> = HashMap::new();
    let mut queue: VecDeque<(usize, i64)> = VecDeque::new();

    // Start at (0,0) with probability 1
    state_prob.insert((0usize, 0i64), 1.0f64);
    queue.push_back((0usize, 0i64));

    // final_pmf: indices 0..k_cap-1 are exact Pr(K=k). Index k_cap aggregates >= k_cap.
    let mut final_pmf: Vec<f64> = vec![0.0f64; k_cap + 1];

    while let Some((k, spent)) = queue.pop_front() {
        // Safely take the probability mass for this state and mark it processed.
        // We use get_mut so we can set it to 0 (mark) without removing the key.
        let prob: f64 = match state_prob.get_mut(&(k, spent)) {
            Some(v) => {
                let val: f64 = *v;
                // Mark as processed. Because transitions always increase `spent` (costs>0),
                // no future contribution to the exact same (k,spent) will appear after
                // we've seen *all* predecessors; so setting to 0 here is safe.
                *v = 0.0f64;
                val
            }
            None => {
                // If there is no entry, there is nothing to do.
                continue;
            }
        };

        if prob == 0.0 {
            continue;
        }

        // If we've already reached or exceeded the cap k_cap, aggregate into the tail
        // bucket and do not propagate further. This computes Pr(K >= k_cap) exactly
        // because once you reach >= k_cap successes the event "final K >= k_cap" is
        // guaranteed regardless of future attempts.
        if k >= k_cap {
            final_pmf[k_cap] += prob;
            continue;
        }

        // If there is no cost/probability information for this k, treat it as a terminal
        // state: we cannot make the next attempt (we require C_k and p_k to exist).
        if k >= upgrade_arr.len() {
            final_pmf[k.min(k_cap)] += prob; // k < k_cap here, but use min for safety
            continue;
        }

        let c_k: i64 = upgrade_arr[k].special_cost;
        // If we cannot afford the next attempt, terminal at current k
        if spent + c_k > budget {
            final_pmf[k] += prob;
            continue;
        }

        let next_spent: i64 = spent + c_k;
        let p_k: f64 = upgrade_arr[k].base_chance as f64;

        // Success transition -> (k+1, next_spent)
        let success_mass: f64 = prob * p_k;
        if success_mass > 0.0 {
            let key_succ: (usize, i64) = (k + 1, next_spent);
            // either update existing entry or insert+enqueue if new
            let was_new = match state_prob.get_mut(&key_succ) {
                Some(v) => {
                    let was_zero: bool = *v == 0.0;
                    *v += success_mass;
                    was_zero
                }
                None => {
                    state_prob.insert(key_succ, success_mass);
                    true
                }
            };
            if was_new {
                queue.push_back(key_succ);
            }
        }

        // Failure transition -> (k, next_spent)
        let fail_mass = prob * (1.0 - p_k);
        if fail_mass > 0.0 {
            let key_fail = (k, next_spent);
            let was_new = match state_prob.get_mut(&key_fail) {
                Some(v) => {
                    let was_zero = *v == 0.0;
                    *v += fail_mass;
                    was_zero
                }
                None => {
                    state_prob.insert(key_fail, fail_mass);
                    true
                }
            };
            if was_new {
                queue.push_back(key_fail);
            }
        }
    }

    // Sanity: probabilities should sum approximately to 1. Return PMF.
    final_pmf
}

fn calc_failure_lim(avail_special: i64, cost: i64) -> i64 {
    (avail_special as f64 / cost as f64).floor() as i64
}

fn construct_geometric_weights(max_taps: i64, base_chance: f64) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::with_capacity(max_taps as usize + 1);
    let mut cum_chance: f64 = 1.0;
    for _ in 0..(max_taps) {
        out.push((cum_chance * base_chance) as f32);
        cum_chance *= 1.0_f64 - base_chance;
    }
    out.push(cum_chance as f32); // chance to fail
    out
}

fn tap_map_generator(count_limit: usize, prob_dist: &Vec<f64>) -> Vec<usize> {
    let cum_weights: Vec<f64> = prob_dist
        .iter()
        .enumerate()
        .scan(0.0, |s, (i, &x)| {
            *s += x;
            Some(if i + 1 == prob_dist.len() { 1.0 } else { *s })
        })
        .collect();

    let mut tap_map: Vec<usize> = Vec::with_capacity(count_limit as usize);
    let mut cur_samples: i64 = 0;
    let mut temp_samples: f64;
    let mut j: usize = 0;
    let mut rng: ThreadRng = thread_rng();
    for i in 0..cum_weights.len() {
        temp_samples = (cur_samples as f64).max(cum_weights[i] * (count_limit as f64)); // not using round juice here because i need to keep track of the float(round juice was written for monte carlos later)
        if temp_samples - temp_samples.floor() as f64 > rng.gen_range(0.0..1.0) {
            cur_samples = temp_samples.floor() as i64 + 1;
        } else {
            cur_samples = temp_samples.floor() as i64;
        }
        for _ in j..(cur_samples as usize) {
            tap_map.push(i);
            j += 1;
        }
    }
    tap_map.shuffle(&mut thread_rng());
    return tap_map;
}

fn round_juice(this_juice_cost: f64, rng: &mut rand::prelude::ThreadRng) -> i64 {
    let juice_cost: i64;
    if this_juice_cost - this_juice_cost.floor() as f64 > rng.gen_range(0.0..1.0) {
        juice_cost = this_juice_cost.floor() as i64 + 1;
    } else {
        juice_cost = this_juice_cost.floor() as i64;
    }
    juice_cost
}
pub fn monte_carlos_data(
    data_size: usize,
    upgrade_arr: &Vec<Upgrade>,
    unlock_costs: &Vec<i64>,
    avail_special: i64,
    rigged: bool,
    use_true_rng: bool,
) -> Vec<Vec<i64>> {
    // dbg!(&upgrade_arr[0]);

    let mut cost_data: Vec<Vec<i64>> = vec![vec![0; 9]; data_size as usize];
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let mut juice_ind: usize;
    if rigged {
        let mut rolled_tap: usize;
        for (_, upgrade) in upgrade_arr.iter().enumerate() {
            for trial_num in 0..data_size as usize {
                rolled_tap = (upgrade.prob_dist_len as f64 * trial_num as f64 / data_size as f64)
                    .floor() as usize;
                for cost_type in 0..7 {
                    cost_data[trial_num][cost_type] +=
                        upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
                }
                if !upgrade.is_normal_honing {
                    if upgrade.is_weapon {
                        juice_ind = 7;
                    } else {
                        juice_ind = 8;
                    }
                    cost_data[trial_num][juice_ind] +=
                        round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
                }
            }
        }
    } else {
        let mut special_wa_table: weighted_rand::table::WalkerTable;
        let mut rolled_special_cost: i64;
        let mut special_budgets: Vec<i64> = vec![avail_special; data_size];
        let mut tap_wa_table: weighted_rand::table::WalkerTable;
        let mut rolled_tap: usize;
        let mut special_dist: Vec<f32>;
        let mut special_pass_arr: Vec<usize> = vec![0; data_size];
        let mut prob_dist: Vec<f32>;
        if use_true_rng {
            for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
                if upgrade.is_normal_honing {
                    special_dist = construct_geometric_weights(
                        calc_failure_lim(avail_special, upgrade.special_cost),
                        upgrade.base_chance,
                    );

                    special_wa_table = WalkerTableBuilder::new(&special_dist).build();
                    for trial_num in 0..data_size as usize {
                        if special_budgets[trial_num] <= 0 {
                            continue;
                        } else {
                            rolled_special_cost = (special_wa_table.next_rng(&mut rng) as i64 + 1)
                                * upgrade.special_cost;
                            special_budgets[trial_num] -= rolled_special_cost;
                            if special_budgets[trial_num] > 0 {
                                special_pass_arr[trial_num] += 1;
                                debug_assert!(special_pass_arr[trial_num] == upgrade_index + 1);
                            }
                        }
                    }
                }
            }
            for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
                prob_dist = upgrade.prob_dist.iter().map(|&n| n as f32).collect();
                // tap_map = tap_map_generator(data_size, &prob_dist);
                tap_wa_table = WalkerTableBuilder::new(&prob_dist).build();
                for trial_num in 0..data_size as usize {
                    if upgrade_index < special_pass_arr[trial_num] {
                        continue;
                    }
                    rolled_tap = tap_wa_table.next_rng(&mut rng);
                    for cost_type in 0..7 {
                        cost_data[trial_num][cost_type] +=
                            upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
                    }
                    if !upgrade.is_normal_honing {
                        if upgrade.is_weapon {
                            juice_ind = 7;
                        } else {
                            juice_ind = 8;
                        }
                        cost_data[trial_num][juice_ind] +=
                            round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
                    }
                }
            }
        } else {
            let mut tap_map: Vec<usize>;

            let special_map: Vec<usize> = tap_map_generator(
                data_size,
                &final_pmf_tail_aggregate(avail_special, &upgrade_arr),
            );
            for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
                tap_map = tap_map_generator(data_size, &upgrade.prob_dist);

                for trial_num in 0..data_size as usize {
                    if upgrade_index < special_map[trial_num] {
                        continue;
                    }
                    rolled_tap = tap_map[trial_num];
                    for cost_type in 0..7 {
                        cost_data[trial_num][cost_type] +=
                            upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
                    }
                    if !upgrade.is_normal_honing {
                        if upgrade.is_weapon {
                            juice_ind = 7;
                        } else {
                            juice_ind = 8;
                        }
                        cost_data[trial_num][juice_ind] +=
                            round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
                    }
                }
            }
        }
    }
    // apply unlock costs
    for i in 0..cost_data.len() {
        cost_data[i][3] += unlock_costs[0];
        cost_data[i][6] += unlock_costs[1];
    }
    cost_data
}
