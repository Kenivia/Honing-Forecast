use crate::parser::Upgrade;

use core::f64;

use num::complex::Complex64;
use quad_rs::{EvaluationError, Integrable, Integrator};
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

use std::f64::consts::PI;
use std::ops::Range;

fn newton(upgrade_arr: &[Upgrade], budget: f64, price_arr: &[f64]) -> (f64, f64, f64, f64) {
    let mut theta: f64 = 0.0;
    let (mut ks, mut ks1, mut ks2) = (0.0, 0.0, 0.0);
    for _ in 0..10 {
        (ks, ks1, ks2) = ks_012(upgrade_arr, theta, price_arr);
        // dbg!(ks1, budget);
        let f = ks1 - budget;
        if f.abs() < 1e-12 {
            break;
        }
        theta = theta - f / ks2;
        // dbg!(theta, ks, ks1, ks2);
    }
    (theta, ks, ks1, ks2)
}

// fn ks_complex(upgrade_arr: &[Upgrade], theta: Complex64) -> Complex64 {
//     let mut total_k = Complex64::new(0.0, 0.0);

//     for upgrade in upgrade_arr {
//         // Compute log MGF for this upgrade's distribution
//         total_k += log_mgf_single_complex(&upgrade, theta);
//     }

//     total_k
// }

// fn log_mgf_single_complex(upgrade: &Upgrade, theta: Complex64) -> Complex64 {
//     let mut alpha_arr: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
//     let mut max_alpha: f64 = f64::NEG_INFINITY;
//     for (index, l) in upgrade.log_prob_dist.iter().enumerate() {
//         let this_alpha: f64 =
//             l + theta.re * (index as f64 + upgrade.tap_offset as f64) * upgrade.eqv_gold_per_tap;
//         alpha_arr.push(this_alpha);
//         max_alpha = this_alpha.max(max_alpha);
//     }

//     // form b_j = exp(alpha_j - m) (real in [0,1]) and complex terms b_j * e^{i v j k}
//     // sum them with compensated complex summation
//     let mut sum: Complex64 = Complex64::new(0.0, 0.0);

//     for (index, a) in alpha_arr.iter().enumerate() {
//         sum += Complex64::new(
//             a - max_alpha,
//             theta.im * (index as f64 + upgrade.tap_offset as f64) * upgrade.eqv_gold_per_tap,
//         )
//         .exp(); // can use kahan here
//     }
//     // dbg!(
//     //     theta,
//     //     sum,
//     //     max_alpha,
//     //     sum.ln(),
//     //     sum.ln() + max_alpha,
//     //     &alpha_arr
//     // );
//     sum.ln() + max_alpha
// }

pub fn ks_012(upgrade_arr: &[Upgrade], theta: f64, price_arr: &[f64]) -> (f64, f64, f64) {
    let mut total_k: f64 = 0.0;
    let mut total_k1: f64 = 0.0;
    let mut total_k2: f64 = 0.0;

    for upgrade in upgrade_arr {
        let mut alpha_arr: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
        let mut max_alpha: f64 = f64::NEG_INFINITY;
        for (index, l) in upgrade.log_prob_dist.iter().enumerate() {
            let this_alpha: f64 =
                l + theta * (index as f64 + upgrade.tap_offset as f64) * upgrade.eqv_gold_per_tap;
            alpha_arr.push(this_alpha);
            max_alpha = this_alpha.max(max_alpha);
        }

        let mut s: f64 = 0.0;
        let mut mean: f64 = 0.0;
        let mut second: f64 = 0.0;
        // dbg!(&alpha_arr);
        let mut u_arr: Vec<f64> = Vec::with_capacity(upgrade.log_prob_dist.len());
        for aj in alpha_arr.iter() {
            let u: f64 = (aj - max_alpha).exp(); // this can for sure be optimized into a polynomial TODO
            s += u;
            u_arr.push(u);
        }

        for (index, &u) in u_arr.iter().enumerate() {
            if u == 0.0 {
                continue;
            }
            let w = u / s;
            let x = (index as f64 + upgrade.tap_offset as f64) * upgrade.eqv_gold_per_tap as f64; // value of X = j*k
            mean += x * w;
            second += (x * x) * w;
        }

        total_k += max_alpha + s.ln();
        total_k1 += mean;
        total_k2 += (second - mean * mean).max(0.0);
    }
    (total_k, total_k1, total_k2)
}

pub fn saddlepoint_approximation(upgrade_arr: &[Upgrade], budget: f64, price_arr: &[f64]) -> f64 {
    let (theta_hat, ks, _ks1, ks2) = newton(upgrade_arr, budget, price_arr);

    let w_hat: f64 =
        if theta_hat > 0.0 { 1.0 } else { -1.0 } * (2.0 * (theta_hat * budget - ks)).sqrt();
    let u_hat: f64 = theta_hat * ks2.sqrt();
    // dbg!(w_hat, u_hat, 2.0 * (theta_hat * budget - ks));
    let normal_dist: Normal = Normal::new(0.0, 1.0).unwrap(); // TODO can i pre initialize this or is there no point
    normal_dist.cdf(w_hat) + normal_dist.pdf(w_hat) * (1.0 / w_hat - 1.0 / u_hat)
    //P(S<B)≈Φ(w^)+ϕ(w^)(w^1​−u^1​)
}
// 1. Define a helper struct to hold the context for the integration

// struct SaddlePointIntegrand<'a> {
//     upgrade_arr: &'a [Upgrade],
//     theta_hat: f64,
//     ks: f64, // precomputed K(theta) real part
//     budget: f64,
// }

