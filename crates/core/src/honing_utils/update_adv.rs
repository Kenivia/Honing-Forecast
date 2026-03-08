use crate::{
    advanced_honing::compute::compute_adv_dist_wrapper, constants::juice_info::JuiceInfo,
    upgrade::Upgrade,
};
use ahash::AHashMap;

use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
impl Upgrade {
    pub fn update_support_adv(&mut self, juice_info: &JuiceInfo) {
        let c_len: usize = self.adv_dists[0].len();
        let j_len: usize = self.adv_dists[1].len();
        let s_len: usize = self.adv_dists[2].len();

        for t_index in 0..7 {
            let mut this_mats_costs: Vec<f64> = Vec::with_capacity(c_len);
            let mut cost_so_far: f64 = self.unlock_costs[t_index];
            let this_cost: f64 = self.costs[t_index];
            for (index, _p) in self.adv_dists[0].iter().enumerate() {
                this_mats_costs.push(cost_so_far);

                if index >= c_len - 1 {
                    break;
                }

                cost_so_far += this_cost;
            }

            self.cost_dist[t_index].update_payload(
                this_mats_costs,
                self.state.hash,
                &self.adv_dists[0],
                this_cost,
                true,
                self.alr_failed,
            );
        }

        for &id in juice_info.adv_uindex_to_id[self.upgrade_index].iter() {
            let mut weap_cost: f64 = 0.0;
            let mut armor_cost: f64 = 0.0;
            let mut weap_support: Vec<f64> =
                Vec::with_capacity(if id == 0 { j_len } else { s_len });
            let mut armor_support: Vec<f64> =
                Vec::with_capacity(if id == 0 { j_len } else { s_len });

            let amt = juice_info.access(id, self.upgrade_index).adv_amt_used as f64;
            let this_dist = if id == 0 {
                &self.adv_dists[1]
            } else {
                &self.adv_dists[2]
            };
            for _ in this_dist.iter() {
                weap_support.push(weap_cost);
                armor_support.push(armor_cost);
                if self.is_weapon {
                    weap_cost += amt;
                } else {
                    armor_cost += amt;
                }
            }

            self.weap_juice_costs[id].update_payload(
                weap_support,
                self.state.hash,
                this_dist,
                amt,
                true,
                self.alr_failed,
            );

            self.armor_juice_costs[id].update_payload(
                armor_support,
                self.state.hash,
                this_dist,
                amt,
                true,
                self.alr_failed,
            );
        }
    }

    pub fn update_dist_adv(&mut self, adv_cache: &mut AHashMap<AdvConfig, AdvDistTriplet>) {
        self.update_adv_config();
        if !adv_cache.contains_key(&self.adv_config) {
            let out = compute_adv_dist_wrapper(&self.adv_config);
            adv_cache.insert(self.adv_config, out);
        }
        let this = adv_cache[&self.adv_config].clone();
        self.adv_dists[0].update_payload(this.cost, self.state.hash);
        self.adv_dists[1].update_payload(this.juice, self.state.hash);
        self.adv_dists[2].update_payload(this.scroll, self.state.hash);
    }
}
