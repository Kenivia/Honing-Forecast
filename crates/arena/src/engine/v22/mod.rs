mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v22, v18 but with restarts scaling with progress";
