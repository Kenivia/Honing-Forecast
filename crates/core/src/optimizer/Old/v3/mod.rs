mod simulated_annealing;
#[allow(unused_imports)]
pub use simulated_annealing::solve;

pub const NOTES: &str =
    "v3, v2 but temperature cutoff occurs earlier and iteration per temp is higher";
