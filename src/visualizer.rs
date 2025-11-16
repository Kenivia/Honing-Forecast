use crate::helpers::{generate_first_deltas, get_one_tap_pity};
use crate::parser::{PreparationOutputs, Upgrade, probability_distribution};
// use crate::value_estimation::explore_one;
use crate::helpers::compute_gold_cost_from_raw;
#[cfg(test)]
use crate::test_utils::PROB_MODE;
use itertools::{Itertools, iproduct};

use rayon::prelude::*;
use std::f64;
use std::sync::{Arc, RwLock};
// use std::time::Instant;

#[cfg(test)]
pub fn brute(
    input_budgets: &[i64],
    prep_outputs: &PreparationOutputs,
) -> Vec<Vec<Vec<(f64, String)>>> {
    let u0 = &prep_outputs.upgrade_arr[0];
    let u1 = &prep_outputs.upgrade_arr[1];

    let len0 = u0.full_juice_len + 1;
    let len1 = u1.full_juice_len + 1;

    // === Precompute supports for every juice value ===
    // let support = Instant::now();
    let supports0: Vec<(Vec<([i64; 9], f64)>, Vec<f64>)> = precompute_supports(u0, len0);
    let supports1: Vec<(Vec<([i64; 9], f64)>, Vec<f64>)> = precompute_supports(u1, len1);
    // dbg!(support.elapsed());
    let num_p = 101; // 0..=99 for 1%–100%, 100 = worst-case (100%)
    let flat_size = num_p * len0 * len1;

    let flat_results: Arc<RwLock<Vec<(f64, String)>>> = Arc::new(RwLock::new(vec![
        (
            if PROB_MODE { 0.0_f64 } else { f64::MAX },
            "uninitiated".to_owned(),
        );
        flat_size
    ]));

    let stride_p = len0 * len1;
    let stride0 = len1;
    let worst_cost: f64 = compute_gold_cost_from_raw(
        &get_one_tap_pity(&prep_outputs.upgrade_arr, &prep_outputs.unlock_costs)[1],
        &input_budgets,
        &prep_outputs.mats_value,
    );
    let best_cost: f64 = compute_gold_cost_from_raw(
        &get_one_tap_pity(&prep_outputs.upgrade_arr, &prep_outputs.unlock_costs)[0],
        &input_budgets,
        &prep_outputs.mats_value,
    );

    // === Parallel over every (juice0, juice1) pair ===
    iproduct!(0..supports0.len(), 0..supports1.len())
        .par_bridge()
        .for_each(|(j0, j1)| {
            let combined = build_combined_prob_dist(
                &supports0[j0],
                &supports1[j1],
                input_budgets,
                &prep_outputs.mats_value,
            );

            let quantiles = compute_quantiles(combined, worst_cost, 34567.0);

            // lock and write
            // let mut vec = flat_results.lock().unwrap();
            for (i, val) in quantiles.iter().enumerate() {
                // Compute idx first (no lock)
                let idx = i * stride_p
                    + supports0[j0].1.iter().filter(|x| **x > 0.0).count() * stride0
                    + supports1[j1].1.iter().filter(|x| **x > 0.0).count();

                // Shared read lock
                {
                    let vec = flat_results.read().unwrap();
                    let current = &vec[idx];

                    let should_update = (PROB_MODE && val.0 > current.0)
                        || (!PROB_MODE && val.0 < current.0)
                        || (val.0 == current.0 && current.1 == "uninitiated");

                    if !should_update {
                        return;
                    }
                }

                // Exclusive write lock
                {
                    let mut vec = flat_results.write().unwrap();
                    let current = &vec[idx];
                    // Re-check
                    if (PROB_MODE && val.0 > current.0)
                        || (!PROB_MODE && val.0 < current.0)
                        || (val.0 == current.0 && current.1 == "uninitiated")
                    {
                        vec[idx] = val.clone();
                    }
                }
            }
        });

    let result_vec = Arc::try_unwrap(flat_results)
        .expect("no other refs")
        .into_inner()
        .unwrap();
    // === Reshape flat → nested Vec<Vec<Vec<f64>>> (as your original signature) ===
    let mut out: Vec<Vec<Vec<(f64, String)>>> =
        vec![vec![vec![(0.0_f64, "uninitiated".to_owned()); len1]; len0]; num_p];
    for p in 0..num_p {
        let base_p = p * stride_p;
        for j0 in 0..len0 {
            let base0 = base_p + j0 * stride0;
            for j1 in 0..len1 {
                out[p][j0][j1] = result_vec[base0 + j1].clone();
            }
        }
    }

    out
}

// ==============================================================

pub fn arrangements<T: Clone + Default>(p: T, n: usize, k: usize) -> Vec<Vec<T>> {
    // if impossible, return empty (could also return vec![vec![T::default(); n]] when k==0)
    if k > n {
        return Vec::new();
    }

    // handle k == 0: single vector of defaults
    if k == 0 {
        return vec![vec![T::default(); n]];
    }

    let mut out = Vec::new();

    // iterate combinations of indices 0..n taken k at a time
    for comb in (0..n).combinations(k) {
        let mut v = vec![T::default(); n];
        for &idx in &comb {
            v[idx] = p.clone();
        }
        out.push(v);
    }

    out
}

