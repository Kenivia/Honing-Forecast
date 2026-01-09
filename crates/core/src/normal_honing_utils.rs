use crate::constants::JuiceInfo;
use crate::state::StateBundle;
use crate::upgrade::Upgrade;
pub fn generate_first_deltas(delta: f64, length: usize, non_zeros: usize) -> Vec<f64> {
    [vec![delta; non_zeros], vec![0.0; length - non_zeros]].concat()
}
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

pub fn add_juice_gold_cost(
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

// prob distribution of normal honing, adjusting for any juice usage
pub fn probability_distribution(
    base: f64,
    artisan_rate: f64,
    extra_arr: &[f64],
    zero: f64,
) -> Vec<f64> {
    let mut raw_chances: Vec<f64> = vec![zero];
    let mut artisan: f64 = 0.0_f64;
    let mut count: usize = 0;

    loop {
        let min_count: f64 = std::cmp::min(count, 10) as f64;
        let mut current_chance: f64 =
            base + (min_count * base) * 0.1 + extra_arr.get(count).unwrap_or(&0.0);

        if artisan >= 1.0 {
            current_chance = 1.0;
            raw_chances.push(current_chance);
            break;
        }

        raw_chances.push(current_chance);
        count += 1;
        artisan += (46.51_f64 / 100.0) * current_chance * artisan_rate;
        if current_chance == 1.0 {
            break; // for upgrades that have 100% passrate immediately
        }
    }

    // convert raw per-try chances into per-tap probability distribution
    let mut chances = vec![0.0_f64; raw_chances.len()];
    let mut cum_chance = 1.0_f64;
    for (idx, &element) in raw_chances.iter().enumerate() {
        chances[idx] = cum_chance * element;
        cum_chance *= 1.0 - element;
    }

    chances
}
