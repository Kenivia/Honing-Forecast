use crate::constants::{FLOAT_TOL, SPECIAL_TOL};
use ahash::AHashMap;
use smallvec::SmallVec;
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;

pub const DEBUG_AVERAGE: bool = false;
pub const DEBUG_AVG_INDEX: i64 = 7;

fn mix(a: u64, b: u64) -> u64 {
    let mut x = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b;
    x ^= x >> 32;
    x = x.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    x ^ (x >> 32)
}

impl StateBundle {
    /// notably this is the average of the unbiased distribution(it's not the mean of the biased one)
    pub fn simple_avg(&self, support_index: i64, skip_count: usize) -> f64 {
        let mut mean: f64 = 0.0;
        for (index, &u_index) in self.special_state.iter().enumerate() {
            let support = self.support_from_index(u_index, support_index);
            mean += if index < skip_count { support.access_min(true) } else { support.mean };
        }
        mean
    }

    /// Special case of ui_average_gold_metric, just separating them to keep things clean
    ///
    /// See Saddlepoint Approximation.pdf for more info on the math.
    pub fn optimizer_average_gold_metric(&mut self, probe: &mut Probe) -> f64 {
        self.update_prob_dist();
        self.update_cost_dist();
        self.compute_special_probs(false);
        probe.count(Counter::StatesEvaluated);

        let skips = self.live_skips();
        let mut memo = std::mem::take(&mut self.gold_memo);
        let sums = if self.tuning.max_memo_size > 0 && !skips.is_empty() { self.memo_sums(&skips) } else { Vec::new() };
        let tuning_key = self.tuning.evaluator_key();
        let mut total_gold: f64 = 0.0;
        let plan_bands = &self.prep_output.bands[self.prep_output.optimizer_plan];
        for (support_index, thresh_price_pairs) in plan_bands.iter().enumerate() {
            total_gold += self.material_average_gold(
                &mut memo,
                support_index as i64,
                &skips,
                thresh_price_pairs,
                sums.get(support_index).map(|sums| &sums[..]),
                tuning_key,
                probe,
            );
        }
        self.gold_memo = memo;

        total_gold
    }

    /// See Saddlepoint Approximation.pdf for more info on the math.
    /// One metric and one per material breakdown for every plan in the payload, plus the
    /// average amount of each material used (which does not depend on the plan).
    pub fn ui_average_gold_metric(
        &mut self,
        probe: &mut Probe,
    ) -> (Vec<f64>, Vec<f64>, Vec<Vec<f64>>) {
        self.update_prob_dist();
        self.update_cost_dist();
        self.compute_special_probs(false);
        probe.count(Counter::StatesEvaluated);

        let num_mats = self.prep_output.table.len();
        let num_plans = self.prep_output.plans.len();
        let mut gold_breakdown: Vec<Vec<f64>> = vec![vec![0.0; num_mats]; num_plans];
        let mut avg_used: Vec<f64> = vec![0.0; num_mats];
        let mut metric_per_plan: Vec<f64> = vec![0.0; num_plans];
        let skips = self.live_skips();
        let mut memo = std::mem::take(&mut self.gold_memo);
        let sums = if self.tuning.max_memo_size > 0 && !skips.is_empty() { self.memo_sums(&skips) } else { Vec::new() };
        let tuning_key = self.tuning.evaluator_key();
        for plan_index in 0..num_plans {
            for support_index in 0..num_mats {
                let bands = &self.prep_output.bands[plan_index][support_index];
                let this = self.material_average_gold(
                    &mut memo,
                    support_index as i64,
                    &skips,
                    bands,
                    sums.get(support_index).map(|sums| &sums[..]),
                    tuning_key,
                    probe,
                );
                gold_breakdown[plan_index][support_index] += this;
                metric_per_plan[plan_index] += this;
                if plan_index == 0 {
                    for &(skip_count, special_prob) in &skips {
                        avg_used[support_index] += special_prob * self.simple_avg(support_index as i64, skip_count)
                    }
                }
            }
        }
        self.gold_memo = memo;

        (metric_per_plan, avg_used, gold_breakdown)
    }

