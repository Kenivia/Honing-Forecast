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
    pub num_breakpoints: usize,

    pub test_case: i64,
    pub juice_info: JuiceInfo,
}

pub type MaterialInput = Vec<Vec<(f64, f64)>>; // [material type][treatment plan].0 = owned, .1 = price
pub type UpgradeInput = Vec<(
    usize,                              // piece type,
    usize,                              // upgrade_index
    bool,                               // is_normal_honing
    Option<usize>,                      // normal_progress
    Vec<(bool, usize)>,                 // state
    bool,                               // unlock
    bool,                               // succeeeded
    Option<(usize, usize, bool, bool)>, // adv_progress
)>;

impl PreparationOutput {
    pub fn initialize(
        raw_material_info: MaterialInput,
        inp_optimizer_plan: Option<Vec<usize>>,
        upgrade_info: UpgradeInput,
        special_budget: i64,
        express_event: bool,
        tier: usize,
    ) -> (
        PreparationOutput,
        Vec<Upgrade>,
        AHashMap<AdvConfig, AdvDistTriplet>,
    ) {
        let juice_info: JuiceInfo =
            get_priced_juice_info(&BASE_JUICE_INFOS[tier], &raw_material_info, express_event);
        let mut adv_cache: AHashMap<AdvConfig, AdvDistTriplet> = AHashMap::new();

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

        let num_breakpoints = raw_material_info[0].len();
        assert!(num_breakpoints > 0);
        let out: PreparationOutput = Self {
            // upgrade_arr,
            raw_material_info,
            optimizer_material_info,
            optimizer_plan,
            num_breakpoints,
            special_budget,
            test_case: -1, // arena will overwrite this
            juice_info,
        };

        (out, upgrade_arr, adv_cache)
    }
}

/// Constructs vector of Upgrade objects according to what upgrades were selected and the appropriate juice applied
pub fn parser(
    upgrade_info: UpgradeInput,
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

    for (
        piece_type,
        upgrade_index,
        is_normal_honing,
        normal_progress,
        state,
        unlock,
        success,
        adv_progress,
    ) in upgrade_info
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
        let this_unlocked: bool = unlock;
        let this_succeeded: bool = success;
        let this_state_given: Vec<(bool, usize)> = state;

        if is_normal_honing {
            let special_cost: i64 =
                special_leap_cost[if piece_type == 5 { 1 } else { 0 }][upgrade_index];
            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_index];
            let this_progress: usize = normal_progress.unwrap();
            out.push(Upgrade::new_normal(
                normal_hone_chances[upgrade_index],
                this_cost,
                special_cost,
                piece_type == 5,
                piece_type,
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
        } else {
            let this_adv_progress: (usize, usize, bool, bool) = adv_progress.unwrap();

            out.push(Upgrade::new_adv(
                this_cost,
                piece_type == 5,
                piece_type,
                upgrade_index,
                this_unlock,
                this_succeeded,
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
