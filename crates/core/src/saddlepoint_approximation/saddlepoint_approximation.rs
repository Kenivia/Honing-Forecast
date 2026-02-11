use crate::brute::MAX_BRUTE_SIZE;
use crate::constants::FLOAT_TOL;
use crate::performance::Performance;
use crate::saddlepoint_approximation::cumulants::KsTuple;
use crate::state_bundle::StateBundle;
use crate::upgrade::Support;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};
use std::f64::{INFINITY, NAN};

pub const DEBUG_SA: bool = false;
pub const MIN_LATTICE_SPAN: f64 = 1.0;

fn float_gcd(inp_a: f64, inp_b: f64) -> f64 {
    let mut a = inp_a;
    let mut b = inp_b;
    while b > FLOAT_TOL {
        (a, b) = (b, a % b);
        if (b - a).abs() < FLOAT_TOL {
            b = 0.0;
        }
    }
    a
}

/// Find out how much continuity correction to perform
fn lattice_span<'a, I>(meta_support_arr: I) -> f64
where
    I: Iterator<Item = &'a Support>,
{
    let mut cur_span: f64 = MIN_LATTICE_SPAN;
    let mut found_non_zeros: bool = false;

    for support in meta_support_arr {
        if support.gap_size < FLOAT_TOL || support.ignore {
            continue;
        }
        if !found_non_zeros {
            cur_span = support.gap_size;
            found_non_zeros = true;
        } else {
            cur_span = float_gcd(support.gap_size, cur_span);
        }
        if cur_span < MIN_LATTICE_SPAN {
            return MIN_LATTICE_SPAN; // always do a little bit of cont correction ig cos why not
        }
    }
    cur_span
}

impl StateBundle {
    pub fn support_size_too_big(&self, support_index: i64, skip_count: usize) -> bool {
        let mut out: usize = 0;
        for a in self.extract_collapsed_pair(support_index, skip_count) {
            if out == 0 {
                out = a.len();
            } else {
                out *= a.len();
                if out >= MAX_BRUTE_SIZE {
                    return true;
                }
            }
        }
        false
    }

    pub fn saddlepoint_approximation_wrapper(
        &self,
        support_index: i64,
        skip_count: usize,
        inp_budget: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        performance: &mut Performance,
    ) -> f64 {
        let (min_value, max_value) = self.find_min_max(support_index, skip_count);
        if inp_budget > max_value + FLOAT_TOL {
            performance.trivial_count += 1;
            return 1.0;
        }

        if inp_budget < min_value - FLOAT_TOL {
            performance.trivial_count += 1;
            return 0.0;
        };
        if self.support_size_too_big(support_index, skip_count) {
            let min_delta = self
                .extract_all_support_with_meta(support_index)
                .skip(skip_count) // this one genuinely can skip
                .map(|x| {
                    if x.gap_size.is_finite() {
                        x.gap_size
                    } else {
                        panic!("non finite gap size")
                    }
                })
                .fold(INFINITY, |prev, next| {
                    if next > FLOAT_TOL {
                        prev.min(next)
                    } else {
                        prev
                    }
                }); // UM this should be the size of the second gap so might not be accurate for combined cost ? CBB

            let span = lattice_span(
                self.extract_all_support_with_meta(support_index)
                    .skip(skip_count),
            ); // this one also
            let budget = ((inp_budget / span).floor() * span)
                .min(max_value - span)
                .max(min_value)
                + span / 2.0;

            if min_delta.is_finite() // trivial empty case with fold being the default
                && max_value - min_value > min_delta // only one or two outcomes
                && min_value + min_delta + 1.0 < budget // brute should only have to handle like n = number of upgrades cases or so. In general any higher (e.g. 2* min_delta) and this explodes insanely quickly 
                && budget < max_value - min_delta - 1.0
            {
                let res: KsTuple = self.ks(
                    0.0,
                    compute_biased,
                    simple_mean_log,
                    support_index,
                    skip_count,
                    performance,
                );
                let mean_var_skew: (f64, f64, f64) = (res.1, res.2, res.3);
                let (soft_low_limit, guess, soft_high_limit) =
                    self.min_guess_max_triplet(budget, min_value, max_value, mean_var_skew);
                return self.saddlepoint_approximation(
                    support_index,
                    skip_count,
                    min_value,
                    max_value,
                    span,
                    budget,
                    compute_biased,
                    simple_mean_log,
                    mean_var_skew,
                    performance,
                    (soft_low_limit, guess, soft_high_limit),
                    min_delta,
                );
            }
        }
        performance.brute_count += 1;

        self.brute_success_prob(
            support_index,
            skip_count,
            inp_budget,
            self.simple_avg(support_index, skip_count),
            compute_biased,
        )
    }

