use crate::constants::RATE_SCALE;
use crate::constants::juice_info::JuiceInfo;
use crate::honing_utils::materials::NUM_BASE_MATS;
use crate::honing_utils::support::Outline;
use crate::my_dbg;
use crate::optimizer::state::State;
use crate::upgrade::Upgrade;
use serde::{Deserialize, Serialize};

/// A normal tap as the game has it: every chance and the artisan are integers out of RATE_SCALE.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq)]
pub struct Tap {
    pub base: i64,
    /// added per failed tap, up to bonus_max
    pub bonus: i64,
    pub bonus_max: i64,
    pub event: i64,
    /// the artisan at which a tap cannot fail
    pub threshold: i64,
    /// of a failed tap's chance, what goes to the artisan
    pub artisan_percent: i64,
}

impl Tap {
    pub fn rate(&self, failed: usize, juice: i64) -> i64 {
        (self.base + (self.bonus * failed as i64).min(self.bonus_max) + self.event + juice)
            .min(RATE_SCALE)
    }
    pub fn step(&self, artisan: i64, rate: i64) -> i64 {
        artisan + rate * self.artisan_percent / 100
    }
    pub fn full(&self, artisan: i64) -> bool {
        artisan >= self.threshold
    }
    pub fn fraction(&self, artisan: i64) -> f64 {
        artisan as f64 / self.threshold as f64
    }
    pub fn from_fraction(&self, fraction: f64) -> i64 {
        (fraction * self.threshold as f64).round() as i64
    }
    /// the chance with nothing used, by failed taps, up to where the bonus stops
    pub fn fail_chances(&self) -> Vec<f64> {
        let steps = if self.bonus > 0 {
            (self.bonus_max + self.bonus - 1) / self.bonus
        } else {
            0
        };
        (0..=steps as usize)
            .map(|failed| chance(self.rate(failed, 0)))
            .collect()
    }
}

pub fn chance(rate: i64) -> f64 {
    rate as f64 / RATE_SCALE as f64
}

pub fn get_extra_arr(state: &State, juice_info: &JuiceInfo, upgrade: &Upgrade) -> Vec<i64> {
    state
        .iter()
        .map(|ids| {
            let mut rate: i64 = 0;

            // an id twice on a tap is paid for once, so it counts once
            for id in ids
                .iter()
                .enumerate()
                .filter(|(at, id)| !ids[..*at].contains(id))
                .map(|(_, id)| id)
            {
                if !juice_info.normal_uindex_to_id[upgrade.piece_type_usize][upgrade.upgrade_index]
                    .contains(id)
                {
                    my_dbg!(state, upgrade,)
                }
                rate += juice_info
                    .access(*id, upgrade.piece_type_usize, upgrade.upgrade_index)
                    .normal_rate;
            }
            rate
        })
        .collect()
}

/// the distribution and the artisan before each tap, as a fraction
pub fn new_prob_dist(
    state: &State,
    juice_info: &JuiceInfo,
    upgrade: &Upgrade,
) -> (Vec<f64>, Vec<f64>) {
    probability_distribution(
        &upgrade.tap,
        &get_extra_arr(state, juice_info, upgrade),
        upgrade.start_artisan,
        upgrade.starting_num_taps,
    )
}

// prob distribution of normal honing, adjusting for any juice usage
pub fn probability_distribution(
    tap: &Tap,
    juice: &[i64],
    start_artisan: i64,
    starting_num_taps: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut raw_chances: Vec<f64> = Vec::new();
    raw_chances.push(0.0); // chance to succeed w 0 tap, the probablity has been moved into special but it's still needed here for the unlock cost
    let mut artisan: i64 = start_artisan;
    let mut trail = vec![tap.fraction(artisan)];
    let mut count: usize = 0;

    loop {
        if tap.full(artisan) {
            raw_chances.push(1.0);
            break;
        }
        let rate = tap.rate(count + starting_num_taps, *juice.get(count).unwrap_or(&0));
        raw_chances.push(chance(rate));
        count += 1;
        artisan = tap.step(artisan, rate);
        trail.push(tap.fraction(artisan));
        if rate == RATE_SCALE {
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

    (chances, trail)
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
            let start = if self.unlocked {
                0.0
            } else {
                self.unlock_costs[t_index]
            };
            self.cost_dist[t_index].update_even(
                start,
                self.costs[t_index],
                self.state.hash,
                &self.normal_dist,
                &outline,
            );
        }

        // a juice is only counted on the taps the state uses it on
        for &id in juice_info.normal_uindex_to_id[self.piece_type_usize][self.upgrade_index].iter()
        {
            let amount = juice_info
                .access(id, self.piece_type_usize, self.upgrade_index)
                .normal_amt_used as f64;
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
            );
        }
    }

    pub fn update_dist_normal(&mut self, juice_info: &JuiceInfo) {
        if self.normal_dist.prob_state_hash == self.state.hash {
            return;
        }
        let (prob_dist, trail) = new_prob_dist(&self.state, juice_info, self);
        self.tap_artisan = trail;
        self.normal_dist.update_payload(prob_dist, self.state.hash);
    }
}
