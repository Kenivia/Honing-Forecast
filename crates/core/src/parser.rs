use crate::constants::*;
use crate::helpers::{calc_unlock, eqv_gold_per_tap};
use crate::normal_honing_utils::probability_distribution;
use crate::upgrade::Upgrade;
use serde::{Deserialize, Serialize};
use std::f64::NAN;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationOutput {
    pub unlock_costs: Vec<i64>,
    pub budgets: Vec<i64>,
    pub price_arr: Vec<f64>,
    pub leftover_values: Vec<f64>,
    pub test_case: i64,
    pub eqv_gold_budget: f64,
    pub juice_info: JuiceInfo,
    pub juice_books_owned: Vec<(f64, f64)>,
    pub effective_budgets: Vec<f64>, // this is just equal to budget now, I used to subtract unlock cost but now it's baked into the random variables
    pub already_spent: Option<(Vec<i64>, Vec<i64>, Vec<i64>, f64)>,
    pub flat_alr_spent: Option<Vec<f64>>,
}

/// convert everything to gold for success_prob_metric, currently unused
pub fn actual_eqv_gold(
    price_arr: &[f64],
    budgets: &[i64],
    juice_info: &JuiceInfo,
    juice_books_owned: &[(f64, f64)],
) -> f64 {
    let mut total = 0.0;
    for i in 0..7 {
        total += price_arr[i] * budgets[i] as f64;
    }
    for (id, i) in juice_books_owned.iter().enumerate() {
        total += i.0 * juice_info.one_gold_cost_id[id].0;
        total += i.1 * juice_info.one_gold_cost_id[id].1;
    }

    total
}

/// this mustve been vibe coded
fn copy_leftover<T: Clone>(inp_leftover_values: &[T], original: &[T]) -> Vec<T> {
    let out: Vec<T>;
    if inp_leftover_values.is_empty() {
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

        let juice_info: JuiceInfo = get_avail_juice_combs(juice_prices, &leftover_juice_values);
        let juice_books_owned: Vec<(f64, f64)> = juice_books_budget
            .iter()
            .map(|(a, b)| (*a as f64, *b as f64))
            .collect();
        let eqv_gold_budget: f64 =
            actual_eqv_gold(&price_arr, &budgets, &juice_info, &juice_books_owned);

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
            upgrade.eqv_gold_per_tap = eqv_gold_per_tap(upgrade, inp_price_arr);
            if !upgrade.is_normal_honing {
                let l_len = upgrade.prob_dist.len();
                let mut this_combined = Vec::with_capacity(l_len);
                let mut cost_so_far: f64 = upgrade.unlock_costs[0] as f64 * price_arr[3]
                    + upgrade.unlock_costs[1] as f64 * price_arr[6];

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
                    &upgrade.prob_dist,
                    NAN,
                    // cost_so_far,
                    false,
                    upgrade.alr_failed,
                );
            }
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
        let effective_budgets: Vec<f64> = budgets[0..7].iter().map(|x| *x as f64).collect();

        let out: PreparationOutput = Self {
            // upgrade_arr,
            unlock_costs,
            budgets,
            price_arr,

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

        (out, upgrade_arr)
    }
}

/// Constructs vector of Upgrade objects according to what upgrades were selected and the appropriate juice applied
/// just an absolute shitshow of a parser but it works so
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
                .is_some_and(|arr| arr[piece_type][upgrade_index]);

            let this_succeeded: bool = succeeded_grid
                .as_ref()
                .is_some_and(|arr| arr[piece_type][upgrade_index]);

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
                .is_some_and(|arr| arr[piece_type][upgrade_index]);

            let this_succeeded: bool = succeeded_grid
                .as_ref()
                .is_some_and(|arr| arr[piece_type][upgrade_index]);

            let start = relevant_data[0][0] as usize;

            let mut prob_dist = Vec::with_capacity(rows);
            let mut juice_dist = Vec::with_capacity(rows);

            for row in 0..rows {
                if row < start || row <= this_progress || this_succeeded {
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
