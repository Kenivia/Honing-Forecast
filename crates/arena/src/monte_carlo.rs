//! Simulates honing directly and compares the gold it averages to what the evaluator says.
//!
//! Free taps and advanced honing are simulated from the game rules. Normal honing samples
//! the upgrade's own tap distribution, so this checks everything built on top of that
//! distribution, not the distribution itself.

use crate::one_adv_sim::one_sim;
use hf_core::materials::NUM_BASE_MATS;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use hf_core::upgrade::Upgrade;
use itertools::izip;
use rand::Rng;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

const BATCH: usize = 100_000;

#[derive(Clone, Serialize, Deserialize)]
pub struct McSpec {
    pub samples: usize,
    /// how many standard errors of sampling noise to allow
    pub z: f64,
    /// how much approximation error to allow on top, relative to the value
    pub rel_tol: f64,
}

#[derive(Serialize, Deserialize)]
pub struct McRow {
    pub material: String,
    pub analytic: f64,
    pub mean: f64,
    pub se: f64,
    pub z: f64,
    pub pass: bool,
}

/// Instead of actually sampling the distribution, we guarantee that every value has exactly the expected number of occurances
fn tap_map_generator<R: Rng>(count_limit: usize, prob_dist: &[f64], rng: &mut R) -> Vec<usize> {
    let mut tap_map: Vec<usize> = Vec::with_capacity(count_limit);

    let mut current_point: f64 = rng.random::<f64>();
    let mut cum_prob: f64 = 0.0;

    for (i, &p) in prob_dist.iter().enumerate() {
        cum_prob += p;
        let target_boundary: f64 = cum_prob * (count_limit as f64);
        while current_point < target_boundary && tap_map.len() < count_limit {
            tap_map.push(i);
            current_point += 1.0;
        }
    }

    // the distribution may sum to slightly under 1
    while tap_map.len() < count_limit {
        tap_map.push(prob_dist.len() - 1);
    }
    tap_map.shuffle(rng);
    tap_map
}

fn sample_truncated_geometric<R: Rng>(p: f64, max_taps: i64, rng: &mut R) -> i64 {
    if p >= 1.0 {
        return 1;
    }
    let u: f64 = rng.random_range(0.0..1.0);
    let k: i64 = (u.log(1.0 - p).ceil() as i64).max(1);
    if k > max_taps { max_taps + 1 } else { k }
}

/// juice_before[tap][id] = amount of the juice used by the taps before succeeding on `tap`
fn juice_before(upgrade: &Upgrade, state_bundle: &StateBundle) -> Vec<Vec<f64>> {
    let juice_info = &state_bundle.prep_output.juice_info;
    let len = upgrade.normal_dist.len();
    let mut so_far: Vec<f64> = vec![0.0; juice_info.num_juice_avail];
    let mut out: Vec<Vec<f64>> = Vec::with_capacity(len);
    for tap in 0..len {
        out.push(so_far.clone());
        // the pity tap takes no juice
        if tap + 2 >= len {
            continue;
        }
        for &id in upgrade.state.get(tap).into_iter().flatten() {
            so_far[id] += juice_info
                .access(id, upgrade.piece_type_usize, upgrade.upgrade_index)
                .normal_amt_used as f64;
        }
    }
    out
}

