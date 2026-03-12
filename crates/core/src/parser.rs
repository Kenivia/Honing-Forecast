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
    pub special_budget: i64,

    pub bound_mats: [f64; 7],
    pub trade_mats: [f64; 7],

    pub market_mats_price: Vec<f64>,
    pub trade_mats_price: Vec<f64>,
    pub left_mats_price: Vec<f64>,

    pub bound_juice: Vec<(f64, f64)>,
    pub trade_juice: Vec<(f64, f64)>,

    pub test_case: i64,
    pub juice_info: JuiceInfo,
    pub already_spent: Option<(Vec<i64>, Vec<i64>, Vec<i64>, f64)>,
    pub flat_alr_spent: Option<Vec<f64>>,
}

impl PreparationOutput {
    pub fn initialize(
        normal_hone_ticks: &[Vec<bool>],
        adv_ticks: &[Vec<bool>],
        express_event: bool,

        inp_bound_mats: &[i64],
        inp_trade_mats: &[i64],

        inp_market_mats_price: &[f64],
        inp_trade_mats_price: &[f64],
        inp_left_mats_price: &[f64],

        inp_bound_juice: &[(i64, i64)],
        inp_trade_juice: &[(i64, i64)],

        inp_juice_market_price: &[(f64, f64)],
        inp_juice_trade_price: &[(f64, f64)],
        inp_juice_left_price: &[(f64, f64)],

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
        let price_arr: Vec<f64> = inp_market_mats_price.to_vec();
        let tradable_leftover_arr: Vec<f64> = inp_trade_mats_price.to_vec();
        let bound_budgets: [f64; 7] = array::from_fn(|index| inp_bound_mats[index] as f64);

        let trade_budgets: [f64; 7] = array::from_fn(|index| inp_trade_mats[index] as f64);

        let juice_info: JuiceInfo = get_priced_juice_info(
            &BASE_JUICE_INFOS[tier],
            inp_juice_market_price,
            inp_juice_trade_price,
            inp_juice_left_price,
            express_event,
        );
        let bound_trade_juice: Vec<(f64, f64)> = inp_bound_juice
            .iter()
            .map(|(a, b)| (*a as f64, *b as f64))
            .collect();

        let tradable_juice_budgets: Vec<(f64, f64)> = inp_trade_juice
            .iter()
            .map(|(a, b)| (*a as f64, *b as f64))
            .collect();
        let mut adv_cache: AHashMap<AdvConfig, AdvDistTriplet> = AHashMap::new();

        let upgrade_arr: Vec<Upgrade> = parser(
            normal_hone_ticks,
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
            bound_mats: bound_budgets.try_into().unwrap(),
            trade_mats: trade_budgets.try_into().unwrap(),
            special_budget: inp_bound_mats[7],
            market_mats_price: price_arr,

            test_case: -1, // arena will overwrite this

            juice_info,
            bound_juice: bound_trade_juice,
            // sellable_toggles, //TODO READ THIS FROM AN ACUTAL INPUT LATEr cant be bother rn
            left_mats_price: inp_left_mats_price.to_vec(),
            already_spent: None,
            flat_alr_spent: None,

            trade_mats_price: tradable_leftover_arr,

            trade_juice: tradable_juice_budgets,
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
