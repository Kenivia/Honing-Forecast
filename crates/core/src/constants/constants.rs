use crate::constants::{
    data::{Data, RawData},
    juice_info::JuiceInfo,
};
use once_cell::sync::Lazy;

pub const TEST_PAYLOAD_PATH: &str = "/test_cases/payloads";
pub const FLOAT_TOL: f64 = 1e-9; // -12 is known to cause problems with brute 
pub const SPECIAL_TOL: f64 = 1e-7;
pub const MONTE_CARLO_CONFIDENCE: f64 = 0.999;
pub const MONTE_CARLO_PRECISION: f64 = 0.001; // percentage error
pub const SIMULATED_ANNEALING_DIFF_TOL: f64 = 1e-2;
pub const MONTE_CARLO_COUNT: usize = 1_000_000;
pub const BUCKET_COUNT: usize = 50;

pub const DATA: Lazy<Vec<Data>> = Lazy::new(|| {
    let mut out = Vec::new();

    let json = include_str!("./T4 Feb 2026.json");
    let raw_data: RawData = serde_json::from_str(json).unwrap();
    out.push(Data::from(raw_data));

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
