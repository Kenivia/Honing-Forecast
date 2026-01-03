use super::saddlepoint_approximation::saddlepoint_approximation_wrapper;
use crate::helpers::find_non_zero_min_vec;
use crate::parser::PreparationOutput;
use crate::parser::Upgrade;
use crate::state::StateBundle;

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
    let special_owned: f64 = prep_output.budgets[7] as f64;

    let mut log_prob_dist_arr: Vec<Vec<f64>> = vec![vec![]; upgrade_arr.len()];
    let mut prob_dist_arr: Vec<Vec<f64>> = vec![vec![]; upgrade_arr.len()];

    let mut support_arr = vec![vec![]; upgrade_arr.len()];
    let mut done = false;
    for (upgrade_index, repeat_count) in state_bundle.special_state.iter() {
        let upgrade: &Upgrade = &upgrade_arr[*upgrade_index];
        let this_special_cost: f64 = upgrade.special_cost as f64;

        let this_attempt_count: &mut usize = &mut attempt_count[*upgrade_index];

        for _ in 0..*repeat_count {
            *this_attempt_count += 1;

            if *this_attempt_count > 1 {
                log_prob_dist_arr[*upgrade_index][*this_attempt_count - 2] = log_base_chance_arr
                    [*upgrade_index]
                    + log_base_chance_one_minus_arr[*upgrade_index]
                        * (*this_attempt_count - 2) as f64;

                prob_dist_arr[*upgrade_index][*this_attempt_count - 2] = upgrade.base_chance
                    * (1.0 - upgrade.base_chance).powi((*this_attempt_count - 2) as i32);
            }

            support_arr[*upgrade_index].push(this_special_cost * *this_attempt_count as f64);

            let end_plus_second_to_last: f64 =
                (1.0 - upgrade.base_chance).powi((*this_attempt_count - 1).max(0) as i32);
            log_prob_dist_arr[*upgrade_index].push(end_plus_second_to_last.ln());
            prob_dist_arr[*upgrade_index].push(end_plus_second_to_last);

            // dbg!(&log_prob_dist_arr, &support_arr,);

            let alpha = saddlepoint_approximation_wrapper(
                &log_prob_dist_arr,
                &prob_dist_arr,
                &support_arr,
                find_non_zero_min_vec(&support_arr, &log_prob_dist_arr),
                support_arr.iter().map(|x| x.last().unwrap_or(&0.0)).sum(),
                special_owned,
                &mut 0.0,
            );
            // dbg!(needed, alpha, &support_arr);

            let new: f64 = alpha * not_succeeded_shadow[*upgrade_index] * upgrade.base_chance;
            result[*upgrade_index] += new;

            if alpha < 1e-4 {
                done = true;
                break;
            }

            not_succeeded_shadow[*upgrade_index] *= 1.0 - upgrade.base_chance;
        }
        if done {
            break;
        }
    }
    result
}
