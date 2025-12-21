use chrono::Local;
use hf_arena::engine::{NOTES, solve};
use hf_arena::parse_test_cases::parse_csv;
use hf_core::parser::PreparationOutputs;
use serde::Serialize;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Error, Write};
use std::path::Path;
#[derive(Debug, Serialize)]
struct Header {
    version: String,
    build_time: String,
    notes: String,
}
#[derive(Debug, Serialize)]
struct Output {
    test_case: i64,
    trial_num: i64,
    wall_time: f64,
    states_evaled: i64,
    prob: f64,
    state: String,
    seed: i64,
}

// Include the generated-file as a separate module
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
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
    let file_name: String = format!("results_{}_{}.jsonl", job_id, task_id);
    let now = Local::now();
    let header: Header = Header {
        version: built_info::FEATURES_LOWERCASE_STR.to_string(),
        build_time: now.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        notes: NOTES.to_string(),
    };
    write_jsonl(&header, &file_name);
    let test_cases: Vec<PreparationOutputs> =
        parse_csv(Path::new("test_cases.csv")).expect("Failed to read test_case.csv");
    for case in test_cases {
        // if case already ran, skip it (maybe add a flag to rerun)
        // otherwise, call solve, write results after each solve call
        //
    }
}
