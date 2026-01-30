mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v10, but no longer roll a nnumebroto mutate its always the max";
