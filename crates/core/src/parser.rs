use serde::{Deserialize, Serialize};

use crate::constants::{
    ADV_DATA_10_20, ADV_DATA_10_20_JUICE, ADV_DATA_30_40, ADV_DATA_30_40_JUICE, ADV_HONE_COST,
    JuiceInfo, NORMAL_HONE_CHANCES, SPECIAL_LEAPS_COST, get_avail_juice_combs,
    get_event_modified_adv_unlock_cost, get_event_modified_armor_costs,
    get_event_modified_armor_unlock_cost, get_event_modified_artisan,
    get_event_modified_weapon_costs, get_event_modified_weapon_unlock_cost,
};
use crate::helpers::{calc_unlock, eqv_gold_per_tap};
use crate::normal_honing_utils::{
    add_up_golds, apply_price_generic, apply_price_leftovers, probability_distribution,
};
use crate::upgrade::Upgrade;
// use crate::monte_carlo::monte_carlo_one;
// use crate::value_estimation::{
//     est_juice_value, est_special_honing_value, extract_special_strings, juice_to_array,
// };
// use rand::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationOutput {
    pub unlock_costs: Vec<i64>,
    pub budgets: Vec<i64>,

    pub price_arr: Vec<f64>,
    pub leftover_values: Vec<f64>,

    pub budgets_no_gold: Vec<i64>,
    pub test_case: i64,
    pub eqv_gold_budget: f64,
    pub juice_info: JuiceInfo,
    pub juice_books_owned: Vec<(i64, i64)>, // juice_books_owned[id].0 = weap owned
    // pub sellable_toggles: Vec<bool>,
    // pub upgrade_arr: Vec<Upgrade>,
    pub effective_budgets: Vec<i64>,
    pub already_spent: Option<(Vec<i64>, Vec<i64>, Vec<i64>, f64)>,
}

fn compute_already_spent(
    upgrade_arr: &mut Vec<Upgrade>,
    prep_output: &PreparationOutput,
) -> (Vec<i64>, Vec<i64>, Vec<i64>, f64) {
    let mut out_mats_float: Vec<f64> = vec![0.0; 7];
    let mut out_weapon_float: Vec<f64> = vec![0.0; prep_output.juice_info.num_avail];
    let mut out_armor_float: Vec<f64> = vec![0.0; prep_output.juice_info.num_avail];
    for upgrade in upgrade_arr.iter_mut() {
        upgrade.update_this_prob_dist(prep_output);
        upgrade.update_this_individual_support(prep_output);
        for i in 0..7 {
            out_mats_float[i] += upgrade.cost_dist[i].support[upgrade.alr_failed];
        }
        if upgrade.alr_failed > 0 {
            if !upgrade.unlocked {
                web_sys::console::log_1(&format!("{:?}", upgrade).into());
            }
            assert!(upgrade.unlocked)
        }
        if upgrade.unlocked {
            out_mats_float[3] += upgrade.unlock_costs[0] as f64;
            out_mats_float[6] += upgrade.unlock_costs[1] as f64;
        }

        for i in 0..prep_output.juice_info.num_avail {
            out_weapon_float[i] += upgrade.weap_juice_costs[i].support[upgrade.alr_failed];
            out_armor_float[i] += upgrade.armor_juice_costs[i].support[upgrade.alr_failed];
        }
    }
    let zipped = out_weapon_float
        .iter()
        .copied()
        .zip(out_armor_float.iter().copied())
        .collect::<Vec<(f64, f64)>>();
    let applied: (Vec<f64>, Vec<(f64, f64)>) =
        apply_price_generic(&out_mats_float, &zipped, &prep_output, false);
    (
        out_mats_float
            .iter()
            .map(|x| x.round() as i64)
            .collect::<Vec<i64>>(),
        out_weapon_float
            .iter()
            .map(|x| x.round() as i64)
            .collect::<Vec<i64>>(),
        out_armor_float
            .iter()
            .map(|x| x.round() as i64)
            .collect::<Vec<i64>>(),
        add_up_golds(&applied.0, &applied.1),
    )
}
pub fn actual_eqv_gold(
    price_arr: &[f64],
    budgets: &[i64],
    juice_info: &JuiceInfo,
    unlock_costs: &[i64],
    juice_books_owned: &[(i64, i64)],
) -> f64 {
    let mut total = 0.0;
    for i in 0..7 {
        total += price_arr[i] * budgets[i] as f64;
    }
    for (id, i) in juice_books_owned.iter().enumerate() {
        total += i.0 as f64 * juice_info.one_gold_cost_id[id].0 as f64;
        total += i.1 as f64 * juice_info.one_gold_cost_id[id].1 as f64;
    }
    total -= unlock_costs[0] as f64 * price_arr[3];
    total -= unlock_costs[1] as f64 * price_arr[6];

    total
}

fn copy_leftover<T: Clone>(inp_leftover_values: &[T], original: &[T]) -> Vec<T> {
    let out: Vec<T>;
    if inp_leftover_values.len() == 0 {
        out = original.to_vec();
    } else if inp_leftover_values.len() == original.len() {
        out = inp_leftover_values.to_vec();
    } else {
        panic!("bad leftover input");
    }
    out
}

