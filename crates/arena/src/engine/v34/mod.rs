mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v34, v31 but with 1.02 special affinity ";
