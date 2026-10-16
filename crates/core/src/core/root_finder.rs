//! Finds theta such that K'(theta) = budget, or close enough
//! This is necessary as part of saddlepoint approximation

use super::cumulants::KsTuple;
use crate::probe::{Counter, Probe};
use crate::state_bundle::StateBundle;
use crate::{constants::FLOAT_TOL, my_dbg};
use std::f64::{INFINITY, NEG_INFINITY};
// there's definitely some fishy business going on with advanced honing

impl StateBundle {
    /// Generalized newton's method, because we have access to higher derivatives for free.
    ///
    /// We use the heuristic where if we're ever too close to min_value or max_value, we divide the theta by 10
    /// This is because the curve is extremely flat around these edges, and normal householder won't converge
    /// and we know the curve is centered at 0 (mean achieved at 0), and this is the heuristic that appears to work
    pub fn householder(
        &self,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        guess: f64,
        min_value: f64,
        max_value: f64,
        min_delta: f64,
        mut seed: Option<KsTuple>,
        probe: &mut Probe,
    ) -> Option<(f64, KsTuple)> {
        let mut lower = NEG_INFINITY;
        let mut upper = INFINITY;
        let mut y_lower = NEG_INFINITY;
        let mut y_upper = INFINITY;
        let mut theta = guess;

        for iter in 0..self.tuning.max_root_find_iter {
            // the cumulants at the guess may be known already
            let evaluated = seed.is_none();
            let this = seed.take().unwrap_or_else(|| self.raw_ks(theta, support_index, skip_count, probe));
            let (_, mut y, dy, dy2, dy3) = this.access();

            y -= budget;

            if y < 0.0 {
                lower = theta;
                y_lower = y_lower.max(y);
            } else {
                upper = theta;
                y_upper = y_upper.min(y);
            }

            if y.abs() < self.tuning.y_value_tol {
                return Some((theta, this));
            }

            let near_edge = (min_value - budget - y).abs() < min_delta
                || (max_value - budget - y).abs() < min_delta;
            let delta = if near_edge {
                theta * 0.1 - theta
            } else {
                let dy_sq = dy.powi(2);
                let y_sq = y.powi(2);
                -(6.0 * y * dy_sq - 3.0 * y_sq * dy2)
                    / (6.0 * dy_sq * dy - 6.0 * y * dy * dy2 + y_sq * dy3)
            };

            let mut proposed_theta = theta + delta;
            proposed_theta =
                proposed_theta.clamp(-3.0 * theta.abs().max(1e-8), 3.0 * theta.abs().max(1e-8));

            probe.count(Counter::RootIter);

            if lower < proposed_theta && proposed_theta < upper && y_lower <= y && y <= y_upper {
                // A last step this short lands where the taylor series about here is as good
                // as another evaluation. Newton on the series, which costs nothing, for as
                // long as it stays that short: a short step is not a small residual, and the
                // series is worthless further out. Not from cumulants that are themselves off
                // a series.
                let sd = dy.sqrt();
                if evaluated && !near_edge {
                    let mut step = proposed_theta - theta;
                    for _ in 0..4 {
                        if !(step.abs() * sd < self.tuning.taylor_skip) {
                            break;
                        }
                        let there = this.moved(step);
                        if (there.1 - budget).abs() < self.tuning.y_value_tol {
                            probe.count(Counter::TaylorSkip);
                            return Some((theta + step, there));
                        }
                        step -= (there.1 - budget) / there.2;
                    }
                }
                theta = proposed_theta;
                probe.count(Counter::Householder);
            } else {
                // fall back to bisection, but shouldn't really happen anymore
                probe.count(Counter::Bisection);
                if upper.is_finite() && lower.is_finite() {
                    theta = 0.5 * (upper + lower)
                } else if upper.is_finite() {
                    if (theta - upper).abs() < FLOAT_TOL {
                        if theta > 0.0 {
                            theta *= 0.5;
                        } else {
                            theta *= 2.0;
                        }
                    } else {
                        theta = 2.0 * (theta + upper);
                    }
                } else if lower.is_finite() {
                    if (theta - lower).abs() < FLOAT_TOL {
                        if theta > 0.0 {
                            theta *= 2.0;
                        } else {
                            theta *= 0.5;
                        }
                    } else {
                        theta = 2.0 * (theta + lower);
                    }
                } else {
                    panic!(
                        "theta {:?} lower {:?} upper {:?} y {:?} guess {:?} budget {:?} iter {:?}",
                        theta, lower, upper, y, guess, budget, iter
                    );
                }
            }
            // last_y = y;
        }

        my_dbg!(theta, lower, upper, guess, budget,);
        None
    }
}
