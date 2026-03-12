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
    pub fn average_gold_metric(
        &mut self,
        compute_breakdown: bool,
        performance: &mut Performance,
    ) -> f64 {
        self.update_dist();
        self.update_individual_support();
        self.compute_special_probs(false);
        performance.states_evaluated += 1;

        #[cfg(feature = "wasm")]
        let mut breakdown: Vec<f64> = vec![0.0; self.prep_output.juice_info.total_num_avail];

        let mut total_gold: f64 = 0.0;
        for (skip_count, &special_prob) in
            self.special_cache[&self.special_state].iter().enumerate()
        {
            if special_prob < SPECIAL_TOL {
                continue;
            }
            for (
                support_index,
                (bound_budget, price, leftover, tradable_budget, tradable_leftover),
            ) in izip!(
                self.flattened_bound_budgets(),
                self.flattened_full_price(),
                self.flattened_leftover(),
                self.flattened_tradable_budgets(),
                self.flattened_tradable_leftover(),
            )
            .enumerate()
            {
                let this_avg: f64 = self.one_dimension_average_gold(
                    support_index as i64,
                    skip_count,
                    bound_budget,
                    tradable_budget,
                    price,
                    tradable_leftover,
                    leftover,
                    performance,
                );
                let this = special_prob * this_avg;
                total_gold += special_prob * this_avg;
                #[cfg(feature = "wasm")]
                if compute_breakdown {
                    breakdown[support_index] += this;
                }
            }
        }
        #[cfg(feature = "wasm")]
        if compute_breakdown {
            for x in breakdown.iter_mut() {
                *x = x.ceil()
            }
            self.average_breakdown = Some(breakdown);
        }
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
        mut budget: f64,
        tradable_threshold: f64,
        price: f64,
        tradable_leftover: f64,
        leftover_value: f64,
        performance: &mut Performance,
    ) -> f64 {
        let simple_mean: f64 = self.simple_avg(support_index, skip_count);

        if (price - leftover_value).abs() < FLOAT_TOL {
            // this also includes price = 0 (unless leftover is high for some reason)
            return price * (budget - simple_mean);
        }
        let trade_same_value: bool = (tradable_leftover - leftover_value).abs() < FLOAT_TOL; // ig this only happens for 1g cost or user specifies
        if trade_same_value {
            budget = tradable_threshold;
        }
        let simple_mean_log = simple_mean.ln();
        if trade_same_value || (budget - tradable_threshold).abs() < FLOAT_TOL {
            let biased_prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                budget,
                true,
                simple_mean_log,
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
            // the signs are backwards compared to the UI / documentation
            let out: f64 = price * (budget - simple_mean)
                + (leftover_value - price) * (budget * prob - biased_prob * simple_mean);

            return out;
        } else {
            let trade_biased_prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                tradable_threshold,
                true,
                simple_mean_log,
                performance,
            );
            let trade_prob = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                tradable_threshold,
                false,
                NAN,
                performance,
            );
            let biased_prob: f64 = self.saddlepoint_approximation_wrapper(
                support_index,
                skip_count,
                budget,
                true,
                simple_mean_log,
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
            // leaving it in this form for now
            let out: f64 = price * (simple_mean - tradable_threshold)
                - price * (trade_biased_prob * simple_mean - trade_prob * tradable_threshold)
                + tradable_leftover * (trade_biased_prob * simple_mean - trade_prob * budget)
                - tradable_leftover * (biased_prob * simple_mean - prob * budget)
                + leftover_value * (biased_prob * simple_mean - prob * budget);

            return -out; // note the negative here (i wrote this part after the white paper)
        }
    }
}
