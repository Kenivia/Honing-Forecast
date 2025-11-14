use crate::constants::{
    ADV_DATA_10_20, ADV_DATA_10_20_JUICE, ADV_DATA_30_40, ADV_DATA_30_40_JUICE, ADV_HONE_COST,
    NORMAL_HONE_CHANCES, NORMAL_JUICE_COST, SPECIAL_LEAPS_COST, get_event_modified_armor_costs,
    get_event_modified_artisan, get_event_modified_weapon_costs,
};
use crate::helpers::{
    average_juice_cost, calc_unlock, compress_runs, generate_first_deltas, sort_by_indices,
};
// use crate::monte_carlo::monte_carlo_one;
use crate::value_estimation::{
    est_juice_value, est_special_honing_value, extract_special_strings, juice_to_array,
};
// use rand::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug)]
pub struct PreparationOutputs {
    pub upgrade_arr: Vec<Upgrade>,
    pub unlock_costs: Vec<i64>,
    pub budgets: Vec<i64>,
    pub juice_strings_armor: Vec<String>,
    pub juice_strings_weapon: Vec<String>,
    pub mats_value: Vec<f64>,
    pub special_strings: Vec<String>,
}

pub fn preparation(
    hone_counts: &[Vec<i64>],
    input_budgets: &[i64],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    user_mats_value: &[f64],
    adv_hone_strategy: &str,
) -> PreparationOutputs {
    let mats_value: Vec<f64> = user_mats_value.to_vec();
    let unlock_costs: Vec<i64> = calc_unlock(hone_counts, adv_counts, express_event);

    let mut upgrade_arr: Vec<Upgrade> = parser(
        hone_counts,
        adv_counts,
        &adv_hone_strategy.to_string(),
        express_event,
    );
    let mut budgets: Vec<i64> = input_budgets.to_vec();

    // Add average juice costs to budgets for all upgrades
    if adv_hone_strategy == "Juice on grace" {
        let (avg_red_juice, avg_blue_juice): (i64, i64) = average_juice_cost(&upgrade_arr);
        budgets[7] -= avg_red_juice;
        budgets[8] -= avg_blue_juice;
    }
    // for upgrade in upgrade_arr.iter_mut() {
    //     let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
    //     for i in 0..upgrade.full_juice_len {
    //         upgrade.cost_data_arr.push(vec![]); // this will contain different free taps eventually i think
    //         upgrade.prob_dist = probability_distribution(
    //             upgrade.base_chance,
    //             upgrade.artisan_rate,
    //             &generate_first_deltas(
    //                 upgrade.base_chance,
    //                 upgrade.prob_dist_len, // this is excessive but its fine
    //                 i,
    //             ),
    //         );
    //         let this_data: Vec<[i64; 10]> = monte_carlo_one(100000, upgrade, 0, i as i64, &mut rng);
    //         upgrade.cost_data_arr[i].push(this_data);
    //     }
    // }

    //XXXXXXXXXXXXXXXXXXXXXXXXXXXXX defunct code here to keep rust analyzer happy
    est_juice_value(&mut upgrade_arr, &mats_value);
    let (juice_strings_armor, juice_strings_weapon): (Vec<String>, Vec<String>) =
        juice_to_array(&mut upgrade_arr, budgets[8], budgets[7]);
    let value_per_special_leap: Vec<f64> = est_special_honing_value(&mut upgrade_arr, &mats_value);
    let mut special_indices: Vec<usize> = (0..value_per_special_leap.len()).collect();
    special_indices
        .sort_by(|&a, &b| value_per_special_leap[b].total_cmp(&value_per_special_leap[a]));
    sort_by_indices(&mut upgrade_arr, special_indices.clone());
    let special_strings: Vec<String> =
        compress_runs(extract_special_strings(&upgrade_arr), true, vec![]);
    //XXXXXXXXXXXXXXXXXXXXXXXXXXXXX defunct code here to keep rust analyzer happy

    PreparationOutputs {
        upgrade_arr,
        unlock_costs,
        budgets,
        juice_strings_armor,
        juice_strings_weapon,
        mats_value,
        special_strings,
    }
}

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
    pub upgrade_plus_num: usize,
    pub special_value: f64,
    pub full_juice_len: usize,
    pub cost_data_arr: Vec<Vec<Vec<[i64; 10]>>>, // cost_data_arr[juice_count][special_count] = cost_data for that decision
}

