use hf_adv_hone_data::utils::{MAX_COUNT, Output, get_all_perms};
use hf_core::adv_hone::{MAX_NUM_STATE, tuple_to_index};
use hf_core::helpers::write_jsonl;
use itertools::iproduct;
use serde::{Deserialize, Serialize};

use std::ops::{Deref, DerefMut};
use std::usize::MAX;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};
#[derive(Serialize, Deserialize, Debug)]

pub struct AdvHoneData {
    data: Vec<Vec<(usize, Vec<[f64; 3]>)>>,
}

impl Default for AdvHoneData {
    fn default() -> Self {
        Self {
            data: vec![vec![(MAX, Vec::new()); MAX_NUM_STATE]; MAX_NUM_STATE],
        }
    }
}

impl Deref for AdvHoneData {
    type Target = Vec<Vec<(usize, Vec<[f64; 3]>)>>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl DerefMut for AdvHoneData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
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
fn write_bin(data: &AdvHoneData, path: &String) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // Convert &[Vec3] → &[u8]
    let bytes = postcard::to_stdvec(data).unwrap();
    file.write_all(&bytes)?;
    Ok(())
}
fn main() {
    for (double_balls, is_30_40, starting_xp, cur_balls) in get_bin_perms().into_iter() {
        let file_name: String = "./Advanced_Honing_Data.jsonl".to_owned();
        let file = File::open(file_name).expect("Failed to rewrite base file");
        let reader = BufReader::new(file);

        let this_path = format!(
            "./Advanced_Honing_Data/{}_{}_{}_{}.bin",
            double_balls,
            is_30_40,
            starting_xp * 10,
            cur_balls,
        );
        dbg!(&this_path);
        let mut this_out: AdvHoneData = AdvHoneData::default();
        for (line, ((a, b), (c, d), this_is_30_40, this_double_balls)) in
            reader.lines().zip(get_all_perms().into_iter())
        {
            if !(this_is_30_40 == is_30_40 && this_double_balls == double_balls) {
                continue;
            }
            let output = serde_json::from_str::<Output>(
                &line.expect("Failed to parse existing result file"),
            )
            .expect("Failed to parse existing result file");
            let sum: usize = output.cost_dist[cur_balls][starting_xp]
                .iter()
                .sum::<usize>();
            if sum == 0 {
                continue;
            }
            let inv_sum = (sum as f64).recip();
            let relecant_vec = &mut this_out[tuple_to_index(a, b)][tuple_to_index(c, d)];
            let mut last_valid: usize = 0;
            let mut seen_valid: bool = false;
            for i in 0..MAX_COUNT {
                let this_freq = output.cost_dist[cur_balls][starting_xp][i];
                if this_freq == 0 && !seen_valid {
                    // 0.0
                    continue;
                };
                if !seen_valid {
                    relecant_vec.0 = i;
                }
                seen_valid = true;

                let inv_this = if this_freq == 0 {
                    0.0
                } else {
                    last_valid = i;
                    (this_freq as f64).recip()
                };
                relecant_vec.1.push([
                    output.juice_dist[cur_balls][starting_xp][i] as f64 * inv_this,
                    output.scroll_dist[cur_balls][starting_xp][i] as f64 * inv_this,
                    output.cost_dist[cur_balls][starting_xp][i] as f64 * inv_sum,
                ]);
            }
            relecant_vec.1.truncate(last_valid - relecant_vec.0);
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
