//! What every threshold of one material shares under one free tap outcome: its range, lattice and cumulants

use crate::core::probability::cumulants::KsTuple;
use crate::state_bundle::StateBundle;
use crate::support::Support;
use num::Integer;
use std::f64::INFINITY;

pub const MIN_LATTICE_SPAN: f64 = 1.0;

/// The spacing of the lattice the sum lives on, which is how much continuity correction
/// to perform, and the smallest gap of any one upgrade. An upgrade with one outcome only
/// moves the lattice, so it is left out.
fn span_of<'a, I>(meta_support_arr: I) -> (f64, f64)
where
    I: Iterator<Item = &'a Support>,
{
    let mut span: i64 = 0;
    let mut min_delta: f64 = INFINITY;
    for support in meta_support_arr {
        if support.access_collapsed(false).len() < 2 {
            continue;
        }
        if !support.gap_size.is_finite() {
            panic!("non finite gap size")
        }
        min_delta = min_delta.min(support.gap_size);
        span = span.gcd(&(support.gap_size.round() as i64));
    }
    if min_delta < MIN_LATTICE_SPAN {
        return (MIN_LATTICE_SPAN, min_delta); // always do a little bit of cont correction ig cos why not
    }
    ((span as f64).max(MIN_LATTICE_SPAN), min_delta)
}

pub struct Shape {
    pub min_value: f64,
    pub max_value: f64,
    pub span: f64,
    pub min_delta: f64,
    pub too_big: bool,
    pub mean: f64,
    /// the cumulants at zero
    pub zero: KsTuple,
}

impl StateBundle {
    pub fn lattice_span(&self, support_index: i64, skip_count: usize) -> (f64, f64) {
        span_of(
            self.extract_all_support_with_meta(support_index)
                .skip(skip_count), // this one genuinely can skip
        )
    }

    pub fn shape(&self, support_index: i64, skip_count: usize) -> Shape {
        let (mut min_value, mut max_value, mut mean) = (0.0, 0.0, 0.0);
        let mut product: usize = 1;
        let mut span: i64 = 0;
        let mut min_delta: f64 = INFINITY;
        let mut zero = KsTuple::default();
        for (index, &u_index) in self.special_state.iter().enumerate() {
            let support = self.support_from_index(u_index, support_index);
            let skipped = index < skip_count;
            min_value += support.access_min(skipped);
            max_value += support.access_max(skipped);
            if skipped {
                mean += support.access_min(true);
                if !support.ignore {
                    zero.1 += support.access_min(true);
                }
                continue;
            }
            mean += support.mean;
            if !support.ignore {
                zero = zero.add(support.zero);
            }
            let outcomes = support.access_collapsed(false).len();
            product = product.saturating_mul(outcomes);
            if outcomes >= 2 {
                if !support.gap_size.is_finite() {
                    panic!("non finite gap size")
                }
                min_delta = min_delta.min(support.gap_size);
                span = span.gcd(&(support.gap_size.round() as i64));
            }
        }
        Shape {
            min_value,
            max_value,
            // always do a little bit of cont correction ig cos why not
            span: if min_delta < MIN_LATTICE_SPAN {
                MIN_LATTICE_SPAN
            } else {
                (span as f64).max(MIN_LATTICE_SPAN)
            },
            min_delta,
            too_big: product > self.tuning.max_brute_size,
            mean,
            zero,
        }
    }
}