fn precompute_supports(
    upgrade: &Upgrade,
    max_juice: usize,
) -> Vec<(Vec<([i64; 9], f64)>, Vec<f64>)> {
    let mut supports: Vec<(Vec<([i64; 9], f64)>, Vec<f64>)> = Vec::new();

    for juice in 0..=max_juice {
        let first_deltas = generate_first_deltas(upgrade.base_chance, upgrade.prob_dist_len, juice);
        let dist_first_deltas =
            probability_distribution(upgrade.base_chance, upgrade.artisan_rate, &first_deltas);
        let chance_arrangements: Vec<Vec<f64>> =
            arrangements(upgrade.base_chance, dist_first_deltas.len(), juice);

        for chance_delta_arr in chance_arrangements.iter() {
            let this_dist = probability_distribution(
                upgrade.base_chance,
                upgrade.artisan_rate,
                &chance_delta_arr,
            );
            let mut list = Vec::with_capacity(dist_first_deltas.len());
            let mut juice_count_so_far: i64 = 0;
            for tap in 0..this_dist.len() {
                let prob: f64 = this_dist[tap];
                if prob == 0.0 {
                    continue;
                }

                let taps_real = tap as i64 + upgrade.tap_offset;
                let mut this_costs = [0_i64; 9];

                for c in 0..7 {
                    this_costs[c] = taps_real * upgrade.costs[c];
                }
                if upgrade.is_normal_honing {
                    if tap < chance_delta_arr.len() && chance_delta_arr[tap] > 0.0 {
                        juice_count_so_far += 1;
                    }
                    let j_idx = if upgrade.is_weapon { 7 } else { 8 };
                    this_costs[j_idx] = juice_count_so_far * upgrade.one_juice_cost;
                }

                list.push((this_costs, prob));
            }
            supports.push((list, chance_delta_arr.clone()));
        }
    }
    supports
}

fn build_combined_prob_dist(
    input1: &(Vec<([i64; 9], f64)>, Vec<f64>),
    input2: &(Vec<([i64; 9], f64)>, Vec<f64>),
    input_budgets: &[i64],
    mats_value: &Vec<f64>,
) -> (Vec<(f64, f64)>, String) {
    let cap: usize = input1.0.len() * input2.0.len();
    let mut combined: Vec<(f64, f64)> = Vec::with_capacity(cap);

    for entries_1 in input1.0.iter() {
        for entries_2 in input2.0.iter() {
            let costs_1 = entries_1.0;
            let p1 = entries_1.1;
            let costs_2 = entries_2.0;
            let p2 = entries_2.1;
            let mut costs = [0_i64; 9];
            for i in 0..9 {
                costs[i] = costs_1[i] + costs_2[i];
            }

            let gold: f64 = compute_gold_cost_from_raw(&costs, input_budgets, mats_value);
            let prob: f64 = p1 * p2;

            if prob > 0.0 {
                combined.push((gold, prob));
            }
        }
    }

    (combined, encode_positions(&input1.1, &input2.1))
}

pub fn encode_positions(v1: &[f64], v2: &[f64]) -> String {
    let mut s1 = String::new();
    let mut s2 = String::new();

    for i in 0..v1.len() {
        // first vector row
        if v1[i] > 0.0 {
            s1.push('1');
        } else {
            s1.push('0');
        }
    }
    for i in 0..v2.len() {
        // second vector row
        if v2[i] > 0.0 {
            s2.push('2');
        } else {
            s2.push('0');
        }
    }

    format!("{s1} {s2}")
}

#[cfg(test)]
fn compute_quantiles(
    mut input: (Vec<(f64, f64)>, String),
    worst_cost: f64,
    best_cost: f64,
) -> [(f64, String); 101] {
    // .0 = gold cost, .1 = probability

    let mut res: Vec<(f64, String)> = Vec::with_capacity(101);

    let mut cum_cost: f64 = 0.0;
    let mut cum_chance: f64 = 0.0;
    let outcomes = &mut input.0;
    outcomes.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut i: usize = 0;
    let n: usize = outcomes.len();

    for seggment in 0..=100 {
        // just evenly dividing the best to the worst rn
        if PROB_MODE {
            let target: f64 = seggment as f64 * (worst_cost - best_cost) / 100.0 + best_cost;

            while i < n && cum_cost < target {
                cum_cost = outcomes[i].0;
                cum_chance += outcomes[i].1;
                i += 1;
            }
            let candidate: f64 = if i > 0 { cum_chance } else { 0.0 };

            res.push((candidate, input.1.clone()));
        } else {
            while i < n && cum_chance < seggment as f64 / 100.0 {
                cum_chance += outcomes[i].1;
                i += 1;
            }
            res.push((outcomes[i.min(n - 1)].0, input.1.clone()))
        }
    }
    res.try_into().unwrap()
    // res[100] = worst;
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::constants::RNG_SEED;
    use crate::parser::preparation;
    use crate::test_utils::*;
    use std::time::Instant;

    #[test]
    fn brute_arrangement_test() {
        let start = Instant::now();
        let suffix: &str = if PROB_MODE { "_prob" } else { "_gold" };
        let test_name = format!("brute_arrangement_test{suffix}");
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 10 { 1 } else { 0 }).collect(),
            (0..25).map(|x| if x == 10 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = false;
        let input_budgets = vec![
            3240, 9240, 46, 17740, 36, 0, 108000, 90, 90, 0,
            // 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        // let data_size: usize = 100000;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value,
            // data_size,
            RNG_SEED,
            PROB_MODE
        );

        let prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );
        let result: Vec<Vec<Vec<(f64, String)>>> = brute(&input_budgets, &prep_outputs);
        dbg!(result.len());
        // let result: Vec<Vec<i64>> = out.clone();
        if let Some(_cached_result) =
            read_cached_data::<Vec<Vec<Vec<(f64, String)>>>>(test_name.as_str(), &hash)
        {
            // my_assert!(*result, cached_result);
        } else {
            write_cached_data(test_name.as_str(), &hash, &result);
        }
        dbg!(start.elapsed());
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
