mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;
mod scaler;
pub const NOTES: &str = "v24, v23 with adaptive special affinity";
