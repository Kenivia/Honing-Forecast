use crate::upgrade::PieceType::{self, Armor, Vambrace, Weapon};

use super::constants::DATA;

pub fn get_event_extra_chance(express_event: bool, tier: usize) -> Vec<Vec<f64>> {
    if !express_event {
        return vec![vec![0.0; 25]; 3];
    }
    DATA[tier].EVENT_NORMAL_EXTRA_CHANCE.clone()
}
pub fn get_artisan(express_event: bool, tier: usize) -> Vec<Vec<f64>> {
    if !express_event {
        return vec![vec![1.0; 25]; 3];
    }

    DATA[tier].EVENT_ARTISAN_MULTIPLIER.clone()
}

pub fn get_special_leap_cost(tier: usize) -> Vec<Vec<i64>> {
    DATA[tier].SPECIAL_LEAPS_COST.clone()
}

pub fn get_normal_hone_chances(tier: usize) -> Vec<Vec<f64>> {
    DATA[tier].NORMAL_CHANCES.clone()
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

    let multiplier_arr = match (is_adv, express_event, is_unlock) {
        (false, false, false) => &vec![vec![1.0_f64; 25]; 7],
        (false, false, true) => &vec![vec![1.0_f64; 25]; 7],
        (false, true, false) => &DATA[tier].EVENT_NORMAL_COST_MULTIPLIER,
        (false, true, true) => &DATA[tier].EVENT_NORMAL_UNLOCK_MULTIPLIER,
        (true, false, false) => &vec![vec![1.0_f64; 25]; 7],
        (true, false, true) => &vec![vec![1.0_f64; 25]; 7],
        (true, true, false) => &DATA[tier].EVENT_ADV_COST_MULTIPLIER,
        (true, true, true) => &DATA[tier].EVENT_ADV_UNLOCK_MULTIPLIER,
    };

    let mut result = vec![vec![0.0_f64; if is_adv { 4 } else { 25 }]; 7];
    for (cost_type, row) in base.iter().enumerate() {
        for (u_index, base_val) in row.iter().enumerate() {
            let multiplier = multiplier_arr[cost_type][u_index];
            result[cost_type][u_index] = (base_val * multiplier).ceil();
        }
    }
    result
}
