use crate::helpers::generate_first_deltas;
use crate::parser::{PreparationOutputs, Upgrade, probability_distribution};
// use crate::value_estimation::explore_one;
use crate::helpers::compute_gold_cost_from_raw;
use itertools::{Itertools, iproduct};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

// pub fn decision_space_iterator(
//     max_juice_counts: Vec<i64>,
// ) -> impl Iterator<Item = (Vec<i64>, Vec<i64>)> {
//     // let mut max_juice_counts: Vec<i64> = Vec::with_capacity(upgrade_arr.len());
//     // for upgrade in upgrade_arr {
//     //     max_juice_counts.push(upgrade.full_juice_len as i64);
//     // }
//     let juice_decision_space: Vec<Vec<i64>> = max_juice_counts
//         .into_iter()
//         .map(|x: i64| (0..x).collect())
//         .collect();

//     // decision_space.push();
//     // dbg!(&juice_decision_space);
//     iproduct!(
//         juice_decision_space.into_iter().multi_cartesian_product(),
//         vec![vec![0]].into_iter() //(0..upgrade_arr.len() as i64).permutations(10.min(upgrade_arr.len()))
//     )
// }

// fn brute(
//     input_budgets: &[i64],
//     prep_outputs: &PreparationOutputs,
//     data_size: usize,
// ) -> Vec<Vec<Vec<f64>>> {
//     let mut out: Vec<Vec<Vec<f64>>> =
//         vec![
//             vec![
//                 vec![-1.0; prep_outputs.upgrade_arr[1].full_juice_len];
//                 prep_outputs.upgrade_arr[0].full_juice_len
//             ];
//             101
//         ];

//     for decision in decision_space_iterator(
//         prep_outputs
//             .upgrade_arr
//             .iter()
//             .map(|x| x.full_juice_len as i64)
//             .collect(),
//     ) {
//         // dbg!(&decision);
//         let result = explore_one(&decision, &input_budgets, prep_outputs, data_size);
//         for i in 0..result.len() {
//             out[i][decision.0[0] as usize][decision.0[1] as usize] = result[i];
//         }
//     }
//     out
// }

pub fn brute(
    input_budgets: &[i64],
    prep_outputs: &PreparationOutputs,
    _data_size: usize,
) -> Vec<Vec<Vec<f64>>> {
    let u0 = &prep_outputs.upgrade_arr[0];
    let u1 = &prep_outputs.upgrade_arr[1];

    let len0 = u0.full_juice_len;
    let len1 = u1.full_juice_len;

    // === Precompute supports for every juice value ===
    let supports0 = precompute_supports(u0, len0);
    let supports1 = precompute_supports(u1, len1);

    let num_p = 101; // 0..=99 for 1%–100%, 100 = worst-case (100%)
    let flat_size = num_p * len0 * len1;
    let mut flat_results = Arc::new(Mutex::new(vec![0.0_f64; flat_size]));

    let stride_p = len0 * len1;
    let stride0 = len1;

    // === Parallel over every (juice0, juice1) pair ===
    iproduct!(0..len0, 0..len1)
        .par_bridge()
        .for_each(|(j0, j1)| {
            let combined = build_combined_prob_dist(
                &supports0[j0],
                &supports1[j1],
                input_budgets,
                &prep_outputs.mats_value,
            );

            let quantiles = compute_quantiles(combined);

            // lock and write
            let mut vec = flat_results.lock().unwrap();
            for (p, &val) in quantiles.iter().enumerate() {
                let idx = p * stride_p + j0 * stride0 + j1;
                vec[idx] = val;
            }
        });

    let result_vec = Arc::try_unwrap(flat_results)
        .expect("no other refs")
        .into_inner()
        .unwrap();
    // === Reshape flat → nested Vec<Vec<Vec<f64>>> (as your original signature) ===
    let mut out = vec![vec![vec![0.0_f64; len1]; len0]; num_p];
    for p in 0..num_p {
        let base_p = p * stride_p;
        for j0 in 0..len0 {
            let base0 = base_p + j0 * stride0;
            for j1 in 0..len1 {
                out[p][j0][j1] = result_vec[base0 + j1];
            }
        }
    }

    out
}

