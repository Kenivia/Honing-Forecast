
use crate::constants::{MONTE_CARLO_CONFIDENCE, MONTE_CARLO_COUNT, MONTE_CARLO_PRECISION, SIMULATED_ANNEALING_DIFF_TOL};
use crate::engine::ACTIVE_FEATURE;
use crate::engine::{NOTES, solve};
use crate::helpers::my_pct_diff;
use crate::monte_carlo::{ verify_result_with_monte_carlo};
use crate::payload::parse_to_state_bundles;
use crate::performance::{Performance, PerformanceToWrite};
use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
use crate::state_bundle::StateBundle;
use chrono::Local;
use core::panic;
use std::f64::NAN;
use rand::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions, remove_file};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::path::Path;
use std::thread::available_parallelism;
use std::time::Instant;

const NUM_TESTS_TO_RUN: i64 = if DEBUG_AVERAGE { 1 } else { 5};
const METRICS: [(&str, i64); 1] = [("Avg", 1)];

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

pub fn run_tests(payload_path_string: String, is_test: bool) {
    let payload_path = Path::new(&payload_path_string);
    let payload_name = payload_path.file_name().unwrap().to_str().unwrap();

    let thread_num = available_parallelism()
        .unwrap()
        .get()
        .saturating_sub(1)
        .max(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_num)
        .build_global()
        .unwrap();
    println!("Using {} threads", thread_num);

    let file_name: String = if is_test {
        format!(
            "../../test_results/{}_{}.jsonl",
            ACTIVE_FEATURE.replace("default, ", "").to_owned(),
            payload_name
        )
    } else {
        format!(
            "./crates/arena/results/{}_{}.jsonl",
            ACTIVE_FEATURE.replace("default, ", "").to_owned(),
            payload_name // current_time_string().replace(":", "-"),
        )
    };

    let mut seen_tests: HashMap<(String, String, i64), f64> = HashMap::new();
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
            seen_tests.insert((out.test_case, out.metric_type, out.trial_num), out.best);
        }
    }
    // my_dbg!(&seen_tests);
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
    // my_dbg!(&seen_tests);

    let test_cases: Vec<(String, StateBundle)> = parse_to_state_bundles(Path::new(
        &(if is_test {
            "../../test_payloads".to_string()
        } else {
            payload_path_string.clone()
        }),
    ));

    if test_cases.len() == 0 {
        panic!(
            "No payload jsons found in {:?}, does the folder exist? {:?} ",
            payload_path_string,
            Path::new(&payload_path_string)
        )
    }
    let mut zipped_test_cases: Vec<(String, StateBundle, String, i64, i64)> = Vec::new();
    for trial_num in 1..=if is_test { 1 } else { NUM_TESTS_TO_RUN } {
        for z in test_cases.iter() {
            let (test_case_name, state_bundle) = z;
            for (metric_type_str, metric_type_num) in METRICS.iter() {
                let key = (
                    test_case_name.clone(),
                    metric_type_str.to_string(),
                    trial_num,
                );
                if !is_test
                    && (seen_tests.contains_key(&key)
                        || (test_case_name.contains("adv")
                            && ACTIVE_FEATURE
                                .strip_prefix('v')
                                .unwrap()
                                .parse::<i64>()
                                .unwrap()
                                < 35))
                {
                    continue;
                }
                // my_dbg!(key);
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
            let mut instant: Instant = Instant::now();
            let key = (
                test_case_name.clone(),
                metric_type_string.clone(),
                *trial_num,
            );

            let seed = if is_test {
                6942067
            } else {         let mut seed_rng: ThreadRng = rand::rng();
                seed_rng.next_u64()
            };
            let mut rng: StdRng = StdRng::seed_from_u64(seed);
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
                let  mc_result  =
                    verify_result_with_monte_carlo(state_bundle.metric, MONTE_CARLO_CONFIDENCE,  MONTE_CARLO_PRECISION * state_bundle.metric.abs(),MONTE_CARLO_COUNT,&mut state_bundle, &mut rng);
                let monte_carlo_mean = if metric_type_string == "SA" || metric_type_string == "brute" {
      panic!("havnt bothered to implement stat testing for success rates")
                } else {
                    mc_result.mean
                };
                let mut  dummy_performance = Performance::new();
                dummy_performance.states_evaluated = mc_result.samples as i64;
                let verification_output: Output = Output {
                    test_case: test_case_name.clone(),
                    trial_num: 0,
                    wall_time: instant.elapsed().as_secs_f64(),

                    best : monte_carlo_mean,
                    state: state_bundle.encode_all(),
                    seed,
                    time_finished: current_time_string(),
                 prob_leftover:   mc_result.prob_leftover.clone(),
                    metric_type: "MC_".to_string() + metric_type_string,
                    performance: dummy_performance.to_write(),
                    best_state_performance: dummy_performance.to_write(),
                };
                if !mc_result.is_match{
                    panic!(
                        "Monte Carlo and metric did not agree with confidence {}% n = {}. 
                         {} Test: {} Trial: {} MC: {} +-{}% SA: {}% My diff: {}%",
                    MONTE_CARLO_CONFIDENCE*100.0, mc_result.samples,ACTIVE_FEATURE,  test_case_name,trial_num, monte_carlo_mean,  MONTE_CARLO_PRECISION* 100.0, state_bundle.metric, my_pct_diff(monte_carlo_mean, state_bundle.metric) * 100.0
                    );
                }
             let seen = if seen_tests.contains_key(&key) {seen_tests[&key]}else {NAN};
                if is_test && seen_tests.contains_key(&key) {
       
                    if my_pct_diff(seen, state_bundle.metric) > SIMULATED_ANNEALING_DIFF_TOL {
                        panic!(
                            "Previous result and metric did not agree. {} Test: {} Trial: {} prev: {} new: {} diff w/ prev: {}%",
                            ACTIVE_FEATURE,  test_case_name,trial_num,   seen, state_bundle.metric,my_pct_diff(seen, state_bundle.metric)* 100.0
                        ); 
                        // idk why the engine is giving different results (and MC actually) with seeded rng
                    }
                } else {
                    write_jsonl(&verification_output, &file_name)
                        .expect("Failed to write to result file");
                }
                
   println!(
                        "Done {} Test: {} Trial: {} MC: {} +-{}%  n={} SA: {} diff w/ MC: {}%   prev: {} diff w/ prev: {}%",
                      ACTIVE_FEATURE,  test_case_name,trial_num, monte_carlo_mean, MONTE_CARLO_PRECISION * 100.0,  mc_result.samples, state_bundle.metric, my_pct_diff(monte_carlo_mean, state_bundle.metric) * 100.0,seen, my_pct_diff(seen, state_bundle.metric)* 100.0
                    );

            }
            else{
                        println!( 
                            "Done {} Test: {:?} Metric: {} Trial: {} ",
                            ACTIVE_FEATURE, test_case_name, metric_type_string, trial_num
                        );
            }
            if !(is_test && seen_tests.contains_key(&key)) {
                write_jsonl(&output, &file_name).expect("Failed to write to result file");
            }

    }
    );
    // }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::TEST_PAYLOAD_PATH;
    #[test]
    fn integration_test() {
        run_tests(TEST_PAYLOAD_PATH.to_string(), true);
    }
}
