mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v29, v28 but with more preference for special (1.01)";
