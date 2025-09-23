use crate::parser::Upgrade;
use rand::Rng;
use rand::prelude::*;

use weighted_rand::builder::*;

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

fn tap_map_generator(
    count_limit: usize,
    prob_dist: &Vec<f64>,
    mut rng: &mut ThreadRng,
) -> Vec<usize> {
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
    for i in 0..cum_weights.len() {
        temp_samples = (cur_samples as f64).max(cum_weights[i] * (count_limit as f64)); // not using round juice here because i need to keep track of the float(round juice was written for monte carlo later)
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
    tap_map.shuffle(&mut rng);
    return tap_map;
}

fn round_juice(this_juice_cost: f64, rng: &mut ThreadRng) -> i64 {
    let juice_cost: i64;
    if this_juice_cost - this_juice_cost.floor() as f64 > rng.gen_range(0.0..1.0) {
        juice_cost = this_juice_cost.floor() as i64 + 1;
    } else {
        juice_cost = this_juice_cost.floor() as i64;
    }
    juice_cost
}
pub fn monte_carlo_data(
    data_size: usize,
    upgrade_arr: &Vec<Upgrade>,
    unlock_costs: &Vec<i64>,
    avail_special: i64,
    rigged: bool,
    use_true_rng: bool,
) -> Vec<Vec<i64>> {
    let mut cost_data: Vec<Vec<i64>> = vec![vec![0; 9]; data_size as usize];
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let mut juice_ind: usize;
    let mut rolled_tap: usize;
    if rigged {
        for (_, upgrade) in upgrade_arr.iter().enumerate() {
            for trial_num in 0..data_size as usize {
                rolled_tap = ((upgrade.prob_dist_len - 1) as f64 * (trial_num + 1) as f64
                    / data_size as f64)
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
        let mut special_dist: Vec<f32>;
        let mut special_pass_arr: Vec<usize> = vec![0; data_size];
        let mut prob_dist: Vec<f32>;
        // if use_true_rng || avail_special as f64 / 12 as f64 > 100.0_f64 {
        if avail_special > 0 {
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
        }
        // This just doesn't seem worth it
        // }
        // else {
        //     special_pass_arr = tap_map_generator(
        //         data_size,
        //         &final_pmf_tail_aggregate(avail_special, &upgrade_arr),
        //     );
        // }
        if use_true_rng {
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
            // This is called latin hypercube sampling apparently
            let mut tap_map: Vec<usize>;
            for (upgrade_index, upgrade) in upgrade_arr.iter().enumerate() {
                tap_map = tap_map_generator(data_size, &upgrade.prob_dist, &mut rng);
                for trial_num in 0..data_size as usize {
                    if upgrade_index < special_pass_arr[trial_num] {
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
