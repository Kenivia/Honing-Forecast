// use crate::constants::*;
use assert_float_eq::assert_f32_near;

fn average_tap(prob_dist: &Vec<f32>, start_index: usize) -> f32 {
    let mut out: f32 = start_index as f32;
    // println!("{:?}", prob_dist[start_index..].iter().sum::<f32>() as f32);
    assert_f32_near!(
        prob_dist[start_index..].iter().sum::<f32>() as f32,
        1.0 as f32,
        10
    );
    // let sum_before_start: f32 = prob_dist[..start_index].iter().sum();
    for (index, item) in prob_dist.iter().skip(start_index).enumerate() {
        out += item * (index + start_index + 1) as f32;
    }
    out
}

pub fn est_special_honing_value(
    prob_dist_arr: &Vec<Vec<f32>>,
    hone_costs: &Vec<Vec<i64>>,
    special_costs: &Vec<i64>,
    mats_value: &Vec<f32>,
) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::with_capacity(prob_dist_arr.len());
    let mut average: f32;
    let mut this_sum: f32;
    let cost_type_count: usize = hone_costs.len();
    assert!(mats_value.len() == cost_type_count);
    for (piece, prob_dist) in prob_dist_arr.iter().enumerate() {
        average = average_tap(&prob_dist, 0);
        this_sum = 0.0;
        for cost_type in 0..cost_type_count {
            this_sum += mats_value[cost_type] * average * hone_costs[cost_type][piece] as f32
                / special_costs[piece] as f32;
        }
        out.push(this_sum);
    }

    out
}
