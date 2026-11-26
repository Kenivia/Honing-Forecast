//! The gold memo's keys: what each material's result under each free tap outcome is made of,
//! from one pass over the upgrades that also takes the shared coarse step

use crate::core::common::mix;
use crate::core::lattice::step::StepScan;
use crate::state_bundle::StateBundle;
use smallvec::SmallVec;

/// one material's memo keys out of `memo_sums`, None where the memo is off
pub fn memo_slice(sums: &[u64], material: usize, outcomes: usize) -> Option<&[u64]> {
    (!sums.is_empty()).then(|| &sums[material * outcomes..(material + 1) * outcomes])
}

impl StateBundle {
    /// What each material's key says about the upgrades, for every free tap outcome:
    /// `[material * outcomes + outcome]`. One pass over the upgrades, which are stored by upgrade.
    /// The memo's keys where it is on (`memo_sums`) and each wanted material's shared coarse step
    pub fn memo_sums_and_steps(
        &self,
        skips: &[(usize, f64)],
        wanted: &[bool],
    ) -> (Vec<u64>, Vec<Option<f64>>) {
        if self.tuning.max_memo_size > 0 && !skips.is_empty() {
            self.memo_sums(skips, wanted)
        } else {
            let all: SmallVec<[usize; 8]> = skips.iter().map(|s| s.0).collect();
            (Vec::new(), self.shared_steps(&all, wanted))
        }
    }

    /// Each wanted material's shared coarse step (`shared_steps`) comes out of the same pass.
    fn memo_sums(&self, skips: &[(usize, f64)], wanted: &[bool]) -> (Vec<u64>, Vec<Option<f64>>) {
        let num_mats = self.upgrade_arr[0].cost_dist.len();
        let n = self.special_state.len();
        let k = skips.len();
        let mut out = vec![0u64; num_mats * k];
        let (first, last) = (skips[0].0, skips[skips.len() - 1].0);
        let mut run: SmallVec<[u64; 32]> = smallvec::smallvec![0u64; num_mats];
        // what a free tap leaves of the upgrades before each outcome
        let mut slot = 0;
        for index in 0..=last.min(n) {
            while slot < skips.len() && skips[slot].0 == index {
                for (sums, skipped) in out.chunks_exact_mut(k).zip(&run) {
                    sums[slot] = mix(*skipped, 1);
                }
                slot += 1;
            }
            if index == n {
                break;
            }
            let u_index = self.special_state[index];
            for (sum, support) in run.iter_mut().zip(&self.upgrade_arr[u_index].cost_dist) {
                *sum = sum.wrapping_add(mix(u_index as u64, support.access_min(true).to_bits()));
            }
        }
        // and the states of the ones from it on
        run.fill(0);
        let scan_steps = self.tuning.coarse_shared && self.tuning.coarse_cells > 0.0;
        let mut scans: SmallVec<[StepScan; 32]> =
            smallvec::smallvec![StepScan::default(); num_mats];
        let mut steps = vec![None::<f64>; num_mats];
        let mut slot = skips.len();
        for index in (first..=n).rev() {
            if index < n {
                let u_index = self.special_state[index];
                let upgrade = &self.upgrade_arr[u_index];
                if scan_steps {
                    self.scan_upgrade(&mut scans, &upgrade.cost_dist, wanted);
                }
                for (sum, support) in run.iter_mut().zip(&upgrade.cost_dist) {
                    let made_of = if support.ignore {
                        0f64.to_bits()
                    } else {
                        upgrade.state.hash
                    };
                    *sum = sum.wrapping_add(mix(u_index as u64, made_of));
                }
            }
            while slot > 0 && skips[slot - 1].0 >= index {
                slot -= 1;
                for (sums, sum) in out.chunks_exact_mut(k).zip(&run) {
                    sums[slot] = sums[slot].wrapping_add(*sum);
                }
                if scan_steps && skips[slot].0 == index {
                    self.take_steps(&mut steps, &scans, wanted);
                }
            }
        }
        (out, steps)
    }
}
