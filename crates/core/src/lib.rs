pub mod advanced_honing;
pub mod constants;
pub mod core;
pub mod helpers;
pub mod materials;

pub mod honing_utils;
pub mod optimizer;
pub mod parser;
pub mod payload;
pub mod performance;
pub mod state;
pub mod state_bundle;
pub mod support;
pub mod timer;
pub mod upgrade;

#[cfg(feature = "wasm")]
pub mod histogram;

#[cfg(feature = "wasm")] // and this module is not in wasm because it is needed in the engine
pub mod js_interface;

// Monte carlo verification is parked while the material interface changes; a proper
// test suite for the core replaces it.
// #[cfg(feature = "run_tests")]
// pub mod verification;
