//! The coarse lattice's step: each outcome's own, from the standard deviation of its sum, and the
//! finest of them that a shared pass takes (`Tuning::coarse_shared`)

use crate::eval::shape::shape_span;
use crate::honing_utils::state_bundle::StateBundle;
use crate::honing_utils::support::Support;
use num::Integer;

/// What decides the own coarse step of an outcome, gathered from the last upgrade back
#[derive(Clone, Copy)]
pub struct StepScan {
    pub span: i64,
    min_delta: f64,
    outcomes: usize,
    range: f64,
    variance: f64,
    /// the exact lattice is still small enough
    pub exact: bool,
}

impl Default for StepScan {
    fn default() -> Self {
        StepScan {
            span: 0,
            min_delta: f64::INFINITY,
            outcomes: 0,
            range: 0.0,
            variance: 0.0,
            exact: true,
        }
    }
}

impl StepScan {
    pub fn add(&mut self, this: &Support, max_lattice_work: usize) {
        if !this.ignore {
            self.variance += this.zero.2;
        }
        let pairs = this.access_collapsed(false).len();
        if pairs < 2 {
            return;
        }
        self.min_delta = self.min_delta.min(this.gap_size);
        // the span and the exact lattice only matter while every gap is at least 1
        if self.min_delta < 1.0 {
            self.exact = false;
            return;
        }
        let gap = this.gap_size.round() as i64;
        if self.span == 0 || gap % self.span != 0 {
            self.span = self.span.gcd(&gap);
        }
        if self.exact {
            self.outcomes += pairs;
            self.range += this.access_max(false) - this.access_min(false);
            self.exact = self.range / (self.span.max(1) as f64) * self.outcomes as f64
                <= max_lattice_work as f64;
        }
    }
}

impl StateBundle {
    /// Each wanted material's shared coarse step over these outcomes, the finest of their own,
    /// None where the exact lattice answers them all or nothing varies. One pass from the last upgrade
    /// back, as `exact_spans` and `shape` would see each outcome; an outcome under every threshold
    /// counts too, which needs no skipped minimum.
    pub fn shared_steps(&self, skips: &[usize], wanted: &[bool]) -> Vec<Option<f64>> {
        // there may be no upgrades
        let num_mats = self.prep_output.table.len();
        let mut out = vec![None::<f64>; num_mats];
        if !self.tuning.coarse_shared || self.tuning.coarse_cells <= 0.0 || skips.is_empty() {
            return out;
        }
        let n = self.special_state.len();
        let mut scans = vec![StepScan::default(); num_mats];
        let mut slot = skips.len();
        for index in (skips[0]..=n).rev() {
            if index < n {
                let upgrade = &self.upgrade_arr[self.special_state[index]];
                self.scan_upgrade(&mut scans, &upgrade.cost_dist, wanted);
            }
            while slot > 0 && skips[slot - 1] >= index {
                slot -= 1;
                if skips[slot] == index {
                    self.take_steps(&mut out, &scans, wanted);
                }
            }
        }
        out
    }

    /// one upgrade into each wanted material's scan
    pub fn scan_upgrade(&self, scans: &mut [StepScan], supports: &[Support], wanted: &[bool]) {
        for ((scan, support), _) in scans.iter_mut().zip(supports).zip(wanted).filter(|x| *x.1) {
            scan.add(support, self.tuning.max_lattice_work);
        }
    }

    /// the outcome the scans have reached into each wanted material's step
    pub fn take_steps(&self, steps: &mut [Option<f64>], scans: &[StepScan], wanted: &[bool]) {
        for ((step, scan), _) in steps.iter_mut().zip(scans).zip(wanted).filter(|x| *x.1) {
            if let Some(own) = self.own_step(scan) {
                *step = Some(step.map_or(own, |s| s.min(own)));
            }
        }
    }

    fn own_step(&self, scan: &StepScan) -> Option<f64> {
        let sd = scan.variance.sqrt();
        (!scan.exact && scan.min_delta.is_finite() && sd > 0.0)
            .then(|| self.coarse_step(sd, scan.min_delta, shape_span(scan.min_delta, scan.span)))
    }

    /// `coarse_cells` grid points a standard deviation, on the ladder with `coarse_cache`, a
    /// whole number of spans where there is a lattice
    pub fn coarse_step(&self, sd: f64, min_delta: f64, span: f64) -> f64 {
        let mut step = sd / self.tuning.coarse_cells;
        if self.tuning.coarse_cache && self.tuning.coarse_shared {
            // on a ladder, so it holds while moves change sd a little
            step = ((step.log2() * 8.0).floor() / 8.0).exp2();
        }
        if min_delta >= 1.0 {
            step = span * (step / span).floor().max(1.0);
        }
        step
    }
}
