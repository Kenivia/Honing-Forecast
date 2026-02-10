mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub const NOTES: &str = "v7, v6 but with restarts correctly suppressed by improvements (and taking averarge of the scaler but i dont think that matters too much)";