impl PreparationOutput {
    pub fn initialize(
        hone_ticks: &[Vec<bool>],
        input_budgets: &[i64],
        adv_ticks: &[Vec<bool>],
        express_event: bool,
        inp_price_arr: &[f64],
        adv_hone_strategy: &str,
        juice_books_budget: &[(i64, i64)],
        juice_prices: &[(f64, f64)],
        inp_leftover_values: &[f64],
        inp_leftover_juice_values: &[(f64, f64)],
        progress_grid: Option<Vec<Vec<usize>>>,
        state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
        unlock_grid: Option<Vec<Vec<bool>>>,
    ) -> (PreparationOutput, Vec<Upgrade>) {
        let price_arr: Vec<f64> = inp_price_arr.to_vec();

        let leftover_values = copy_leftover(inp_leftover_values, inp_price_arr);
        let leftover_juice_values = copy_leftover(inp_leftover_juice_values, juice_prices);

        let unlock_costs: Vec<i64> = calc_unlock(hone_ticks, adv_ticks, express_event);

        let budgets: Vec<i64> = input_budgets.to_vec();

        let mut budgets_no_gold: Vec<i64> = budgets.clone();
        budgets_no_gold[5] = 0;
        // let sellable_toggles: Vec<bool> = vec![
        //     true, true, true, true, true, true, true, false, false, false, false, false, false,
        //     false,
        // ];

        let juice_info: JuiceInfo = get_avail_juice_combs(juice_prices, &leftover_juice_values);

        let juice_books_owned: Vec<(i64, i64)> = juice_books_budget.to_vec();
        let eqv_gold_budget: f64 = actual_eqv_gold(
            &price_arr,
            &budgets,
            &juice_info,
            &unlock_costs,
            &juice_books_owned,
        );
        let mut effective_budgets: Vec<i64> = budgets[0..7].to_vec();
        effective_budgets[3] -= unlock_costs[0];
        effective_budgets[6] -= unlock_costs[1];
        // web_sys::console::log_1(&"4".into());
        let mut upgrade_arr: Vec<Upgrade> = parser(
            hone_ticks,
            adv_ticks,
            &adv_hone_strategy.to_string(),
            express_event,
            juice_info.num_avail,
            progress_grid,
            state_grid,
            unlock_grid,
        );

        for upgrade in upgrade_arr.iter_mut() {
            // let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);

            upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, inp_price_arr);

            // let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
            // upgrade.eqv_gold_per_juice = user_price_arr[juice_ind] * upgrade.one_juice_cost as f64;
        }
        for upgrade in upgrade_arr.iter_mut() {
            // JUST GONNA ASSUME THAT not have juice => not have book or book => juice or first element is always juice (if there's a first element)
            let both_avail: usize = juice_info.ids[upgrade.upgrade_index].len();
            if both_avail > 0 {
                upgrade.juice_avail = true;
            }
            upgrade.books_avail = (both_avail - 1).max(0) as i64;
        }

        let mut out: PreparationOutput = Self {
            // upgrade_arr,
            unlock_costs,
            budgets,
            price_arr,
            budgets_no_gold,
            test_case: -1, // arena will overwrite this
            eqv_gold_budget,
            juice_info,
            juice_books_owned,
            // sellable_toggles, //TODO READ THIS FROM AN ACUTAL INPUT LATEr cant be bother rn
            leftover_values,
            effective_budgets,
            already_spent: None,
        };
        out.already_spent = Some(compute_already_spent(&mut upgrade_arr, &out));
        (out, upgrade_arr)
    }
    pub fn eqv_gold_unlock(&self) -> f64 {
        // a bit redundent but whatever
        let mut c: f64 = 0.0;

        c += self.unlock_costs[0] as f64 * self.price_arr[3];
        c += self.unlock_costs[1] as f64 * self.price_arr[6];

        c
    }
}

