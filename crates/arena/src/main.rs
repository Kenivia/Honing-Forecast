use chrono::Local;
use hf_arena::engine::{NOTES, solve};
use hf_arena::parse_test_cases::parse_csv;
use hf_core::parser::PreparationOutputs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::path::Path;

static NUM_TESTS_TO_RUN: i64 = 10; // TODO this should be replaced by statistical tests like fishtest eventually

#[derive(Debug, Serialize)]
struct Header {
    version: String,
    build_time: String,
    notes: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Output {
    test_number: i64,
    trial_num: i64,
    wall_time: f64,
    states_evaled: i64,
    prob: f64,
    state: String,
    seed: i64,
    time_finished: String,
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

    let mut seen_tests: HashMap<i64, i64> = HashMap::new();

    if Path::new(&file_name).exists() {
        let file: File = File::open(&file_name).unwrap(); // this really shouldnt go wrong 
        let reader: BufReader<File> = BufReader::new(file);
        for line in reader.lines() {
            let out: Output = serde_json::from_str::<Output>(
                &line.expect("Failed to parse existing result file"),
            )
            .expect("Failed to parse existing result file");
            *seen_tests.entry(out.test_number).or_insert(0) += 1;
        }
    } else {
        let header: Header = Header {
            version: built_info::FEATURES_LOWERCASE_STR.to_string(),
            build_time: current_time_string(),
            notes: NOTES.to_string(),
        };
        write_jsonl(&header, &file_name).expect("Failed to write to result file");
    }

    let test_cases: Vec<PreparationOutputs> =
        parse_csv(Path::new("test_cases.csv")).expect("Failed to read test_case.csv");
    for case in test_cases {
        if seen_tests.contains_key(&case.test_number)
            && seen_tests[&case.test_number] > NUM_TESTS_TO_RUN
        {
            continue;
        }

        solve();

        let output: Output;
        write_jsonl(&output, &file_name).expect("Failed to write to result file");
        *seen_tests.entry(case.test_number).or_insert(0) += 1;

        // if case already ran, skip it (maybe add a flag to rerun)
        // otherwise, call solve, write results after each solve call
        //
    }
}
