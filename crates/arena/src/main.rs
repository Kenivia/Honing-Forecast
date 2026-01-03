use chrono::Local;
use hf_arena::engine::{NOTES, solve};
use hf_arena::parse_test_cases::parse_csv;
use hf_core::average::{
    add_up_golds, apply_price_leftovers, apply_price_naive, average_gold_wrapper,
};
use hf_core::brute::brute_naive_wrapper;
use hf_core::monte_carlo::monte_carlo_data;
use hf_core::parser::PreparationOutput;
use hf_core::saddlepoint_approximation::normal_sa::{compute_leftover_probs, honing_sa_metric};
use hf_core::state::StateBundle;

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions, remove_file};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::path::Path;
use std::time::Instant;

static NUM_TESTS_TO_RUN: i64 = 10; // TODO this should be replaced by statistical tests like fishtest eventually
static MONTE_CARLO_COUNT: usize = 1_000_000;
type EvalFn = fn(&mut StateBundle, &mut PreparationOutput, &mut i64) -> f64;

static METRICS: [(&str, EvalFn); 3] = [
    ("SA", honing_sa_metric),
    ("Avg", average_gold_wrapper),
    ("Brute", brute_naive_wrapper),
];

#[derive(Debug, Serialize)]
struct Header {
    version: String,
    build_time: String,
    notes: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Output {
    test_case: i64,
    metric_type: String,
    trial_num: i64,
    wall_time: f64,
    states_evaled: i64,
    best: f64,
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
    let job_id: String = env::var("SLURM_JOB_ID").unwrap_or_else(|_| "local".to_string());
    let task_id: String = env::var("SLURM_ARRAY_TASK_ID").unwrap_or_else(|_| "0".to_string());
    let file_name: String = format!(
        "Result_{}_{}_{}.jsonl",
        built_info::FEATURES_LOWERCASE_STR.to_string(),
        job_id,
        task_id
    );

    let mut seen_tests: HashMap<(i64, String), i64> = HashMap::new();
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
    dbg!(&seen_tests);
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

    let mut test_cases: Vec<(PreparationOutput, Vec<bool>)> =
        parse_csv(Path::new("bloated_test_cases.csv"));

    for _ in 0..NUM_TESTS_TO_RUN {
        for (prep_output, tests_to_run) in test_cases.iter_mut() {
            for (index, (metric_type_str, metric_function)) in METRICS.iter().enumerate() {
                if !tests_to_run[index] {
                    continue;
                }
                let metric_type = metric_type_str.to_string();
                let mut instant: Instant = Instant::now();
                let key = (prep_output.test_case, metric_type.clone());
                if seen_tests.contains_key(&key) && seen_tests[&key] >= NUM_TESTS_TO_RUN {
                    continue;
                }
                let seed: u64 = seed_rng.next_u64();
                let mut rng: StdRng = StdRng::seed_from_u64(seed);
                let mut states_evaled: i64 = 0;
                println!("Test case {:?}", key);

                let mut state_bundle: StateBundle =
                    solve(prep_output, &mut rng, &mut states_evaled, metric_function);

                let trial_num = seen_tests.entry(key).or_insert(0);
                *trial_num += 1;
                let output: Output = Output {
                    test_case: prep_output.test_case,
                    trial_num: *trial_num,
                    wall_time: instant.elapsed().as_secs_f64(),
                    states_evaled,
                    best: state_bundle.metric,
                    state: state_bundle.encode_all(),
                    seed,
                    time_finished: current_time_string(),
                    prob_leftover: compute_leftover_probs(prep_output, &mut state_bundle),
                    metric_type: metric_type.clone(),
                };

                instant = Instant::now();
                if *trial_num == 1 {
                    let (cost_data, juice_data) = monte_carlo_data(
                        MONTE_CARLO_COUNT,
                        &mut state_bundle,
                        prep_output,
                        &mut rng,
                    );
                    let mut success_count: i64 = 0;
                    let mut average: f64 = 0.0;
                    let mut leftover_counts: Vec<i64> =
                        vec![0; 7 + prep_output.juice_info.one_gold_cost.len() * 2];
                    for (r_index, row) in cost_data.iter().enumerate() {
                        let float_row: Vec<f64> = row.iter().map(|x| *x as f64).collect();
                        let float_juice: Vec<(f64, f64)> = juice_data[r_index]
                            .iter()
                            .map(|x| (x.0 as f64, x.1 as f64))
                            .collect();
                        let (mats_gold_leftover, juice_gold_leftover) =
                            apply_price_leftovers(&float_row, &float_juice, prep_output);

                        let (mats_gold_naive, juice_gold_naive) =
                            apply_price_naive(&float_row, &float_juice, prep_output);
                        let gold_eqv_naive: f64 = add_up_golds(&mats_gold_naive, &juice_gold_naive);
                        if gold_eqv_naive > 0.0 {
                            success_count += 1;
                        }
                        average += add_up_golds(&mats_gold_leftover, &juice_gold_leftover);

                        let mut leftover_index: usize = 0;
                        for (index, mat) in row.iter().enumerate() {
                            if *mat < prep_output.budgets[index] {
                                leftover_counts[leftover_index] += 1;
                            }
                            leftover_index += 1;
                        }
                        for (index, juice) in juice_data[r_index].iter().enumerate() {
                            if juice.0 < prep_output.juice_books_owned[index].0 {
                                leftover_counts[leftover_index] += 1;
                            }
                            leftover_index += 1;
                        }
                        for (index, juice) in juice_data[r_index].iter().enumerate() {
                            if juice.1 < prep_output.juice_books_owned[index].1 {
                                leftover_counts[leftover_index] += 1;
                            }
                            leftover_index += 1;
                        }
                    }

                    let prob_leftover: Vec<f64> = leftover_counts
                        .into_iter()
                        .map(|x| x as f64 / MONTE_CARLO_COUNT as f64)
                        .collect();
                    let verification_output: Output = Output {
                        test_case: prep_output.test_case,
                        trial_num: 0,
                        wall_time: instant.elapsed().as_secs_f64(),
                        states_evaled: 1,
                        best: if metric_type == "SA" || metric_type == "brute" {
                            success_count as f64 / MONTE_CARLO_COUNT as f64
                        } else {
                            average / MONTE_CARLO_COUNT as f64
                        },
                        state: state_bundle.encode_all(),
                        seed,
                        time_finished: current_time_string(),
                        prob_leftover,
                        metric_type: "MC_".to_string() + &metric_type,
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
