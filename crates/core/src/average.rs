use crate::parser::{PreparationOutput, Upgrade};
use crate::saddlepoint_approximation::normal_sa::init_dist;
use crate::saddlepoint_approximation::special_sa::special_probs;
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
    let mut avg_mats = vec![0.0; 7];
    let mut avg_juices = vec![(0.0, 0.0); prep_output.juice_info.amt_used_id.len()];

    avg_mats[3] += prep_output.unlock_costs[0] as f64;
    avg_mats[6] += prep_output.unlock_costs[1] as f64;

    let mut upgrade_expectations: Vec<(Vec<f64>, Vec<(f64, f64)>)> =
        Vec::with_capacity(prep_output.upgrade_arr.len());
    for (u_index, upgrade) in prep_output.upgrade_arr.iter().enumerate() {
        let exp = expected_contribution(upgrade, &state_bundle.state[u_index], prep_output);
        upgrade_expectations.push(exp);
    }

    for (k, &special_prob) in special_probs(prep_output, state_bundle).iter().enumerate() {
        if k > 0 && special_prob < 1e-7 {
            break;
        }

        for &u_index in state_bundle.special_state.iter().skip(k) {
            let (mats_u, juices_u) = &upgrade_expectations[u_index];

            for i in 0..7 {
                avg_mats[i] += special_prob * mats_u[i];
            }

            for j in 0..avg_juices.len() {
                avg_juices[j].0 += special_prob * juices_u[j].0;
                avg_juices[j].1 += special_prob * juices_u[j].1;
            }
        }
    }

    (avg_mats, avg_juices)
}
fn expected_contribution(
    upgrade: &Upgrade,
    state_row: &[(bool, usize)],
    prep_output: &PreparationOutput,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    let mut exp_mats = vec![0.0; 7];
    let mut exp_juices = vec![(0.0, 0.0); prep_output.juice_info.amt_used_id.len()];

    let mut mats_so_far = vec![0.0; 7];
    let mut juice_so_far = vec![0.0; prep_output.juice_info.amt_used_id.len()];

    for (p_index, &p) in upgrade.prob_dist.iter().enumerate() {
        let (juice, book_index) = state_row[p_index];

        for i in 0..7 {
            exp_mats[i] += p * mats_so_far[i];
            mats_so_far[i] += upgrade.costs[i] as f64;
        }

        for id in 0..exp_juices.len() {
            if upgrade.is_weapon {
                exp_juices[id].0 += p * juice_so_far[id];
            } else {
                exp_juices[id].1 += p * juice_so_far[id];
            }

            let juice_amt = prep_output.juice_info.amt_used_id[id][upgrade.upgrade_index] as f64;

            if id == 0 && juice {
                juice_so_far[id] += juice_amt;
            } else if id > 0 && prep_output.juice_info.ids[upgrade.upgrade_index][book_index] == id
            {
                juice_so_far[id] += juice_amt;
            }
        }
    }

    (exp_mats, exp_juices)
}