impl Upgrade {
    pub fn new_normal(
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        special_cost: i64,
        is_weapon: bool,
        artisan_rate: f64,
        upgrade_plus_num: usize,
    ) -> Self {
        let prob_dist_len: usize = prob_dist.len();
        let base_chance: f64 = prob_dist.first().copied().unwrap_or(0.0);
        let full_juice_len: usize = probability_distribution(
            base_chance,
            artisan_rate,
            &generate_first_deltas(
                base_chance,
                prob_dist_len, // this is excessive but its fine
                prob_dist_len,
            ),
        )
        .len();
        Self {
            is_normal_honing: true,
            prob_dist: prob_dist.clone(),
            original_prob_dist: prob_dist,
            base_chance,
            costs,
            one_juice_cost: NORMAL_JUICE_COST[upgrade_plus_num],
            adv_juice_cost: vec![],
            special_cost,
            juice_values: vec![],
            prob_dist_len,
            is_weapon,
            artisan_rate,
            tap_offset: 1,
            upgrade_plus_num,
            special_value: -1.0_f64,
            full_juice_len,
            cost_data_arr: vec![], // to be filled
        }
    }

    pub fn new_adv(
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        one_juice_cost: i64,
        adv_juice_cost: Vec<f64>,
        is_weapon: bool,
        adv_cost_start: i64,
        upgrade_plus_num: usize,
    ) -> Self {
        let prob_dist_len: usize = prob_dist.len();
        assert!(prob_dist_len == adv_juice_cost.len());

        Self {
            is_normal_honing: false,
            prob_dist: prob_dist.clone(),
            original_prob_dist: prob_dist,
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
            upgrade_plus_num,
            special_value: -1.0_f64,
            full_juice_len: 1, // need to sort this out
            cost_data_arr: vec![],
            // failure_raw_delta: -1,
            // failure_delta_order: -1,
        }
    }
}

// prob distribution of normal honing, adjusting for any juice usage
pub fn probability_distribution(base: f64, artisan_rate: f64, extra_arr: &[f64]) -> Vec<f64> {
    let mut raw_chances: Vec<f64> = Vec::new();
    let mut artisan: f64 = 0.0_f64;
    let mut count: usize = 0;

    loop {
        let min_count: f64 = std::cmp::min(count, 10) as f64;
        let mut current_chance: f64 =
            base + (min_count * base) * 0.1 + extra_arr.get(count).unwrap_or(&0.0);

        if artisan >= 1.0 {
            current_chance = 1.0;
            raw_chances.push(current_chance);
            break;
        }

        raw_chances.push(current_chance);
        count += 1;
        artisan += (46.51_f64 / 100.0) * current_chance * artisan_rate;
        if current_chance == 1.0 {
            break; // for upgrades that have 100% passrate immediately
        }
    }

    // convert raw per-try chances into per-tap probability distribution
    let mut chances = vec![0.0_f64; raw_chances.len()];
    let mut cum_chance = 1.0_f64;
    for (idx, &element) in raw_chances.iter().enumerate() {
        chances[idx] = cum_chance * element;
        cum_chance *= 1.0 - element;
    }
    chances
}

