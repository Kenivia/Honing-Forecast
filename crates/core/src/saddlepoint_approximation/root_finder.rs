use std::f64::{INFINITY, NAN, NEG_INFINITY};

use crate::{
    constants::FLOAT_TOL, performance::Performance, saddlepoint_approximation::ks::KsTuple,
    state_bundle::StateBundle,
};
pub static THETA_TOL: f64 = 1e-10;
pub static MAX_ROOT_FIND_ITER: usize = 20;
static Y_VALUE_TOL: f64 = 1e-7;
// pub static THETA_LIMIT: f64 = 1e2;

impl StateBundle {
    pub fn householder(
        &self,
        compute_biased: bool,
        simple_mean_log: f64,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        // low: f64,
        guess: f64,
        // high: f64,
        min_value: f64,
        max_value: f64,
        min_delta: f64,
        performance: &mut Performance,
    ) -> Option<(f64, f64, usize, KsTuple)> {
        let mut lower = NEG_INFINITY;
        let mut upper = INFINITY;

        let mut y_lower = NEG_INFINITY;
        let mut y_upper = INFINITY;

        let mut theta = guess;

        let mut init_y: f64 = NAN;
        let mut debug_record: Vec<(f64, f64, f64, f64, f64, f64)> = Vec::new();

        let mut last_y: f64 = NAN;
        for iter in 0..MAX_ROOT_FIND_ITER {
            let this = self.ks(
                theta,
                compute_biased,
                simple_mean_log,
                support_index,
                skip_count,
                performance,
            );
            let (_, mut y, dy, dy2, dy3) = this.access();
            if iter == 0 {
                init_y = y;
            }

            y -= budget;

            if y < 0.0 {
                lower = theta;
                y_lower = y_lower.max(y);
            } else {
                upper = theta;
                y_upper = y_upper.min(y);
            }

            if y.abs() < Y_VALUE_TOL {
                return Some((theta, init_y, iter, this));
            }

            let delta = if (min_value - budget - y).abs() < min_delta
                || (max_value - budget - y).abs() < min_delta
            {
                theta * 0.1 - theta
            } else if compute_biased {
                (-2.0 * y * dy) / (-y * dy2 + 2.0 * dy.powi(2))
            } else {
                let dy_sq = dy.powi(2);
                let y_sq = y.powi(2);
                -(6.0 * y * dy_sq - 3.0 * y_sq * dy2)
                    / (6.0 * dy_sq * dy - 6.0 * y * dy * dy2 + y_sq * dy3)
            };

            let mut proposed_theta = theta + delta;
            debug_record.push((theta, proposed_theta, y, dy, dy2, dy3));
            proposed_theta =
                proposed_theta.clamp(-3.0 * theta.abs().max(1e-8), 3.0 * theta.abs().max(1e-8));

            performance.newton_iterations += 1;

            if lower < proposed_theta && proposed_theta < upper && y_lower <= y && y <= y_upper {
                theta = proposed_theta;
                performance.householder_count += 1;
            } else {
                performance.bisection_count += 1;
                if upper.is_finite() && lower.is_finite() {
                    theta = (lower * y_upper - upper * y_lower) / (y_upper - y_lower);
                } else if upper.is_finite() {
                    if (theta - upper).abs() < FLOAT_TOL {
                        if theta > 0.0 {
                            theta = 0.5 * (theta);
                        } else {
                            theta = 2.0 * (theta);
                        }
                    } else {
                        theta = 2.0 * (theta + upper);
                    }
                } else if lower.is_finite() {
                    if (theta - lower).abs() < FLOAT_TOL {
                        if theta > 0.0 {
                            theta = 2.0 * (theta);
                        } else {
                            theta = 0.5 * (theta);
                        }
                    } else {
                        theta = 2.0 * (theta + lower);
                    }
                } else {
                    panic!(
                        "theta {:?} lower {:?} upper {:?} y {:?} guess {:?} compute_biased {:?} budget {:?} iter {:?}",
                        theta, lower, upper, y, guess, compute_biased, budget, iter
                    );
                }
            }
            last_y = y;
        }

        dbg!(theta, lower, upper, guess, compute_biased, budget,);
        dbg!(debug_record);
        None
    }
}
