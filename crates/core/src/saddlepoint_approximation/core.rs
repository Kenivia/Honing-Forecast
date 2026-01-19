use std::f64::NAN;

use crate::{constants::FLOAT_TOL, state_bundle::StateBundle};
pub static THETA_TOL: f64 = 1e-10;
// pub static THETA_LIMIT: f64 = 1e2;

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

            let mut sum: f64 = 0.0;
            let mut mean: f64 = 0.0;
            let mut second: f64 = 0.0;
            let mut third: f64 = 0.0;
            let mut fourth: f64 = 0.0;

            // if theta == 0.0 && DEBUG {
            //     dbg!(&alpha_arr);
            // }
            let support_len = meta_support.access_collapsed().len();
            let mut u_arr: Vec<f64> = Vec::with_capacity(support_len);
            let biggest_shift = if theta >= 0.0 {
                theta * meta_support.access_collapsed().iter().last().unwrap().0
            } else {
                theta * meta_support.access_collapsed().iter().next().unwrap().0 // this is just 0 
            };

            if meta_support.linear {
                let step = meta_support
                    .access_collapsed()
                    .iter()
                    .skip(1)
                    .next()
                    .unwrap()
                    .0;

                if theta >= 0.0 {
                    let decay_factor = (-step * theta).exp();
                    let mut current_exp_val = 1.0;
                    // dbg!(decay_factor, biggest_shift, step, theta);
                    for (_s, p) in meta_support.access_collapsed().iter().rev() {
                        // let correct_u = p * (_s * theta - biggest_shift).exp();

                        let u = p * current_exp_val;
                        // dbg!(correct_u, u);
                        sum += u;
                        u_arr.push(u);
                        current_exp_val *= decay_factor;
                    }
                    // dbg!(decay_factor, step);
                    u_arr.reverse();
                    // panic!();
                } else {
                    let decay_factor = (step * theta).exp();
                    let mut current_exp_val = 1.0;

                    for (_s, p) in meta_support.access_collapsed().iter() {
                        // let correct_u = p * (_s * theta - biggest_shift).exp();
                        let u = p * current_exp_val;

                        // dbg!(correct_u, u, current_exp_val);
                        sum += u;
                        u_arr.push(u);

                        current_exp_val *= decay_factor;
                    }
                    // dbg!(decay_factor, step, meta_support.access_collapsed());
                }
                // panic!();
            } else {
                for (s, p) in meta_support.access_collapsed().iter() {
                    let u: f64 = p * (s * theta - biggest_shift).exp();
                    sum += u;
                    u_arr.push(u);
                }
            }
            if sum == 0.0 || !sum.is_finite() {
                dbg!(
                    &u_arr,
                    theta,
                    meta_support.linear,
                    meta_support.access_collapsed()
                );
                return (NAN, NAN, NAN, NAN, NAN);
                // panic!();
            }
            for (&u, pair) in u_arr.iter().zip(meta_support.access_collapsed().iter()) {
                if u == 0.0 {
                    //   l = -inf , p = 0
                    continue;
                }
                let w = u / sum;
                let x = pair.0;

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
                total_k += biggest_shift + sum.ln();
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

    pub fn my_householder(
        &self,
        init_theta: f64,
        compute_biased: bool,
        mean_log: f64,
        support_index: i64,
        skip_count: usize,
        budget: f64,
        min_value: f64,
        max_value: f64, // limit: f64,
        mean_var: (f64, f64),
    ) -> Option<(f64, f64, usize)> {
        // e ^ like 718 or soemtihng overflows, using 700 to make sure summing a few of these wont overflow
        let (low, guess, high) = self.min_guess_max_triplet(
            // budget,
            // max_value,
            // min_value,
            support_index,
            skip_count,
            // mean_var,
            // compute_biased,
        );
        let root = self.find_root(
            guess,
            low,
            high,
            THETA_TOL,
            20,
            &(false, true, true, true, true),
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
        min_theta: f64,
        max_theta: f64,
        theta_tol: f64,
        max_iter: usize,
        toggle: &(bool, bool, bool, bool, bool),
        compute_biased: bool,
        mean_log: f64,
        support_index: i64,
        skip_count: usize,
        budget: f64,
    ) -> Option<(f64, f64, usize)> {
        let mut lower = min_theta;
        let mut upper = max_theta;

        let mut theta = init_theta;

        theta = theta.min(max_theta).max(min_theta);

        let mut init_y: f64 = NAN; // this is K'(0) = the mean 
        let mut debug_record: Vec<(f64, f64, f64, f64, f64)> = Vec::new();

        for iter in 0..max_iter {
            // 2. Evaluate Function and Derivatives
            let (_, mut y, dy, dy2, dy3) = self.ks(
                theta,
                toggle,
                compute_biased,
                mean_log,
                support_index,
                skip_count,
            );
            if iter == 0 {
                init_y = y;
            }

            y -= budget;
            debug_record.push((theta, y, dy, dy2, dy3));

            if y < 0.0 {
                lower = theta;
            } else {
                upper = theta;
            }

            if (upper - lower) < theta_tol || y.abs() < FLOAT_TOL {
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
            if proposed_theta > lower && proposed_theta < upper && dy.abs() > 0.1 {
                theta = proposed_theta.clamp(min_theta, max_theta);
            } else {
                theta = 0.5 * (lower + upper);
            }
        }

        dbg!(debug_record);
        None
    }
    //     let mut theta: f64 = init_theta.max(min_theta).min(max_theta);

    //     let mut count: usize = 0;
    //     // dbg!("start");
    //     let (int_y, mut y, mut dy, mut dy2, mut dy3) = self.ks(
    //         theta,
    //         toggle,
    //         compute_biased,
    //         mean_log,
    //         support_index,
    //         skip_count,
    //     );
    //     y -= budget;

    //     let mut debug_record: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
    //     loop {
    //         if dy == 0.0 {
    //             dbg!(y, dy, theta, min_theta, max_theta,);
    //             dbg!(debug_record);
    //             return None;
    //         }

    //         let proposed_delta: f64;

    //         if compute_biased {
    //             proposed_delta = (-2.0 * y * dy) / (-y * dy2 + 2.0 * dy.powi(2))
    //         } else {
    //             let dy_sq = dy.powi(2);
    //             let y_sq = y.powi(2);
    //             proposed_delta = -(6.0 * y * dy_sq - 3.0 * y_sq * dy2)
    //                 / (6.0 * dy_sq * dy - 6.0 * y * dy * dy2 + y_sq * dy3);
    //         }

    //         let proposed_delta = proposed_delta.clamp(min_theta - theta, max_theta - theta);
    //         if (proposed_delta).abs() < theta_tol {
    //             return Some((theta + proposed_delta, theta, count + 1));
    //         }

    //         let mut cur_delta: f64 = proposed_delta;
    //         let mut damping_factor = 1.0;
    //         let mut new_theta: f64 = theta + cur_delta;
    //         let (mut new_y, mut new_dy, mut new_dy2, mut new_dy3): (f64, f64, f64, f64);
    //         loop {
    //             if !new_theta.is_finite() {
    //                 dbg!(new_theta);
    //                 dbg!(debug_record);
    //                 return None;
    //             }
    //             (_, new_y, new_dy, new_dy2, new_dy3) = self.ks(
    //                 new_theta,
    //                 toggle,
    //                 compute_biased,
    //                 mean_log,
    //                 support_index,
    //                 skip_count,
    //             );
    //             new_y -= budget;
    //             debug_record.push((new_theta, new_y, new_dy, new_dy2, new_dy3));
    //             if new_y.abs() > y.abs() || new_dy == 0.0 {
    //                 // dbg!(y, new_y, theta, proposed_delta);

    //                 damping_factor *= 0.5;
    //                 cur_delta = proposed_delta * damping_factor;
    //                 new_theta = theta + cur_delta;
    //             } else {
    //                 break;
    //             }
    //         }

    //         // assert!(new_theta < max && new_theta > min);

    //         count += 1;

    //         if (new_theta - theta).abs() < theta_tol {
    //             return Some((new_theta, theta, count + 1));
    //         }
    //         if count >= max_iter {
    //             dbg!(new_y, new_dy, min_theta, max_theta);
    //             dbg!(debug_record);
    //             return None;
    //         }

    //         theta = new_theta;
    //         (y, dy, dy2, dy3) = (new_y, new_dy, new_dy2, new_dy3);
    //     }
    // }
}
