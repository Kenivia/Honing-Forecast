use super::saddlepoint_approximation::saddlepoint_approximation;
use crate::constants::JuiceInfo;
use crate::helpers::find_non_zero_min;
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
    let this_price: (f64, f64) = juice_info.gold_costs[upgrade.upgrade_index][index];
    if upgrade.is_weapon {
        *cost_so_far += this_price.0;
    } else {
        *cost_so_far += this_price.1;
    }
}
fn generate_combined(
    prep_output: &mut PreparationOutput,
    state_bundle: &StateBundle,
) -> Vec<Vec<f64>> {
    let u_len: usize = prep_output.upgrade_arr.len();
    let mut combined_costs: Vec<Vec<f64>> = Vec::with_capacity(u_len);
    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        combined_costs.push(Vec::with_capacity(upgrade.prob_dist.len()));
        let mut cost_so_far: f64 = 0.0;
        for (p_index, _) in state_bundle.log_prob_dist_arr[u_index].iter().enumerate() {
            combined_costs[u_index].push(cost_so_far);
            cost_so_far += upgrade.eqv_gold_per_tap;
            let (juice, book) = state_bundle.state[u_index][p_index];
            if juice {
                add_juice_gold_cost(&prep_output.juice_info, &upgrade, &mut cost_so_far, 0);
            }
            if book > 0 {
                add_juice_gold_cost(
                    &prep_output.juice_info,
                    &upgrade,
                    &mut cost_so_far,
                    book as usize,
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
    let j_len: usize = prep_output.juice_info.one_gold_cost_id.len();

    let mut mats_costs: Vec<Vec<Vec<f64>>> = vec![Vec::with_capacity(u_len); 7];
    let mut weap_juices_costs: Vec<Vec<Vec<f64>>> = vec![Vec::with_capacity(u_len); j_len];
    let mut armor_juices_costs: Vec<Vec<Vec<f64>>> = vec![Vec::with_capacity(u_len); j_len];

    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        let l_len: usize = state_bundle.log_prob_dist_arr[u_index].len();
        for t_index in 0..7 {
            mats_costs[t_index].push(Vec::with_capacity(l_len));
            let mut cost_so_far = 0.0;
            for _ in state_bundle.log_prob_dist_arr[u_index].iter() {
                mats_costs[t_index][u_index].push(cost_so_far);
                cost_so_far += upgrade.costs[t_index] as f64 * prep_output.price_arr[t_index];
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

                for (p_index, _) in state_bundle.log_prob_dist_arr[u_index].iter().enumerate() {
                    this_weap.push(costs_so_far.0);
                    this_armor.push(costs_so_far.1);
                    let (juice, book) = state_bundle.state[u_index][p_index];
                    if juice {
                        add_juice_gold_cost(
                            &prep_output.juice_info,
                            &upgrade,
                            if upgrade.is_weapon {
                                &mut costs_so_far.0
                            } else {
                                &mut costs_so_far.1
                            },
                            0,
                        );
                    }
                    if book > 0 {
                        add_juice_gold_cost(
                            &prep_output.juice_info,
                            &upgrade,
                            if upgrade.is_weapon {
                                &mut costs_so_far.0
                            } else {
                                &mut costs_so_far.1
                            },
                            book as usize,
                        );
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
    state_bundle: &StateBundle,
) -> Vec<f64> {
    let (mats_costs, weap_juices_costs, armor_juices_costs) =
        generate_individual(prep_output, &state_bundle);
    let mut prob_leftover: Vec<f64> = Vec::new();
    for (t_index, support_arr) in mats_costs.iter().enumerate() {
        prob_leftover.push(saddlepoint_approximation(
            &state_bundle.log_prob_dist_arr,
            support_arr,
            find_non_zero_min(support_arr, &state_bundle.log_prob_dist_arr),
            support_arr.iter().map(|x| x.last().unwrap()).sum(),
            prep_output.budgets[t_index] as f64,
            &mut 0.0,
        ));
    }
    for (t_index, support_arr) in weap_juices_costs.iter().enumerate() {
        prob_leftover.push(saddlepoint_approximation(
            &state_bundle.log_prob_dist_arr,
            support_arr,
            find_non_zero_min(support_arr, &state_bundle.log_prob_dist_arr),
            support_arr.iter().map(|x| x.last().unwrap()).sum(),
            prep_output.juice_books_owned[t_index].0 as f64,
            &mut 0.0,
        ));
    }
    for (t_index, support_arr) in armor_juices_costs.iter().enumerate() {
        prob_leftover.push(saddlepoint_approximation(
            &state_bundle.log_prob_dist_arr,
            support_arr,
            find_non_zero_min(support_arr, &state_bundle.log_prob_dist_arr),
            support_arr.iter().map(|x| x.last().unwrap()).sum(),
            prep_output.juice_books_owned[t_index].1 as f64,
            &mut 0.0,
        ));
    }
    prob_leftover
}
pub fn normal_honing_sa_wrapper(
    state_bundle: &mut StateBundle,
    prep_output: &mut PreparationOutput,
    states_evaled: &mut i64,
    // depth: usize,
    // cache: &mut HashMap<(Vec<bool>, usize), Vec<([i64; 9], f64)>>,
) -> f64 {
    let u_len: usize = prep_output.upgrade_arr.len();
    // dbg!(&prep_output, &state_bundle);
    let zero_probs: Vec<f64> = special_probs(prep_output, state_bundle);
    // dbg!(&zero_probs);
    let mut log_prob_dist_arr: Vec<Vec<f64>> = Vec::with_capacity(u_len);
    for (u_index, upgrade) in prep_output.upgrade_arr.iter_mut().enumerate() {
        let new_extra: Vec<f64> = state_bundle.state[u_index]
            .iter()
            .map(|(juice, book)| {
                let mut chance: f64 = upgrade.base_chance;
                if *juice {
                    chance += prep_output.juice_info.chances[upgrade.upgrade_index][0];
                }
                if *book > 0 {
                    chance += prep_output.juice_info.chances[upgrade.upgrade_index][*book];
                }
                chance
            }) //if *x > 0 { upgrade.base_chance } else { 0.0 }) //
            .collect();

        let prob_dist: Vec<f64> = probability_distribution(
            upgrade.base_chance,
            upgrade.artisan_rate,
            &new_extra,
            zero_probs[u_index],
        );
        let log_prob_dist: Vec<f64> = prob_dist.iter().map(|x| x.ln()).collect();
        log_prob_dist_arr.push(log_prob_dist);
        // gold_costs_arr.push(gold_cost_record);
    }
    state_bundle.log_prob_dist_arr = log_prob_dist_arr;
    *states_evaled += 1;
    let combined_costs: Vec<Vec<f64>> = generate_combined(prep_output, state_bundle);
    let min_value: f64 = find_non_zero_min(&combined_costs, &state_bundle.log_prob_dist_arr);

    let max_value: f64 = combined_costs.iter().map(|x| *x.last().unwrap()).sum();

    let out: f64 = saddlepoint_approximation(
        &state_bundle.log_prob_dist_arr,
        &combined_costs,
        min_value,
        max_value,
        prep_output.budget_eqv_gold,
        &mut 0.0, //TODO i mean maybe can cache theta values and warm start with similar states? but then have to make sure derivative at that init_theta is like correct cos sometimes we snap to 1.0 or -1.0 then the derivatives fake
    );
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
