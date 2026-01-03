use super::saddlepoint_approximation::saddlepoint_approximation_wrapper;
use crate::constants::JuiceInfo;
use crate::helpers::find_non_zero_min_prep;
use crate::parser::PreparationOutput;
use crate::parser::Upgrade;
use crate::parser::probability_distribution;
use crate::saddlepoint_approximation::special_sa::special_probs;
use crate::state::StateBundle;

fn add_juice_gold_cost(
    juice_info: &JuiceInfo,
    upgrade: &Upgrade,
    cost_so_far: &mut f64,
    index: usize,
) {
    let this_cost: (f64, f64) = juice_info.gold_costs[upgrade.upgrade_index][index];
    if upgrade.is_weapon {
        *cost_so_far += this_cost.0;
    } else {
        *cost_so_far += this_cost.1;
    }
}

pub fn generate_combined(
    prep_output: &mut PreparationOutput,
    state_bundle: &StateBundle,
) -> Vec<Vec<f64>> {
    let u_len: usize = prep_output.upgrade_arr.len();
    let mut combined_costs: Vec<Vec<f64>> = Vec::with_capacity(u_len);
    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        combined_costs.push(Vec::with_capacity(upgrade.prob_dist.len()));
        let mut cost_so_far: f64 = 0.0;
        for (p_index, _) in upgrade.log_prob_dist.iter().enumerate() {
            combined_costs[u_index].push(cost_so_far);
            cost_so_far += upgrade.eqv_gold_per_tap;
            let (juice, book_index) = state_bundle.state[u_index][p_index];
            if juice {
                add_juice_gold_cost(&prep_output.juice_info, &upgrade, &mut cost_so_far, 0);
            }
            if book_index > 0 {
                add_juice_gold_cost(
                    &prep_output.juice_info,
                    &upgrade,
                    &mut cost_so_far,
                    book_index as usize,
                );
            }
        }
        // combined_costs[u_index].push(cost_so_far);
    }
    combined_costs
}
pub fn generate_individual(
    prep_output: &mut PreparationOutput,
    state_bundle: &StateBundle,
) -> (Vec<Vec<Vec<f64>>>, Vec<Vec<Vec<f64>>>, Vec<Vec<Vec<f64>>>) {
    let u_len: usize = prep_output.upgrade_arr.len();
    let j_len: usize = prep_output.juice_info.one_gold_cost.len();

    let mut mats_costs: Vec<Vec<Vec<f64>>> = vec![Vec::with_capacity(u_len); 7];
    let mut weap_juices_costs: Vec<Vec<Vec<f64>>> = vec![Vec::with_capacity(u_len); j_len];
    let mut armor_juices_costs: Vec<Vec<Vec<f64>>> = vec![Vec::with_capacity(u_len); j_len];

    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        let l_len: usize = upgrade.log_prob_dist.len();
        for t_index in 0..7 {
            mats_costs[t_index].push(Vec::with_capacity(l_len));
            let mut cost_so_far = 0.0;
            for _ in upgrade.log_prob_dist.iter() {
                mats_costs[t_index][u_index].push(cost_so_far);
                cost_so_far += upgrade.costs[t_index] as f64;
            }
            // mats_costs[t_index][u_index].push(cost_so_far);
        }
        for id_to_match in 0..j_len {
            let mut this_weap: Vec<f64> = Vec::with_capacity(l_len);
            let mut this_armor: Vec<f64> = Vec::with_capacity(l_len);
            for (bit_index, _) in prep_output.juice_info.gold_costs[upgrade.upgrade_index]
                .iter()
                .enumerate()
            {
                let id: usize = prep_output.juice_info.ids[upgrade.upgrade_index][bit_index];
                if id_to_match != id {
                    continue;
                }
                let mut costs_so_far: (f64, f64) = (0.0, 0.0);

                for (p_index, _) in upgrade.log_prob_dist.iter().enumerate() {
                    this_weap.push(costs_so_far.0);
                    this_armor.push(costs_so_far.1);
                    let (juice, book_index) = state_bundle.state[u_index][p_index];
                    if juice {
                        if upgrade.is_weapon {
                            costs_so_far.0 +=
                                prep_output.juice_info.amt_used[upgrade.upgrade_index][0] as f64;
                        } else {
                            costs_so_far.1 +=
                                prep_output.juice_info.amt_used[upgrade.upgrade_index][0] as f64;
                        }
                    }
                    if book_index > 0 {
                        if upgrade.is_weapon {
                            costs_so_far.0 += prep_output.juice_info.amt_used[upgrade.upgrade_index]
                                [book_index] as f64;
                        } else {
                            costs_so_far.1 += prep_output.juice_info.amt_used[upgrade.upgrade_index]
                                [book_index] as f64;
                        }
                    }
                }
                // this_weap.push(costs_so_far.0);
                // this_armor.push(costs_so_far.1);
                break;
            }
            if this_armor.len() > 0 {
                weap_juices_costs[id_to_match].push(this_weap);
                armor_juices_costs[id_to_match].push(this_armor);
            } else {
                weap_juices_costs[id_to_match].push(vec![0.0; l_len]);
                armor_juices_costs[id_to_match].push(vec![0.0; l_len]);
            }
        }
    }
    (mats_costs, weap_juices_costs, armor_juices_costs)
}

