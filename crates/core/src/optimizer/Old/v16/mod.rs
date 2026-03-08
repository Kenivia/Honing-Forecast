mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;

pub const NOTES: &str = "v16, based on v11";
