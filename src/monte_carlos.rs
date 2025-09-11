use crate::constants::*;
use rand::Rng;
use rand::prelude::*;

use weighted_rand::builder::*;

fn tap_map_generator(count_limit: i64, prob_dist: &Vec<f32>) -> Vec<usize> {
    let cum_weights: Vec<f32> = prob_dist
        .iter()
        .enumerate()
        .scan(0.0, |s, (i, &x)| {
            *s += x;
            Some(if i + 1 == prob_dist.len() { 1.0 } else { *s })
        })
        .collect();
    let mut tap_map: Vec<usize> = vec![0 as usize; count_limit as usize];
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
            tap_map[j] = i;
            j += 1;
        }
    }
    tap_map.shuffle(&mut thread_rng());
    return tap_map;
}

pub fn monte_carlos_data(
    cost_size: i64,
    prob_dist_arr: &Vec<Vec<f32>>,
    hone_costs: &Vec<Vec<i64>>,
    adv_hone_chances: &Vec<Vec<f32>>,
    adv_hone_costs: &Vec<Vec<Vec<i64>>>,
    unlock_costs: &Vec<i64>,
    use_true_rng: bool,
) -> Vec<Vec<i64>> {
    //

    let mut cost_data: Vec<Vec<i64>> = vec![vec![0; 9]; cost_size as usize];
    if use_true_rng {
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        let mut builder: WalkerTableBuilder;
        let mut wa_table: weighted_rand::table::WalkerTable;
        let mut rolled_tap: usize;
        for piece in 0..prob_dist_arr.len() {
            builder = WalkerTableBuilder::new(&prob_dist_arr[piece]);
            wa_table = builder.build();

            // let tap_map: Vec<usize> = tap_map_generator(cost_size, &prob_dist_arr[piece]);
            for trial_num in 0..cost_size as usize {
                rolled_tap = wa_table.next_rng(&mut rng);
                for cost_type in 0..NORMAL_HONE_ARMOR_COST.len() {
                    cost_data[trial_num][cost_type] +=
                        hone_costs[cost_type][piece] * (rolled_tap as i64 + 1);
                }
            }
        }
        for piece in 0..adv_hone_chances.len() {
            builder = WalkerTableBuilder::new(&adv_hone_chances[piece]);
            wa_table = builder.build();
            for trial_num in 0..cost_size as usize {
                rolled_tap = wa_table.next_rng(&mut rng);
                for cost_type in 0..adv_hone_costs[0].len() {
                    cost_data[trial_num][cost_type] += adv_hone_costs[piece][cost_type][rolled_tap]
                }
            }
        }
    } else {
        let mut rolled_tap: usize;
        let mut tap_map: Vec<usize>;
        for piece in 0..prob_dist_arr.len() {
            tap_map = tap_map_generator(cost_size, &prob_dist_arr[piece]);
            for trial_num in 0..cost_size as usize {
                rolled_tap = tap_map[trial_num];
                for cost_type in 0..NORMAL_HONE_ARMOR_COST.len() {
                    cost_data[trial_num][cost_type] +=
                        hone_costs[cost_type][piece] * (rolled_tap as i64 + 1);
                }
            }
        }
        for piece in 0..adv_hone_chances.len() {
            tap_map = tap_map_generator(cost_size, &adv_hone_chances[piece]);
            for trial_num in 0..cost_size as usize {
                rolled_tap = tap_map[trial_num];
                for cost_type in 0..adv_hone_costs[0].len() {
                    cost_data[trial_num][cost_type] += adv_hone_costs[piece][cost_type][rolled_tap];
                }
            }
        }
    }

    // apply unlock adjustments
    for i in 0..cost_data.len() {
        cost_data[i as usize][3] += unlock_costs[0];
        cost_data[i as usize][6] += unlock_costs[1];
    }
    return cost_data;
}
