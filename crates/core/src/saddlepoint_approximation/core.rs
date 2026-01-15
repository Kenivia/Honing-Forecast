use std::f64::{NAN, NEG_INFINITY};

use crate::state_bundle::StateBundle;
pub static THETA_TOL: f64 = 1e-10;
pub static THETA_LIMIT: f64 = 1e2;

impl StateBundle {
    pub fn ks(
        &self,
        theta: f64,
        toggle: &(bool, bool, bool, bool, bool),
        compute_biased: bool,
        mean_log: f64,
        support_index: i64,
        skip_count: usize,
    ) -> (f64, f64, f64, f64, f64) {
        if compute_biased {
            let new_toggle = (
                toggle.0,
                toggle.0 || toggle.1 || toggle.2 || toggle.3,
                toggle.1 || toggle.2 || toggle.3,
                toggle.2 || toggle.3,
                toggle.3,
            );

            let ksx = self.ks_01234(support_index, skip_count, theta, &new_toggle);
            (
                ksx.0 + ksx.1.ln() - mean_log,
                ksx.1 + ksx.2 / ksx.1,
                ksx.2 + (ksx.3 * ksx.1 - ksx.2.powi(2)) / (ksx.1.powi(2)),
                ksx.3
                    + (2.0 * ksx.2.powi(3) + ksx.4 * ksx.1.powi(2) - 3.0 * ksx.3 * ksx.1 * ksx.2)
                        / ksx.1.powi(3),
                NAN,
            )
        } else {
            self.ks_01234(support_index, skip_count, theta, toggle)
        }
    }
    pub fn ks_01234(
        &self,
        support_index: i64,
        skip_count: usize,
        theta: f64,
        toggle: &(bool, bool, bool, bool, bool),
    ) -> (f64, f64, f64, f64, f64) {
        let mut total_k = 0.0;
        let mut total_k1 = 0.0;
        let mut total_k2 = 0.0;
        let mut total_k3 = 0.0;
        let mut total_k4 = 0.0;

        for meta_support in self.extract_support_with_meta(support_index, skip_count) {
            if meta_support.ignore {
                continue;
            }
            let triplet_arr = meta_support.access_collapsed();

            let mut alpha_arr: Vec<f64> = Vec::with_capacity(triplet_arr.len());
            let mut alpha_shift: f64 = f64::NEG_INFINITY;

            for (s, _p, log_p) in triplet_arr.iter() {
                let this_alpha: f64 = log_p + theta * s;

                alpha_arr.push(this_alpha);
                alpha_shift = this_alpha.max(alpha_shift);
                // sanity_check += p;
            }
            // if (1.0 - sanity_check).abs() > FLOAT_TOL {
            //     dbg!(
            //         sanity_check,
            //         &triplet_arr,
            //         // log_prob_dist_arr
            //         //     .into_iter()
            //         //     .map(|x| x.iter().map(|y| y.exp()).sum::<f64>())
            //         //     .collect::<Vec<f64>>(),
            //         // log_prob_dist_arr
            //         //     .into_iter()
            //         //     .map(|x| x.iter().map(|y| y.exp()).collect())
            //         //     .collect::<Vec<Vec<f64>>>(),
            //     );
            //     panic!();
            // }

            let mut s: f64 = 0.0;
            let mut mean: f64 = 0.0;
            let mut second: f64 = 0.0;
            let mut third: f64 = 0.0;
            let mut fourth: f64 = 0.0;

            // if theta == 0.0 && DEBUG {
            //     dbg!(&alpha_arr);
            // }

            let mut u_arr: Vec<f64> = Vec::with_capacity(triplet_arr.len());
            if meta_support.linear && false {
                let multiplier: f64 = 1.0 / alpha_shift.exp();

                let init_power = alpha_arr.iter().find(|x| **x > NEG_INFINITY).unwrap().exp();
                let mut power = init_power;
                for aj in alpha_arr.iter() {
                    if *aj == NEG_INFINITY {
                        // just to make it explicit, i think exp does this anyway
                        u_arr.push(0.0);
                        continue;
                    }
                    let u: f64 = power * multiplier;
                    s += u;
                    u_arr.push(u);

                    power *= init_power;
                }
            } else {
                for aj in alpha_arr.iter() {
                    if *aj == f64::NEG_INFINITY {
                        // just to make it explicit, i think exp does this anyway
                        u_arr.push(0.0);
                        continue;
                    }
                    let u: f64 = (aj - alpha_shift).exp();
                    s += u;
                    u_arr.push(u);
                }
            }

            for (p_index, &u) in u_arr.iter().enumerate() {
                if u == 0.0 {
                    //   l = -inf , p = 0
                    continue;
                }
                let w = u / s;
                let x = triplet_arr[p_index].0;

                if toggle.1 || toggle.2 || toggle.3 || toggle.4 {
                    mean += x * w;
                    if toggle.2 || toggle.3 || toggle.4 {
                        let x2: f64 = x * x;
                        second += x2 * w;
                        if toggle.3 || toggle.4 {
                            let x3: f64 = x2 * x;
                            third += x3 * w;
                            if toggle.4 {
                                fourth += x3 * x * w;
                            }
                        }
                    }
                }
            }

            if toggle.0 {
                total_k += alpha_shift + s.ln();
            }
            let mut mu2: f64 = -1.0;
            if toggle.1 {
                total_k1 += mean;
            }
            if toggle.2 || toggle.4 {
                mu2 = (second - mean * mean).max(0.0);
                total_k2 += mu2;
            }
            if toggle.3 {
                let mu3 = third - 3.0 * second * mean + 2.0 * mean * mean * mean;
                total_k3 += mu3;
            }
            if toggle.4 {
                if !toggle.2 {
                    mu2 = (second - mean * mean).max(0.0);
                }
                let mu4 = fourth - 4.0 * third * mean + 6.0 * second * mean * mean
                    - 3.0 * mean * mean * mean * mean;
                total_k4 += mu4 - 3.0 * mu2 * mu2;
            }
        }

        (total_k, total_k1, total_k2, total_k3, total_k4)
    }

