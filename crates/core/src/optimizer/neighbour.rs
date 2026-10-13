use std::cmp::Ordering::{Equal, Greater, Less};

use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::constants::juice_info::JuiceInfo;
// use crate::my_dbg;
use crate::state::get_pool_ids;
use crate::upgrade::Upgrade;

// use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
use super::one_batch::SolverStateBundle;

// use ordered_float::Float;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
// use rand::seq::IteratorRandom;
use rand::Rng;
use rand::rngs::SmallRng;

// use std::f64::{MAX, MIN};

impl SolverStateBundle {
    /// Reopens some chests of one stack as another of its options. False, with nothing drawn,
    /// where no stack has a choice.
    fn chest_move(&mut self) -> bool {
        let rate = self.state_bundle.tuning.chest_move_rate;
        let progress = self.progress();
        let movable: Vec<&String> = self
            .state_bundle
            .prep_output
            .chests
            .iter()
            .filter(|(_, chest)| chest.options.len() > 1 && chest.count > 0)
            .map(|(key, _)| key)
            .collect();
        if rate <= 0.0 || movable.is_empty() || !self.rng.random_bool(rate) {
            return false;
        }
        let key = movable[self.rng.random_range(0..movable.len())];
        let state = self.state_bundle.chest_states.get_mut(key).unwrap();
        let filled: Vec<usize> = (0..state.len()).filter(|&option| state[option] > 0).collect();
        let from = filled[self.rng.random_range(0..filled.len())];
        let mut to = self.rng.random_range(0..state.len() - 1);
        if to >= from {
            to += 1;
        }
        let most = ((1.0 - progress).powi(2) * state[from] as f64).ceil().max(1.0) as i64;
        let moved = self.rng.random_range(1..=most);
        state[from] -= moved;
        state[to] += moved;
        self.state_bundle.apply_chest_state(false);
        true
    }

    pub fn neighbour(&mut self) -> bool {
        if self.chest_move() {
            return false;
        }
        let mutate_special = self.rng.random_bool(
            (1.0 - self.state_bundle.special_probs()[0])
                * self.special_affinity,
        );
        if mutate_special {
            let u_len = self.state_bundle.special_state.len();
            let max_dist = u_len; //((u_len as f64) * 0.5).round().max(1.0) as usize;

            // for _ in 0..(0.5 * u_len as f64).ceil() as usize {
            let idx1: usize = self.rng.random_range(0..u_len);
            let offset = self.rng.random_range(1..=max_dist);
            let mut idx2 = if self.rng.random_bool(0.5) {
                (idx1 + offset).min(u_len - 1)
            } else {
                idx1.saturating_sub(offset)
            };
            let elem = self.state_bundle.special_state.remove(idx1);

            if idx2 > idx1 {
                idx2 -= 1;
            }
            self.state_bundle.special_state.insert(idx2, elem);
            // }
        } else {
            let progress = self.progress();
            let non_impact_weight = self.state_bundle.tuning.non_impact_weight;
            // let resolution = self.current_resolution();
            let num_upgrades = self.state_bundle.upgrade_arr.len();
            let max_mutations = (num_upgrades as f64 * progress).ceil().min(1.0) as usize;
            // let num_to_mutate = rng.random_range(1..=max_mutations.max(1));
            let mut already_mutated: Vec<bool> = vec![false; self.state_bundle.upgrade_arr.len()];

            for _ in 0..max_mutations.max(2.min(num_upgrades)) {
                let u_idx =
                    WeightedIndex::new(self.upgrade_impact.iter().zip(already_mutated.iter()).map(
                        |(x, alr)| {
                            if *alr {
                                0.0
                            } else {
                                x * (1.0 - progress) + non_impact_weight * progress
                            }
                        },
                    ))
                    .unwrap()
                    .sample(&mut self.rng);
                already_mutated[u_idx] = true;
                let upgrade = &mut self.state_bundle.upgrade_arr[u_idx];

                upgrade.perturb(
                    progress,
                    &self.state_bundle.prep_output.juice_info,
                    &mut self.rng,
                );
            }
        }
        mutate_special
    }
}