pub fn compute_leftover_probs(
    prep_output: &mut PreparationOutput,
    state_bundle: &mut StateBundle,
) -> Vec<f64> {
    init_dist(state_bundle, prep_output);

    let (mats_costs, weap_juices_costs, armor_juices_costs) =
        generate_individual(prep_output, &state_bundle);
    let mut prob_leftover: Vec<f64> = Vec::new();
    for (t_index, support_arr) in mats_costs.iter().enumerate() {
        prob_leftover.push(honing_sa_wrapper(
            prep_output,
            support_arr,
            prep_output.budgets[t_index] as f64,
        ));
    }
    for (t_index, support_arr) in weap_juices_costs.iter().enumerate() {
        prob_leftover.push(honing_sa_wrapper(
            prep_output,
            support_arr,
            prep_output.juice_books_owned[t_index].0 as f64,
        ));
    }
    for (t_index, support_arr) in armor_juices_costs.iter().enumerate() {
        prob_leftover.push(honing_sa_wrapper(
            prep_output,
            support_arr,
            prep_output.juice_books_owned[t_index].1 as f64,
        ));
    }
    prob_leftover
}

pub fn new_prob_dist(
    state: &Vec<(bool, usize)>,
    juice_info: &JuiceInfo,
    upgrade: &Upgrade,
    zero: f64,
) -> Vec<f64> {
    let new_extra: Vec<f64> = state
        .iter()
        .map(|(juice, book_index)| {
            let mut chance: f64 = 0.0;
            if *juice {
                chance += juice_info.chances[upgrade.upgrade_index][0];
            }
            if *book_index > 0 {
                chance += juice_info.chances[upgrade.upgrade_index][*book_index];
            }
            chance
        }) //if *x > 0 { upgrade.base_chance } else { 0.0 }) //
        .collect();

    let out = probability_distribution(upgrade.base_chance, upgrade.artisan_rate, &new_extra, zero);
    // for o in out.iter() {
    //     if !o.is_finite() || *o < 0.0 {
    //         dbg!(
    //             &out,
    //             &upgrade,
    //             &juice_info.chances[upgrade.upgrade_index],
    //             &new_extra,
    //             zero
    //         );
    //         panic!();
    //     }
    // }

    out
}

pub fn init_dist(state_bundle: &mut StateBundle, prep_output: &mut PreparationOutput) {
    // dbg!(&prep_output, &state_bundle);
    let zero_probs: Vec<f64> = special_probs(prep_output, state_bundle);
    // dbg!(&zero_probs);
    for (u_index, upgrade) in prep_output.upgrade_arr.iter_mut().enumerate() {
        let prob_dist: Vec<f64> = new_prob_dist(
            &state_bundle.state[u_index],
            &prep_output.juice_info,
            upgrade,
            zero_probs[u_index],
        );
        let log_prob_dist: Vec<f64> = prob_dist.iter().map(|x| x.ln()).collect();
        upgrade.prob_dist = prob_dist;
        upgrade.log_prob_dist = log_prob_dist;

        // gold_costs_arr.push(gold_cost_record);
    }
}

