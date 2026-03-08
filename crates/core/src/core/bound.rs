//! There was an attempt at bounding theta, failed miserably
//! Currently it only generates a guess, so the triplet is more like a single child

use crate::state_bundle::StateBundle;
use num::traits::Inv;
use std::f64::NAN;

impl StateBundle {
    pub fn min_guess_max_triplet(
        &self,
        budget: f64,
        min_value: f64,
        max_value: f64,
        mean_var_skew: (f64, f64, f64),
    ) -> (f64, f64, f64) {
        (
            NAN,
            inverse_shifted_sigmoid(min_value, max_value, mean_var_skew, budget),
            NAN,
        )
    }
}

/// Assumes that the cumulant generating function is of the form
/// min_value +  (max_value - min_value) / (1 + [(t + t0)/alpha] ^ -beta)
///
/// which is a sigmoid-shaped curve (shifted logistic) where we can match the following:
/// - top
/// - bottom
/// - 1st derivative
/// - 2nd derivative
/// - (NOT y intercept)
///
/// We invert this to get an initial-guess theta
///
/// The original purpose of this was to prevent input_y close to min/max from diverging/stalling during rootfinding
/// but it's not very good at that but like it makes a decent guess so whatever
pub fn inverse_shifted_sigmoid(
    min_value: f64,
    max_value: f64,
    (mean, var, skew): (f64, f64, f64),
    input_y: f64,
) -> f64 {
    let z = (mean - min_value) / (max_value - min_value);
    let v1 = var / (max_value - min_value);
    let v2 = skew / (max_value - min_value);
    let t0 = (v1 * (1.0 - 2.0 * z) / (z * (1.0 - z)) - v2 / v1).inv();

    let beta = t0 * v1 / (z * (1.0 - z));
    let alpha = t0 * ((1.0 - z) / z).powf(beta.inv());

    let out = alpha * ((input_y - min_value) / (max_value - input_y)).powf(1.0 / beta) - t0;

    if !out.is_finite() {
        panic!(
            "min {:?} max {:?} mean {:?} var {:?} skew {:?} input_y {:?} z {:?} v1 {:?} v2 {:?} t0 {:?} beta {:?} alpha  {:?} ",
            min_value, max_value, mean, var, skew, input_y, z, v1, v2, t0, beta, alpha
        );
    }
    out
}
