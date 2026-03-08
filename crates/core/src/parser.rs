use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::constants::accessor::{
    get_artisan, get_data, get_event_extra_chance, get_normal_hone_chances, get_special_leap_cost,
};
use crate::constants::juice_info::{JuiceInfo, get_priced_juice_info};
use crate::constants::*;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::array;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationOutput {
    pub budgets: [f64; 7],
    pub special_budget: i64,
    pub price_arr: Vec<f64>,
    pub leftover_values: Vec<f64>,
    pub test_case: i64,
    pub juice_info: JuiceInfo,
    pub juice_books_owned: Vec<(f64, f64)>,
    pub already_spent: Option<(Vec<i64>, Vec<i64>, Vec<i64>, f64)>,
    pub flat_alr_spent: Option<Vec<f64>>,
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
        juice_books_budget: &[(i64, i64)],
        juice_prices: &[(f64, f64)],
        inp_leftover_values: &[f64],
        inp_leftover_juice_values: &[(f64, f64)],
        normal_progress_grid: Option<Vec<Vec<usize>>>,
        state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
        unlock_grid: Option<Vec<Vec<bool>>>,
        succeeded_grid: Option<Vec<Vec<bool>>>,
        adv_progress_grid: Option<Vec<Vec<(usize, usize, bool, bool)>>>,
        tier: usize,
    ) -> (
        PreparationOutput,
        Vec<Upgrade>,
        AHashMap<AdvConfig, AdvDistTriplet>,
    ) {
        let price_arr: Vec<f64> = inp_price_arr.to_vec();
        let leftover_values = copy_leftover(inp_leftover_values, inp_price_arr);
        let leftover_juice_values = copy_leftover(inp_leftover_juice_values, juice_prices);

        let budgets: [f64; 7] = array::from_fn(|index| input_budgets[index] as f64);

        let juice_info: JuiceInfo = get_priced_juice_info(
            &BASE_JUICE_INFOS[tier],
            juice_prices,
            &leftover_juice_values,
            express_event,
        );
        let juice_books_owned: Vec<(f64, f64)> = juice_books_budget
            .iter()
            .map(|(a, b)| (*a as f64, *b as f64))
            .collect();
        let mut adv_cache: AHashMap<AdvConfig, AdvDistTriplet> = AHashMap::new();

        let upgrade_arr: Vec<Upgrade> = parser(
            hone_ticks,
            adv_ticks,
            express_event,
            &juice_info,
            normal_progress_grid,
            state_grid,
            unlock_grid,
            succeeded_grid,
            adv_progress_grid,
            tier,
            &mut adv_cache,
        );

        let out: PreparationOutput = Self {
            // upgrade_arr,
            budgets: budgets.try_into().unwrap(),
            special_budget: input_budgets[7],
            price_arr,

            test_case: -1, // arena will overwrite this

            juice_info,
            juice_books_owned,
            // sellable_toggles, //TODO READ THIS FROM AN ACUTAL INPUT LATEr cant be bother rn
            leftover_values,
            already_spent: None,
            flat_alr_spent: None,
        };

        (out, upgrade_arr, adv_cache)
    }
}

/// Constructs vector of Upgrade objects according to what upgrades were selected and the appropriate juice applied
/// just an absolute shitshow of a parser but it works so
pub fn parser(
    normal_ticks: &[Vec<bool>],
    adv_ticks: &[Vec<bool>],
    express_event: bool,
    juice_info: &JuiceInfo,
    progress_arr_opt: Option<Vec<Vec<usize>>>,
    state_given_opt: Option<Vec<Vec<Vec<(bool, usize)>>>>,
    unlock_grid: Option<Vec<Vec<bool>>>,
    succeeded_grid: Option<Vec<Vec<bool>>>,
    adv_progress_grid: Option<Vec<Vec<(usize, usize, bool, bool)>>>,
    tier: usize,
    adv_cache: &mut AHashMap<AdvConfig, AdvDistTriplet>,
) -> Vec<Upgrade> {
    let mut out: Vec<Upgrade> = Vec::new();

    let artisan_rate_arr = get_artisan(express_event, tier);
    let event_extra_arr = get_event_extra_chance(express_event, tier);
    let special_leap_cost = get_special_leap_cost(tier);
    let normal_hone_chances = get_normal_hone_chances(tier);
    let row_len = normal_ticks[0].len(); // 25

    for upgrade_index in 0..row_len {
        for row_ind in 0..normal_ticks.len() {
            let needed = normal_ticks[row_ind][upgrade_index];
            if !needed {
                continue;
            }
            let relevant_cost = get_data(express_event, tier, false, row_ind == 5, false);
            let relevant_unlock = get_data(express_event, tier, false, row_ind == 5, true);

            let special_cost: i64 =
                special_leap_cost[if row_ind == 5 { 1 } else { 0 }][upgrade_index];

            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_index];

            let this_progress: usize = progress_arr_opt
                .as_ref()
                .map_or(0, |arr| arr[row_ind][upgrade_index]);

            let this_unlocked: bool = unlock_grid
                .as_ref()
                .is_some_and(|arr| arr[row_ind][upgrade_index]);

            let this_succeeded: bool = succeeded_grid
                .as_ref()
                .is_some_and(|arr| arr[row_ind][upgrade_index]);

            let this_state_given: Option<Vec<(bool, usize)>> = state_given_opt
                .as_ref()
                .and_then(|sg| Some(sg[row_ind][upgrade_index].clone()));

            let this_cost =
                &Vec::from_iter((0..7).map(|cost_type| relevant_cost[cost_type][upgrade_index]));
            let this_unlock =
                &Vec::from_iter((0..7).map(|cost_type| relevant_unlock[cost_type][upgrade_index]));
            out.push(Upgrade::new_normal(
                normal_hone_chances[upgrade_index],
                this_cost,
                special_cost,
                row_ind == 5,
                row_ind,
                event_artisan_rate,
                upgrade_index,
                juice_info,
                this_progress,
                this_state_given,
                this_unlocked,
                this_unlock,
                this_succeeded,
                event_extra_arr[upgrade_index],
            ));
        }
    }
    let row_len = adv_ticks[0].len();

    for upgrade_index in 0..row_len {
        for row_ind in 0..adv_ticks.len() {
            if !adv_ticks[row_ind][upgrade_index] {
                continue;
            }

            let relevant_cost = get_data(express_event, tier, true, row_ind == 5, false);
            let relevant_unlock = get_data(express_event, tier, true, row_ind == 5, true);

            let this_adv_progress: (usize, usize, bool, bool) = adv_progress_grid
                .as_ref()
                .map_or((0, 0, false, false), |arr| arr[row_ind][upgrade_index]);

            let this_unlocked: bool = unlock_grid
                .as_ref()
                .is_some_and(|arr| arr[row_ind][upgrade_index]);

            let this_succeeded: bool = succeeded_grid
                .as_ref()
                .is_some_and(|arr| arr[row_ind][upgrade_index]);
            let this_cost =
                &Vec::from_iter((0..7).map(|cost_type| relevant_cost[cost_type][upgrade_index]));
            let this_unlock =
                &Vec::from_iter((0..7).map(|cost_type| relevant_unlock[cost_type][upgrade_index]));

            out.push(Upgrade::new_adv(
                this_cost,
                row_ind == 5,
                row_ind,
                upgrade_index,
                this_unlock,
                this_succeeded,
                this_unlocked,
                this_adv_progress,
                express_event,
                juice_info,
                adv_cache,
            ));
        }
    }

    out
}
