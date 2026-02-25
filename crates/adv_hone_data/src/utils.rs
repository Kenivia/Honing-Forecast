use chrono::Duration as ChronoDuration;
use hf_core::adv_hone::{GRACE_FIRST_N, NON_GRACE_FIRST_N};
use itertools::iproduct;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, remove_file},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

pub const MAX_COUNT: usize = 100;
pub const STARTING_POSITIONS: usize = 100;

pub fn get_all_perms() -> Vec<((usize, usize), (usize, usize), usize, usize)> {
    let mut grace = GRACE_FIRST_N.to_vec();
    grace.extend_from_slice(&[GRACE_FIRST_N[GRACE_FIRST_N.len() - 1]; NON_GRACE_FIRST_N.len()]);
    let mut non_grace = vec![0; GRACE_FIRST_N.len()];
    non_grace.extend_from_slice(&NON_GRACE_FIRST_N);
    let permitted_strategy_one = grace.into_iter().zip(non_grace.into_iter());
    let all_perms: Vec<((usize, usize), (usize, usize), usize, usize)> = iproduct!(
        permitted_strategy_one.clone().into_iter(),
        permitted_strategy_one,
        [0, 1_usize].into_iter(),
        [0, 1_usize].into_iter()
    )
    .collect();
    all_perms
}

pub fn format_duration_chrono(d: std::time::Duration) -> String {
    let chrono_duration = ChronoDuration::seconds(d.as_secs() as i64);
    let hours = chrono_duration.num_hours();
    let minutes = chrono_duration.num_minutes() % 60;
    let seconds = chrono_duration.num_seconds() % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn merge_tmp(file_name: &str) {
    let tmp_name = format!("{}.tmp", file_name);

    if !Path::new(&tmp_name).exists() {
        return; // nothing to merge
    }

    println!("Found tmp file. Merging into main file, don't interrupt...");

    // Read main file (if it exists)
    let mut base_outputs: Vec<Output> = if Path::new(file_name).exists() {
        let file = File::open(file_name).expect("Failed to open base file");
        BufReader::new(file)
            .lines()
            .map(|l| {
                serde_json::from_str::<Output>(&l.expect("Failed to read base file line"))
                    .expect("Failed to parse base file line")
            })
            .collect()
    } else {
        Vec::new()
    };

    // Read tmp file
    let tmp_file = File::open(&tmp_name).expect("Failed to open tmp file");
    let tmp_outputs: Vec<Output> = BufReader::new(tmp_file)
        .lines()
        .map(|l| {
            serde_json::from_str::<Output>(&l.expect("Failed to read tmp line"))
                .expect("Failed to parse tmp line")
        })
        .collect();

    // Merge line by line
    for (i, tmp_out) in tmp_outputs.into_iter().enumerate() {
        if i < base_outputs.len() {
            base_outputs[i] = base_outputs[i].clone().combine(tmp_out);
        } else {
            base_outputs.push(tmp_out);
        }
    }
    // Write merged result back to main file
    let file = File::create(file_name).expect("Failed to rewrite base file");
    let mut writer = BufWriter::new(file);

    for out in base_outputs {
        let line = serde_json::to_string(&out).unwrap();
        writer.write_all(line.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }

    writer.flush().unwrap();

    // Remove tmp file
    remove_file(&tmp_name).expect("Failed to remove tmp file");

    println!("Merge complete.");
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Output {
    pub cost_dist: Vec<Vec<Vec<usize>>>,
    pub juice_dist: Vec<Vec<Vec<usize>>>,
    pub scroll_dist: Vec<Vec<Vec<usize>>>,
    pub data_size: usize,
    pub configuration: Configuration,
}

impl Output {
    pub fn new(data_size: usize, configuration: &Configuration) -> Self {
        Self {
            cost_dist: vec![vec![vec![0; MAX_COUNT]; STARTING_POSITIONS]; 7],

            juice_dist: vec![vec![vec![0; MAX_COUNT]; STARTING_POSITIONS]; 7],

            scroll_dist: vec![vec![vec![0; MAX_COUNT]; STARTING_POSITIONS]; 7],

            data_size,
            configuration: configuration.clone(),
        }
    }
    pub fn combine(mut self, other: Self) -> Self {
        for i in 0..7 {
            for j in 0..STARTING_POSITIONS {
                for k in 0..MAX_COUNT {
                    self.cost_dist[i][j][k] += other.cost_dist[i][j][k];
                    self.juice_dist[i][j][k] += other.juice_dist[i][j][k];
                    self.scroll_dist[i][j][k] += other.scroll_dist[i][j][k];
                }
            }
        }
        self.data_size += other.data_size;
        self
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Copy)]
pub struct Configuration {
    pub grace_juice_target: usize,
    pub non_grace_juice_target: usize,
    pub grace_scroll_target: usize,
    pub non_grace_scroll_target: usize,
    pub is_30_40: usize,
    pub double_balls: usize,
}

impl Configuration {
    pub fn new(
        (
            (grace_juice_target, non_grace_juice_target),
            (grace_scroll_target, non_grace_scroll_target),
            is_30_40,
            double_balls,
        ): ((usize, usize), (usize, usize), usize, usize),
    ) -> Self {
        Self {
            grace_juice_target,
            non_grace_juice_target,
            grace_scroll_target,
            non_grace_scroll_target,
            is_30_40,
            double_balls,
        }
    }
}