/// amount of every material used in each of `data_size` simulated runs
fn simulate<R: Rng>(data_size: usize, state_bundle: &StateBundle, rng: &mut R) -> Vec<Vec<f64>> {
    let prep = &state_bundle.prep_output;
    let juice_info = &prep.juice_info;
    let mut cost_data: Vec<Vec<f64>> = vec![vec![0.0; prep.table.len()]; data_size];
    let mut special_left: Vec<i64> = vec![prep.special_budget; data_size];
    let mut highest_upgrade_index_seen: Vec<i64> = vec![-1; 7];

    for u_index in state_bundle.special_state.iter() {
        let upgrade = &state_bundle.upgrade_arr[*u_index];
        if upgrade.is_normal_honing {
            let tap_map: Vec<usize> = tap_map_generator(data_size, &upgrade.normal_dist, rng);
            let juice_before = juice_before(upgrade, state_bundle);
            let special_valid =
                highest_upgrade_index_seen[upgrade.piece_index] <= upgrade.upgrade_index as i64;
            if special_valid {
                highest_upgrade_index_seen[upgrade.piece_index] = upgrade.upgrade_index as i64;
            }

            for (this_cost, this_special_left, rolled_tap) in
                izip!(cost_data.iter_mut(), special_left.iter_mut(), tap_map)
            {
                if !upgrade.unlocked {
                    for row in 0..NUM_BASE_MATS {
                        this_cost[row] += upgrade.unlock_costs[row];
                    }
                }
                let affordable = (*this_special_left / upgrade.special_cost).max(0);
                if special_valid && affordable > 0 {
                    let taps = sample_truncated_geometric(upgrade.base_chance, affordable, rng);
                    *this_special_left -= taps * upgrade.special_cost;
                    if *this_special_left >= 0 {
                        continue;
                    }
                } else {
                    // free taps stop at the first upgrade they cannot be tried on
                    *this_special_left = -1;
                }

                for row in 0..NUM_BASE_MATS {
                    this_cost[row] += upgrade.costs[row] * rolled_tap as f64;
                }
                for &id in juice_info.normal_uindex_to_id[upgrade.piece_type_usize]
                    [upgrade.upgrade_index]
                    .iter()
                {
                    this_cost[prep.table.juice_row(id)] += juice_before[rolled_tap][id];
                }
            }
        } else {
            for (this_cost, this_special_left) in cost_data.iter_mut().zip(special_left.iter_mut())
            {
                *this_special_left = -1;
                if upgrade.adv_config.start_xp == 0 {
                    for row in 0..NUM_BASE_MATS {
                        this_cost[row] += upgrade.unlock_costs[row];
                    }
                }
                let (taps, juice, scroll) = one_sim(rng, &upgrade.adv_config);
                for row in 0..NUM_BASE_MATS {
                    this_cost[row] += upgrade.costs[row] * taps as f64;
                }
                for &id in juice_info.adv_uindex_to_id[upgrade.piece_type_usize]
                    [upgrade.upgrade_index]
                    .iter()
                {
                    let used = if id <= 1 { juice } else { scroll } as f64;
                    let amt = juice_info
                        .access(id, upgrade.piece_type_usize, upgrade.upgrade_index)
                        .adv_amt_used as f64;
                    this_cost[prep.table.juice_row(id)] += amt * used;
                }
            }
        }
    }
    cost_data
}

/// gold value of having used this much, the function the evaluator takes the mean of
pub fn gold(used: f64, bands: &[(f64, f64)]) -> f64 {
    let (last_thresh, last_price) = *bands.last().unwrap();
    let mut out = last_price * (last_thresh - used);
    for pair in bands.windows(2) {
        out += (pair[0].1 - pair[1].1) * (pair[1].0 - used).max(0.0);
    }
    out
}

/// One row per material and one for the total, under the optimizer's plan, against the reference
/// evaluator, not the one the state was optimized with.
pub fn check<R: Rng>(state_bundle: &mut StateBundle, spec: &McSpec, rng: &mut R) -> Vec<McRow> {
    let tuning = std::mem::replace(&mut state_bundle.tuning, Tuning::reference());
    let (metric_per_plan, _, breakdown) =
        state_bundle.ui_average_gold_metric(&mut Probe::default());
    state_bundle.tuning = tuning;
    let prep = &state_bundle.prep_output;
    let bands = &prep.bands[prep.optimizer_plan];
    let num_mats = prep.table.len();

    let mut sum: Vec<f64> = vec![0.0; num_mats + 1];
    let mut sum_sq: Vec<f64> = vec![0.0; num_mats + 1];
    let mut done: usize = 0;
    while done < spec.samples {
        let batch = BATCH.min(spec.samples - done);
        for row in simulate(batch, state_bundle, rng) {
            let mut total: f64 = 0.0;
            for (index, used) in row.iter().enumerate() {
                let this = gold(*used, &bands[index]);
                sum[index] += this;
                sum_sq[index] += this * this;
                total += this;
            }
            sum[num_mats] += total;
            sum_sq[num_mats] += total * total;
        }
        done += batch;
    }

    let n = done as f64;
    let mut analytic: Vec<f64> = breakdown[prep.optimizer_plan].clone();
    analytic.push(metric_per_plan[prep.optimizer_plan]);
    let mut labels: Vec<String> = prep.table.labels.clone();
    labels.push("total".to_string());

    izip!(labels, analytic, sum, sum_sq)
        .map(|(material, analytic, sum, sum_sq)| {
            let mean = sum / n;
            let se = ((sum_sq / n - mean * mean).max(0.0) / (n - 1.0)).sqrt();
            let diff = analytic - mean;
            McRow {
                material,
                analytic,
                mean,
                se,
                z: if se > 0.0 { diff / se } else { 0.0 },
                pass: diff.abs() <= spec.z * se + spec.rel_tol * analytic.abs().max(1.0),
            }
        })
        .collect()
}
