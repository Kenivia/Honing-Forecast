use crate::constants::*;
use weighted_rand::builder::*;

pub fn monte_carlos_data(
    cost_size: i64,
    prob_dist_arr: &Vec<Vec<f32>>,
    hone_costs: &Vec<Vec<i64>>,
    adv_hone_chances: &Vec<Vec<f32>>,
    adv_hone_costs: &Vec<Vec<Vec<i64>>>,
    unlock_costs: &Vec<i64>,
) -> Vec<Vec<i64>> {
    //

    let mut cost_data: Vec<Vec<i64>> = vec![vec![0; 9]; cost_size as usize];

    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    for piece in 0..prob_dist_arr.len() {
        let builder: WalkerTableBuilder = WalkerTableBuilder::new(&prob_dist_arr[piece]);
        let wa_table: weighted_rand::table::WalkerTable = builder.build();
        for trial_num in 0..cost_size {
            for cost_type in 0..NORMAL_HONE_ARMOR_COST.len() {
                cost_data[trial_num as usize][cost_type] +=
                    hone_costs[cost_type][piece] * (wa_table.next_rng(&mut rng) as i64 + 1);
            }
        }
    }

    for piece in 0..adv_hone_chances.len() {
        let builder: WalkerTableBuilder = WalkerTableBuilder::new(&adv_hone_chances[piece]);
        let wa_table: weighted_rand::table::WalkerTable = builder.build();
        for trial_num in 0..cost_size {
            for cost_type in 0..adv_hone_costs.len() {
                cost_data[trial_num as usize][cost_type] +=
                    adv_hone_costs[piece][cost_type][wa_table.next_rng(&mut rng)]
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