// Constructs vector of Upgrade objects according to what upgrades were selected and the appropriate juice applieid
pub fn parser(
    normal_ticks: &[Vec<bool>],
    adv_ticks: &[Vec<bool>],
    adv_hone_strategy: &String,
    express_event: bool,
    num_juice_avail: usize,
    progress_arr_opt: Option<Vec<Vec<usize>>>,
    state_given_opt: Option<Vec<Vec<Vec<(bool, usize)>>>>,
    unlock_grid: Option<Vec<Vec<bool>>>,
) -> Vec<Upgrade> {
    let mut out: Vec<Upgrade> = Vec::new();
    let weapon_unlock_costs: [[i64; 25]; 2] = get_event_modified_weapon_unlock_cost(express_event);
    let armor_unlock_costs: [[i64; 25]; 2] = get_event_modified_armor_unlock_cost(express_event);
    let adv_unlock_costs: [[i64; 8]; 2] = get_event_modified_adv_unlock_cost(express_event);
    let artisan_rate_arr: [f64; 25] = get_event_modified_artisan(express_event);
    for piece_type in 0..normal_ticks.len() {
        let cur_cost: [[i64; 25]; 7] = if piece_type < 5 {
            get_event_modified_armor_costs(express_event)
        } else {
            get_event_modified_weapon_costs(express_event)
        };

        // let mut current_counter: i64 = 0;
        let row_len: usize = normal_ticks[piece_type].len(); // 25
        let mut upgrade_index: usize = 0;

        while upgrade_index < row_len {
            let needed: bool = normal_ticks[piece_type][upgrade_index];
            if !needed {
                upgrade_index += 1;
                continue;
            }
            // if current_counter >= needed {
            //     upgrade_index += 1;
            //     current_counter = 0;
            //     continue;
            // }

            let special_cost: i64 =
                SPECIAL_LEAPS_COST[if piece_type == 5 { 0 } else { 1 }][upgrade_index];
            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_index];
            let this_progress: Option<usize> = if progress_arr_opt.is_none() {
                None
            } else {
                Some(progress_arr_opt.as_ref().unwrap()[piece_type][upgrade_index])
            };

            let this_state_given: Option<Vec<(bool, usize)>> = if state_given_opt.is_none()
                || state_given_opt.as_ref().unwrap()[piece_type][upgrade_index].len() == 0
            {
                None
            } else {
                Some(state_given_opt.as_ref().unwrap()[piece_type][upgrade_index].clone())
            };
            let this_unlocked: bool = if unlock_grid.is_none() {
                false
            } else {
                unlock_grid.as_ref().unwrap()[piece_type][upgrade_index]
            };
            // web_sys::console::log_1(&this_progress.into());
            // web_sys::console::log_1(&format!("{:?}", unlock_grid).into());

            // web_sys::console::log_1(&format!("{:?}", this_unlocked).into());
            // web_sys::console::log_1(&format!("{:?}, {:?}", piece_type, upgrade_index).into());

            let relevant = if piece_type == 5 {
                weapon_unlock_costs
            } else {
                armor_unlock_costs
            };
            out.push(Upgrade::new_normal(
                NORMAL_HONE_CHANCES[upgrade_index],
                probability_distribution(
                    NORMAL_HONE_CHANCES[upgrade_index],
                    event_artisan_rate,
                    &[],
                    0.0,
                    this_progress.unwrap_or(0),
                ),
                std::array::from_fn(|cost_type: usize| cur_cost[cost_type][upgrade_index]),
                special_cost,
                piece_type == 5,
                piece_type,
                event_artisan_rate,
                upgrade_index,
                num_juice_avail,
                this_progress,
                this_state_given,
                this_unlocked,
                vec![relevant[0][upgrade_index], relevant[1][upgrade_index]],
            ));
            // web_sys::console::log_1(&format!("upgrade init done ").into());
            upgrade_index += 1;
            // current_counter += 1;
        }
    }

    // Advanced hone
    let mut this_juice_cost: Vec<f64>;
    let mut prob_dist: Vec<f64>;
    for piece_type in 0..adv_ticks.len() {
        let row_len: usize = adv_ticks[piece_type].len();
        let mut upgrade_index: usize = 0;
        while upgrade_index < row_len {
            let needed: bool = adv_ticks[piece_type][upgrade_index];
            if !needed {
                upgrade_index += 1;
                continue;
            }
            // if current_counter >= needed {

            //     current_counter = 0;
            //     continue;
            // }

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
            let col_index: usize = 2 * upgrade_index + (1 - piece_type);

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
                piece_type == 5,
                piece_type,
                relevant_data[0][0],
                upgrade_index,
                vec![
                    adv_unlock_costs[0][col_index], // um idk if tihs is right i forgot how this worked will figure it out later
                    adv_unlock_costs[1][col_index],
                ],
            ));
            // current_counter += 1;
            upgrade_index += 1;
        }
    }

    out
}

// /// Parser that runs twice to get both the main strategy and the other strategy's probability distributions
// /// Used by Gamba  when toggling on/off juice for a particular adv honing piece
// pub fn parser_with_other_strategy(
//     normal_counts: &[Vec<i64>],
//     adv_counts: &[Vec<i64>],
//     adv_hone_strategy: &String,

//     express_event: bool,
// ) -> (Vec<Upgrade>, Vec<Vec<f64>>) {
//     let main_upgrades: Vec<Upgrade> =
//         parser(normal_counts, adv_counts, adv_hone_strategy, express_event);

//     let other_strategy: String = if adv_hone_strategy == "Juice on grace" {
//         "No juice".to_string()
//     } else {
//         "Juice on grace".to_string()
//     };

//     let other_upgrades: Vec<Upgrade> =
//         parser(normal_counts, adv_counts, &other_strategy, express_event);

//     let other_strategy_prob_dists: Vec<Vec<f64>> = other_upgrades
//         .iter()
//         .filter(|upgrade| !upgrade.is_normal_honing)
//         .map(|upgrade| upgrade.prob_dist.clone())
//         .collect();

//     (main_upgrades, other_strategy_prob_dists)
// }
