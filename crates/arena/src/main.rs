use hf_core::run_tests::run_tests;
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let payload_path_string = if args.len() > 1 {
        args[1].clone()
    } else {
        eprintln!("Usage: {} <path_to_payloads>", args[0]);
        std::process::exit(1);
    };

    run_tests(payload_path_string, false);
}
