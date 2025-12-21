use hf_arena::engine::solve;
use hf_arena::parse_test_cases::read_csv;
use hf_core::parser::PreparationOutputs;
use std::path::Path;

fn main() {
    let test_cases: Vec<PreparationOutputs> =
        read_csv(Path::new("test_cases.csv")).expect("Failed to read test_case.csv");

    for case in test_cases {}
}
