use crate::constants::{FLOAT_TOL, SPECIAL_TOL};
use crate::performance::Performance;
use crate::state_bundle::StateBundle;
use itertools::izip;
use std::f64::NAN;

pub const DEBUG_AVERAGE: bool = false;
pub const DEBUG_AVG_INDEX: i64 = 7;

impl StateBundle {
    /// notably this is the average of the unbiased distribution(it's not the mean of the biased one)
    pub fn simple_avg(&self, support_index: i64, skip_count: usize) -> f64 {
        let mut mean: f64 = 0.0;
        for pair_arr in self.extract_collapsed_pair(support_index, skip_count) {
            let mut this_mean: f64 = 0.0;
            for (s, p) in pair_arr.iter() {
                this_mean += s * p;
            }
            mean += this_mean;
        }
        mean
    }

    /// Computes E[ f_1(X_1) + f_2(X_2) + ... ]
    /// where f_i = gold incurred due to a mat type (described in one_dimension_average_gold)
    /// and X_i is the amount of a particular mat type used
    ///
    /// Note that X_1, X_2 are not independent (in fact for non-juice they are multiples of each other), but that doesn't matter
    ///
    /// We exploit the distributivity of expectation to reduce our multi-dimensional problem into many 1-D ones
    ///
    /// Well actually it calculates this for each outcome for special and takes the weighted average, see special.rs for more info about that  
    pub fn average_gold_metric(&mut self, performance: &mut Performance) -> f64 {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs();
        performance.states_evaluated += 1;

        let mut total_gold: f64 = 0.0;
        for (skip_count, &special_prob) in
            self.special_cache[&self.special_state].iter().enumerate()
        {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            for (support_index, (effective_budget, price, leftover)) in izip!(
                self.flattened_effective_budgets(),
                self.flattened_price(),
                self.flattened_leftover()
            )
            .enumerate()
            {
                let this_avg: f64 = self.one_dimension_average_gold(
                    support_index as i64,
                    skip_count,
                    effective_budget,
                    price,
                    leftover,
                    performance,
                );

                total_gold += special_prob * this_avg;
            }
        }
        total_gold
    }

    /// I initially had two versions of average_gold_metric but um ig this makes more sense, cbb to merge them now
    pub fn average_gold_metric_with_breakdown(&mut self, performance: &mut Performance) -> f64 {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs();
        performance.states_evaluated += 1;
        let mut breakdown: Vec<f64> = vec![0.0; 7 + self.prep_output.juice_info.num_avail * 2];
        let mut total_gold: f64 = 0.0;
        for (skip_count, &special_prob) in
            self.special_cache[&self.special_state].iter().enumerate()
        {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            for (support_index, (effective_budget, price, leftover)) in izip!(
                self.flattened_effective_budgets(),
                self.flattened_price(),
                self.flattened_leftover()
            )
            .enumerate()
            {
                let this_avg: f64 = self.one_dimension_average_gold(
                    support_index as i64,
                    skip_count,
                    effective_budget,
                    price,
                    leftover,
                    performance,
                );
                let this = special_prob * this_avg;
                breakdown[support_index] += this;
                total_gold += this;
            }
        }
        for x in breakdown.iter_mut() {
            *x = x.ceil()
        }
        self.average_breakdown = Some(breakdown);
        total_gold
    }

    /// For the sake of notations we focus on mat type i (every varaible here should have a subscript i but I cbb)
    /// let b = budget of mat type i,
    /// then f = { (X - b) * price if (X - b) > 0, otherwise (X - b) * leftover value (leftover default 0)
    ///
    /// So E[f(X)] = price * E[(X - b)  * I(X > b) ] + leftover value * E[(X - b) * I(X < b)]
    /// where I is the indicator function,
    ///
    /// and we define the biased distribution X' which has the probability distribution p' = p * s / mean at value s,
    /// such that P(X' < b) = SUM (p * s / mean where s < b) = 1/mean * E[X * I(X<b)]
    ///
    /// which, after some algebra to avoid calling SA 4 times is the form you see below
    pub fn one_dimension_average_gold(
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        price: f64,
        leftover_value: f64,
        performance: &mut Performance,
    ) -> f64 {
        let simple_mean: f64 = self.simple_avg(support_index, skip_count);

        if (price - leftover_value).abs() < FLOAT_TOL {
            // this also includes price = 0 (unless leftover is high for some reason)
            return price * (budget - simple_mean);
        }

        let biased_prob: f64 = self.saddlepoint_approximation_wrapper(
            support_index,
            skip_count,
            budget,
            true,
            simple_mean.ln(),
            performance,
        );
        let prob = self.saddlepoint_approximation_wrapper(
            support_index,
            skip_count,
            budget,
            false,
            NAN,
            performance,
        );

        let out: f64 = price * (budget - simple_mean)
            + (leftover_value - price) * (budget * prob - biased_prob * simple_mean);

        if (!out.is_finite() || DEBUG_AVERAGE) && support_index == DEBUG_AVG_INDEX {
            let left = budget - simple_mean;
            let right = budget * prob - biased_prob * (simple_mean);
            let truncated_mean = biased_prob * (simple_mean);
            dbg!(
                simple_mean,
                self.find_min_max(support_index, skip_count),
                budget,
                biased_prob,
                prob,
                truncated_mean,
                left,
                right,
                price,
                leftover_value,
                out,
            );
        }
        out
    }
}
