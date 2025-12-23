use crate::constants::{
    ADV_DATA_10_20, ADV_DATA_10_20_JUICE, ADV_DATA_30_40, ADV_DATA_30_40_JUICE, ADV_HONE_COST,
    JuiceInfo, NORMAL_HONE_CHANCES, NORMAL_JUICE_COST, SPECIAL_LEAPS_COST, get_avail_juice_combs,
    get_event_modified_armor_costs, get_event_modified_artisan, get_event_modified_weapon_costs,
};
use crate::helpers::{
    average_juice_cost, calc_unlock, compute_eqv_gold_values, eqv_gold_per_tap, eqv_gold_unlock,
    generate_first_deltas,
};
// use crate::monte_carlo::monte_carlo_one;
// use crate::value_estimation::{
//     est_juice_value, est_special_honing_value, extract_special_strings, juice_to_array,
// };
// use rand::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug)]
pub struct PreparationOutput {
    pub upgrade_arr: Vec<Upgrade>,
    pub unlock_costs: Vec<i64>,
    pub budgets: Vec<i64>,

    pub mats_value: Vec<f64>,

    pub budgets_no_gold: Vec<i64>,
    pub test_case: i64,
    pub base_gold_budget: f64,
    pub juice_info: JuiceInfo,
    pub juice_books_owned: Vec<(i64, i64)>, // juice_books_owned[id].0 = weap
}
pub fn preparation(
    hone_counts: &[Vec<i64>],
    input_budgets: &[i64],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    user_mats_value: &[f64],
    adv_hone_strategy: &str,
    juice_books_budget: &[(i64, i64)],
) -> PreparationOutput {
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
    for upgrade in upgrade_arr.iter_mut() {
        // let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
        upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, user_mats_value);
        for i in 0..upgrade.full_juice_len {
            // upgrade.support_lengths.push(vec![]); // this will contain different free taps eventually i think
            upgrade.support_lengths.push(
                probability_distribution(
                    upgrade.base_chance,
                    upgrade.artisan_rate,
                    &generate_first_deltas(
                        upgrade.base_chance,
                        upgrade.prob_dist_len, // this is excessive but its fine
                        i,
                    ),
                )
                .len(),
            );
        }

        // let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
        // upgrade.eqv_gold_per_juice = user_mats_value[juice_ind] * upgrade.one_juice_cost as f64;
    }

    let mut budgets_no_gold: Vec<i64> = budgets.clone();
    budgets_no_gold[5] = 0;

    let base_gold_budget: f64 = compute_eqv_gold_values(&budgets, &mats_value)
        - eqv_gold_unlock(&unlock_costs, &mats_value);
    PreparationOutput {
        upgrade_arr,
        unlock_costs,
        budgets,

        mats_value,

        budgets_no_gold,
        test_case: -1, // arena will overwrite this
        base_gold_budget,
        juice_info: get_avail_juice_combs(),
        juice_books_owned: juice_books_budget.to_vec(),
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
    pub upgrade_index: usize,
    pub special_value: f64,
    pub full_juice_len: usize,
    pub support_lengths: Vec<usize>, //Vec<Vec<Vec<[i64; 10]>>>, // cost_data_arr[juice_count][special_count] = cost_data for that decision
    pub eqv_gold_per_tap: f64,
    pub log_prob_dist: Vec<f64>,
    pub gold_cost_record: Vec<f64>,
    // pub juice_arr: Vec<f64>,
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
            one_juice_cost: NORMAL_JUICE_COST[upgrade_index],
            adv_juice_cost: vec![],
            special_cost,
            juice_values: vec![],
            prob_dist_len,
            is_weapon,
            artisan_rate,
            tap_offset: 1,
            upgrade_index,
            special_value: -1.0_f64,
            full_juice_len,
            support_lengths: vec![],    // to be filled
            log_prob_dist: vec![], // will change with each arrangement, maybe use a hashmap later
            eqv_gold_per_tap: -1.0_f64, // dummy value
            gold_cost_record: vec![],
            // juice_arr: vec![],
            // eqv_gold_per_juice: -1.0_f64,
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
            upgrade_index,
            special_value: -1.0_f64,
            full_juice_len: 1, // need to sort this out
            support_lengths: vec![],
            log_prob_dist: vec![], // will change with each arrangement, maybe use a hashmap later
            eqv_gold_per_tap: -1.0_f64, // dummy value
            gold_cost_record: vec![],
            // juice_arr: vec![],
            // eqv_gold_per_juice: -1.0_f64,
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
        let mut upgrade_index: usize = 0;

        while upgrade_index < row_len {
            let needed: i64 = normal_counts[is_weapon][upgrade_index];
            if current_counter >= needed {
                upgrade_index += 1;
                current_counter = 0;
                continue;
            }

            let special_cost: i64 = SPECIAL_LEAPS_COST[is_weapon][upgrade_index];
            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_index];
            out.push(Upgrade::new_normal(
                probability_distribution(
                    NORMAL_HONE_CHANCES[upgrade_index],
                    event_artisan_rate,
                    &[],
                ),
                std::array::from_fn(|cost_type: usize| cur_cost[cost_type][upgrade_index]),
                special_cost,
                is_weapon == 1,
                event_artisan_rate,
                upgrade_index,
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
        let mut upgrade_index: usize = 0;
        while upgrade_index < row_len {
            let needed: i64 = adv_counts[is_weapon][upgrade_index];
            if current_counter >= needed {
                upgrade_index += 1;
                current_counter = 0;
                continue;
            }

            // pick relevant_data based on strategy and level i (i <= 1 -> 10/20, else 30/40)
            let relevant_data: &'static [[i64; 3]] = if adv_hone_strategy == "Juice on grace" {
                if upgrade_index <= 1 {
                    &ADV_DATA_10_20_JUICE
                } else {
                    &ADV_DATA_30_40_JUICE
                }
            } else if upgrade_index <= 1 {
                &ADV_DATA_10_20
            } else {
                &ADV_DATA_30_40
            };

            let rows: usize = relevant_data.len();
            let sum_taps: i64 = relevant_data.iter().map(|row: &[i64; 3]| row[2]).sum(); // 2nd index is frequency
            let col_index: usize = 2 * upgrade_index + (1 - is_weapon);

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
                upgrade_index,
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
