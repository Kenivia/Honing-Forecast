//! A start from shadow prices: each material priced at its marginal value at the state as it
//! is, every normal upgrade takes its best plan under those prices alone, again until the plans
//! settle. The prices are the running average of each round's. The best state seen is kept.

use super::response::Response;
use crate::constants::{FLOAT_TOL, SPECIAL_TOL};
use crate::probe::Probe;
use crate::state::OneState;
use crate::state_bundle::StateBundle;
use smallvec::SmallVec;

impl StateBundle {
    /// What one more unit of each material costs, by row, over the free tap outcomes. Reads
    /// the supports and free tap probabilities of the last evaluation.
    pub fn shadow_prices(&self, probe: &mut Probe) -> Vec<f64> {
        let skips: Vec<(usize, f64)> = self
            .special_probs()
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, weight)| weight >= SPECIAL_TOL)
            .collect();
        let counts: Vec<usize> = skips.iter().map(|skip| skip.0).collect();
        let prep = &self.prep_output;
        prep.bands[prep.optimizer_plan]
            .iter()
            .enumerate()
            .map(|(row, bands)| {
                let last_price = bands[bands.len() - 1].1;
                if !bands
                    .windows(2)
                    .any(|pair| (pair[0].1 - pair[1].1).abs() >= FLOAT_TOL)
                {
                    return last_price;
                }
                let used: f64 = skips
                    .iter()
                    .map(|&(skip, weight)| weight * self.simple_avg(row as i64, skip))
                    .sum();
                let delta = (0.01 * used).max(1.0);
                let shifted = |by: f64| -> SmallVec<[(f64, f64); 4]> {
                    bands
                        .iter()
                        .map(|&(thresh, price)| (thresh + by, price))
                        .collect()
                };
                // one less used, then one more
                let thresholds: Vec<f64> = [delta, -delta]
                    .iter()
                    .flat_map(|&by| bands.iter().map(move |pair| pair.0 + by))
                    .collect();
                let table = self.lattice_table(row as i64, &counts, &thresholds, probe);
                let len = bands.len();
                let mut sum = 0.0;
                for (&(skip, weight), lattice) in skips.iter().zip(&table) {
                    let at = |side: usize, by: f64, probe: &mut Probe| {
                        self.one_dimension_average_gold(
                            row as i64,
                            skip,
                            &shifted(by),
                            lattice
                                .as_ref()
                                .map(|all| &all[side * len..(side + 1) * len]),
                            probe,
                        )
                    };
                    sum += weight * (at(0, delta, probe) - at(1, -delta, probe));
                }
                sum / (2.0 * delta)
            })
            .collect()
    }
}

/// Up to `iterations` rounds of: price each material at its averaged marginal value, give every
/// normal upgrade its best plan under those prices, evaluate. Leaves the best state seen, its
/// metric set, and returns what it took in evaluations.
pub fn shadow_start(state_bundle: &mut StateBundle, iterations: usize, probe: &mut Probe) -> f64 {
    let normal: Vec<usize> = (0..state_bundle.upgrade_arr.len())
        .filter(|&index| state_bundle.upgrade_arr[index].is_normal_honing)
        .collect();
    let plans = |state_bundle: &StateBundle| -> Vec<Vec<OneState>> {
        normal
            .iter()
            .map(|&index| state_bundle.upgrade_arr[index].state.payload.clone())
            .collect()
    };
    let steps = state_bundle.tuning.response_steps;
    let joint = state_bundle.tuning.response_joint;
    let clock = probe.clock();
    state_bundle.metric = state_bundle.metric_router(probe);
    probe.evaluated(clock);
    let mut work = 1.0;
    let mut best = (state_bundle.metric, plans(state_bundle));
    let mut prices: Option<Vec<f64>> = None;
    for round in 0..iterations {
        // the fresh prices averaged over the rounds
        let rate = 1.0 / (round as f64 + 1.0);
        // the prices take about an evaluation
        let fresh = state_bundle.shadow_prices(probe);
        work += 1.0;
        let now: Vec<f64> = match prices {
            None => fresh,
            Some(old) => old
                .iter()
                .zip(&fresh)
                .map(|(old, new)| (1.0 - rate) * old + rate * new)
                .collect(),
        };
        let mut moved = false;
        for &index in &normal {
            let response = Response::linear(state_bundle, index, &now);
            let model = &response.model;
            let was =
                model.canonical(&model.indices(&state_bundle.upgrade_arr[index].state.payload));
            let plan = response.better(&was, joint).1;
            work += response.work(steps);
            if plan != was {
                state_bundle.upgrade_arr[index]
                    .state
                    .update_payload(model.plan(&plan));
                moved = true;
            }
        }
        prices = Some(now);
        if !moved {
            break;
        }
        let clock = probe.clock();
        state_bundle.metric = state_bundle.metric_router(probe);
        probe.evaluated(clock);
        work += 1.0;
        if state_bundle.metric > best.0 {
            best = (state_bundle.metric, plans(state_bundle));
        }
    }
    for (&index, plan) in normal.iter().zip(best.1) {
        if state_bundle.upgrade_arr[index].state.payload != plan {
            state_bundle.upgrade_arr[index].state.update_payload(plan);
        }
    }
    state_bundle.metric = state_bundle.metric_router(probe);
    work
}
