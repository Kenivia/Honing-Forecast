mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;
mod scaler;
pub const NOTES: &str = "v18, v17 with correct cooling stage start time";
