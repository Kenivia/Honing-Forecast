use std::f64;

// use super::saddlepoint_approximation::saddlepoint_approximation_wrapper;
// use crate::helpers::find_non_zero_min_vec;
use crate::parser::PreparationOutput;
// use crate::parser::Upgrade;
use crate::state::StateBundle;

// pub fn special_probs(prep_output: &PreparationOutput, state_bundle: &StateBundle) -> Vec<f64> {
//     let u_len: usize = prep_output.upgrade_arr.len();
//     let upgrade_arr: &Vec<Upgrade> = &prep_output.upgrade_arr;
//     let log_base_chance_arr: Vec<f64> = upgrade_arr.iter().map(|x| x.base_chance.ln()).collect();
//     let log_base_chance_one_minus_arr: Vec<f64> = upgrade_arr
//         .iter()
//         .map(|x| (1.0 - x.base_chance).ln())
//         .collect();

//     let mut attempt_count: Vec<usize> = vec![0; u_len];
//     let mut not_succeeded_shadow: Vec<f64> = vec![1.0; u_len];

//     let mut result: Vec<f64> = vec![0.0; u_len];
//     let special_owned: f64 = prep_output.budgets[7] as f64;

//     let mut log_prob_dist_arr: Vec<Vec<f64>> = vec![vec![]; upgrade_arr.len()];
//     let mut prob_dist_arr: Vec<Vec<f64>> = vec![vec![]; upgrade_arr.len()];

//     let mut support_arr = vec![vec![]; upgrade_arr.len()];
//     let mut done = false;
//     for (upgrade_index, repeat_count) in state_bundle.special_state.iter() {
//         let upgrade: &Upgrade = &upgrade_arr[*upgrade_index];
//         let this_special_cost: f64 = upgrade.special_cost as f64;

//         let this_attempt_count: &mut usize = &mut attempt_count[*upgrade_index];

//         for _ in 0..*repeat_count {
//             *this_attempt_count += 1;

//             // dbg!(&log_prob_dist_arr, &support_arr,);

//             let alpha = saddlepoint_approximation_wrapper(
//                 &log_prob_dist_arr,
//                 &prob_dist_arr,
//                 &support_arr,
//                 find_non_zero_min_vec(&support_arr, &log_prob_dist_arr),
//                 support_arr.iter().map(|x| x.last().unwrap_or(&0.0)).sum(),
//                 special_owned - this_special_cost,
//                 &mut 0.0,
//             );
//             // dbg!(needed, alpha, &support_arr);

//             let new: f64 = alpha * not_succeeded_shadow[*upgrade_index] * upgrade.base_chance;
//             result[*upgrade_index] += new;

//             if alpha < 1e-4 {
//                 done = true;
//                 break;
//             }

//             if *this_attempt_count > 1 {
//                 log_prob_dist_arr[*upgrade_index][*this_attempt_count - 2] = log_base_chance_arr
//                     [*upgrade_index]
//                     + log_base_chance_one_minus_arr[*upgrade_index]
//                         * (*this_attempt_count - 2) as f64;

//                 prob_dist_arr[*upgrade_index][*this_attempt_count - 2] = upgrade.base_chance
//                     * (1.0 - upgrade.base_chance).powi((*this_attempt_count - 2) as i32);
//             }

//             support_arr[*upgrade_index].push(this_special_cost * *this_attempt_count as f64);

//             let end_plus_second_to_last: f64 =
//                 (1.0 - upgrade.base_chance).powi((*this_attempt_count - 1) as i32);
//             log_prob_dist_arr[*upgrade_index].push(
//                 log_base_chance_one_minus_arr[*upgrade_index] * (*this_attempt_count - 1) as f64,
//             );
//             prob_dist_arr[*upgrade_index].push(end_plus_second_to_last);

//             not_succeeded_shadow[*upgrade_index] *= 1.0 - upgrade.base_chance;
//         }
//         if done {
//             break;
//         }
//     }
//     result
// }
pub fn special_probs(prep_output: &PreparationOutput, state_bundle: &StateBundle) -> Vec<f64> {
    let upgrades = &prep_output.upgrade_arr;
    let m = upgrades.len();

    let budget: usize = prep_output.budgets[7] as usize; // total special budget B
    let mut result = vec![0.0_f64; m];

    // active[b] = probability we are still running and have 'b' special left
    let mut active: Vec<f64> = vec![0.0_f64; budget + 1];
    active[budget] = 1.0;

    // optional: track stopped distribution; not needed if you only care about success probs
    // let mut stopped = vec![0.0_f64; budget + 1];

    // Process streaks in order
    for &(upgrade_index, repeat_count) in state_bundle.special_state.iter() {
        let upg = &upgrades[upgrade_index];
        let p = upg.base_chance;
        let one_minus_p = 1.0 - p;
        let this_special_cost = upg.special_cost as usize;

        if repeat_count == 0 || p == 0.0 {
            // Nothing happens in this streak; propagate active as-is
            continue;
        }

        // Precompute geometric probabilities p * (1-p)^(t-1) up to L
        let mut geom = vec![0.0_f64; repeat_count + 1]; // 1-based: geom[t] for t=1..L
        let mut pow = 1.0_f64;
        for t in 1..=repeat_count {
            geom[t] = p * pow;
            pow *= one_minus_p;
        }
        // pow now equals (1-p)^L, but we also need (1-p)^(A-1) often, see below.

        let mut next_active = vec![0.0_f64; budget + 1];

        for b in 0..=budget {
            let mass = active[b];
            if mass == 0.0 {
                continue;
            }

            if b < this_special_cost {
                continue; // do not propagate to next_active
            }

            // Max attempts allowed by budget and streak length
            let max_by_budget = b / this_special_cost;
            let actual_repeated = repeat_count.min(max_by_budget);

            // Success probability on this upgrade from this starting budget
            let fail_all_a = one_minus_p.powi(actual_repeated as i32);
            result[upgrade_index] += mass * (1.0 - fail_all_a);

            for succeed_at in 1..actual_repeated {
                let prob_n_t = geom[succeed_at];
                if prob_n_t == 0.0 {
                    continue;
                }
                let b2 = b - succeed_at * this_special_cost;
                next_active[b2] += mass * prob_n_t;
            }

            // t = A
            let prob_n_a = one_minus_p.powi((actual_repeated - 1) as i32);
            let b2 = b - actual_repeated * this_special_cost;
            next_active[b2] += mass * prob_n_a;
        }

        active = next_active;
    }
    // dbg!(&state_bundle.special_state);
    // dbg!(&result);
    // panic!();
    result
}
