use bytemuck::{Pod, Zeroable};
use hf_adv_hone_data::utils::{MAX_COUNT, Output, get_all_perms};
use hf_core::adv_hone::{MAX_NUM_STATE, tuple_to_index};
use hf_core::helpers::write_jsonl;
use itertools::iproduct;
use serde::Serialize;

use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Serialize)]
struct Vec3 {
    juice: f64,
    scroll: f64,
    chance: f64,
}
impl Default for Vec3 {
    fn default() -> Self {
        Self {
            juice: 0.0,
            scroll: 0.0,
            chance: 0.0,
        }
    }
}

type Grid = [Vec3; MAX_COUNT * MAX_NUM_STATE * MAX_NUM_STATE];
fn default_empty_grid() -> Grid {
    [Vec3::default(); MAX_COUNT * MAX_NUM_STATE * MAX_NUM_STATE]
}

fn get_bin_perms() -> Vec<(usize, usize, usize, usize)> {
    let perms: Vec<(usize, usize, usize, usize)> = iproduct!(
        [0, 1_usize].into_iter(),
        [0, 1_usize].into_iter(),
        (0..100).step_by(1),
        0..7_usize
    )
    .collect();
    perms
}
fn write_bin(data: &Grid, path: &String) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // Convert &[Vec3] → &[u8]
    let bytes = bytemuck::cast_slice(data);
    file.write_all(bytes)?;
    Ok(())
}
fn main() {
    let file_name: String = "./Advanced_Honing_Data.jsonl".to_owned();
    let file = File::open(file_name).expect("Failed to rewrite base file");
    let reader = BufReader::new(file);
    let mut existing_outputs = Vec::new();
    for line in reader.lines() {
        let out =
            serde_json::from_str::<Output>(&line.expect("Failed to parse existing result file"))
                .expect("Failed to parse existing result file");
        existing_outputs.push(out);
    }
    for (double_balls, is_30_40, starting_xp, cur_balls) in get_bin_perms().into_iter() {
        let this_path = format!(
            "./Advanced_Honing_Data/{}_{}_{}_{}.bin",
            double_balls,
            is_30_40,
            starting_xp * 10,
            cur_balls,
        );
        dbg!(&this_path);
        let mut this_out: Grid = default_empty_grid();
        for (output, ((a, b), (c, d), this_is_30_40, this_double_balls)) in
            existing_outputs.iter().zip(get_all_perms().into_iter())
        {
            if !(this_is_30_40 == is_30_40 && this_double_balls == double_balls) {
                continue;
            }
            let index =
                tuple_to_index(a, b) * MAX_NUM_STATE * MAX_COUNT + tuple_to_index(c, d) * MAX_COUNT;

            let sum: usize = output.cost_dist[cur_balls][starting_xp]
                .iter()
                .sum::<usize>();
            if sum == 0 {
                continue;
            }
            let inv_sum = (sum as f64).recip();

            for i in 0..100_usize {
                let inv_this = if output.cost_dist[cur_balls][starting_xp][i] == 0 {
                    0.0
                } else {
                    (output.cost_dist[cur_balls][starting_xp][i] as f64).recip()
                };
                this_out[index + i] = Vec3 {
                    juice: output.juice_dist[cur_balls][starting_xp][i] as f64 * inv_this,
                    scroll: output.scroll_dist[cur_balls][starting_xp][i] as f64 * inv_this,
                    chance: output.cost_dist[cur_balls][starting_xp][i] as f64 * inv_sum,
                };
            }
        }
        let this_jsonl_path = format!(
            "./Advanced_Honing_Data/{}_{}_{}_{}.jsonl",
            double_balls,
            is_30_40,
            starting_xp * 10,
            cur_balls,
        );

        write_jsonl(&this_out.to_vec(), &this_jsonl_path).unwrap();
        write_bin(&this_out, &this_path).unwrap();
    }
}
