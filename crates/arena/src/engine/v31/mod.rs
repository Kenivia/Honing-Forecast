mod simulated_annealing;
pub use simulated_annealing::solve;
mod constants;
mod neighbour;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v31, based on v28 with self-crossover and multi-neighbour ";
