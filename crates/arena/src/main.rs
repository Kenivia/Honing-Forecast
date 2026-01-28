use chrono::Local;
use hf_arena::engine::{NOTES, solve};
use hf_arena::parse_test_cases::parse_payload_jsons;

use hf_core::monte_carlo::monte_carlo_wrapper;

use hf_core::performance::{Performance, PerformanceToWrite};
use hf_core::saddlepoint_approximation::average::DEBUG_AVERAGE;

use hf_core::state_bundle::StateBundle;

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions, remove_file};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::path::Path;
use std::time::Instant;

static NUM_TESTS_TO_RUN: i64 = if DEBUG_AVERAGE { 1 } else { 5 };
static MONTE_CARLO_COUNT: usize = 1_000_000;
static METRICS: [(&str, i64); 2] = [("SA", 0), ("Avg", 1)];
#[derive(Debug, Serialize)]
struct Header {
    version: String,
    build_time: String,
    notes: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Output {
    test_case: String,
    metric_type: String,
    trial_num: i64,
    wall_time: f64,
    best: f64,
    performance: PerformanceToWrite,
    best_state_performance: PerformanceToWrite,
    state: String,
    seed: u64,
    time_finished: String,
    prob_leftover: Vec<f64>,
}

// Include the generated-file as a separate module
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn current_time_string() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

fn write_jsonl<T: Serialize>(data: &T, file_name: &String) -> Result<(), Error> {
    let file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)?;
    let mut line = serde_json::to_string(data).expect("Serialization failed");
    line.push('\n');
    // 3. Write and Force Sync
    let mut writer = BufWriter::new(file);
    writer.write_all(line.as_bytes())?;
    writer.flush()?; // Clears the internal Rust buffer

    // 4. Critical for SLURM: Sync to physical storage
    writer.into_inner()?.sync_all()?;

    Ok(())
}
fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(16)
        .build_global()
        .unwrap();
    let job_id: String = env::var("SLURM_JOB_ID").unwrap_or_else(|_| "local".to_string());
    let task_id: String = env::var("SLURM_ARRAY_TASK_ID").unwrap_or_else(|_| "0".to_string());
    let file_name: String = format!(
        "Result_{}_{}_{}.jsonl",
        built_info::FEATURES_LOWERCASE_STR.to_string(),
        job_id,
        task_id,
        // current_time_string().replace(":", "-"),
    );

    let mut seen_tests: HashMap<(String, String), i64> = HashMap::new();
    let mut at_least_one_line: bool = false;
    if Path::new(&file_name).exists() {
        let file: File = File::open(&file_name).unwrap(); // this really shouldnt go wrong 
        let reader: BufReader<File> = BufReader::new(file);
        for line in reader.lines().skip(1) {
            at_least_one_line = true;
            let out: Output = serde_json::from_str::<Output>(
                &line.expect("Failed to parse existing result file"),
            )
            .expect("Failed to parse existing result file");
            *seen_tests
                .entry((out.test_case, out.metric_type))
                .or_insert(0) += 1;
        }
    }
    // dbg!(&seen_tests);
    if !at_least_one_line {
        if Path::new(&file_name).exists() {
            remove_file(&file_name).expect("Failed to delete empty file");
        }
        let header: Header = Header {
            version: built_info::FEATURES_LOWERCASE_STR.to_string(),
            build_time: current_time_string(),
            notes: NOTES.to_string(),
        };
        write_jsonl(&header, &file_name).expect("Failed to write to result file");
    }

    let mut seed_rng: ThreadRng = rand::rng();

    let test_cases: Vec<(String, StateBundle, Vec<bool>)> =
        parse_payload_jsons(Path::new("test_payloads_bloated"));

    for current_trial in 0..NUM_TESTS_TO_RUN {
        for (test_case_name, state_bundle, tests_to_run) in test_cases.iter() {
            for (index, (metric_type_str, metric_type)) in METRICS.iter().enumerate() {
                if !tests_to_run[index] {
                    continue;
                }
                let metric_type_string = metric_type_str.to_string();
                let mut instant: Instant = Instant::now();
                let key = (test_case_name.clone(), metric_type_string.clone());
                if seen_tests.contains_key(&key) && seen_tests[&key] >= current_trial {
                    continue;
                }
                let seed: u64 = seed_rng.next_u64();
                // let seed: u64 = 886717209566745136;
                let mut rng: StdRng = StdRng::seed_from_u64(seed);

                let trial_num = seen_tests.entry(key.clone()).or_insert(0);
                *trial_num += 1;
                println!("Test case {:?} trial {}", key, trial_num);

                let mut state_performance: Performance = Performance::new();
                let mut state_bundle: StateBundle = solve(
                    &mut rng,
                    *metric_type,
                    state_bundle.clone(),
                    &mut state_performance,
                );

                // Call metric on best state to get standalone performance metrics
                let mut best_state_performance: Performance = Performance::new();
                let _ = state_bundle.metric_router(*metric_type, &mut best_state_performance);

                let output: Output = Output {
                    test_case: test_case_name.clone(),
                    trial_num: *trial_num,
                    wall_time: instant.elapsed().as_secs_f64(),

                    best: state_bundle.metric,
                    state: state_bundle.encode_all(),
                    seed,
                    time_finished: current_time_string(),
                    prob_leftover: state_bundle.compute_leftover_probs(),
                    metric_type: metric_type_string.clone(),
                    performance: state_performance.to_write(),
                    best_state_performance: best_state_performance.to_write(),
                };

                instant = Instant::now();
                if *trial_num == 1 {
                    let (prob_leftover, success_rate, average_rate) =
                        monte_carlo_wrapper(MONTE_CARLO_COUNT, &mut state_bundle, &mut rng);
                    let dummy_performance = Performance::new();
                    let verification_output: Output = Output {
                        test_case: test_case_name.clone(),
                        trial_num: 0,
                        wall_time: instant.elapsed().as_secs_f64(),

                        best: if metric_type_string == "SA" || metric_type_string == "brute" {
                            success_rate
                        } else {
                            average_rate
                        },
                        state: state_bundle.encode_all(),
                        seed,
                        time_finished: current_time_string(),
                        prob_leftover,
                        metric_type: "MC_".to_string() + &metric_type_string,
                        performance: dummy_performance.to_write(),
                        best_state_performance: dummy_performance.to_write(),
                    };
                    write_jsonl(&verification_output, &file_name)
                        .expect("Failed to write to result file");
                }
                write_jsonl(&output, &file_name).expect("Failed to write to result file");

                // if case already ran, skip it (maybe add a flag to rerun)
                // otherwise, call solve, write results after each solve call
                //
            }
        }
    }
}
