use crate::constants::{
    data::{Data, RawData},
    juice_info::JuiceInfo,
};
use once_cell::sync::Lazy;

pub const NUM_NORMAL_UPGRADES: usize = 25;
pub const NUM_ADV_UPGRADES: usize = 4;

pub const ARTISAN_MULTIPLIER: f64 = 10000.0 / 21500.0;
pub const ALLOWED_JUICE_POOLS: Lazy<Vec<Vec<Vec<usize>>>> = Lazy::new(|| {
    vec![
        vec![vec![0, 1], vec![]],
        vec![vec![0, 1], vec![]],
        vec![vec![0], vec![1], vec![]],
    ]
});

pub const NUM_PIECE_TYPES: usize = 3;

pub const FLOAT_TOL: f64 = 1e-9; // -12 is known to cause problems with brute
/// FLOAT_TOL is under the resolution of an f64 past about 8 million, so this grows with the value
pub fn float_tol(value: f64) -> f64 {
    FLOAT_TOL.max(value.abs() * 1e-12)
}
pub const IGNORE_PROB_TOL: f64 = 1e-14; // mostly for adv honing, 10 is known to cause problems, 12 shoudl be fine? using 14 to be giga safe
pub const SPECIAL_TOL: f64 = 1e-7;
pub const BUCKET_COUNT: usize = 50;

pub const DATA: Lazy<Vec<Data>> = Lazy::new(|| {
    let mut out = Vec::new();

    out.push(Data::from(
        serde_json::from_str::<RawData>(include_str!("./T4 July 2026 (Vambrace padded).json"))
            .unwrap(),
    ));

    out.push(Data::from(
        serde_json::from_str::<RawData>(include_str!("./Serca Oct 2026.json")).unwrap(),
    ));

    out
});

pub const BASE_JUICE_INFOS: Lazy<Vec<JuiceInfo>> = Lazy::new(|| {
    let mut out = Vec::new();
    for tier in 0..DATA.len() {
        out.push(JuiceInfo::new(
            &DATA[tier].JUICE_BOOKS_AVAIL,
            &DATA[tier].EVENT_ADV_JUICE_MULTIPLIER,
        ));
    }
    out
});
