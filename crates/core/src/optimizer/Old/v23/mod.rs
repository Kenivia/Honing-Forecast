mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;
mod scaler;
pub const NOTES: &str = "v23, based on v18, using new heuristic on neighbour funciotn";