    /// the free tap outcomes worth evaluating, with their probabilities
    fn live_skips(&self) -> Vec<(usize, f64)> {
        self.special_probs()
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, special_prob)| special_prob >= SPECIAL_TOL)
            .collect()
    }

    /// One material's average gold over the free tap outcomes.
    ///
    /// A move changes one upgrade, so most materials are as they were some evaluations ago:
    /// each outcome's result is kept under a key made of what it depends on. Per upgrade
    /// that is its state where it is honed, and only what a free tap leaves where it is
    /// skipped. The order of the rest does not matter, so the key is a sum.
    fn material_average_gold(
        &self,
        memo: &mut AHashMap<u64, f64>,
        support_index: i64,
        skips: &[(usize, f64)],
        thresh_price_pairs: &[(f64, f64)],
        sums: Option<&[u64]>,
        tuning_key: u64,
        probe: &mut Probe,
    ) -> f64 {
        // what the key says about everything but the upgrades
        let mut base = mix(support_index as u64, tuning_key);
        for &(thresh, price) in thresh_price_pairs {
            base = mix(mix(base, thresh.to_bits()), price.to_bits());
        }
        let mut total: f64 = 0.0;
        let mut missing: SmallVec<[usize; 8]> = SmallVec::new();
        for (slot, &(skip_count, weight)) in skips.iter().enumerate() {
            match sums.and_then(|sums| memo.get(&mix(base, sums[slot]))) {
                Some(&known) => total += weight * known,
                None => missing.push(skip_count),
            }
        }
        if missing.is_empty() {
            return total;
        }
        let needs_probability = thresh_price_pairs.windows(2).any(|pair| (pair[0].1 - pair[1].1).abs() >= FLOAT_TOL);
        let table = if needs_probability {
            let thresholds: SmallVec<[f64; 4]> = thresh_price_pairs.iter().map(|pair| pair.0).collect();
            self.lattice_table(support_index, &missing, &thresholds, probe)
        } else {
            vec![None; missing.len()]
        };
        if memo.len() + missing.len() > self.tuning.max_memo_size {
            memo.clear();
        }
        let mut lattices = table.iter();
        for (slot, &(skip_count, weight)) in skips.iter().enumerate() {
            if !missing.contains(&skip_count) {
                continue;
            }
            let lattice = lattices.next().unwrap().as_deref();
            let this_avg =
                self.one_dimension_average_gold(support_index, skip_count, thresh_price_pairs, lattice, probe);
            total += weight * this_avg;
            if let Some(sums) = sums {
                memo.insert(mix(base, sums[slot]), this_avg);
            }
        }
        total
    }

    /// What each material's key says about the upgrades, for every free tap outcome:
    /// `[material][outcome]`. One pass over the upgrades, which are stored by upgrade.
    fn memo_sums(&self, skips: &[(usize, f64)]) -> Vec<Vec<u64>> {
        let num_mats = self.upgrade_arr[0].cost_dist.len();
        let n = self.special_state.len();
        let mut out = vec![vec![0u64; skips.len()]; num_mats];
        let (first, last) = (skips[0].0, skips[skips.len() - 1].0);
        let mut run = vec![0u64; num_mats];
        // what a free tap leaves of the upgrades before each outcome
        let mut slot = 0;
        for index in 0..=last.min(n) {
            while slot < skips.len() && skips[slot].0 == index {
                for (sums, skipped) in out.iter_mut().zip(&run) {
                    sums[slot] = mix(*skipped, 1);
                }
                slot += 1;
            }
            if index == n {
                break;
            }
            let u_index = self.special_state[index];
            for (sum, support) in run.iter_mut().zip(&self.upgrade_arr[u_index].cost_dist) {
                *sum = sum.wrapping_add(mix(u_index as u64, support.access_min(true).to_bits()));
            }
        }
        // and the states of the ones from it on
        run.fill(0);
        let mut slot = skips.len();
        for index in (first..=n).rev() {
            if index < n {
                let u_index = self.special_state[index];
                let upgrade = &self.upgrade_arr[u_index];
                for (sum, support) in run.iter_mut().zip(&upgrade.cost_dist) {
                    let made_of = if support.ignore { 0f64.to_bits() } else { upgrade.state.hash };
                    *sum = sum.wrapping_add(mix(u_index as u64, made_of));
                }
            }
            while slot > 0 && skips[slot - 1].0 >= index {
                slot -= 1;
                for (sums, sum) in out.iter_mut().zip(&run) {
                    sums[slot] = sums[slot].wrapping_add(*sum);
                }
            }
        }
        out
    }

    /// See Saddlepoint Approximation.pdf for more info on the math. This is a generalized version for n price breakpoints
    pub fn one_dimension_average_gold(
        &self,
        support_index: i64,
        skip_count: usize,
        thresh_price_pairs: &[(f64, f64)],
        lattice: Option<&[(f64, f64)]>,
        probe: &mut Probe,
    ) -> f64 {
        let num_thresholds: usize = thresh_price_pairs.len();

        if num_thresholds == 1 {
            let simple_mean: f64 = self.simple_avg(support_index, skip_count);
            return thresh_price_pairs[0].1 * (thresh_price_pairs[0].0 - simple_mean);
        }
        // exact and shared by every threshold, where it is cheap enough
        if let Some(lattice) = lattice {
            let simple_mean: f64 = self.simple_avg(support_index, skip_count);
            let (last_thresh, last_price) = thresh_price_pairs[num_thresholds - 1];
            let mut out: f64 = last_price * (last_thresh - simple_mean);
            for (index, &(thresh, price)) in thresh_price_pairs.iter().enumerate().skip(1) {
                let prev_price = thresh_price_pairs[index - 1].1;
                if (prev_price - price).abs() < FLOAT_TOL {
                    continue;
                }
                probe.count(Counter::Lattice);
                out += (prev_price - price) * (thresh * lattice[index].0 - lattice[index].1 * simple_mean);
            }
            return out;
        }
        let shape = self.shape(support_index, skip_count);
        let simple_mean: f64 = shape.mean;

        let last_thresh: f64 = thresh_price_pairs[num_thresholds - 1].0;
        let last_price: f64 = thresh_price_pairs[num_thresholds - 1].1;

        let mut out: f64 = last_price * (last_thresh - simple_mean);

        for (index, &(thresh, price)) in thresh_price_pairs.iter().enumerate().skip(1) {
            let prev_price = thresh_price_pairs[index - 1].1;
            // Both probabilities below are multiplied by this gap, so a threshold the price
            // does not change at costs nothing. The top entry is the one that needs this:
            // it has to stay for the term above, because it carries a constant rather than
            // a kink, but it is free whenever the band under it is credited at the buy price.
            if (prev_price - price).abs() < FLOAT_TOL {
                continue;
            }

            out += (prev_price - price) * self.leftover(support_index, skip_count, &shape, thresh, probe);
        }
        return out;
    }

    /// E[(t - S)+] of one material under one free tap outcome, by whatever the metric would use
    pub fn expected_leftover(&self, support_index: i64, skip_count: usize, thresh: f64, probe: &mut Probe) -> f64 {
        let shape = self.shape(support_index, skip_count);
        if let Some(table) = self.lattice_table(support_index, &[skip_count], &[thresh], probe).pop().flatten() {
            probe.count(Counter::Lattice);
            return thresh * table[0].0 - table[0].1 * shape.mean;
        }
        self.leftover(support_index, skip_count, &shape, thresh, probe)
    }
}
