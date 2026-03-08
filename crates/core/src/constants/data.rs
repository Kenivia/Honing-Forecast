use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct RawData {
    // this intermediate struct is to make sure that things have the right shape
    pub EVENT_NORMAL_EXTRA_CHANCE: [f64; 25],
    pub EVENT_ARTISAN_MULTIPLIER: [f64; 25],

    pub EVENT_NORMAL_COST_MULTIPLIER: [[f64; 25]; 7],
    pub EVENT_NORMAL_UNLOCK_MULTIPLIER: [[f64; 25]; 7],
    pub EVENT_ADV_UNLOCK_MULTIPLIER: [[f64; 4]; 7],
    pub EVENT_ADV_COST_MULTIPLIER: [[f64; 4]; 7],

    pub EVENT_ADV_JUICE_MULTIPLIER: Vec<(usize, usize, usize, f64)>,

    pub NORMAL_CHANCES: [f64; 25],
    pub JUICE_BOOKS_AVAIL: Vec<(usize, usize, usize, f64, i64, f64, f64)>,
    pub NORMAL_WEAPON_COST: [[f64; 25]; 7],
    pub NORMAL_ARMOR_COST: [[f64; 25]; 7],
    pub NORMAL_WEAPON_UNLOCK: [[f64; 25]; 7],
    pub NORMAL_ARMOR_UNLOCK: [[f64; 25]; 7],
    pub SPECIAL_LEAPS_COST: [[i64; 25]; 2],

    pub ADV_WEAPON_COST: [[f64; 4]; 7],
    pub ADV_ARMOR_COST: [[f64; 4]; 7],
    pub ADV_WEAPON_UNLOCK: [[f64; 4]; 7],
    pub ADV_ARMOR_UNLOCK: [[f64; 4]; 7],
}
#[allow(non_snake_case)]
pub struct Data {
    pub EVENT_NORMAL_EXTRA_CHANCE: Vec<f64>,
    pub EVENT_ARTISAN_MULTIPLIER: Vec<f64>,
    pub EVENT_NORMAL_COST_MULTIPLIER: Vec<Vec<f64>>,
    pub EVENT_NORMAL_UNLOCK_MULTIPLIER: Vec<Vec<f64>>,
    pub EVENT_ADV_UNLOCK_MULTIPLIER: Vec<Vec<f64>>,
    pub EVENT_ADV_COST_MULTIPLIER: Vec<Vec<f64>>,
    pub EVENT_ADV_JUICE_MULTIPLIER: Vec<(usize, usize, usize, f64)>,
    pub NORMAL_CHANCES: Vec<f64>,
    pub JUICE_BOOKS_AVAIL: Vec<(usize, usize, usize, f64, i64, f64, f64)>,
    pub NORMAL_WEAPON_COST: Vec<Vec<f64>>,
    pub NORMAL_ARMOR_COST: Vec<Vec<f64>>,
    pub NORMAL_WEAPON_UNLOCK: Vec<Vec<f64>>,
    pub NORMAL_ARMOR_UNLOCK: Vec<Vec<f64>>,
    pub SPECIAL_LEAPS_COST: Vec<Vec<i64>>,
    pub ADV_WEAPON_COST: Vec<Vec<f64>>,
    pub ADV_ARMOR_COST: Vec<Vec<f64>>,
    pub ADV_WEAPON_UNLOCK: Vec<Vec<f64>>,
    pub ADV_ARMOR_UNLOCK: Vec<Vec<f64>>,
}

impl From<RawData> for Data {
    fn from(r: RawData) -> Self {
        // Helper closures
        let to_vec = |arr: &[f64]| arr.to_vec();
        let to_vec2d = |arr: &[[f64; 25]]| arr.iter().map(|row| row.to_vec()).collect();
        let to_vec2d_4 = |arr: &[[f64; 4]]| arr.iter().map(|row| row.to_vec()).collect();
        let to_vec2d_i64 = |arr: &[[i64; 25]]| arr.iter().map(|row| row.to_vec()).collect();

        Self {
            EVENT_NORMAL_EXTRA_CHANCE: to_vec(&r.EVENT_NORMAL_EXTRA_CHANCE),
            EVENT_ARTISAN_MULTIPLIER: to_vec(&r.EVENT_ARTISAN_MULTIPLIER),
            EVENT_NORMAL_COST_MULTIPLIER: to_vec2d(&r.EVENT_NORMAL_COST_MULTIPLIER),
            EVENT_NORMAL_UNLOCK_MULTIPLIER: to_vec2d(&r.EVENT_NORMAL_UNLOCK_MULTIPLIER),
            EVENT_ADV_UNLOCK_MULTIPLIER: to_vec2d_4(&r.EVENT_ADV_UNLOCK_MULTIPLIER),
            EVENT_ADV_COST_MULTIPLIER: to_vec2d_4(&r.EVENT_ADV_COST_MULTIPLIER),
            EVENT_ADV_JUICE_MULTIPLIER: r.EVENT_ADV_JUICE_MULTIPLIER,
            NORMAL_CHANCES: to_vec(&r.NORMAL_CHANCES),
            JUICE_BOOKS_AVAIL: r.JUICE_BOOKS_AVAIL,
            NORMAL_WEAPON_COST: to_vec2d(&r.NORMAL_WEAPON_COST),
            NORMAL_ARMOR_COST: to_vec2d(&r.NORMAL_ARMOR_COST),
            NORMAL_WEAPON_UNLOCK: to_vec2d(&r.NORMAL_WEAPON_UNLOCK),
            NORMAL_ARMOR_UNLOCK: to_vec2d(&r.NORMAL_ARMOR_UNLOCK),
            SPECIAL_LEAPS_COST: to_vec2d_i64(&r.SPECIAL_LEAPS_COST),
            ADV_WEAPON_COST: to_vec2d_4(&r.ADV_WEAPON_COST),
            ADV_ARMOR_COST: to_vec2d_4(&r.ADV_ARMOR_COST),
            ADV_WEAPON_UNLOCK: to_vec2d_4(&r.ADV_WEAPON_UNLOCK),
            ADV_ARMOR_UNLOCK: to_vec2d_4(&r.ADV_ARMOR_UNLOCK),
        }
    }
}