// Constructs vector of Upgrade objects according to what upgrades were selected and the appropriate juice applieid
pub fn parser(
    normal_counts: &[Vec<i64>],
    adv_counts: &[Vec<i64>],
    adv_hone_strategy: &String,

    express_event: bool,
) -> Vec<Upgrade> {
    let mut out: Vec<Upgrade> = Vec::new();
    let artisan_rate_arr: [f64; 25] = get_event_modified_artisan(express_event);
    for is_weapon in 0..normal_counts.len() {
        let cur_cost: [[i64; 25]; 7] = if is_weapon == 0 {
            get_event_modified_armor_costs(express_event)
        } else {
            get_event_modified_weapon_costs(express_event)
        };

        let mut current_counter: i64 = 0;
        let row_len: usize = normal_counts[is_weapon].len(); // 25
        let mut upgrade_plus_num: usize = 0;

        while upgrade_plus_num < row_len {
            let needed: i64 = normal_counts[is_weapon][upgrade_plus_num];
            if current_counter >= needed {
                upgrade_plus_num += 1;
                current_counter = 0;
                continue;
            }

            let special_cost: i64 = SPECIAL_LEAPS_COST[is_weapon][upgrade_plus_num];
            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_plus_num];
            out.push(Upgrade::new_normal(
                probability_distribution(
                    NORMAL_HONE_CHANCES[upgrade_plus_num],
                    event_artisan_rate,
                    &[],
                ),
                std::array::from_fn(|cost_type: usize| cur_cost[cost_type][upgrade_plus_num]),
                special_cost,
                is_weapon == 1,
                event_artisan_rate,
                upgrade_plus_num,
            ));
            current_counter += 1;
        }
    }

    // Advanced hone
    let mut this_juice_cost: Vec<f64>;
    let mut prob_dist: Vec<f64>;
    for is_weapon in 0..adv_counts.len() {
        let mut current_counter: i64 = 0;
        let row_len: usize = adv_counts[is_weapon].len();
        let mut upgrade_plus_num: usize = 0;
        while upgrade_plus_num < row_len {
            let needed: i64 = adv_counts[is_weapon][upgrade_plus_num];
            if current_counter >= needed {
                upgrade_plus_num += 1;
                current_counter = 0;
                continue;
            }

            // pick relevant_data based on strategy and level i (i <= 1 -> 10/20, else 30/40)
            let relevant_data: &'static [[i64; 3]] = if adv_hone_strategy == "Juice on grace" {
                if upgrade_plus_num <= 1 {
                    &ADV_DATA_10_20_JUICE
                } else {
                    &ADV_DATA_30_40_JUICE
                }
            } else if upgrade_plus_num <= 1 {
                &ADV_DATA_10_20
            } else {
                &ADV_DATA_30_40
            };

            let rows: usize = relevant_data.len();
            let sum_taps: i64 = relevant_data.iter().map(|row: &[i64; 3]| row[2]).sum(); // 2nd index is frequency
            let col_index: usize = 2 * upgrade_plus_num + (1 - is_weapon);

            prob_dist = Vec::with_capacity(rows);
            this_juice_cost = Vec::with_capacity(rows);

            let cost_val: i64 = ADV_HONE_COST[7][col_index];
            let sum_taps_f: f64 = if sum_taps == 0 { 1.0 } else { sum_taps as f64 };

            for row in relevant_data {
                let taps: i64 = row[2];
                prob_dist.push((taps as f64) / sum_taps_f);
                this_juice_cost.push(cost_val as f64 * row[1] as f64 / 1000.0_f64);
            }

            out.push(Upgrade::new_adv(
                prob_dist,
                std::array::from_fn(|cost_type: usize| ADV_HONE_COST[cost_type][col_index]),
                cost_val,
                this_juice_cost,
                is_weapon == 1,
                relevant_data[0][0],
                upgrade_plus_num,
            ));
            current_counter += 1;
        }
    }

    out
}

/// Parser that runs twice to get both the main strategy and the other strategy's probability distributions
/// Used by Gamba  when toggling on/off juice for a particular adv honing piece
pub fn parser_with_other_strategy(
    normal_counts: &[Vec<i64>],
    adv_counts: &[Vec<i64>],
    adv_hone_strategy: &String,

    express_event: bool,
) -> (Vec<Upgrade>, Vec<Vec<f64>>) {
    let main_upgrades: Vec<Upgrade> =
        parser(normal_counts, adv_counts, adv_hone_strategy, express_event);

    let other_strategy: String = if adv_hone_strategy == "Juice on grace" {
        "No juice".to_string()
    } else {
        "Juice on grace".to_string()
    };

    let other_upgrades: Vec<Upgrade> =
        parser(normal_counts, adv_counts, &other_strategy, express_event);

    let other_strategy_prob_dists: Vec<Vec<f64>> = other_upgrades
        .iter()
        .filter(|upgrade| !upgrade.is_normal_honing)
        .map(|upgrade| upgrade.prob_dist.clone())
        .collect();

    (main_upgrades, other_strategy_prob_dists)
}