fn honing_sa_wrapper(
    prep_output: &mut PreparationOutput,
    support_arr: &[Vec<f64>],
    budget: f64,
) -> f64 {
    let (log_prob_dist_arr, prob_dist_arr) = prep_output.gather_dists();
    saddlepoint_approximation_wrapper(
        &log_prob_dist_arr,
        &prob_dist_arr,
        support_arr,
        find_non_zero_min_prep(support_arr, prep_output),
        support_arr.iter().map(|x| x.last().unwrap()).sum(),
        budget,
        &mut 0.0,
    )
}
pub fn honing_sa_metric(
    state_bundle: &mut StateBundle,
    prep_output: &mut PreparationOutput,
    states_evaled: &mut i64,
) -> f64 {
    init_dist(state_bundle, prep_output);
    *states_evaled += 1;

    let combined_costs: Vec<Vec<f64>> = generate_combined(prep_output, state_bundle);
    let out: f64 = honing_sa_wrapper(prep_output, &combined_costs, prep_output.eqv_gold_budget);

    // dbg!(
    //     prep_output.budget_eqv_gold,
    //     out,
    //     &state_bundle.log_prob_dist_arr.len(),
    //     &state_bundle.log_prob_dist_arr[0].len(),
    //     &combined_costs.len(),
    //     &combined_costs[0].len(),
    //     min_value,
    //     max_value
    // );
    // panic!();
    out
}

// // this feels SO wrong but idk how else to do this
// fn expected_juice_leftover(prep_output: &PreparationOutput, state_bundle: &StateBundle) -> f64 {
//     let mut avg_used: Vec<(f64, f64)> =
//         vec![(0.0, 0.0); prep_output.juice_info.one_gold_cost_id.len()];
//     let mut full_avg: Vec<(f64, f64)> =
//         vec![(0.0, 0.0); prep_output.juice_info.one_gold_cost_id.len()];
//     for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
//         let mut used_so_far: Vec<(i64, i64)> = vec![(0, 0); prep_output.juice_info.ids.len()];
//         let mut max_used: Vec<(i64, i64)> = vec![(0, 0); prep_output.juice_info.ids.len()];
//         for (p_index, p) in upgrade.prob_dist.iter().enumerate() {
//             // dbg!(&state_bundle.state);
//             for (bit_index, bit) in state_bundle.state[u_index][p_index].iter().enumerate() {
//                 // dbg!(&prep_output.juice_info);
//                 let id = prep_output.juice_info.ids[upgrade.upgrade_index][bit_index];

//                 if upgrade.is_weapon {
//                     if *bit {
//                         used_so_far[id].0 +=
//                             prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].0;
//                     }
//                     max_used[id].0 +=
//                         prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].0;
//                 } else {
//                     if *bit {
//                         used_so_far[id].1 +=
//                             prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].1;
//                     }
//                     max_used[id].1 +=
//                         prep_output.juice_info.amt_used[upgrade.upgrade_index][bit_index].1;
//                 }

//                 avg_used[id].0 += p * used_so_far[id].0 as f64;
//                 avg_used[id].1 += p * used_so_far[id].1 as f64;

//                 full_avg[id].0 += p * max_used[id].0 as f64;
//                 full_avg[id].1 += p * max_used[id].1 as f64;
//             }
//         }
//     }
//     // dbg!(
//     //     &prep_output.juice_info,
//     //     &prep_output.juice_books_owned,
//     //     &avg_used
//     // );
//     let mut total_gold: f64 = 0.0;
//     for (id, a) in avg_used.iter().enumerate() {
//         total_gold += ((prep_output.juice_books_owned[id].0 as f64).min(full_avg[id].0) - a.0)
//             .max(0.0) as f64
//             * prep_output.juice_info.one_gold_cost_id[id].0;
//         total_gold += ((prep_output.juice_books_owned[id].1 as f64).min(full_avg[id].1) - a.1)
//             .max(0.0) as f64
//             * prep_output.juice_info.one_gold_cost_id[id].1;
//     }
//     total_gold
// }
