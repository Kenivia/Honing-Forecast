use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::constants::accessor::{
    get_artisan, get_data, get_event_extra_chance, get_normal_hone_chances, get_special_leap_cost,
};
use crate::constants::juice_info::{JuiceInfo, get_priced_juice_info};
use crate::constants::*;
use crate::helpers::distribute_budgets;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationOutput {
    pub special_budget: i64,
    pub raw_material_info: MaterialInput,
    pub optimizer_plan: Vec<usize>,
    pub optimizer_material_info: MaterialInput,
    pub raw_num_breakpoints: usize,
    pub test_case: i64,
    pub juice_info: JuiceInfo,
}

pub type MaterialInput = Vec<Vec<(f64, f64)>>; // [material type][treatment plan].0 = owned, .1 = price

#[derive(Deserialize, Clone, Serialize)]
pub struct OneUpgradeInput {
    pub piece_type: usize,
    pub upgrade_index: usize,
    pub is_normal_honing: bool,
    pub starting_artisan: Option<f64>,
    pub starting_num_taps: Option<usize>,
    pub state: Option<Vec<(bool, usize)>>,
    pub unlocked: bool,
    pub adv_progress: Option<(usize, usize, bool, bool)>,
}

impl PreparationOutput {
    pub fn initialize(
        raw_material_info: MaterialInput,
        inp_optimizer_plan: Option<Vec<usize>>,
        upgrade_info: Vec<OneUpgradeInput>,
        special_budget: i64,
        express_event: bool,
        tier: usize,
        inp_adv_cache: Option<AHashMap<AdvConfig, AdvDistTriplet>>,
    ) -> (
        PreparationOutput,
        Vec<Upgrade>,
        AHashMap<AdvConfig, AdvDistTriplet>,
    ) {
        let juice_info: JuiceInfo =
            get_priced_juice_info(&BASE_JUICE_INFOS[tier], &raw_material_info, express_event);
        let mut adv_cache: AHashMap<AdvConfig, AdvDistTriplet> = if inp_adv_cache.is_none() {
            AHashMap::new()
        } else {
            let mut inp: AHashMap<AdvConfig, AdvDistTriplet> = inp_adv_cache.unwrap();

            inp.retain(|key: &AdvConfig, _| {
                upgrade_info.iter().any(|upgrade| {
                    if !upgrade.is_normal_honing && upgrade.adv_progress.is_some() {
                        let (start_xp, start_balls, next_free, next_big) =
                            upgrade.adv_progress.unwrap();

                        return start_xp == key.start_xp
                            && start_balls == key.start_balls
                            && next_free == key.next_free
                            && next_big == key.next_big
                            && (express_event && upgrade.upgrade_index < 2) == key.double_balls
                            && ((upgrade.upgrade_index >= 2) == key.is_30_40);
                    }
                    false
                })
            });
            inp
        };

        let upgrade_arr: Vec<Upgrade> = parser(
            upgrade_info,
            express_event,
            &juice_info,
            tier,
            &mut adv_cache,
        );
        let optimizer_plan = if inp_optimizer_plan.is_none() {
            (0..raw_material_info.len()).collect::<Vec<usize>>()
        } else {
            inp_optimizer_plan.unwrap()
        };
        let optimizer_material_info = distribute_budgets(&raw_material_info, &optimizer_plan);
        // my_dbg!(
        //     &raw_material_info,
        //     &optimizer_material_info,
        //     &optimizer_plan
        // );
        let raw_num_breakpoints = raw_material_info[0].len();
        assert!(raw_num_breakpoints > 0);
        let out: PreparationOutput = Self {
            // upgrade_arr,
            raw_material_info,
            optimizer_material_info,
            optimizer_plan,
            raw_num_breakpoints,
            special_budget,
            test_case: -1, // arena will overwrite this
            juice_info,
        };

        (out, upgrade_arr, adv_cache)
    }
}

/// Constructs vector of Upgrade objects according to what upgrades were selected and the appropriate juice applied
pub fn parser(
    upgrade_info: Vec<OneUpgradeInput>,
    express_event: bool,
    juice_info: &JuiceInfo,
    tier: usize,
    adv_cache: &mut AHashMap<AdvConfig, AdvDistTriplet>,
) -> Vec<Upgrade> {
    let mut out: Vec<Upgrade> = Vec::new();

    let artisan_rate_arr = get_artisan(express_event, tier);
    let event_extra_arr = get_event_extra_chance(express_event, tier);
    let special_leap_cost = get_special_leap_cost(tier);
    let normal_hone_chances = get_normal_hone_chances(tier);

    for OneUpgradeInput {
        piece_type,
        upgrade_index,
        is_normal_honing,
        starting_artisan,
        starting_num_taps,
        state,
        unlocked,
        adv_progress,
    } in upgrade_info
    {
        let relevant_cost = get_data(
            express_event,
            tier,
            !is_normal_honing,
            piece_type == 5,
            false,
        );
        let relevant_unlock = get_data(
            express_event,
            tier,
            !is_normal_honing,
            piece_type == 5,
            true,
        );
        let this_cost =
            &Vec::from_iter((0..7).map(|cost_type| relevant_cost[cost_type][upgrade_index]));
        let this_unlock =
            &Vec::from_iter((0..7).map(|cost_type| relevant_unlock[cost_type][upgrade_index]));
        let this_unlocked: bool = unlocked;
        let this_state_given: Vec<(bool, usize)> = state.unwrap_or(Vec::new());

        if is_normal_honing {
            let special_cost: i64 =
                special_leap_cost[if piece_type == 5 { 1 } else { 0 }][upgrade_index];
            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_index];
            let starting_artisan: f64 = starting_artisan.unwrap();
            let starting_num_taps: usize = starting_num_taps.unwrap_or(0);
            out.push(Upgrade::new_normal(
                normal_hone_chances[upgrade_index],
                this_cost,
                special_cost,
                piece_type == 5,
                piece_type,
                event_artisan_rate,
                upgrade_index,
                juice_info,
                starting_artisan,
                starting_num_taps,
                this_state_given,
                this_unlocked,
                this_unlock,
                event_extra_arr[upgrade_index],
            ));
        } else {
            let this_adv_progress: (usize, usize, bool, bool) = adv_progress.unwrap();

            out.push(Upgrade::new_adv(
                this_cost,
                piece_type == 5,
                piece_type,
                upgrade_index,
                this_unlock,
                this_unlocked,
                this_adv_progress,
                express_event,
                juice_info,
                adv_cache,
                this_state_given,
            ));
        }
    }

    out
}
