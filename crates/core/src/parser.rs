//! What is derived once from a payload: the prepared inputs and the upgrades.
use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::constants::accessor::{get_data, get_multiplier, get_special_leap_cost, get_tap};
use crate::constants::juice_info::JuiceInfo;
use crate::honing_utils::chest::Chests;
use crate::honing_utils::materials::{MaterialTable, NUM_BASE_MATS, OneMaterial, ValuationPlan};
use crate::optimizer::state::OneState;
use crate::upgrade::{Upgrade, piece_index_to_type, piece_type_to_usize};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

pub type AdvCache = AHashMap<AdvConfig, AdvDistTriplet>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationOutput {
    /// with what the chests give, see apply_chest_state
    pub special_budget: i64,
    pub base_special_budget: i64,
    pub table: MaterialTable,
    pub materials: Vec<OneMaterial>, // row order
    /// before any chest is opened
    pub base_materials: Vec<OneMaterial>,
    pub chests: Chests,
    /// what a chest's special leaps are called
    pub special_label: String,
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

/// What every upgrade of a payload is built with.
pub struct UpgradeCtx<'a> {
    pub express_event: bool,
    pub tier: usize,
    pub juice_info: &'a JuiceInfo,
    pub special_leap_cost: Vec<Vec<i64>>,
}

impl<'a> UpgradeCtx<'a> {
    pub fn new(express_event: bool, tier: usize, juice_info: &'a JuiceInfo) -> Self {
        UpgradeCtx {
            express_event,
            tier,
            juice_info,
            special_leap_cost: get_special_leap_cost(tier),
        }
    }
}

/// The given cache, less the entries no upgrade of this payload starts from.
pub fn kept_adv_cache(given: Option<AdvCache>, upgrades: &[OneUpgradeInput]) -> AdvCache {
    let mut cache = given.unwrap_or_default();
    cache.retain(|key, _| {
        upgrades.iter().any(|upgrade| {
            let Some((start_xp, start_balls, next_free, next_big)) = upgrade.adv_progress else {
                return false;
            };
            !upgrade.is_normal_honing
                && start_xp == key.start_xp
                && start_balls == key.start_balls
                && next_free == key.next_free
                && next_big == key.next_big
                && upgrade.double_balls == key.double_balls
                && ((upgrade.upgrade_index >= 2) == key.is_30_40)
        })
    });
    cache
}

impl Upgrade {
    pub fn from_input(
        input: OneUpgradeInput,
        ctx: &UpgradeCtx,
        adv_cache: &mut AdvCache,
    ) -> Upgrade {
        let OneUpgradeInput {
            piece_index,
            upgrade_index,
            is_normal_honing,
            starting_artisan,
            starting_num_taps,
            state,
            unlocked,
            adv_progress,
            double_balls,
        } = input;
        let UpgradeCtx {
            express_event,
            tier,
            juice_info,
            ..
        } = *ctx;
        let piece_type = piece_index_to_type(piece_index);
        let piece_type_usize = piece_type_to_usize(piece_type);
        let column = |table: Vec<Vec<f64>>| -> Vec<f64> {
            (0..NUM_BASE_MATS)
                .map(|cost_type| table[cost_type][upgrade_index])
                .collect()
        };
        let is_adv = !is_normal_honing;
        let cost = column(get_data(express_event, tier, is_adv, piece_type, false));
        let unlock = column(get_data(express_event, tier, is_adv, piece_type, true));
        let state = state.unwrap_or_default();

        let mut upgrade = if is_normal_honing {
            Upgrade::new_normal(
                get_tap(express_event, tier, piece_type_usize, upgrade_index),
                &cost,
                ctx.special_leap_cost[piece_type_usize][upgrade_index],
                piece_index,
                upgrade_index,
                juice_info,
                starting_artisan.unwrap(),
                starting_num_taps.unwrap_or(0),
                state,
                unlocked,
                &unlock,
            )
        } else {
            Upgrade::new_adv(
                &cost,
                piece_index,
                upgrade_index,
                &unlock,
                unlocked,
                adv_progress.unwrap(),
                double_balls,
                juice_info,
                adv_cache,
                state,
            )
        };
        upgrade.cost_multipliers = column(get_multiplier(express_event, tier, is_adv, false));
        upgrade
    }
}
