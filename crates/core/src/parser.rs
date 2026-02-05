use std::f64::NAN;
use std::iter;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::constants::{
    ADV_DATA_10_20_DOUBLE, ADV_DATA_10_20_JUICE, ADV_DATA_30_40, ADV_DATA_30_40_JUICE,
    DEFAULT_ADV_HONE_COST, JuiceInfo, NORMAL_HONE_CHANCES, SPECIAL_LEAPS_COST,
    get_avail_juice_combs, get_event_extra_chance, get_event_modified_adv_costs,
    get_event_modified_adv_unlock_cost, get_event_modified_armor_costs,
    get_event_modified_armor_unlock_cost, get_event_modified_artisan,
    get_event_modified_weapon_costs, get_event_modified_weapon_unlock_cost,
};
use crate::helpers::{calc_unlock, eqv_gold_per_tap};
use crate::normal_honing_utils::{add_up_golds, apply_price_generic, probability_distribution};
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
    pub juice_books_owned: Vec<(f64, f64)>, // juice_books_owned[id].0 = weap owned
    // pub sellable_toggles: Vec<bool>,
    // pub upgrade_arr: Vec<Upgrade>,
    pub effective_budgets: Vec<f64>,
    pub already_spent: Option<(Vec<i64>, Vec<i64>, Vec<i64>, f64)>,
    pub flat_alr_spent: Option<Vec<f64>>,
}

fn compute_already_spent(
    upgrade_arr: &mut Vec<Upgrade>,
    prep_output: &PreparationOutput,
) -> ((Vec<i64>, Vec<i64>, Vec<i64>, f64), Vec<f64>, Vec<f64>) {
    let mut out_mats_float: Vec<f64> = vec![0.0; 7];
    let mut out_weapon_float: Vec<f64> = vec![0.0; prep_output.juice_info.num_avail];
    let mut out_armor_float: Vec<f64> = vec![0.0; prep_output.juice_info.num_avail];

    let mut unlock_offset: Vec<f64> = vec![0.0; 2];
    for upgrade in upgrade_arr.iter_mut() {
        upgrade.update_this_prob_dist(prep_output);
        upgrade.update_this_individual_support(prep_output);
        for i in 0..7 {
            out_mats_float[i] += upgrade.cost_dist[i].support[upgrade.alr_failed];
        }
        if upgrade.alr_failed > 0 {
            #[cfg(target_arch = "wasm32")]
            if !upgrade.unlocked {
                web_sys::console::log_1(&format!("{:?}", upgrade).into());
            }
            assert!(upgrade.unlocked)
        }
        // if upgrade.unlocked {
        //     out_mats_float[3] += upgrade.unlock_costs[0] as f64;
        //     out_mats_float[6] += upgrade.unlock_costs[1] as f64;
        //     unlock_offset[0] += upgrade.unlock_costs[0] as f64;
        //     unlock_offset[1] += upgrade.unlock_costs[1] as f64;
        // }

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
    let out_flat: Vec<f64> = out_mats_float
        .iter()
        .chain(out_weapon_float.iter().interleave(out_armor_float.iter()))
        .cloned()
        .collect();
    (
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
        ),
        out_flat,
        unlock_offset,
    )
}
pub fn actual_eqv_gold(
    price_arr: &[f64],
    budgets: &[i64],
    juice_info: &JuiceInfo,
    unlock_costs: &[i64],
    juice_books_owned: &[(f64, f64)],
) -> f64 {
    let mut total = 0.0;
    for i in 0..7 {
        total += price_arr[i] * budgets[i] as f64;
    }
    for (id, i) in juice_books_owned.iter().enumerate() {
        total += i.0 as f64 * juice_info.one_gold_cost_id[id].0;
        total += i.1 as f64 * juice_info.one_gold_cost_id[id].1;
    }
    // total -= unlock_costs[0] as f64 * price_arr[3];
    // total -= unlock_costs[1] as f64 * price_arr[6];

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
        succeeded_grid: Option<Vec<Vec<bool>>>,
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

        let juice_books_owned: Vec<(f64, f64)> = juice_books_budget
            .iter()
            .map(|(a, b)| (*a as f64, *b as f64))
            .collect();
        let eqv_gold_budget: f64 = actual_eqv_gold(
            &price_arr,
            &budgets,
            &juice_info,
            &unlock_costs,
            &juice_books_owned,
        );

        // web_sys::console::log_1(&"4".into());
        let mut upgrade_arr: Vec<Upgrade> = parser(
            hone_ticks,
            adv_ticks,
            &adv_hone_strategy.to_string(),
            express_event,
            &juice_info,
            progress_grid,
            state_grid,
            unlock_grid,
            succeeded_grid,
        );

        for upgrade in upgrade_arr.iter_mut() {
            // let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
            // web_sys::console::log_1(
            //     &format!("{:?} {:?} ", upgrade.state.payload, upgrade.succeeded).into(),
            // );
            upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, inp_price_arr);
            if !upgrade.is_normal_honing {
                let l_len = upgrade.prob_dist.len();
                let mut this_combined = Vec::with_capacity(l_len);
                let mut cost_so_far: f64 = 0.0;

                for (p_index, _) in upgrade.prob_dist.iter().enumerate() {
                    this_combined.push(
                        cost_so_far
                            + (upgrade.adv_juice_cost[p_index]
                                * if upgrade.is_weapon {
                                    juice_info.one_gold_cost_id[0].0
                                } else {
                                    juice_info.one_gold_cost_id[0].1
                                }),
                    );

                    if p_index >= l_len - 1 {
                        break;
                    }

                    cost_so_far += upgrade.eqv_gold_per_tap;
                }
                upgrade.combined_gold_costs.update_payload(
                    this_combined,
                    upgrade.state.hash,
                    &mut upgrade.prob_dist,
                    NAN,
                    // cost_so_far,
                    false,
                );
            }

            // let juice_ind: usize = if upgrade.is_weapon { 7 } else { 8 };
            // upgrade.eqv_gold_per_juice = user_price_arr[juice_ind] * upgrade.one_juice_cost as f64;
        }
        for upgrade in upgrade_arr.iter_mut() {
            if !upgrade.is_normal_honing {
                continue;
            }
            // JUST GONNA ASSUME THAT not have juice => not have book or book => juice or first element is always juice (if there's a first element)
            let both_avail: usize = juice_info.ids[upgrade.upgrade_index].len();
            if both_avail > 0 {
                upgrade.juice_avail = true;
            }
            upgrade.books_avail = (both_avail - 1).max(0) as i64;
        }
        let mut effective_budgets: Vec<f64> = budgets[0..7].iter().map(|x| *x as f64).collect();
        // effective_budgets[3] -= unlock_costs[0] as f64;
        // effective_budgets[6] -= unlock_costs[1] as f64;

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
            flat_alr_spent: None,
        };
        // let (alr, flat_alr_spent, unlock_offset) = compute_already_spent(&mut upgrade_arr, &out);
        // out.already_spent = Some(alr.clone());
        // out.flat_alr_spent = Some(flat_alr_spent.clone());
        // for (index, e_budget) in out
        //     .effective_budgets
        //     .iter_mut()
        //     .chain(
        //         out.juice_books_owned
        //             .iter_mut()
        //             .flat_map(|x| iter::once(&mut x.0).chain(iter::once(&mut x.1))),
        //     )
        //     .enumerate()
        // {
        //     *e_budget -= flat_alr_spent[index];
        //     if index == 3 {
        //         *e_budget += unlock_offset[0]; // don't double count unlock
        //     }
        //     if index == 6 {
        //         *e_budget += unlock_offset[1];
        //     }
        // }
        // // web_sys::console::log_1(
        // //     &format!(
        // //         "{:?} {:?} {:?}",
        // //         out.juice_books_owned, out.flat_alr_spent, out.already_spent
        // //     )
        // //     .into(),
        // // );
        (out, upgrade_arr)
    }
    // pub fn eqv_gold_unlock(&self) -> f64 {
    //     // a bit redundent but whatever
    //     let mut c: f64 = 0.0;

    //     // c += self.unlock_costs[0] as f64 * self.price_arr[3];
    //     // c += self.unlock_costs[1] as f64 * self.price_arr[6];

    //     c
    // }
}

