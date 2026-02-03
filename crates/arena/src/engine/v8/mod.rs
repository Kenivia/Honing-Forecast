mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub const NOTES: &str = "v8, v7 but with 2x max iter (stays at low temp for significantly longer) and does not take average scaler";
