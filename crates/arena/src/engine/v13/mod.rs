mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub const NOTES: &str = "v13, v10 but slower annealing";
