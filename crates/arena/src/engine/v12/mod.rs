mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v12, v10 but with faster annealing";
