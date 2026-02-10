//! Rudimentary runner for testing the engines on test cases
use chrono::Local;
use hf_arena::parse_test_cases::parse_payload_jsons;
use hf_core::engine::ACTIVE_FEATURE;
use hf_core::engine::{NOTES, solve};
use hf_core::monte_carlo::monte_carlo_wrapper;
use hf_core::performance::{Performance, PerformanceToWrite};
use hf_core::saddlepoint_approximation::average::DEBUG_AVERAGE;
use hf_core::state_bundle::StateBundle;
use rand::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::{File, OpenOptions, remove_file};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::path::Path;
use std::thread::available_parallelism;
use std::time::Instant;

const NUM_TESTS_TO_RUN: i64 = if DEBUG_AVERAGE { 1 } else { 5 };
const MONTE_CARLO_COUNT: usize = 1_000_000;
const METRICS: [(&str, i64); 2] = [("SA", 0), ("Avg", 1)];

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
    let args: Vec<String> = env::args().collect();
    let payload_path_string = if args.len() > 1 {
        args[1].clone()
    } else {
        eprintln!("Usage: {} <path_to_payloads>", args[0]);
        std::process::exit(1);
    };
    let payload_path = Path::new(&payload_path_string);
    let payload_name = payload_path.file_name().unwrap().to_str().unwrap();

    let thread_num = available_parallelism()
        .unwrap()
        .get()
        .saturating_sub(2)
        .max(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_num)
        .build_global()
        .unwrap();
    println!("Using {} threads", thread_num);
    let job_id: String = env::var("SLURM_JOB_ID").unwrap_or_else(|_| "local".to_string());
    let task_id: String = env::var("SLURM_ARRAY_TASK_ID").unwrap_or_else(|_| "0".to_string());
    let file_name: String = format!(
        "./crates/arena/Results/{}_{}_{}_{}.jsonl",
        ACTIVE_FEATURE.replace("default, ", "").to_owned(),
        job_id,
        task_id,
        payload_name // current_time_string().replace(":", "-"),
    );

    let mut seen_tests: HashSet<(String, String, i64)> = HashSet::new();
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
            seen_tests.insert((out.test_case, out.metric_type, out.trial_num));
        }
    }
    // dbg!(&seen_tests);
    if !at_least_one_line {
        if Path::new(&file_name).exists() {
            remove_file(&file_name).expect("Failed to delete empty file");
        }
        let header: Header = Header {
            version: ACTIVE_FEATURE.replace("default, ", "").to_owned(),
            build_time: current_time_string(),
            notes: NOTES.to_string(),
        };
        write_jsonl(&header, &file_name)
            .unwrap_or_else(|_| panic!("Failed to write to result file {}", file_name));
    }
    // dbg!(&seen_tests);

    let test_cases: Vec<(String, StateBundle, Vec<bool>)> =
        parse_payload_jsons(Path::new(&payload_path_string));

    if test_cases.len() == 0 {
        panic!(
            "No payload jsons found in {:?}, does the folder exist? {:?} ",
            args[1],
            Path::new(&payload_path_string)
        )
    }
    let mut zipped_test_cases: Vec<(String, StateBundle, String, i64, i64)> = Vec::new();
    for trial_num in 1..=NUM_TESTS_TO_RUN {
        for z in test_cases.iter() {
            let (test_case_name, state_bundle, tests_to_run) = z;
            for (index, (metric_type_str, metric_type_num)) in METRICS.iter().enumerate() {
                let key = (
                    test_case_name.clone(),
                    metric_type_str.to_string(),
                    trial_num,
                );
                if !tests_to_run[index] || seen_tests.contains(&key) {
                    continue;
                }
                // dbg!(key);
                zipped_test_cases.push((
                    test_case_name.clone(),
                    state_bundle.clone(),
                    metric_type_str.to_string(),
                    *metric_type_num,
                    trial_num,
                ));
            }
        }
    }
    // for current_trial in{
    zipped_test_cases.par_iter().for_each(
        |(test_case_name, state_bundle, metric_type_string, metric_type_num, trial_num)| {
            let mut seed_rng: ThreadRng = rand::rng();

            let mut instant: Instant = Instant::now();
            let key = (test_case_name.clone(), metric_type_string.clone());
            // if seen_tests.contains_key(&key) && seen_tests[&key] >= *current_trial {
            //     continue;
            // }
            let seed: u64 = seed_rng.next_u64();
            // let seed: u64 = 886717209566745136;
            let mut rng: StdRng = StdRng::seed_from_u64(seed);

            // let trial_num = seen_tests.entry(key.clone()).or_insert(0);
            // *trial_num += 1;
            println!(
                "Version {} Test case {:?} trial {}",
                ACTIVE_FEATURE, key, trial_num
            );

            let mut state_performance: Performance = Performance::new();
            let mut this_state_bundle = state_bundle.clone();
            this_state_bundle.metric_type = *metric_type_num;
            let mut state_bundle: StateBundle =
                solve(&mut rng, this_state_bundle, &mut state_performance);

            // Call metric on best state to get standalone performance metrics
            let mut best_state_performance: Performance = Performance::new();
            let _ = state_bundle.metric_router(&mut best_state_performance);

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
                    metric_type: "MC_".to_string() + metric_type_string,
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
        },
    );
    // }
}
