use std::f64::{INFINITY, NAN, NEG_INFINITY};

use crate::{constants::FLOAT_TOL, performance::Performance, state_bundle::StateBundle};
pub static THETA_TOL: f64 = 1e-10;
pub static MAX_ROOT_FIND_ITER: usize = 20;
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
        performance: &mut Performance,
    ) -> Option<(f64, f64, usize)> {
        let mut lower = NEG_INFINITY;
        let mut upper = INFINITY;

        let mut theta = guess;

        // theta = theta.min(max_theta).max(min_theta);

        let mut init_y: f64 = NAN; // this is K'(0) = the mean 
        let mut debug_record: Vec<(f64, f64, f64, f64, f64)> = Vec::new();

        for iter in 0..MAX_ROOT_FIND_ITER {
            // 2. Evaluate Function and Derivatives
            let (_, mut y, dy, dy2, dy3) = self.ks(
                theta,
                compute_biased,
                simple_mean_log,
                support_index,
                skip_count,
                performance,
            );
            if iter == 0 {
                init_y = y;
            }

            y -= budget;
            debug_record.push((theta, y, dy, dy2, dy3));
            //
            if y < 0.0 {
                lower = theta;
            } else {
                upper = theta;
            }

            if (upper - lower) < THETA_TOL || y.abs() < FLOAT_TOL {
                return Some((theta, init_y, iter));
            }

            let delta = if compute_biased {
                (-2.0 * y * dy) / (-y * dy2 + 2.0 * dy.powi(2))
            } else {
                let dy_sq = dy.powi(2);
                let y_sq = y.powi(2);
                -(6.0 * y * dy_sq - 3.0 * y_sq * dy2)
                    / (6.0 * dy_sq * dy - 6.0 * y * dy * dy2 + y_sq * dy3)
            };

            let proposed_theta = theta + delta;

            // last_theta = theta;
            performance.newton_iterations += 1;
            if proposed_theta > lower && proposed_theta < upper && dy.abs() > 0.1 {
                theta = proposed_theta;
                performance.householder_count += 1;
            } else {
                if upper.is_finite() && lower.is_finite() {
                    theta = 0.5 * (upper + lower);
                } else if upper.is_finite() {
                    if (theta - upper).abs() < FLOAT_TOL {
                        // implicitly y > 0.0
                        if theta > 0.0 {
                            theta = 0.5 * (theta);
                        } else {
                            theta = 2.0 * (theta);
                        }
                    } else {
                        // shouldnt really happen? idk
                        theta = 0.5 * (theta + upper);
                    }
                } else if lower.is_finite() {
                    if (theta - lower).abs() < FLOAT_TOL {
                        // implicitly y < 0.0
                        if theta > 0.0 {
                            theta = 2.0 * (theta);
                        } else {
                            theta = 0.5 * (theta);
                        }
                    } else {
                        // shouldnt really happen? idk
                        theta = 0.5 * (theta + lower);
                    }
                } else {
                    //  not possible unless theta is nan or ks output was nan
                    panic!(
                        "theta {:?} lower {:?} upper {:?} y {:?} guess {:?} compute_biased {:?} budget {:?} iter {:?}",
                        theta, lower, upper, y, guess, compute_biased, budget, iter
                    );
                }

                performance.bisection_count += 1;
            }
        }

        // dbg!(debug_record);

        web_sys::console::log_1(&format!("{:?}", &debug_record).into());
        web_sys::console::log_1(
            &format!(
                "theta {:?} lower {:?} upper {:?} guess {:?} compute_biased {:?} budget {:?}",
                theta, lower, upper, guess, compute_biased, budget,
            )
            .into(),
        );
        None
    }
}
