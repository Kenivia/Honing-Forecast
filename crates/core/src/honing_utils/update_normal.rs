use crate::constants::ARTISAN_MULTIPLIER;
use crate::constants::juice_info::JuiceInfo;
use crate::materials::NUM_BASE_MATS;
use crate::my_dbg;
use crate::state::State;
use crate::support::Outline;
use crate::upgrade::Upgrade;

pub fn get_extra_arr(state: &State, juice_info: &JuiceInfo, upgrade: &Upgrade) -> Vec<f64> {
    state
        .iter()
        .map(|ids| {
            let mut chance: f64 = 0.0;

            // an id twice on a tap is paid for once, so it counts once
            for id in ids.iter().enumerate().filter(|(at, id)| !ids[..*at].contains(id)).map(|(_, id)| id) {
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
        if self.cost_dist[0].support_state_hash == self.state.hash {
            return;
        }

        let l_len: usize = self.normal_dist.len();

        let outline = Outline::of(&self.normal_dist);
        for t_index in 0..NUM_BASE_MATS {
            let start = if self.unlocked { 0.0 } else { self.unlock_costs[t_index] };
            self.cost_dist[t_index].update_even(start, self.costs[t_index], self.state.hash, &self.normal_dist, &outline);
        }

        // a juice is only counted on the taps the state uses it on
        for &id in juice_info.normal_uindex_to_id[self.piece_type_usize][self.upgrade_index].iter() {
            let amount = juice_info.access(id, self.piece_type_usize, self.upgrade_index).normal_amt_used as f64;
            let mut so_far: f64 = 0.0;
            let mut support: Vec<f64> = Vec::with_capacity(l_len);
            for index in 0..l_len {
                support.push(so_far);
                if index + 2 < l_len && self.state.get(index).is_some_and(|ids| ids.contains(&id)) {
                    so_far += amount;
                }
            }
            self.cost_dist[NUM_BASE_MATS + id].update_payload(
                support,
                self.state.hash,
                &self.normal_dist,
                amount,
                true,
            );
        }
    }

    pub fn update_dist_normal(&mut self, juice_info: &JuiceInfo) {
        if self.normal_dist.prob_state_hash == self.state.hash {
            return;
        }
        let prob_dist: Vec<f64> = new_prob_dist(&self.state, juice_info, self);

        self.normal_dist.update_payload(prob_dist, self.state.hash);
    }
}
