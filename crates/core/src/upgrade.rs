use crate::constants::NORMAL_JUICE_COST;

use crate::normal_honing_utils::{generate_first_deltas, probability_distribution};
use serde::{Deserialize, Serialize};
// the parser function turns a selection of upgrades into an array of Upgrade objects
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upgrade {
    pub is_normal_honing: bool,
    pub prob_dist: Vec<f64>,
    pub original_prob_dist: Vec<f64>,
    pub base_chance: f64,
    pub costs: [i64; 7],
    pub one_juice_cost: i64,
    pub adv_juice_cost: Vec<f64>, // array corresponding to column 2 in the ADV_DATA
    pub special_cost: i64,
    pub juice_values: Vec<f64>, // juice values
    pub prob_dist_len: usize,
    pub is_weapon: bool,
    pub artisan_rate: f64,
    pub tap_offset: i64,
    pub upgrade_index: usize,
    pub special_value: f64,
    pub full_juice_len: usize,
    pub support_lengths: Vec<usize>, //Vec<Vec<Vec<[i64; 10]>>>, // cost_data_arr[juice_count][special_count] = cost_data for that decision
    pub eqv_gold_per_tap: f64,
    pub juice_avail: bool,
    pub books_avail: i64,
    // pub juice_arr: Vec<f64>,
    pub log_prob_dist: Vec<f64>,
    pub state: Vec<(bool, usize)>, // state for this upgrade - (juice_used, book_index) per tap
    pub cost_dist: Vec<Vec<f64>>,
    pub weap_juice_costs: Vec<Vec<f64>>,
    pub armor_juice_costs: Vec<Vec<f64>>,
    pub combined_gold_costs: Vec<f64>,
}

impl Upgrade {
    pub fn new_normal(
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        special_cost: i64,
        is_weapon: bool,
        artisan_rate: f64,
        upgrade_index: usize,
    ) -> Self {
        let prob_dist_len: usize = prob_dist.len();
        let base_chance: f64 = prob_dist[1];
        let full_juice_len: usize = probability_distribution(
            base_chance,
            artisan_rate,
            &generate_first_deltas(
                base_chance,
                prob_dist_len, // this is excessive but its fine
                prob_dist_len,
            ),
            0.0,
        )
        .len();
        Self {
            is_normal_honing: true,
            prob_dist: prob_dist.clone(),
            original_prob_dist: prob_dist.clone(),
            base_chance,
            costs,
            one_juice_cost: NORMAL_JUICE_COST[upgrade_index],
            adv_juice_cost: vec![],
            special_cost,
            juice_values: vec![],
            prob_dist_len,
            is_weapon,
            artisan_rate,
            tap_offset: 0,
            upgrade_index,
            special_value: -1.0_f64,
            full_juice_len,
            support_lengths: vec![], // to be filled
            // log_prob_dist: vec![], // will change with each arrangement, maybe use a hashmap later
            eqv_gold_per_tap: -1.0_f64, // dummy value
            // gold_cost_record: vec![],
            // juice_arr: vec![],
            // eqv_gold_per_juice: -1.0_f64,
            juice_avail: upgrade_index > 2, // will overwrite this in prep initialization anyway
            books_avail: -1,                // will overwrite in prep
            log_prob_dist: vec![],
            state: vec![(false, 0); prob_dist.len()], // initialize state with default values
            cost_dist: vec![],
            weap_juice_costs: vec![],
            armor_juice_costs: vec![],
            combined_gold_costs: vec![],
        }
    }

    pub fn new_adv(
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        one_juice_cost: i64,
        adv_juice_cost: Vec<f64>,
        is_weapon: bool,
        adv_cost_start: i64,
        upgrade_index: usize,
    ) -> Self {
        let prob_dist_len: usize = prob_dist.len();
        assert!(prob_dist_len == adv_juice_cost.len());

        Self {
            is_normal_honing: false,
            prob_dist: prob_dist.clone(),
            original_prob_dist: prob_dist.clone(),
            base_chance: 0.0,
            costs,
            one_juice_cost,
            adv_juice_cost,
            special_cost: 0,
            juice_values: vec![],
            prob_dist_len,
            is_weapon,
            artisan_rate: 0.0,
            tap_offset: adv_cost_start,
            upgrade_index,
            special_value: -1.0_f64,
            full_juice_len: 1, // need to sort this out
            support_lengths: vec![],
            // log_prob_dist: vec![], // will change with each arrangement, maybe use a hashmap later
            eqv_gold_per_tap: -1.0_f64, // dummy value
            // gold_cost_record: vec![],
            // juice_arr: vec![],
            // eqv_gold_per_juice: -1.0_f64,
            // failure_raw_delta: -1,
            // failure_delta_order: -1,
            juice_avail: upgrade_index > 2, // will overwrite this in prep initialization anyway
            books_avail: -1,                // will overwrite in prep
            log_prob_dist: vec![],
            state: vec![(false, 0); prob_dist.len()], // initialize state with default values
            cost_dist: vec![],
            weap_juice_costs: vec![],
            armor_juice_costs: vec![],
            combined_gold_costs: vec![],
        }
    }
}
