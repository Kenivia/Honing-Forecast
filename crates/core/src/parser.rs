use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::constants::accessor::{
    get_artisan, get_data, get_event_extra_chance, get_normal_hone_chances, get_special_leap_cost,
};
use crate::constants::juice_info::{JuiceInfo, get_event_adjusted_juice_info};
use crate::constants::*;
use crate::materials::{MaterialTable, NUM_BASE_MATS, OneMaterial, ValuationPlan};
use crate::state::OneState;
use crate::upgrade::{PieceType, Upgrade, piece_index_to_type, piece_type_to_usize};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationOutput {
    pub special_budget: i64,
    pub table: MaterialTable,
    pub materials: Vec<OneMaterial>, // row order
    pub plans: Vec<ValuationPlan>,
    pub optimizer_plan: usize, // index into plans
    /// bands[plan][row], derived from materials and plans
    #[serde(skip)]
    pub bands: Vec<Vec<Vec<(f64, f64)>>>,
    pub test_case: i64,
    pub juice_info: JuiceInfo,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct OneUpgradeInput {
    pub piece_index: usize,
    pub upgrade_index: usize,
    pub is_normal_honing: bool,
    pub starting_artisan: Option<f64>,
    pub starting_num_taps: Option<usize>,
    pub state: Option<Vec<OneState>>,
    pub unlocked: bool,
    pub adv_progress: Option<(usize, usize, bool, bool)>,
    pub double_balls: bool,
}

impl PreparationOutput {
    pub fn initialize(
        material_labels: Vec<String>,
        keyed_materials: AHashMap<String, OneMaterial>,
        plans: Vec<ValuationPlan>,
        optimizer_plan: usize,
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
            get_event_adjusted_juice_info(&BASE_JUICE_INFOS[tier], express_event);
        let table = MaterialTable::new(material_labels, juice_info.num_juice_avail);
        let materials: Vec<OneMaterial> = table.in_row_order(&keyed_materials);
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
        assert!(optimizer_plan < plans.len());
        let bands: Vec<Vec<Vec<(f64, f64)>>> = plans
            .iter()
            .map(|plan| materials.iter().map(|mat| plan.bands(mat)).collect())
            .collect();
        let out: PreparationOutput = Self {
            table,
            materials,
            plans,
            optimizer_plan,
            bands,
            special_budget,
            test_case: -1,
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
        piece_index,
        upgrade_index,
        is_normal_honing,
        starting_artisan,
        starting_num_taps,
        state,
        unlocked,
        adv_progress,
        double_balls,
    } in upgrade_info
    {
        let piece_type: PieceType = piece_index_to_type(piece_index);
        let piece_type_usize: usize = piece_type_to_usize(piece_type);
        let relevant_cost = get_data(express_event, tier, !is_normal_honing, piece_type, false);
        let relevant_unlock = get_data(express_event, tier, !is_normal_honing, piece_type, true);
        let this_cost = &Vec::from_iter(
            (0..NUM_BASE_MATS).map(|cost_type| relevant_cost[cost_type][upgrade_index]),
        );
        let this_unlock = &Vec::from_iter(
            (0..NUM_BASE_MATS).map(|cost_type| relevant_unlock[cost_type][upgrade_index]),
        );
        let this_unlocked: bool = unlocked;
        let this_state_given: Vec<OneState> = state.unwrap_or(Vec::new());

        if is_normal_honing {
            let special_cost: i64 = special_leap_cost[piece_type_usize][upgrade_index];
            let event_artisan_rate: f64 = artisan_rate_arr[piece_type_usize][upgrade_index];
            let starting_artisan: f64 = starting_artisan.unwrap();
            let starting_num_taps: usize = starting_num_taps.unwrap_or(0);
            out.push(Upgrade::new_normal(
                normal_hone_chances[piece_type_usize][upgrade_index],
                this_cost,
                special_cost,
                piece_index,
                event_artisan_rate,
                upgrade_index,
                juice_info,
                starting_artisan,
                starting_num_taps,
                this_state_given,
                this_unlocked,
                this_unlock,
                event_extra_arr[piece_type_usize][upgrade_index],
            ));
        } else {
            let this_adv_progress: (usize, usize, bool, bool) = adv_progress.unwrap();

            out.push(Upgrade::new_adv(
                this_cost,
                piece_index,
                upgrade_index,
                this_unlock,
                this_unlocked,
                this_adv_progress,
                double_balls,
                juice_info,
                adv_cache,
                this_state_given,
            ));
        }
    }

    out
}
