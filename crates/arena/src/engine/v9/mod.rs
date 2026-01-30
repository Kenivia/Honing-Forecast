mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v9, v8 but with 4x bigger batch size";
