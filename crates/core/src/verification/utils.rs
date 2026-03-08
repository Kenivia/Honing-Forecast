use crate::{
    constants::juice_info::JuiceInfo, parser::PreparationOutput, state_bundle::StateBundle,
    upgrade::Upgrade,
};

pub fn add_up_golds(mats_gold: &Vec<f64>, juice_gold: &Vec<(f64, f64)>) -> f64 {
    mats_gold.iter().fold(0.0, |last, new| last + *new)
        + juice_gold
            .iter()
            .fold(0.0, |last, new| last + new.0 + new.1)
}

// these three functions below were vibe coded
/// Applies price/leftover pricing to concrete consumption values.
/// Used by Monte Carlo simulation where we have actual consumption, not expectations.
/// positive diff = leftover (use leftover_value), negative diff = shortage (use price)
pub fn apply_price_leftovers(
    mats: &[f64],
    juice: &[(f64, f64)],
    state_bundle: &StateBundle,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    apply_price_generic(mats, juice, &state_bundle.prep_output, false)
}

/// Applies naive (linear) pricing to concrete consumption values.
/// Used by Monte Carlo simulation. This is equivalent to apply_price_leftovers
/// when leftover_price = price.
pub fn apply_price_naive(
    mats: &[f64],
    juice: &[(f64, f64)],
    state_bundle: &StateBundle,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    apply_price_generic(mats, juice, &state_bundle.prep_output, true)
}

/// Generic price application for concrete consumption values.
/// When `naive` is true, uses price for both leftover and shortage.
pub fn apply_price_generic(
    mats: &[f64],
    juice: &[(f64, f64)],
    prep_output: &PreparationOutput,
    naive: bool,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    let mut mats_gold = vec![0.0; mats.len()];
    let mut juice_gold = vec![(0.0, 0.0); juice.len()];

    for (index, gold) in mats_gold.iter_mut().enumerate() {
        let diff: f64 = prep_output.budgets[index] as f64 - mats[index];
        *gold = diff
            * if naive {
                prep_output.price_arr[index]
            } else if diff > 0.0 {
                prep_output.leftover_values[index]
            } else {
                prep_output.price_arr[index]
            };
    }

    for (id, (weap, armor)) in juice_gold.iter_mut().enumerate() {
        let weap_diff: f64 = prep_output.juice_books_owned[id].0 - juice[id].0;
        let armor_diff: f64 = prep_output.juice_books_owned[id].1 - juice[id].1;

        *weap = weap_diff
            * if naive {
                prep_output.juice_info.all_juices[id].prices.0
            } else if weap_diff > 0.0 {
                prep_output.juice_info.all_juices[id].leftover_values.0
            } else {
                prep_output.juice_info.all_juices[id].prices.0
            };

        *armor = armor_diff
            * if naive {
                prep_output.juice_info.all_juices[id].prices.1
            } else if armor_diff > 0.0 {
                prep_output.juice_info.all_juices[id].leftover_values.1
            } else {
                prep_output.juice_info.all_juices[id].prices.1
            };
    }

    (mats_gold, juice_gold)
}

pub fn add_juice_gold_cost(
    juice_info: &JuiceInfo,
    upgrade: &Upgrade,
    cost_so_far: &mut f64,
    id: usize,
) {
    *cost_so_far += juice_info.all_juices[id][&upgrade.upgrade_index].normal_amt_used as f64
        * if upgrade.is_weapon {
            juice_info.all_juices[id].prices.0
        } else {
            juice_info.all_juices[id].prices.1
        };
}

pub fn encode_one_positions(v1: &[(bool, usize)]) -> String {
    v1.iter()
        .map(|(uppercase, num)| {
            let letter: char = if *num == 0 {
                'x'
            } else {
                (b'a' + (*num as u8 - 1)) as char
            };

            if *uppercase {
                letter.to_ascii_uppercase()
            } else {
                letter
            }
        })
        .collect()
}

impl StateBundle {
    pub fn encode_all(&self) -> String {
        let mut strings = Vec::new();
        strings.push(format!("{:?}", self.special_state));
        for (index, upgrade) in self.upgrade_arr.iter().enumerate() {
            strings.push(
                self.upgrade_arr[index].name_string.clone()
                    + ": "
                    + &encode_one_positions(&upgrade.state),
            );
        }
        strings.join("\n")
    }
}