// impl<'a> Integrable for SaddlePointIntegrand<'a> {
//     type Input = f64;
//     type Output = f64;

//     fn integrand(&self, t: &Self::Input) -> Result<Self::Output, EvaluationError<Self::Input>> {
//         let theta = self.theta_hat;
//         let s = Complex64::new(theta, *t);

//         // K(s) as complex
//         let k_complex = ks_complex(self.upgrade_arr, s);

//         // exponent = K(s) - K(theta) - i * t * B
//         let exponent =
//             k_complex - Complex64::new(self.ks, 0.0) - Complex64::new(0.0, (*t) * self.budget);

//         let u = exponent.exp(); // complex

//         // real-form g(t) = (theta * Im(u) - t * Re(u)) / (theta^2 + t^2)
//         let denom = theta * theta + (*t) * (*t);
//         let numerator = theta * u.im - (*t) * u.re;
//         Ok(numerator / denom)
//     }
// }

// pub fn saddlepoint_inversion(upgrade_arr: &[Upgrade], budget: f64, price_arr: &[f64]) -> f64 {
//     let (theta_hat, ks, _ks1, ks2) = newton(upgrade_arr, budget, price_arr);

//     // Calculate integration bounds
//     let t_max: f64 = 200_f64; // 6.0 / ks2.sqrt();

//     let pre_factor: f64 = (ks + theta_hat * budget).exp();
//     dbg!(
//         t_max,
//         ks,
//         _ks1,
//         ks2,
//         theta_hat,
//         theta_hat * budget,
//         pre_factor
//     );
//     // 3. Set up the integrator
//     let integrator = Integrator::default()
//         .relative_tolerance(1e-6)
//         .with_maximum_iter(123000);

//     // 4. Create the integrand struct
//     let integrand: SaddlePointIntegrand = SaddlePointIntegrand {
//         upgrade_arr,
//         theta_hat,
//         ks,
//         budget,
//     };

//     // 5. Perform the integration
//     let range = Range {
//         start: 0.0,
//         end: t_max,
//     };
//     let result = integrator.integrate(integrand, range).unwrap();
//     1.0 - pre_factor * (0.5 + (1.0 / PI) * result.result.result.unwrap())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::constants::RNG_SEED;
    use crate::helpers::eqv_gold_unlock;
    use crate::parser::preparation;
    use crate::parser::{PreparationOutputs, Upgrade, probability_distribution};
    use crate::test_utils::*;
    use std::time::Instant;
    #[test]
    fn saddle_approx_test() {
        let start = Instant::now();
        let test_name = format!("saddle_approx_test");
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
            (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
        ];
        // let hone_counts: Vec<Vec<i64>> =
        //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let input_budgets = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value,
            RNG_SEED,
            PROB_MODE
        );

        let mut prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );

        for upgrade in prep_outputs.upgrade_arr.iter_mut() {
            let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
            for i in upgrade.prob_dist.iter() {
                log_prob_dist.push(i.ln());
            }
            upgrade.log_prob_dist = log_prob_dist;
        }
        let result: f64 = saddlepoint_approximation(
            &prep_outputs.upgrade_arr,
            34987.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
            &user_mats_value,
        );
        dbg!(result);
        if let Some(_cached_result) = read_cached_data::<f64>(test_name.as_str(), &hash) {
        } else {
            write_cached_data(test_name.as_str(), &hash, &result);
        }
        dbg!(start.elapsed());
    }

    // #[test]
    // fn saddle_inversion_test() {
    //     let start = Instant::now();
    //     let test_name = format!("saddle_inversion_test");
    //     let hone_counts: Vec<Vec<i64>> = vec![
    //         (0..25).map(|x| if x == 9 { 5 } else { 0 }).collect(),
    //         (0..25).map(|x| if x == 9 { 1 } else { 0 }).collect(),
    //     ];
    //     // let hone_counts: Vec<Vec<i64>> =
    //     //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
    //     let adv_counts: Vec<Vec<i64>> =
    //         vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

    //     let adv_hone_strategy: &str = "No juice";
    //     let express_event: bool = true;
    //     let input_budgets = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    //     let user_mats_value = DEFAULT_GOLD_VALUES;
    //     let hash: String = calculate_hash!(
    //         &hone_counts,
    //         &adv_counts,
    //         adv_hone_strategy,
    //         express_event,
    //         &input_budgets,
    //         &user_mats_value,
    //         RNG_SEED,
    //         PROB_MODE
    //     );

    //     let mut prep_outputs: PreparationOutputs = preparation(
    //         &hone_counts,
    //         &input_budgets,
    //         &adv_counts,
    //         express_event,
    //         &user_mats_value,
    //         adv_hone_strategy,
    //     );

    //     for upgrade in prep_outputs.upgrade_arr.iter_mut() {
    //         let mut log_prob_dist: Vec<f64> = Vec::with_capacity(upgrade.prob_dist.len());
    //         for i in upgrade.prob_dist.iter() {
    //             log_prob_dist.push(i.ln());
    //         }
    //         upgrade.log_prob_dist = log_prob_dist;
    //     }
    //     let result: f64 = saddlepoint_inversion(
    //         &prep_outputs.upgrade_arr,
    //         161161.0 - eqv_gold_unlock(&prep_outputs.unlock_costs, &prep_outputs.mats_value),
    //         &user_mats_value,
    //     );
    //     dbg!(result);
    //     if let Some(_cached_result) = read_cached_data::<f64>(test_name.as_str(), &hash) {
    //     } else {
    //         write_cached_data(test_name.as_str(), &hash, &result);
    //     }
    //     dbg!(start.elapsed());
    // }
}
