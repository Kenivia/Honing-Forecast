use crate::constants::JuiceInfo;

use crate::parser::{Upgrade, probability_distribution};
use crate::performance::Performance;
use crate::saddlepoint_approximation::success_prob::honing_sa_wrapper;
use crate::state::StateBundle;

/// Sums up gold values from materials and juices
pub fn add_up_golds(mats_gold: &Vec<f64>, juice_gold: &Vec<(f64, f64)>) -> f64 {
    mats_gold.iter().fold(0.0, |last, new| last + *new)
        + juice_gold
            .iter()
            .fold(0.0, |last, new| last + new.0 + new.1)
}

/// Applies price/leftover pricing to concrete consumption values.
/// Used by Monte Carlo simulation where we have actual consumption, not expectations.
/// positive diff = leftover (use leftover_value), negative diff = shortage (use price)
pub fn apply_price_leftovers(
    mats: &[f64],
    juice: &[(f64, f64)],
    state_bundle: &StateBundle,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    apply_price_generic(mats, juice, state_bundle, false)
}

/// Applies naive (linear) pricing to concrete consumption values.
/// Used by Monte Carlo simulation. This is equivalent to apply_price_leftovers
/// when leftover_price = price.
pub fn apply_price_naive(
    mats: &[f64],
    juice: &[(f64, f64)],
    state_bundle: &StateBundle,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    apply_price_generic(mats, juice, state_bundle, true)
}

/// Generic price application for concrete consumption values.
/// When `naive` is true, uses price for both leftover and shortage.
fn apply_price_generic(
    mats: &[f64],
    juice: &[(f64, f64)],
    state_bundle: &StateBundle,
    naive: bool,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    let mut mats_gold = vec![0.0; mats.len()];
    let mut juice_gold = vec![(0.0, 0.0); juice.len()];

    for (index, gold) in mats_gold.iter_mut().enumerate() {
        let diff: f64 = state_bundle.prep_output.budgets[index] as f64 - mats[index];
        *gold = diff
            * if naive {
                state_bundle.prep_output.price_arr[index]
            } else if diff > 0.0 {
                state_bundle.prep_output.leftover_values[index]
            } else {
                state_bundle.prep_output.price_arr[index]
            };
    }

    for (id, (weap, armor)) in juice_gold.iter_mut().enumerate() {
        let weap_diff: f64 = state_bundle.prep_output.juice_books_owned[id].0 as f64 - juice[id].0;
        let armor_diff: f64 = state_bundle.prep_output.juice_books_owned[id].1 as f64 - juice[id].1;

        *weap = weap_diff
            * if naive {
                state_bundle.prep_output.juice_info.one_gold_cost_id[id].0
            } else if weap_diff > 0.0 {
                state_bundle.prep_output.juice_info.one_leftover_value_id[id].0
            } else {
                state_bundle.prep_output.juice_info.one_gold_cost_id[id].0
            };

        *armor = armor_diff
            * if naive {
                state_bundle.prep_output.juice_info.one_gold_cost_id[id].1
            } else if armor_diff > 0.0 {
                state_bundle.prep_output.juice_info.one_leftover_value_id[id].1
            } else {
                state_bundle.prep_output.juice_info.one_gold_cost_id[id].1
            };
    }

    (mats_gold, juice_gold)
}

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

pub fn generate_combined(state_bundle: &StateBundle) -> Vec<Vec<f64>> {
    let prep_output = &state_bundle.prep_output;
    let u_len: usize = prep_output.upgrade_arr.len();
    let mut combined_costs: Vec<Vec<f64>> = Vec::with_capacity(u_len);
    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        combined_costs.push(Vec::with_capacity(upgrade.prob_dist.len()));
        let mut cost_so_far: f64 = 0.0;
        for (p_index, _) in upgrade.log_prob_dist.iter().enumerate() {
            combined_costs[u_index].push(cost_so_far);
            cost_so_far += upgrade.eqv_gold_per_tap;
            let (juice, book_index) = upgrade.state[p_index];
            if juice {
                add_juice_gold_cost(&prep_output.juice_info, upgrade, &mut cost_so_far, 0);
            }
            if book_index > 0 {
                add_juice_gold_cost(
                    &prep_output.juice_info,
                    upgrade,
                    &mut cost_so_far,
                    book_index,
                );
            }
        }
        // combined_costs[u_index].push(cost_so_far);
    }
    combined_costs
}
pub fn generate_individual(
    state_bundle: &StateBundle,
) -> (Vec<Vec<Vec<f64>>>, Vec<Vec<Vec<f64>>>, Vec<Vec<Vec<f64>>>) {
    let prep_output = &state_bundle.prep_output;
    let u_len: usize = prep_output.upgrade_arr.len();
    let j_len: usize = prep_output.juice_info.one_gold_cost_id.len();

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
                    let (juice, book_index) = upgrade.state[p_index];
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

pub fn compute_leftover_probs(state_bundle: &mut StateBundle) -> Vec<f64> {
    init_dist(state_bundle);

    let (mut mats_costs, mut weap_juices_costs, mut armor_juices_costs) =
        generate_individual(state_bundle);
    let mut prob_leftover: Vec<f64> = Vec::new();
    let mut dummy_performance = Performance::new();
    for (t_index, support_arr) in mats_costs.iter_mut().enumerate() {
        prob_leftover.push(honing_sa_wrapper(
            state_bundle,
            support_arr,
            state_bundle.prep_output.budgets[t_index] as f64,
            &mut dummy_performance,
        ));
    }
    for (t_index, support_arr) in weap_juices_costs.iter_mut().enumerate() {
        prob_leftover.push(honing_sa_wrapper(
            state_bundle,
            support_arr,
            state_bundle.prep_output.juice_books_owned[t_index].0 as f64,
            &mut dummy_performance,
        ));
    }
    for (t_index, support_arr) in armor_juices_costs.iter_mut().enumerate() {
        prob_leftover.push(honing_sa_wrapper(
            state_bundle,
            support_arr,
            state_bundle.prep_output.juice_books_owned[t_index].1 as f64,
            &mut dummy_performance,
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

pub fn init_dist(state_bundle: &mut StateBundle) {
    // TODO add a toggle for computing log or not
    // dbg!(&prep_output, &state_bundle);
    // let zero_probs: Vec<f64> = special_probs(prep_output, state_bundle);
    // dbg!(&zero_probs);
    for upgrade in state_bundle.prep_output.upgrade_arr.iter_mut() {
        let prob_dist: Vec<f64> = new_prob_dist(
            &upgrade.state,
            &state_bundle.prep_output.juice_info,
            upgrade,
            0.0,
        );
        let log_prob_dist: Vec<f64> = prob_dist.iter().map(|x| x.ln()).collect();
        upgrade.prob_dist = prob_dist;
        upgrade.log_prob_dist = log_prob_dist;

        // gold_costs_arr.push(gold_cost_record);
    }
}
