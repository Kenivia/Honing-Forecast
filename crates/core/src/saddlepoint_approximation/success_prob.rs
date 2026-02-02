use std::f64::NAN;

use crate::constants::{BUCKET_COUNT, SPECIAL_TOL};

use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use itertools::Itertools;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct HistogramOutputs {
    cum_percentiles: Vec<Vec<(f64, f64)>>,
    average: Vec<f64>,
    budgets: Vec<f64>, // just for convenience
}
impl StateBundle {
    pub fn success_prob_metric(&mut self, performance: &mut Performance) -> f64 {
        performance.states_evaluated += 1;

        self.update_dist();
        self.update_combined();
        self.compute_special_probs();
        let budget = self.prep_output.eqv_gold_budget;
        let out: f64 = self.honing_sa_wrapper(-1, budget, performance);

        out
    }

    pub fn success_prob_for_analysis(&self, budget: f64) -> f64 {
        let mut dummy_performance = Performance::new();
        let out: f64 = self.honing_sa_wrapper(-1, budget, &mut dummy_performance);

        out
    }
    pub fn honing_sa_wrapper(
        &self,
        support_index: i64,
        budget: f64,
        performance: &mut Performance,
    ) -> f64 {
        let mut out: f64 = 0.0;

        let special_probs = self.special_cache[&self.special_state].clone();

        for (skip_count, &special_prob) in special_probs.iter().enumerate() {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            // dbg!(&support_arr[index..]);
            let this_prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                budget,
                false,
                NAN,
                performance,
            );

            out += special_prob * this_prob;
        }

        out
    }
    pub fn compute_leftover_probs(&mut self) -> Vec<f64> {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs();
        let mut prob_leftover: Vec<f64> =
            Vec::with_capacity(self.flattened_effective_budgets().try_len().unwrap());

        let items: Vec<_> = self.flattened_effective_budgets().enumerate().collect();
        let mut dummy_performance = Performance::new();
        for (support_index, effective_budget) in items {
            prob_leftover.push(self.honing_sa_wrapper(
                support_index as i64,
                effective_budget,
                &mut dummy_performance,
            ));
        }

        prob_leftover
    }
    pub fn histogram(&mut self) -> HistogramOutputs {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs();
        let special_probs = self.special_cache[&self.special_state].clone();
        let mut dummy_performance = Performance::new();
        let mut cum_percentiles: Vec<Vec<(f64, f64)>> =
            vec![
                vec![(0.0, 0.0); BUCKET_COUNT + 1];
                self.flattened_effective_budgets().try_len().unwrap()
            ];
        let num_sup = self.flattened_effective_budgets().count();
        let mut average: Vec<f64> = Vec::with_capacity(num_sup);

        let already_spent: (Vec<i64>, Vec<i64>, Vec<i64>, f64) =
            self.prep_output.already_spent.clone().unwrap();
        let flattened_spent: Vec<f64> = already_spent
            .0
            .iter()
            .chain(already_spent.1.iter())
            .chain(already_spent.2.iter())
            .map(|x| *x as f64)
            .collect();
        for support_index in 0..num_sup {
            for (index, this_prob) in cum_percentiles[support_index].iter_mut().enumerate() {
                let this_budget =
                    index as f64 * self.pity()[support_index] as f64 / BUCKET_COUNT as f64;
                *this_prob = (
                    this_budget,
                    self.honing_sa_wrapper(
                        support_index as i64,
                        this_budget,
                        &mut dummy_performance,
                    ),
                );
            }
        }
        for support_index in 0..num_sup {
            let mut out: f64 = 0.0;
            for (skip_count, &special_prob) in special_probs.iter().enumerate() {
                if special_prob < SPECIAL_TOL {
                    continue;
                }
                out += special_prob * self.simple_avg(support_index as i64, skip_count);
            }
            average.push(out - flattened_spent[support_index])
        }

        HistogramOutputs {
            cum_percentiles,
            average,
            budgets: self
                .prep_output
                .budgets
                .iter()
                .map(|x| *x as f64)
                .chain(
                    self.prep_output
                        .juice_books_owned
                        .iter()
                        .map(|x| x.0 as f64),
                )
                .chain(
                    self.prep_output
                        .juice_books_owned
                        .iter()
                        .map(|x| x.1 as f64),
                )
                .collect(),
        }
    }
    pub fn compute_leftover_probs_for_analysis(&self) -> Vec<f64> {
        // assume initialized properly
        let mut prob_leftover: Vec<f64> =
            Vec::with_capacity(self.flattened_effective_budgets().try_len().unwrap());

        let items: Vec<_> = self.flattened_effective_budgets().enumerate().collect();
        let mut dummy_performance = Performance::new();
        for (support_index, effective_budget) in items {
            prob_leftover.push(self.honing_sa_wrapper(
                support_index as i64,
                effective_budget,
                &mut dummy_performance,
            ));
        }

        prob_leftover
    }
}