    pub fn my_newton(
        &self,
        init_theta: f64,
        compute_biased: bool,
        mean_log: f64,
        support_index: i64,
        skip_count: usize,
        budget: f64,
    ) -> Option<(f64, f64, usize)> {
        let root = self.find_root(
            init_theta,
            -THETA_LIMIT,
            THETA_LIMIT,
            THETA_TOL,
            20,
            &(false, true, true, false, false),
            compute_biased,
            mean_log,
            support_index,
            skip_count,
            budget,
        );
        root
    }

    pub fn find_root(
        &self,
        init_theta: f64,
        min: f64,
        max: f64,
        tol: f64,
        max_iter: usize,
        toggle: &(bool, bool, bool, bool, bool),
        compute_biased: bool,
        mean_log: f64,
        support_index: i64,
        skip_count: usize,
        budget: f64,
    ) -> Option<(f64, f64, usize)> {
        let mut theta: f64 = init_theta.max(min).min(max);

        let mut count: usize = 0;

        let (_, mut y, mut dy, _, _) = self.ks(
            theta,
            toggle,
            compute_biased,
            mean_log,
            support_index,
            skip_count,
        );
        y -= budget;
        loop {
            // dbg!(count, theta, y, dy);
            // if y.abs() < 1e-12 {
            //     // this is largely irrelevant because we're interested in theta
            //     return Some(theta);
            // }

            if dy == 0.0 {
                // let (y_min, _) = func(min);
                // let (y_max, _) = func(max);
                dbg!(y, dy, theta);
                // this shouldn't happen no more
                return None;
                // return Some((
                //     if y_min.abs() < y_max.abs() { min } else { max },
                //     if y_min.abs() < y_max.abs() {
                //         min + THETA_TOL
                //     } else {
                //         max - THETA_TOL
                //     },
                // ));
            }

            let proposed_delta: f64 = -y / dy;
            let mut cur_delta: f64 = proposed_delta;
            let mut damping_factor = 1.0;
            let mut new_theta: f64 = theta + cur_delta;
            let (mut new_y, mut new_dy): (f64, f64);
            loop {
                (_, new_y, new_dy, _, _) = self.ks(
                    new_theta,
                    toggle,
                    compute_biased,
                    mean_log,
                    support_index,
                    skip_count,
                );
                new_y -= budget;

                // If the error (magnitude of y) got worse, reduce step size
                if new_y.abs() > y.abs() || new_dy == 0.0 {
                    damping_factor *= 0.5;
                    cur_delta =
                        proposed_delta.signum() * proposed_delta.abs().min(1.0) * damping_factor;
                    new_theta = theta + cur_delta;

                    if damping_factor < 1e-3 {
                        break;
                    }
                } else {
                    // The step is good, it reduced the error
                    break;
                }
            }
            if !new_theta.is_finite() {
                dbg!(y, dy, proposed_delta, theta);
                return None;
            }

            // assert!(new_theta < max && new_theta > min);

            count += 1;

            if (new_theta - theta).abs() < tol || count >= max_iter {
                return Some((new_theta, theta, count + 1));
            }

            theta = new_theta;
            (y, dy) = (new_y, new_dy);
        }
    }
}
