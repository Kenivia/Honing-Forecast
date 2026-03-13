use ahash::AHashMap;

use crate::support::Support;
use crate::{
    advanced_honing::utils::{AdvConfig, AdvDistTriplet},
    constants::juice_info::JuiceInfo,
    state_bundle::StateBundle,
    upgrade::Upgrade,
};
impl Upgrade {
    pub fn update_this_prob_dist(
        &mut self,
        adv_cache: &mut AHashMap<AdvConfig, AdvDistTriplet>,
        juice_info: &JuiceInfo,
    ) {
        if self.is_normal_honing {
            self.update_dist_normal(juice_info);
        } else {
            self.update_dist_adv(adv_cache);
        }
    }

    pub fn update_this_support(&mut self, juice_info: &JuiceInfo) {
        if self.is_normal_honing {
            self.update_support_normal(juice_info);
        } else {
            self.update_support_adv(juice_info);
        }
    }
}

impl StateBundle {
    /// These are only used for generating the histogram
    /// note that for the purpose of making consistent & comparable x-axises (or whatever the plural is) I should evaluate the same state every time, but then juice won't agree with that so maybe i should just write a special case for that? idk cbb
    pub fn one_tap(&self) -> Vec<i64> {
        let mut out = vec![0i64; self.prep_output.juice_info.total_num_avail];
        for (support_index, cost) in out.iter_mut().enumerate() {
            *cost += (self
                .extract_all_support_with_meta(support_index as i64)
                .map(|support| support.access_min(false)) // does not consider skipped to make a better graph
                .sum::<f64>())
            .ceil() as i64;
        }

        out
    }
    pub fn pity(&self) -> Vec<i64> {
        let mut out = vec![0i64; self.prep_output.juice_info.total_num_avail];
        for (support_index, cost) in out.iter_mut().enumerate() {
            *cost += (self
                .extract_all_support_with_meta(support_index as i64)
                .map(|support| support.access_max(false))
                .sum::<f64>())
            .ceil() as i64;
        }

        out
    }
    /// This thing is called insanely often and it's kinda takes longer than i'd like (like +10% overall its kinda crazy)
    /// TODO ig
    pub fn find_min_max(&self, support_index: i64, skip_count: usize) -> (f64, f64) {
        let min_value = self
            .extract_all_support_with_meta(support_index)
            .enumerate()
            .map(|(index, x)| x.access_min(skip_count > index))
            .sum();
        let max_value = self
            .extract_all_support_with_meta(support_index)
            .enumerate()
            .map(|(index, x)| x.access_max(skip_count > index))
            .sum();
        (min_value, max_value)
    }

    /// Returns an iterator of Support instances, in the order specified by special_state
    ///
    /// We don't use a &Vec to avoid re-ordering / allocating, currently I think it only needs to allocate for the iterator (due to box)
    /// Can probably turn this into a macro or something
    pub fn extract_all_support_with_meta(
        &self,
        support_index: i64,
    ) -> Box<dyn DoubleEndedIterator<Item = &Support> + '_> {
        Box::new(self.special_state.iter().map(move |&u_index| {
            let upgrade = &self.upgrade_arr[u_index];
            if support_index < 0 {
                unreachable!()
            } else {
                &upgrade.cost_dist[support_index as usize]
            }
        }))
    }

    pub fn support_from_index(&self, u_index: usize, support_index: i64) -> &Support {
        let upgrade = &self.upgrade_arr[u_index];

        if support_index < 0 {
            unreachable!()
        } else {
            &upgrade.cost_dist[support_index as usize]
        }
    }

    pub fn extract_collapsed_pair(
        &self,
        support_index: i64,
        skip_count: usize,
    ) -> Box<dyn DoubleEndedIterator<Item = &Vec<(f64, f64)>> + '_> {
        Box::new(
            self.special_state
                .iter()
                .enumerate()
                .map(move |(index, &u_index)| {
                    let upgrade = &self.upgrade_arr[u_index];
                    if support_index < 0 {
                        unreachable!()
                    } else {
                        upgrade.cost_dist[support_index as usize]
                            .access_collapsed(skip_count > index)
                    }
                }),
        )
    }

    pub fn update_individual_support(&mut self) {
        for upgrade in self.upgrade_arr.iter_mut() {
            upgrade.update_this_support(&self.prep_output.juice_info);
        }
    }

    pub fn update_dist(&mut self) {
        for upgrade in self.upgrade_arr.iter_mut() {
            upgrade.update_this_prob_dist(&mut self.adv_cache, &self.prep_output.juice_info);
        }
    }
}
