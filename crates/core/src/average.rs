use crate::parser::PreparationOutput;
use crate::saddlepoint_approximation::normal_sa::init_dist;

use crate::state::StateBundle;
pub fn average_gold_wrapper(
    state_bundle: &mut StateBundle,
    prep_output: &mut PreparationOutput,
    states_evaled: &mut i64,
) -> f64 {
    init_dist(state_bundle, prep_output);
    *states_evaled += 1;

    let (avg_mats, avg_juices) = individual_averages(state_bundle, prep_output);
    let (mats_gold, juice_gold) = apply_price_leftovers(&avg_mats, &avg_juices, prep_output);
    add_up_golds(&mats_gold, &juice_gold)
}

pub fn add_up_golds(mats_gold: &Vec<f64>, juice_gold: &Vec<(f64, f64)>) -> f64 {
    mats_gold.iter().fold(0.0, |last, new| last + *new)
        + juice_gold
            .iter()
            .fold(0.0, |last, new| last + new.0 + new.1)
}

pub fn apply_price_leftovers(
    mats: &[f64],
    juice: &[(f64, f64)],
    prep_output: &mut PreparationOutput,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    // positive = leftover, negative = bad
    let mut mats_gold = vec![0.0; mats.len()];
    let mut juice_gold = vec![(0.0, 0.0); juice.len()];
    for (index, gold) in mats_gold.iter_mut().enumerate() {
        let diff: f64 = prep_output.budgets[index] as f64 - mats[index];
        *gold = diff
            * if diff > 0.0 {
                prep_output.leftover_values[index]
            } else {
                prep_output.price_arr[index]
            };
    }

    for (id, (weap, armor)) in juice_gold.iter_mut().enumerate() {
        let weap_diff: f64 = prep_output.juice_books_owned[id].0 as f64 - juice[id].0;
        let armor_diff: f64 = prep_output.juice_books_owned[id].1 as f64 - juice[id].1;
        if weap_diff > 0.0 {
            *weap = weap_diff * prep_output.juice_info.one_leftover_value[id].0
        } else {
            *weap = weap_diff * prep_output.juice_info.one_gold_cost[id].0
        }

        if armor_diff > 0.0 {
            *armor = armor_diff * prep_output.juice_info.one_leftover_value[id].1
        } else {
            *armor = armor_diff * prep_output.juice_info.one_gold_cost[id].1
        }
    }
    (mats_gold, juice_gold)
}

pub fn apply_price_naive(
    mats: &[f64],
    juice: &[(f64, f64)],
    prep_output: &mut PreparationOutput,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    // positive = leftover, negative = bad
    let mut mats_gold = vec![0.0; mats.len()];
    let mut juice_gold = vec![(0.0, 0.0); juice.len()];
    for (index, gold) in mats_gold.iter_mut().enumerate() {
        let diff: f64 = prep_output.budgets[index] as f64 - mats[index];
        *gold = diff * prep_output.price_arr[index]
    }

    for (id, (weap, armor)) in juice_gold.iter_mut().enumerate() {
        let weap_diff: f64 = prep_output.juice_books_owned[id].0 as f64 - juice[id].0;
        let armor_diff: f64 = prep_output.juice_books_owned[id].1 as f64 - juice[id].1;

        *weap = weap_diff * prep_output.juice_info.one_gold_cost[id].0;

        *armor = armor_diff * prep_output.juice_info.one_gold_cost[id].1;
    }
    (mats_gold, juice_gold)
}
pub fn individual_averages(
    state_bundle: &mut StateBundle,
    prep_output: &mut PreparationOutput,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    let mut avg_mats: Vec<f64> = vec![0.0; 7];
    let mut avg_juices: Vec<(f64, f64)> =
        vec![(0.0, 0.0); prep_output.juice_info.amt_used_id.len()];

    avg_mats[3] += prep_output.unlock_costs[0] as f64;
    avg_mats[6] += prep_output.unlock_costs[1] as f64;

    for (u_index, upgrade) in prep_output.upgrade_arr.iter_mut().enumerate() {
        let mut mats_so_far: Vec<f64> = vec![0.0; 7];
        let mut juice_so_far: Vec<f64> = vec![0.0; prep_output.juice_info.amt_used_id.len()];

        for (p_index, p) in upgrade.prob_dist.iter().enumerate() {
            let (juice, book_index) = state_bundle.state[u_index][p_index];
            for (index, avg_mat) in avg_mats.iter_mut().enumerate() {
                *avg_mat += p * mats_so_far[index];
                mats_so_far[index] += upgrade.costs[index] as f64;
            }

            for (id, (avg_weap, avg_armor)) in avg_juices.iter_mut().enumerate() {
                if upgrade.is_weapon {
                    *avg_weap += p * juice_so_far[id];
                } else {
                    *avg_armor += p * juice_so_far[id];
                }
                let juice_amt =
                    prep_output.juice_info.amt_used_id[id][upgrade.upgrade_index] as f64;
                if id == 0 && juice {
                    juice_so_far[id] += juice_amt;
                } else if id > 0
                    && prep_output.juice_info.ids[upgrade.upgrade_index][book_index] == id
                {
                    juice_so_far[id] += juice_amt;
                }
            }
        }
    }
    (avg_mats, avg_juices)
}
