use ahash::AHashMap;
use smallvec::smallvec;

use crate::constants::ARTISAN_MULTIPLIER;
use crate::constants::juice_info::JuiceInfo;
use crate::my_dbg;
use crate::state::State;
use crate::upgrade::Upgrade;

pub fn get_extra_arr(state: &State, juice_info: &JuiceInfo, upgrade: &Upgrade) -> Vec<f64> {
    state
        .iter()
        .map(|ids| {
            let mut chance: f64 = 0.0;

            for id in ids {
                if !juice_info.normal_uindex_to_id[upgrade.piece_type_usize][upgrade.upgrade_index]
                    .contains(id)
                {
                    my_dbg!(state, upgrade,)
                }
                chance += juice_info
                    .access(*id, upgrade.piece_type_usize, upgrade.upgrade_index)
                    .normal_chance;
            }
            chance
        })
        .collect()
}

pub fn new_prob_dist(state: &State, juice_info: &JuiceInfo, upgrade: &Upgrade) -> Vec<f64> {
    probability_distribution(
        upgrade.base_chance,
        upgrade.artisan_rate,
        &get_extra_arr(state, juice_info, upgrade),
        upgrade.starting_artisan,
        upgrade.starting_num_taps,
        upgrade.extra_chance,
    )
}

// prob distribution of normal honing, adjusting for any juice usage
pub fn probability_distribution(
    base: f64,
    artisan_rate: f64,
    extra_arr: &[f64],
    starting_artisan: f64,
    starting_num_taps: usize,
    event_extra: f64,
) -> Vec<f64> {
    let mut raw_chances: Vec<f64> = Vec::new();
    raw_chances.push(0.0); // chance to succeed w 0 tap, the probablity has been moved into special but it's still needed here for the unlock cost
    let mut artisan: f64 = starting_artisan;
    let mut count: usize = 0;

    loop {
        let min_count: f64 = std::cmp::min(count + starting_num_taps, 10) as f64;

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
        artisan += ARTISAN_MULTIPLIER * current_chance * artisan_rate;
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

    // for (idx, element) in chances.iter_mut().enumerate() {
    //     if idx <= alr_failed {
    //         *element = 0.0;
    //     }
    // }
    // let total = chances.iter().sum::<f64>();
    // if total > FLOAT_TOL {
    //     for element in chances.iter_mut() {
    //         *element /= total;
    //     }
    // } else {
    //     *chances.iter_mut().last().unwrap() = 1.0;
    // }

    chances
}

impl Upgrade {
    pub fn update_support_normal(&mut self, juice_info: &JuiceInfo) {
        assert!(self.is_normal_honing);

        let l_len: usize = self.normal_dist.len();

        for t_index in 0..7 {
            let mut this_mats_costs: Vec<f64> = Vec::with_capacity(l_len);
            let mut cost_so_far: f64 = if self.unlocked {
                0.0
            } else {
                self.unlock_costs[t_index]
            };
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
            );
        }
        let possible_ids =
            &juice_info.normal_uindex_to_id[self.piece_type_usize][self.upgrade_index];

        let mut out: AHashMap<usize, (f64, Vec<f64>, f64)> =
            AHashMap::from_iter(possible_ids.iter().map(|id| {
                (
                    *id,
                    (
                        0.0,
                        Vec::new(),
                        juice_info
                            .access(*id, self.piece_type_usize, self.upgrade_index)
                            .normal_amt_used as f64,
                    ),
                )
            }));
        for (index, _) in self.normal_dist.iter().enumerate() {
            for id in possible_ids.iter() {
                let current = out[id].0;
                out.get_mut(&id).unwrap().1.push(current);
            }

            if index >= l_len - 2 {
                continue;
            }

            for actual_id in self.state.get(index).unwrap_or(&smallvec![]) {
                out.get_mut(&actual_id).unwrap().0 += out[actual_id].2;
            }
            // if *juice && id <= 1 {
            //     weap_cost += amt;
            // }
            // if *book == id && id > 1 {
            //     weap_cost += amt;
            // }
        }
        for (id, (_, support, gap_size)) in out {
            self.cost_dist[id + 7].update_payload(
                support,
                self.state.hash,
                &self.normal_dist,
                gap_size,
                true,
            );
        }
    }

    pub fn update_dist_normal(&mut self, juice_info: &JuiceInfo) {
        let prob_dist: Vec<f64> = new_prob_dist(&self.state, juice_info, self);

        self.normal_dist.update_payload(prob_dist, self.state.hash);
    }
}
