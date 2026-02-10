mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub const NOTES: &str = "v6, parallelsim with a rudimentary idea of genetic algorithm(top 10 list of best state that we randomly restart to), also this restart scheme is EXTREMELY AGGRESIVE(unintended bug) but lets see how it goes";