    /// Lugganani-Rice formula
    pub fn saddlepoint_approximation(
        &self,
        support_index: i64,
        skip_count: usize,
        min_value: f64,
        max_value: f64,
        span: f64,
        budget: f64,
        compute_biased: bool,
        simple_mean_log: f64,
        mean_var: (f64, f64, f64),
        performance: &mut Performance,
        guess_triplet: (f64, f64, f64),
        min_delta: f64,
    ) -> f64 {
        performance.sa_count += 1;
        let k1_zero = mean_var.0;
        let (_, guess, _) = guess_triplet;

        let result_opt = self.householder(
            compute_biased,
            simple_mean_log,
            support_index,
            skip_count,
            budget,
            guess,
            min_value,
            max_value,
            min_delta,
            performance,
        );

        let result: (f64, f64, usize, KsTuple) = result_opt.unwrap();
        let theta_hat = result.0;
        let ks_tuple: KsTuple = result.3;
        let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap();

        let w = |t: f64, ks_inp: f64| t.signum() * (2.0 * (t * budget - ks_inp)).sqrt();
        let u = |t: f64, ks2_inp: f64| 2.0 / span * (span * t / 2.0).sinh() * ks2_inp.sqrt(); // second continuity correction 
        let w_hat = w(theta_hat, ks_tuple.0);
        let u_hat = u(theta_hat, ks_tuple.2);

        let sa_out: f64 =
            normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat);

        let mut approx: f64 = -6.9;
        let mut actual_out = sa_out;

        // the threshold is complete trial-and-error heuristic
        if (k1_zero - budget).abs() / (k1_zero.max(budget).max(1.0)) < 1e-5 {
            performance.edgeworth_count += 1;
            let std = ks_tuple.2.sqrt();
            let z = (budget - ks_tuple.1) / std;

            let gamma3 = ks_tuple.3 / std.powi(3);
            let pdf = normal_dist.pdf(z);
            let cdf = normal_dist.cdf(z);
            let cdf_correction = pdf
                * ((gamma3 / 6.0) * (z.powi(2) - 1.0)
                    + if compute_biased {
                        0.0
                    } else {
                        let gamma4 = ks_tuple.4 / std.powi(4);
                        (gamma4 / 24.0) * (z.powi(3) - 3.0 * z)
                            + (gamma3 * gamma3 / 72.0) * (z.powi(5) - 10.0 * z.powi(3) + 15.0 * z)
                    });

            approx = cdf - cdf_correction;
            actual_out = approx;
        } else {
            performance.lugganani_count += 1;
        }

        // most of the time if something's wrong it's probably because the proabability distribution didn't add up to 1 for some reason
        // i should probably add a check for that somewhere but whatever
        if DEBUG_SA
            || !(-FLOAT_TOL..=1.0 + FLOAT_TOL).contains(&actual_out)
            || !actual_out.is_finite()
        {
            dbg!(theta_hat, k1_zero);
            dbg!(w_hat, u_hat, sa_out);

            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(
                &format!(
                    "{:?}",
                    self.extract_collapsed_pair(support_index, skip_count)
                        .into_iter()
                        .collect::<Vec<&Vec<(f64, f64)>>>()
                )
                .into(),
            );

            dbg!(
                self.extract_collapsed_pair(support_index, skip_count)
                    .collect::<Vec<&Vec<(f64, f64)>>>()
            );

            dbg!(
                budget,
                min_value,
                max_value,
                k1_zero,
                compute_biased,
                support_index,
                // low_limit,
                guess,
                // high_limit
            );

            println!(
                "zero {:?}",
                self.ks(
                    0.0,
                    compute_biased,
                    simple_mean_log,
                    support_index,
                    skip_count,
                    performance,
                )
            );

            println!(
                "guess {:?}",
                self.ks(
                    guess,
                    compute_biased,
                    simple_mean_log,
                    support_index,
                    skip_count,
                    performance,
                )
            );

            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(
                &format!(
                    "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} ",
                    compute_biased,
                    theta_hat,
                    ks_tuple,
                    k1_zero,
                    support_index,
                    2.0 * (theta_hat * budget - ks_tuple.0),
                    w_hat,
                    u_hat,
                    normal_dist.cdf(w_hat),
                    normal_dist.pdf(w_hat),
                    1.0 / w_hat - 1.0 / u_hat,
                    min_value,
                    budget,
                    self.simple_avg(support_index, skip_count),
                    max_value,
                    sa_out,
                    approx,
                    actual_out
                )
                .into(),
            );

            dbg!(
                compute_biased,
                theta_hat,
                ks_tuple,
                k1_zero,
                support_index,
                2.0 * (theta_hat * budget - ks_tuple.0),
                w_hat,
                u_hat,
                normal_dist.cdf(w_hat),
                normal_dist.pdf(w_hat),
                1.0 / w_hat - 1.0 / u_hat,
                min_value,
                budget,
                self.simple_avg(support_index, skip_count),
                max_value,
                sa_out,
                approx,
                actual_out
            );
            println!("==============================");
            panic!();
        }

        actual_out
    }
}