impl Upgrade {
    fn perturb(&mut self, progress: f64, juice_info: &JuiceInfo, rng: &mut SmallRng) {
        if self.is_normal_honing {
            // hashed by apply_new_streak_info
            self.perturb_normal(progress, juice_info, rng);
        } else {
            self.perturb_adv(progress, juice_info, rng);
            self.state.update_hash();
        }
    }
    fn perturb_normal(&mut self, progress: f64, juice_info: &JuiceInfo, rng: &mut SmallRng) {
        let piece_type = self.piece_type_usize;
        let upgrade_index = self.upgrade_index;
        let len = self.state.len();

        let max_change_len = ((1.0 - progress).powi(2) * len as f64).ceil().max(3.0) as i64;

        let mut streak_info = self.state.streak_cache.take().unwrap_or_else(|| {
            self.state
                .compute_streak_info(piece_type, upgrade_index, juice_info)
        });

        let pool_ids_list = self
            .state
            .pool_ids_list
            .get_or_insert_with(|| get_pool_ids(piece_type, upgrade_index, juice_info));

        for (p, pool_ids) in pool_ids_list.iter().enumerate() {
            if pool_ids.is_empty() {
                // Nothing usable in this pool
                streak_info.front[p] = (None, 0);
                streak_info.back[p] = (None, 0);
                continue;
            }

            let (front_id, front_len) = streak_info.front[p];
            let (back_id, back_len) = streak_info.back[p];
            let total = (front_len + back_len) as i64;
            
            let new_total =
                (total + rng.random_range(-max_change_len..max_change_len)).clamp(0, len as i64);
            let delta = new_total - total;

            // split the required change randomly between the two streaks.
            let delta_front = match delta.cmp(&0) {
                Greater => rng.random_range(0..=delta),
                Less => rng.random_range(delta..=0),
                Equal => 0,
            };
            let delta_back = delta - delta_front;

            let mut new_front_len = (front_len as i64 + delta_front).max(0) as usize;
            let mut new_back_len = (back_len as i64 + delta_back).max(0) as usize;

            if new_front_len + new_back_len > len {
                let excess = new_front_len + new_back_len - len;
                let back_clip = excess.min(new_back_len);
                new_back_len -= back_clip;
                let remaining = excess - back_clip;
                new_front_len = new_front_len.saturating_sub(remaining);
            }

            // a streak going from empty -> non-empty gets a random id
            // this might be enough as the order change mechanism? Idk it's not important rn
            let new_front_id = if new_front_len == 0 {
                None
            } else if front_len == 0 {
                Some(pool_ids[rng.random_range(0..pool_ids.len())])
            } else {
                front_id
            };

            let new_back_id = if new_back_len == 0 {
                None
            } else if back_len == 0 {
                Some(pool_ids[rng.random_range(0..pool_ids.len())])
            } else {
                back_id
            };

            streak_info.front[p] = (new_front_id, new_front_len);
            streak_info.back[p] = (new_back_id, new_back_len);
        }

        self.state
            .apply_new_streak_info(&streak_info, piece_type, upgrade_index, juice_info);
        self.state.streak_cache = Some(streak_info);
        // assert!(self.state.iter().all(|x| x.iter().all(|y| {
        //     juice_info.normal_uindex_to_id[self.piece_type_usize][self.upgrade_index].contains(y)
        // })));
        // my_dbg!(&self.state);
    }

    fn perturb_adv(&mut self, progress: f64, juice_info: &JuiceInfo, rng: &mut SmallRng) {
        let max_change_len = ((1.0 - progress).powi(2) * MAX_ADV_STATE as f64)
            .ceil()
            .max(2.0) as i64;

        assert!(
            self.state.len()
                == juice_info.adv_uindex_to_id[self.piece_type_usize][self.upgrade_index].len()
        );
        for val in self.state.iter_mut() {
            val[0] = val[0]
                .saturating_add_signed(rng.random_range(-max_change_len..max_change_len) as isize)
                .min(MAX_ADV_STATE);
        }
    }
}
