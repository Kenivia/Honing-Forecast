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

pub type TreatmentsType = [usize; 4];
pub const UI_TREATMENTS: [TreatmentsType; 2] = [
    [0, 1, 1, 3], // char-bound <bound| roster <bound| tradable <bound| market
    [0, 1, 2, 3], //  char-bound <bound| roster <bound| tradable <trade| market
];

pub const TEST_PAYLOAD_PATH: &str = "/test_cases/payloads";
pub const FLOAT_TOL: f64 = 1e-9; // -12 is known to cause problems with brute
pub const IGNORE_PROB_TOL: f64 = 1e-14; // mostly for adv honing, 10 is known to cause problems, 12 shoudl be fine? using 14 to be giga safe
pub const SPECIAL_TOL: f64 = 1e-7;
pub const BUCKET_COUNT: usize = 50;

// testing thresholds
pub const MONTE_CARLO_CONFIDENCE: f64 = 0.999;
pub const MONTE_CARLO_PRECISION: f64 = 0.001; // percentage error
pub const SIMULATED_ANNEALING_DIFF_TOL: f64 = 1e-3; // run to run variance, this may need to change if optimizer is changed to be more varied
pub const MONTE_CARLO_COUNT: usize = 1_000_000; // per batch size, this is automatically increased until confidence is reached

pub const DATA: Lazy<Vec<Data>> = Lazy::new(|| {
    let mut out = Vec::new();

    out.push(Data::from(
        serde_json::from_str::<RawData>(include_str!("./T4 July 2026 (Vambrace padded).json"))
            .unwrap(),
    ));

    out.push(Data::from(
        serde_json::from_str::<RawData>(include_str!("./Serca Aug 2026 (Vambrace).json")).unwrap(),
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
