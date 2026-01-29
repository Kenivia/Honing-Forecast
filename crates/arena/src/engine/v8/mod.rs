mod simulated_annealing;
pub use simulated_annealing::solve;
mod core;
mod one_batch;
mod scaler;
pub static NOTES: &str = "v8, v7 but with 2x iters per temp (just more states explored) and 2x max iter (stays at low temp for significantly longer)";