// Constructs vector of Upgrade objects according to what upgrades were selected and the appropriate juice applieid
pub fn parser(
    normal_ticks: &[Vec<bool>],
    adv_ticks: &[Vec<bool>],
    adv_hone_strategy: &String,
    express_event: bool,
    juice_info: &JuiceInfo,
    progress_arr_opt: Option<Vec<Vec<usize>>>,
    state_given_opt: Option<Vec<Vec<Vec<(bool, usize)>>>>,
    unlock_grid: Option<Vec<Vec<bool>>>,
    succeeded_grid: Option<Vec<Vec<bool>>>,
) -> Vec<Upgrade> {
    let mut out: Vec<Upgrade> = Vec::new();
    let weapon_unlock_costs: [[i64; 25]; 2] = get_event_modified_weapon_unlock_cost(express_event);
    let armor_unlock_costs: [[i64; 25]; 2] = get_event_modified_armor_unlock_cost(express_event);
    let adv_unlock_costs: [[i64; 8]; 2] = get_event_modified_adv_unlock_cost(express_event);
    let adv_tap_costs: [[i64; 8]; 8] = get_event_modified_adv_costs(express_event);
    let artisan_rate_arr: [f64; 25] = get_event_modified_artisan(express_event);
    let event_extra_arr: [f64; 25] = get_event_extra_chance(express_event);
    let row_len = normal_ticks[0].len(); // 25

    for upgrade_index in 0..row_len {
        for piece_type in 0..normal_ticks.len() {
            let needed = normal_ticks[piece_type][upgrade_index];
            if !needed {
                continue;
            }

            let cur_cost: [[i64; 25]; 7] = if piece_type < 5 {
                get_event_modified_armor_costs(express_event)
            } else {
                get_event_modified_weapon_costs(express_event)
            };

            let special_cost: i64 =
                SPECIAL_LEAPS_COST[if piece_type == 5 { 0 } else { 1 }][upgrade_index];

            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_index];

            let this_progress: usize = progress_arr_opt
                .as_ref()
                .map_or(0, |arr| arr[piece_type][upgrade_index]);

            let this_unlocked: bool = unlock_grid
                .as_ref()
                .map_or(false, |arr| arr[piece_type][upgrade_index]);

            let this_succeeded: bool = succeeded_grid
                .as_ref()
                .map_or(false, |arr| arr[piece_type][upgrade_index]);

            let clean_prob_dist = probability_distribution(
                NORMAL_HONE_CHANCES[upgrade_index],
                event_artisan_rate,
                &[],
                0.0,
                0,
                false,
                None,
                event_extra_arr[upgrade_index],
            );

            let this_state_given: Option<Vec<(bool, usize)>> =
                state_given_opt.as_ref().and_then(|sg| {
                    let v = &sg[piece_type][upgrade_index];
                    if v.len() == clean_prob_dist.len() {
                        Some(v.clone())
                    } else {
                        None
                    }
                });

            let relevant = if piece_type == 5 {
                weapon_unlock_costs
            } else {
                armor_unlock_costs
            };

            out.push(Upgrade::new_normal(
                NORMAL_HONE_CHANCES[upgrade_index],
                // clean_prob_dist,
                std::array::from_fn(|cost_type| cur_cost[cost_type][upgrade_index]),
                special_cost,
                piece_type == 5,
                piece_type,
                event_artisan_rate,
                upgrade_index,
                juice_info,
                this_progress,
                this_state_given,
                this_unlocked,
                vec![relevant[0][upgrade_index], relevant[1][upgrade_index]],
                this_succeeded,
                event_extra_arr[upgrade_index],
            ));
        }
    }
    let row_len = adv_ticks[0].len();

    for upgrade_index in 0..row_len {
        for row_ind in 0..adv_ticks.len() {
            let needed = adv_ticks[row_ind][upgrade_index];
            if !needed {
                continue;
            }

            let piece_type = if row_ind == 5 { 0 } else { 1 };

            let relevant_data: &'static [[i64; 3]] = if adv_hone_strategy == "No x2 grace" {
                if upgrade_index <= 1 {
                    &ADV_DATA_10_20_JUICE
                } else {
                    &ADV_DATA_30_40_JUICE
                }
            } else if upgrade_index <= 1 {
                &ADV_DATA_10_20_DOUBLE
            } else {
                &ADV_DATA_30_40
            };

            let rows = relevant_data.last().unwrap()[0] as usize + 1;
            let sum_taps: i64 = relevant_data.iter().map(|row| row[2]).sum();
            let col_index: usize = 2 * upgrade_index + piece_type;

            let one_juice_cost: i64 = adv_tap_costs[7][col_index];
            let mut sum_taps_f: f64 = if sum_taps == 0 { 1.0 } else { sum_taps as f64 };

            let this_progress: usize = progress_arr_opt
                .as_ref()
                .map_or(0, |arr| arr[piece_type][upgrade_index]);

            let this_unlocked: bool = unlock_grid
                .as_ref()
                .map_or(false, |arr| arr[piece_type][upgrade_index]);

            let this_succeeded: bool = succeeded_grid
                .as_ref()
                .map_or(false, |arr| arr[piece_type][upgrade_index]);

            let start = relevant_data[0][0] as usize;

            let mut prob_dist = Vec::with_capacity(rows);
            let mut juice_dist = Vec::with_capacity(rows);

            for row in 0..rows {
                if row < start || row <= this_progress || (row > this_progress && this_succeeded) {
                    prob_dist.push(0.0);
                    juice_dist.push(0.0);
                    if row >= start {
                        sum_taps_f -= relevant_data[row - start][2] as f64;
                    }
                    continue;
                }

                juice_dist
                    .push(one_juice_cost as f64 * relevant_data[row - start][1] as f64 / 1000.0);

                if this_succeeded {
                    prob_dist.push(1.0);
                } else {
                    prob_dist.push(relevant_data[row - start][2] as f64 / sum_taps_f);
                }
            }

            out.push(Upgrade::new_adv(
                prob_dist,
                std::array::from_fn(|cost_type| adv_tap_costs[cost_type][col_index]),
                one_juice_cost,
                juice_dist,
                row_ind == 5,
                row_ind,
                relevant_data[0][0],
                upgrade_index,
                vec![
                    adv_unlock_costs[0][col_index],
                    adv_unlock_costs[1][col_index],
                ],
                false,
                juice_info.num_avail,
                this_unlocked,
            ));
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

//     let other_strategy: String = if adv_hone_strategy == "No x2 grace" {
//         "x2 grace".to_string()
//     } else {
//         "No x2 grace".to_string()
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
