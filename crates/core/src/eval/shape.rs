//! What every threshold of one material shares under one free tap outcome: its range, lattice and cumulants

use crate::eval::saddlepoint::cumulants::KsTuple;
use crate::honing_utils::state_bundle::StateBundle;
use num::Integer;

pub const MIN_LATTICE_SPAN: f64 = 1.0;

/// `Shape::span` from the smallest gap and the gcd of the gaps
pub fn shape_span(min_delta: f64, span: i64) -> f64 {
    // always do a little bit of cont correction ig cos why not
    if min_delta < MIN_LATTICE_SPAN {
        MIN_LATTICE_SPAN
    } else {
        (span as f64).max(MIN_LATTICE_SPAN)
    }
}

pub struct Shape {
    pub min_value: f64,
    pub max_value: f64,
    /// the spacing of the lattice the sum lives on, which is how much continuity correction to
    /// perform; an upgrade with one outcome only moves the lattice, so it is left out
    pub span: f64,
    /// the smallest gap of any one upgrade
    pub min_delta: f64,
    /// how many outcomes the upgrades with more than one have between them
    pub outcomes: usize,
    pub too_big: bool,
    pub mean: f64,
    /// the cumulants at zero
    pub zero: KsTuple,
}

impl StateBundle {
    pub fn shape(&self, support_index: i64, skip_count: usize) -> Shape {
        let (mut min_value, mut max_value, mut mean) = (0.0, 0.0, 0.0);
        let mut product: usize = 1;
        let mut span: i64 = 0;
        let mut min_delta: f64 = f64::INFINITY;
        let mut varied: usize = 0;
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
                varied += outcomes;
            }
        }
        Shape {
            min_value,
            max_value,
            span: shape_span(min_delta, span),
            min_delta,
            outcomes: varied,
            too_big: product > self.tuning.max_brute_size,
            mean,
            zero,
        }
    }
}
