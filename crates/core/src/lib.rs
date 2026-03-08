pub mod advanced_honing;
pub mod constants;
pub mod core;
pub mod helpers;
pub mod honing_utils;
pub mod optimizer;
pub mod parser;
pub mod payload;
pub mod performance;
pub mod state_bundle;
pub mod support;
pub mod upgrade;

#[cfg(feature = "wasm")] // and this module is not in wasm because it is needed in the engine
pub mod js_interface;

#[cfg(feature = "run_tests")]
pub mod timer;
#[cfg(feature = "run_tests")] // this module is not in arena because it is also needed for tests
pub mod verification;
