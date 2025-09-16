use crate::parser::Upgrade;
use rand::Rng;
use rand::prelude::*;

use weighted_rand::builder::*;

fn calc_failure_lim(avail_special: i64, cost: i64) -> i64 {
    (avail_special as f64 / cost as f64).floor() as i64 + 1 // just using 12 because its the lowest special leap cost possible
}

fn construct_geometric_weights(max_taps: i64, base_chance: f32) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::with_capacity((max_taps + 1) as usize);
    let mut cum_chance: f32 = 1.0;
    for _ in 0..(max_taps) {
        out.push(cum_chance * base_chance);
        cum_chance *= 1.0 - base_chance;
    }
    out.push(cum_chance); // chance to fail
    out
}

fn tap_map_generator(count_limit: usize, prob_dist: &Vec<f32>) -> Vec<usize> {
    let cum_weights: Vec<f32> = prob_dist
        .iter()
        .enumerate()
        .scan(0.0, |s, (i, &x)| {
            *s += x;
            Some(if i + 1 == prob_dist.len() { 1.0 } else { *s })
        })
        .collect();

    let mut tap_map: Vec<usize> = Vec::with_capacity(count_limit as usize);
    let mut cur_samples: i64 = 0;
    let mut temp_samples: f32;
    let mut j: usize = 0;
    let mut rng: ThreadRng = thread_rng();
    for i in 0..cum_weights.len() {
        temp_samples = (cur_samples as f32).max(cum_weights[i] * (count_limit as f32));
        if temp_samples - temp_samples.floor() as f32 > rng.gen_range(0.0..1.0) {
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

fn round_juice(this_juice_cost: f32, rng: &mut rand::prelude::ThreadRng) -> i64 {
    //  this_juice_cost = upgrade.juice_cost[rolled_tap];
    let juice_cost: i64;
    if this_juice_cost - this_juice_cost.floor() as f32 > rng.gen_range(0.0..1.0) {
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

    let mut cost_data: Vec<Vec<i64>> = vec![vec![0; 8]; data_size as usize];
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    if rigged {
        let mut rolled_tap: usize;
        for (_, upgrade) in upgrade_arr.iter().enumerate() {
            for trial_num in 0..data_size as usize {
                rolled_tap = (upgrade.prob_dist_len as f64 * trial_num as f64 / data_size as f64)
                    .floor() as usize;
                for cost_type in 0..7 {
                    cost_data[trial_num][cost_type] +=
                        upgrade.costs[cost_type] * (rolled_tap as i64 + 1);
                }
                if !upgrade.is_normal_honing {
                    cost_data[trial_num][7] +=
                        round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
                }
            }
        }
    } else {
        let mut special_wa_table: weighted_rand::table::WalkerTable;
        let mut rolled_special_cost: i64;
        let mut special_budgets: Vec<i64> = vec![avail_special; data_size];
        if use_true_rng {
            let mut builder: WalkerTableBuilder;
            let mut tap_wa_table: weighted_rand::table::WalkerTable;
            let mut rolled_tap: usize;
            for (_, upgrade) in upgrade_arr.iter().enumerate() {
                builder = WalkerTableBuilder::new(&upgrade.prob_dist);
                tap_wa_table = builder.build();

                if upgrade.is_normal_honing {
                    let special_dist: Vec<f32> = construct_geometric_weights(
                        calc_failure_lim(avail_special, upgrade.special_cost),
                        upgrade.base_chance,
                    );
                    special_wa_table = WalkerTableBuilder::new(&special_dist).build();

                    for trial_num in 0..data_size as usize {
                        if special_budgets[trial_num] > 0 {
                            rolled_special_cost = (special_wa_table.next_rng(&mut rng) as i64 + 1)
                                * upgrade.special_cost;
                            special_budgets[trial_num] -= rolled_special_cost;
                            if special_budgets[trial_num] > 0 {
                                continue;
                            }
                        }
                        rolled_tap = tap_wa_table.next_rng(&mut rng);
                        for cost_type in 0..7 {
                            cost_data[trial_num][cost_type] +=
                                upgrade.costs[cost_type] * (rolled_tap as i64 + 1);
                        }
                    }
                } else {
                    for trial_num in 0..data_size as usize {
                        rolled_tap = tap_wa_table.next_rng(&mut rng);
                        for cost_type in 0..7 {
                            cost_data[trial_num][cost_type] += upgrade.costs[cost_type]
                                * (rolled_tap as i64 + upgrade.adv_cost_start);
                        }
                        if !upgrade.is_normal_honing {
                            cost_data[trial_num][7] +=
                                round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
                        }
                    }
                }
            }
        } else {
            // not working rn

            // let mut rolled_tap: usize;
            // let mut tap_map: Vec<usize>;
            // for (_, upgrade) in upgrade_arr.iter().enumerate() {
            //     tap_map = tap_map_generator(data_size, &upgrade.prob_dist);
            //     for trial_num in 0..data_size as usize {
            //         rolled_tap = tap_map[trial_num];
            //         for cost_type in 0..7 {
            //             cost_data[trial_num][cost_type] +=
            //                 upgrade.costs[cost_type] * (rolled_tap as i64 + 1);
            //         }
            //         if !upgrade.is_normal_honing {
            //             cost_data[trial_num][7] +=
            //                 round_juice(upgrade.adv_juice_cost[rolled_tap], &mut rng);
            //         }
            //     }
            // }
        }
    }
    // apply unlock costs
    for i in 0..cost_data.len() {
        cost_data[i][3] += unlock_costs[0];
        cost_data[i][6] += unlock_costs[1];
    }
    cost_data
}
