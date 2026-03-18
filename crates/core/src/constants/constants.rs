use crate::constants::{
    data::{Data, RawData},
    juice_info::JuiceInfo,
};
use once_cell::sync::Lazy;

pub const TEST_PAYLOAD_PATH: &str = "/test_cases/payloads";
pub const FLOAT_TOL: f64 = 1e-9; // -12 is known to cause problems with brute 
pub const IGNORE_PROB_TOL: f64 = 1e-12; // mostly for adv honing, 10 is known to cause problems, using 12 to be safe
pub const SPECIAL_TOL: f64 = 1e-7;
pub const BUCKET_COUNT: usize = 50;

// testing thresholds
pub const MONTE_CARLO_CONFIDENCE: f64 = 0.999;
pub const MONTE_CARLO_PRECISION: f64 = 0.001; // percentage error
pub const SIMULATED_ANNEALING_DIFF_TOL: f64 = 1e-2; // run to run variance, this may need to change if optimizer is changed to be more varied
pub const MONTE_CARLO_COUNT: usize = 1_000_000; // per batch size, this is automatically increased until confidence is reached

pub const DATA: Lazy<Vec<Data>> = Lazy::new(|| {
    let mut out = Vec::new();

    out.push(Data::from(
        serde_json::from_str::<RawData>(include_str!("./T4 Feb 2026.json")).unwrap(),
    ));

    out.push(Data::from(
        serde_json::from_str::<RawData>(include_str!("./Serca March 2026.json")).unwrap(),
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