// ==============================================================

type SupportEntry = ([i64; 9], f64);

fn precompute_supports(upgrade: &Upgrade, max_juice: usize) -> Vec<Vec<SupportEntry>> {
    let mut supports = vec![Vec::new(); max_juice];

    for juice in 0..max_juice {
        let first_deltas = generate_first_deltas(upgrade.base_chance, upgrade.prob_dist_len, juice);
        let dist =
            probability_distribution(upgrade.base_chance, upgrade.artisan_rate, &first_deltas);

        let mut list = Vec::with_capacity(dist.len());
        for tap in 0..dist.len() {
            let prob = dist[tap];
            if prob == 0.0 {
                // huge speedup in practice
                continue;
            }

            let taps_real = tap as i64 + upgrade.tap_offset;
            let mut delta = [0_i64; 9];

            for c in 0..7 {
                delta[c] = taps_real * upgrade.costs[c];
            }
            if upgrade.is_normal_honing {
                let j_idx = if upgrade.is_weapon { 7 } else { 8 };
                let juice_used = taps_real.min(juice as i64);
                delta[j_idx] = juice_used * upgrade.one_juice_cost;
            }

            list.push((delta, prob));
        }
        supports[juice] = list;
    }
    supports
}

fn build_combined_prob_dist(
    list0: &[SupportEntry],
    list1: &[SupportEntry],
    input_budgets: &[i64],
    mats_value: &Vec<f64>,
) -> Vec<(f64, f64)> {
    let cap = list0.len() * list1.len();
    let mut combined = Vec::with_capacity(cap);

    for &(delta0, p0) in list0 {
        for &(delta1, p1) in list1 {
            let mut cost = [0_i64; 9];
            for i in 0..9 {
                cost[i] = delta0[i] + delta1[i];
            }

            let gold = compute_gold_cost_from_raw(&cost, input_budgets, mats_value);
            let prob = p0 * p1;

            if prob > 0.0 {
                combined.push((gold, prob));
            }
        }
    }
    combined
}

fn compute_quantiles(mut outcomes: Vec<(f64, f64)>) -> [f64; 101] {
    if outcomes.is_empty() {
        return [0.0; 101];
    }

    outcomes.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut res = [0.0_f64; 101];
    let mut cum = 0.0_f64;
    let mut i = 0;
    let n = outcomes.len();
    let worst = outcomes[n - 1].0;

    // 0..=99 → 1% to 100% quantiles, 100 → explicit worst-case
    for percentile in 1..=100 {
        let target = percentile as f64 * 0.01;

        while i < n && cum < target {
            cum += outcomes[i].1;
            i += 1;
        }
        res[percentile - 1] = if i > 0 { outcomes[i - 1].0 } else { worst };
    }
    res[100] = worst;

    res
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::RNG_SEED;
    use crate::parser::{parser, preparation};
    use crate::test_utils::*;
    use crate::{calculate_hash, my_assert};

    #[test]
    fn brute_test() {
        let test_name: &str = "brute_test";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
            (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let input_budgets = vec![
            324000, 924000, 4680, 1774000, 3600, 406800, 10800000, 900, 900, 0,
        ];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        let data_size: usize = 100000;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value,
            data_size,
            RNG_SEED
        );

        let prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );
        let result: Vec<Vec<Vec<f64>>> = brute(&input_budgets, &prep_outputs, data_size);
        dbg!(result.len());
        // let result: Vec<Vec<i64>> = out.clone();
        if let Some(cached_result) = read_cached_data::<Vec<Vec<Vec<f64>>>>(test_name, &hash) {
            // my_assert!(*result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result);
        }
        // let result: Vec<(Vec<i64>, Vec<i64>)> = brute(&mut upgrade_arr);
        // dbg!(result.len());
        // // let result: Vec<Vec<i64>> = out.clone();
        // if let Some(cached_result) = read_cached_data::<Vec<(Vec<i64>, Vec<i64>)>>(test_name, &hash)
        // {
        //     my_assert!(*result, cached_result);
        // } else {
        //     write_cached_data(test_name, &hash, &result);
        // }
    }
}
