use crate::constants::FLOAT_TOL;
use crate::constants::juice_info::JuiceInfo;

use crate::upgrade::Upgrade;

pub fn new_prob_dist(
    state: &Vec<(bool, usize)>,
    juice_info: &JuiceInfo,
    upgrade: &Upgrade,
    zero: f64,
) -> Vec<f64> {
    let new_extra: Vec<f64> = state
        .iter()
        .map(|(juice, id)| {
            let mut chance: f64 = 0.0;
            if *juice {
                chance += juice_info.access(0, upgrade.upgrade_index).normal_chance;
            }
            if *id > 0 {
                chance += juice_info.access(*id, upgrade.upgrade_index).normal_chance;
            }
            chance
        })
        .collect();

    probability_distribution(
        upgrade.base_chance,
        upgrade.artisan_rate,
        &new_extra,
        zero,
        upgrade.alr_failed,
        upgrade.succeeded,
        upgrade.extra_chance,
    )
}

// prob distribution of normal honing, adjusting for any juice usage
pub fn probability_distribution(
    base: f64,
    artisan_rate: f64,
    extra_arr: &[f64],
    zero: f64,
    alr_failed: usize,
    succeeded: bool,
    event_extra: f64,
) -> Vec<f64> {
    if succeeded {
        let mut v = vec![0.0; alr_failed + 1];
        v[alr_failed] = 1.0;
        return v;
    }
    let mut raw_chances: Vec<f64> = Vec::new();
    raw_chances.push(zero);
    let mut artisan: f64 = 0.0_f64;
    let mut count: usize = 0;

    loop {
        let min_count: f64 = std::cmp::min(count, 10) as f64;

        let mut current_chance: f64 =
            (base + event_extra + (min_count * base) * 0.1 + extra_arr.get(count).unwrap_or(&0.0))
                .min(1.0);
        if artisan >= 1.0 {
            current_chance = 1.0;
            raw_chances.push(current_chance);
            break;
        }
        raw_chances.push(current_chance);
        count += 1;
        artisan += 0.4651_f64 * current_chance * artisan_rate;
        if current_chance == 1.0 {
            break; // for upgrades that have 100% passrate immediately or upgrades that have above 100% success rate (juicing last few taps of like +4 or something)
        }
    }

    let mut chances = vec![0.0_f64; raw_chances.len()];
    let mut cum_chance = 1.0_f64;

    for (idx, &element) in raw_chances.iter().enumerate() {
        chances[idx] = cum_chance * element;
        cum_chance *= 1.0 - element;
    }

    for (idx, element) in chances.iter_mut().enumerate() {
        if idx <= alr_failed {
            *element = 0.0;
        }
    }
    let total = chances.iter().sum::<f64>();
    if total > FLOAT_TOL {
        for element in chances.iter_mut() {
            *element /= total;
        }
    } else {
        *chances.iter_mut().last().unwrap() = 1.0;
    }

    chances
}

impl Upgrade {
    pub fn update_support_normal(&mut self, juice_info: &JuiceInfo) {
        assert!(self.is_normal_honing);

        let l_len: usize = self.normal_dist.len();

        for t_index in 0..7 {
            let mut this_mats_costs: Vec<f64> = Vec::with_capacity(l_len);
            let mut cost_so_far: f64 = self.unlock_costs[t_index];
            let this_cost: f64 = self.costs[t_index];
            for (index, _p) in self.normal_dist.iter().enumerate() {
                this_mats_costs.push(cost_so_far);

                if index >= l_len - 1 {
                    break;
                }

                cost_so_far += this_cost;
            }

            self.cost_dist[t_index].update_payload(
                this_mats_costs,
                self.state.hash,
                &self.normal_dist,
                this_cost,
                true,
                self.alr_failed,
            );
        }

        for &id in juice_info.normal_uindex_to_id[self.upgrade_index].iter() {
            let mut weap_cost: f64 = 0.0;
            let mut armor_cost: f64 = 0.0;
            let mut weap_support: Vec<f64> = Vec::with_capacity(l_len);
            let mut armor_support: Vec<f64> = Vec::with_capacity(l_len);

            let amt = juice_info.access(id, self.upgrade_index).normal_amt_used as f64;
            for (index, _) in self.normal_dist.iter().enumerate() {
                let (juice, book) = self.state.get(index).unwrap_or(&(false, 0));
                weap_support.push(weap_cost);
                armor_support.push(armor_cost);
                if index >= l_len - 2 {
                    continue;
                }
                if *juice && id == 0 {
                    if self.is_weapon {
                        weap_cost += amt;
                    } else {
                        armor_cost += amt;
                    }
                }
                if *book == id && id > 0 {
                    if self.is_weapon {
                        weap_cost += amt;
                    } else {
                        armor_cost += amt;
                    }
                }
            }

            self.weap_juice_costs[id].update_payload(
                weap_support,
                self.state.hash,
                &self.normal_dist,
                amt,
                true,
                self.alr_failed,
            );

            self.armor_juice_costs[id].update_payload(
                armor_support,
                self.state.hash,
                &self.normal_dist,
                amt,
                true,
                self.alr_failed,
            );
        }
    }

    pub fn update_dist_normal(&mut self, juice_info: &JuiceInfo) {
        let prob_dist: Vec<f64> = new_prob_dist(&self.state, juice_info, self, 0.0);

        self.normal_dist.update_payload(prob_dist, self.state.hash);
    }
}
