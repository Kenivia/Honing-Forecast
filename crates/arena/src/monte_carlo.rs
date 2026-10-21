//! Simulates honing directly and compares the gold it averages to what the evaluator says.
//!
//! Free taps and advanced honing are simulated from the game rules. Normal honing samples
//! the upgrade's own tap distribution, so this checks everything built on top of that
//! distribution, not the distribution itself.

use hf_core::advanced_honing::sim::one_sim;
use hf_core::gold_dist::tap_map_generator;
use hf_core::materials::{NUM_BASE_MATS, gold};
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use hf_core::upgrade::Upgrade;
use itertools::izip;
use rand::Rng;
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

/// The metric of one unlocked normal upgrade without free taps, summed exactly over the tap it succeeds on
#[cfg(test)]
pub fn exact_single(state_bundle: &StateBundle) -> f64 {
    let prep = &state_bundle.prep_output;
    let bands = &prep.bands[prep.optimizer_plan];
    single_outcomes(state_bundle)
        .iter()
        .map(|(p, used)| {
            p * used
                .iter()
                .zip(bands)
                .map(|(used, bands)| gold(*used, bands))
                .sum::<f64>()
        })
        .sum()
}

/// (probability, amount of every material used) for each tap one upgrade can succeed on
#[cfg(test)]
pub fn single_outcomes(state_bundle: &StateBundle) -> Vec<(f64, Vec<f64>)> {
    let prep = &state_bundle.prep_output;
    let upgrade = &state_bundle.upgrade_arr[0];
    assert!(state_bundle.upgrade_arr.len() == 1 && upgrade.unlocked && prep.special_budget == 0);
    let juice_before = juice_before(upgrade, state_bundle);
    let ids = &prep.juice_info.normal_uindex_to_id[upgrade.piece_type_usize][upgrade.upgrade_index];
    upgrade
        .normal_dist
        .iter()
        .enumerate()
        .map(|(tap, p)| {
            let mut used = vec![0.0; prep.table.len()];
            for row in 0..NUM_BASE_MATS {
                used[row] = upgrade.costs[row] * tap as f64;
            }
            for &id in ids {
                used[prep.table.juice_row(id)] += juice_before[tap][id];
            }
            (*p, used)
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::dummy;
    use hf_core::gold_dist::{gold_distribution, gold_samples};
    use hf_core::optimizer::scramble;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// the sampled spend against the metric and against free taps played out by the game rules
    #[test]
    fn gold_distribution_matches_monte_carlo() {
        const N: usize = 100_000;
        for payload in dummy().steps {
            let mut rng = StdRng::seed_from_u64(1);
            let mut sb = scramble(&mut rng, StateBundle::init_from_payload(payload), 20);
            sb.tuning = Tuning::reference();
            let (metric_per_plan, _, _) = sb.ui_average_gold_metric(&mut Probe::default());
            let bands = &sb.prep_output.bands[0];
            let ours = gold_samples(&sb, 0, N, &mut rng);

            let n = N as f64;
            let mean = ours.iter().sum::<f64>() / n;
            let var = ours.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
            let diff = mean + metric_per_plan[0];
            assert!(
                diff.abs() <= 4.0 * (var / n).sqrt() + 1e-3 * mean.abs().max(1.0),
                "{mean} {}",
                -metric_per_plan[0]
            );

            let mut theirs: Vec<f64> = simulate(N, &sb, &mut rng)
                .iter()
                .map(|row| -row.iter().zip(bands).map(|(u, b)| gold(*u, b)).sum::<f64>())
                .collect();
            theirs.sort_by(f64::total_cmp);
            let dist = gold_distribution(&sb, 0);
            for &(x, chance) in &dist.cdf[1..dist.cdf.len() - 1] {
                let sim = theirs.partition_point(|&s| s <= x) as f64 / n;
                assert!((chance - sim).abs() < 0.01, "{x} {chance} {sim}");
            }
            assert!(ours[0] >= dist.luckiest.0 - 1e-6 && ours[N - 1] <= dist.pity.0 + 1e-6);
            assert!(dist.luckiest.1.is_finite() && dist.pity.1.is_finite());
        }
    }
}
