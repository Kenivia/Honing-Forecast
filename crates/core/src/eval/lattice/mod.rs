//! The whole distribution of a sum, exactly, where the lattice it lives on is small.
//!
//! The saddlepoint formula needs many comparable upgrades, and is far off without any
//! sign of it where one upgrade outweighs the rest. That is mostly juices and books,
//! whose totals are small numbers, so few lattice points. Where it is too big, the metric
//! takes a coarse lattice of a set number of points a standard deviation (`Coarse`), and the
//! plain probability the saddlepoint formula.

pub mod coarse;
pub mod exact;
pub mod step;

pub use coarse::CoarseMemo;
pub use exact::Lattice;

use crate::honing_utils::state_bundle::StateBundle;
use crate::probe::Probe;

impl StateBundle {
    /// The lattice's answers at every threshold, plain and biased, for each of these free
    /// tap outcomes at once. Skipping k upgrades leaves the sum of the rest, so the sums are
    /// built from the last upgrade backwards and each one is the next with one more upgrade.
    /// Nothing past the largest threshold is kept. None where `lattice` would be.
    pub fn lattice_table(
        &self,
        support_index: i64,
        skips: &[usize],
        thresholds: &[f64],
        probe: &mut Probe,
    ) -> Vec<Option<Vec<(f64, f64)>>> {
        let mut wanted = vec![false; self.prep_output.table.len()];
        wanted[support_index as usize] = true;
        let step = self.shared_steps(skips, &wanted)[support_index as usize];
        self.lattice_table_with(support_index, skips, thresholds, None, step, probe)
    }

    /// `lattice_table`, the coarse lattices' grid cells kept in `coarse` where given, and a
    /// shared coarse pass on `shared_step` where given: the exact lattice where it is small
    /// enough (`exact_table`), a coarse one for the rest (`coarse_table`)
    pub fn lattice_table_with(
        &self,
        support_index: i64,
        skips: &[usize],
        thresholds: &[f64],
        coarse: Option<&mut CoarseMemo>,
        shared_step: Option<f64>,
        probe: &mut Probe,
    ) -> Vec<Option<Vec<(f64, f64)>>> {
        let mut out: Vec<Option<Vec<(f64, f64)>>> = vec![None; skips.len()];
        self.exact_table(support_index, skips, thresholds, &mut out);
        self.coarse_table(
            support_index,
            skips,
            thresholds,
            &mut out,
            coarse,
            shared_step,
            probe,
        );
        out
    }
}
