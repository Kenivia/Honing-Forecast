use hf_adv_hone_data::one_sim::{SimTracker, one_sim};
use hf_adv_hone_data::utils::{
    Configuration, Output, format_duration_chrono, get_all_perms, merge_tmp,
};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    thread::available_parallelism,
    time::Instant,
};
const BATCH_SIZE: usize = 1_000_000;

fn one_batch(configuration: &Configuration, batch_size: usize) -> Output {
    let num_threads = rayon::current_num_threads();
    let chunk_size = (batch_size - batch_size % num_threads) / num_threads;

    (0..num_threads)
        .into_par_iter()
        .map(|_| {
            let mut local_out = Output::new(chunk_size, configuration);
            let mut rng = SmallRng::from_os_rng(); // Excellent choice!

            // Allocate our tracker once per thread
            let mut tracker = SimTracker::new();

            for _ in 0..chunk_size {
                one_sim(&mut rng, configuration, &mut local_out, &mut tracker);
            }

            local_out
        })
        .reduce(
            || Output::new(0, configuration),
            |acc, local_out| acc.combine(local_out),
        )
}
fn main() {
    let thread_num = available_parallelism()
        .unwrap()
        .get()
        .saturating_sub(1)
        .max(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_num)
        .build_global()
        .unwrap();

    let file_name: String = "./Advanced_Honing_Data.jsonl".to_owned();
    merge_tmp(&file_name);
    let mut existing_outputs = Vec::new();
    if Path::new(&file_name).exists() {
        let file: File = File::open(&file_name).unwrap(); // this really shouldnt go wrong
        let reader: BufReader<File> = BufReader::new(file);
        for line in reader.lines() {
            let out: Output = serde_json::from_str::<Output>(
                &line.expect("Failed to parse existing result file"),
            )
            .expect("Failed to parse existing result file");
            existing_outputs.push(out);
        }
    }
    let actual_existing = existing_outputs.len();
    let mut existing_data_size: usize = 0;
    let all_perms = get_all_perms();
    for (index, x) in all_perms.iter().enumerate() {
        if index >= actual_existing {
            existing_outputs.push(Output::new(0, &Configuration::new(*x)));
        } else {
            existing_data_size += existing_outputs[index].data_size;
        }
    }

    let tmp_name = format!("{}.tmp", file_name);
    let output = File::create(&tmp_name).expect("create tmp failed");
    let mut writer = BufWriter::new(output);
    let mut start: Instant = Instant::now();
    let mut timer_started = false;
    let mut done_data_size: usize = existing_data_size;
    for (x, existing) in all_perms.iter().zip(existing_outputs.into_iter()) {
        let this_config = Configuration::new(*x);
        assert!(existing.configuration == this_config);
        let alr_done = existing.data_size;
        let actual_doing = BATCH_SIZE.saturating_sub(alr_done);
        if actual_doing < thread_num {
            continue;
        }
        if !timer_started {
            start = Instant::now();
            timer_started = true;
        }

        done_data_size += actual_doing;
        let out = one_batch(&this_config, actual_doing);

        let new_line = serde_json::to_string(&out).unwrap();
        writer.write_all(new_line.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
        writer.flush().unwrap();
        let total_progress = done_data_size.max(1) as f64 / (BATCH_SIZE * all_perms.len()) as f64;
        let cur_this_run_progress = (done_data_size - existing_data_size).max(1) as f64
            / (BATCH_SIZE * all_perms.len() - existing_data_size) as f64;
        let time_so_far = start.elapsed();
        let estimated_total =
            std::time::Duration::from_secs_f64(time_so_far.as_secs_f64() / cur_this_run_progress);
        let estimated_remaining = estimated_total.saturating_sub(time_so_far);
        let progress_percent = (total_progress * 100.0) as u32;

        println!(
            "Finished {:?}, Batch size : {}, Progress: {}%, Time so far: {}, Estimated time remaining: {}",
            x,
            BATCH_SIZE.saturating_sub(alr_done),
            progress_percent,
            format_duration_chrono(time_so_far),
            format_duration_chrono(estimated_remaining)
        );
    }

    merge_tmp(&file_name);
}
