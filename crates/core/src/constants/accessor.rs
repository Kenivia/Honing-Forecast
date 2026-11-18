use crate::upgrade::PieceType::{self, Armor, Vambrace, Weapon};

use super::constants::{DATA, RATE_SCALE};
use crate::honing_utils::update_normal::Tap;

/// the game's tap rule of a normal upgrade, its tables being out of 10,000
pub fn get_tap(
    express_event: bool,
    tier: usize,
    piece_type_usize: usize,
    upgrade_index: usize,
) -> Tap {
    let data = &DATA[tier];
    let at = |table: &Vec<Vec<i64>>| table[piece_type_usize][upgrade_index] * (RATE_SCALE / 10000);
    Tap {
        base: at(&data.NORMAL_CHANCES),
        bonus: at(&data.NORMAL_FAIL_BONUS),
        bonus_max: at(&data.NORMAL_FAIL_BONUS_MAX),
        event: if express_event {
            at(&data.EVENT_NORMAL_EXTRA_CHANCE)
        } else {
            0
        },
        threshold: at(&data.ARTISAN_THRESHOLD),
        artisan_percent: if express_event {
            data.EVENT_ARTISAN_MULTIPLIER[piece_type_usize][upgrade_index]
        } else {
            100
        },
    }
}

pub fn get_special_leap_cost(tier: usize) -> Vec<Vec<i64>> {
    DATA[tier].SPECIAL_LEAPS_COST.clone()
}

/// the event's cost multiplier per material and upgrade, all 1 without the event
pub fn get_multiplier(
    express_event: bool,
    tier: usize,
    is_adv: bool,
    is_unlock: bool,
) -> Vec<Vec<f64>> {
    match (is_adv, express_event, is_unlock) {
        (_, false, _) => vec![vec![1.0_f64; 25]; 7],
        (false, true, false) => DATA[tier].EVENT_NORMAL_COST_MULTIPLIER.clone(),
        (false, true, true) => DATA[tier].EVENT_NORMAL_UNLOCK_MULTIPLIER.clone(),
        (true, true, false) => DATA[tier].EVENT_ADV_COST_MULTIPLIER.clone(),
        (true, true, true) => DATA[tier].EVENT_ADV_UNLOCK_MULTIPLIER.clone(),
    }
}

pub fn get_data(
    express_event: bool,
    tier: usize,
    is_adv: bool,
    piece_type: PieceType,
    is_unlock: bool,
) -> Vec<Vec<f64>> {
    let base = match (is_adv, piece_type, is_unlock) {
        (false, Armor, false) => &DATA[tier].NORMAL_ARMOR_COST,
        (false, Armor, true) => &DATA[tier].NORMAL_ARMOR_UNLOCK,
        (false, Weapon, false) => &DATA[tier].NORMAL_WEAPON_COST,
        (false, Weapon, true) => &DATA[tier].NORMAL_WEAPON_UNLOCK,
        (false, Vambrace, false) => &DATA[tier].NORMAL_VAMBRACE_COST,
        (false, Vambrace, true) => &DATA[tier].NORMAL_VAMBRACE_UNLOCK,
        (true, Armor, false) => &DATA[tier].ADV_ARMOR_COST,
        (true, Armor, true) => &DATA[tier].ADV_ARMOR_UNLOCK,
        (true, Weapon, false) => &DATA[tier].ADV_WEAPON_COST,
        (true, Weapon, true) => &DATA[tier].ADV_WEAPON_UNLOCK,
        _ => &vec![],
    };

    let multiplier_arr = &get_multiplier(express_event, tier, is_adv, is_unlock);
    let mut result = vec![vec![0.0_f64; if is_adv { 4 } else { 25 }]; 7];
    for (cost_type, row) in base.iter().enumerate() {
        for (u_index, base_val) in row.iter().enumerate() {
            let multiplier = multiplier_arr[cost_type][u_index];
            result[cost_type][u_index] = (base_val * multiplier).ceil();
        }
    }
    result
}
