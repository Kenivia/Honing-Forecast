use super::saddlepoint_approximation::{FLOAT_TOL, saddlepoint_approximation};
use crate::helpers::find_non_zero_min;
use crate::parser::PreparationOutput;
use crate::parser::Upgrade;
use crate::state::StateBundle;
use std::collections::HashSet;

fn normalize_factor(n: i32, p: f64) -> f64 {
    (1.0 - (1.0 - p).powi(n)).ln()
}
pub fn special_probs(prep_output: &PreparationOutput, state_bundle: &StateBundle) -> Vec<f64> {
    let u_len: usize = prep_output.upgrade_arr.len();
    let upgrade_arr: &Vec<Upgrade> = &prep_output.upgrade_arr;
    let log_base_chance_arr: Vec<f64> = upgrade_arr.iter().map(|x| x.base_chance.ln()).collect();
    let log_base_chance_one_minus_arr: Vec<f64> = upgrade_arr
        .iter()
        .map(|x| (1.0 - x.base_chance).ln())
        .collect();

    let mut attempt_count: Vec<usize> = vec![0; u_len];
    let mut not_succeeded_shadow: Vec<f64> = vec![1.0; u_len];

    let mut result: Vec<f64> = vec![0.0; u_len];
    let mut special_owned: f64 = prep_output.budgets[7] as f64;
    let mut minimal_cost: f64 = 0.0;
    let mut seen: HashSet<usize> = HashSet::new();

    let mut log_prob_dist_arr = vec![vec![]; upgrade_arr.len()];
    let mut support_arr = vec![vec![]; upgrade_arr.len()];
    let mut done = false;
    for (upgrade_index, repeat_count) in state_bundle.special_state.iter() {
        // let mut theta: f64 = 0.0;
        let upgrade: &Upgrade = &upgrade_arr[*upgrade_index];
        let this_special_cost: f64 = upgrade.special_cost as f64;
        if !seen.contains(upgrade_index) {
            minimal_cost += this_special_cost;
            seen.insert(*upgrade_index);
        }
        let this_attempt_count: &mut usize = &mut attempt_count[*upgrade_index];
        for _ in 0..*repeat_count {
            *this_attempt_count += 1;

            support_arr[*upgrade_index].push(this_special_cost * *this_attempt_count as f64);

            let norm_factor: f64 =
                normalize_factor(*this_attempt_count as i32, upgrade.base_chance);
            for (index, l) in log_prob_dist_arr[*upgrade_index].iter_mut().enumerate() {
                *l = log_base_chance_arr[*upgrade_index]
                    + log_base_chance_one_minus_arr[*upgrade_index] * index as f64
                    - norm_factor;
            }
            log_prob_dist_arr[*upgrade_index].push(
                log_base_chance_arr[*upgrade_index]
                    + log_base_chance_one_minus_arr[*upgrade_index]
                        * (*this_attempt_count - 1) as f64
                    - norm_factor,
            );
            let mut alpha: f64 = 0.0; // prob that we got enough special leaps left, but the calculation assumes infinite budget(does not consider the times where we stop because we ran out)
            let needed: f64 = special_owned - this_special_cost; // default to trivial case (prob_got_enough_special_left = 0)

            if needed > minimal_cost + FLOAT_TOL {
                // dbg!(&log_prob_dist_arr, &support_arr,);
                alpha = saddlepoint_approximation(
                    &log_prob_dist_arr,
                    &support_arr,
                    find_non_zero_min(&support_arr, &log_prob_dist_arr),
                    support_arr.iter().map(|x| x.last().unwrap_or(&0.0)).sum(),
                    needed,
                    &mut 0.0,
                );
                // dbg!(needed, alpha, &support_arr);
            }

            let new: f64 = alpha * not_succeeded_shadow[*upgrade_index] * upgrade.base_chance;
            result[*upgrade_index] += new;

            if new < 1e-7 {
                done = true;
                break;
            }

            special_owned = needed;
            not_succeeded_shadow[*upgrade_index] *= 1.0 - upgrade.base_chance;
        }
        if done {
            break;
        }
    }
    result
}
