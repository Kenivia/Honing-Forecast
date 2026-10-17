use std::cmp::Ordering::{Equal, Greater, Less};

use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::constants::SPECIAL_TOL;
use crate::constants::juice_info::JuiceInfo;
// use crate::my_dbg;
use crate::probe::Probe;
use crate::state::get_pool_ids;
use crate::upgrade::Upgrade;

// use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
use super::one_batch::SolverStateBundle;
use super::response::Response;

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
        let filled: Vec<usize> = (0..state.len())
            .filter(|&option| state[option] > 0)
            .collect();
        let from = filled[self.rng.random_range(0..filled.len())];
        let mut to = self.rng.random_range(0..state.len() - 1);
        if to >= from {
            to += 1;
        }
        let most = ((1.0 - progress).powi(2) * state[from] as f64)
            .ceil()
            .max(1.0) as i64;
        let moved = self.rng.random_range(1..=most);
        state[from] -= moved;
        state[to] += moved;
        self.state_bundle.apply_chest_state(false);
        true
    }

    /// what the rest of the state is to each upgrade: everything a response depends on
    fn rest_of(&self) -> Vec<u64> {
        let hasher = ahash::RandomState::with_seeds(1, 2, 3, 4);
        let part = |index: usize| {
            hasher.hash_one((index, self.state_bundle.upgrade_arr[index].state.hash))
        };
        let count = self.state_bundle.upgrade_arr.len();
        let all = (0..count).fold(
            hasher.hash_one((
                &self.state_bundle.special_state,
                &self.state_bundle.chest_states,
            )),
            |sum, index| sum.wrapping_add(part(index)),
        );
        (0..count)
            .map(|index| all.wrapping_sub(part(index)))
            .collect()
    }

    /// A normal upgrade that does not have its best plan against the rest takes it. False
    /// where every one has it already.
    fn respond(&mut self, progress: f64) -> bool {
        let tuning = self.state_bundle.tuning.clone();
        let mut rest = self.rest_of();
        loop {
            let open: Vec<usize> = (0..self.settled.len())
                .filter(|&index| {
                    let this = &self.state_bundle.upgrade_arr[index];
                    this.is_normal_honing && self.settled[index] != (this.state.hash, rest[index])
                })
                .collect();
            if open.is_empty() {
                return false;
            }
            let index =
                open[WeightedIndex::new(open.iter().map(|&index| self.upgrade_impact[index]))
                    .unwrap()
                    .sample(&mut self.rng)];
            let kicked = tuning.response_kick > 0.0
                && self.settled.len() > 1
                && self.rng.random_bool(tuning.response_kick);
            if kicked {
                let mut other = self.rng.random_range(0..self.settled.len() - 1);
                if other >= index {
                    other += 1;
                }
                let block = self.state_bundle.block();
                self.state_bundle.upgrade_arr[other].perturb(
                    progress,
                    block,
                    &self.state_bundle.prep_output.juice_info,
                    &mut self.rng,
                );
                rest = self.rest_of();
            }
            let response = Response::of(&mut self.state_bundle, index, &mut Probe::default());
            let this = &mut self.state_bundle.upgrade_arr[index];
            let now = response
                .model
                .canonical(&response.model.indices(&this.state.payload));
            let plan = response.better(&now, tuning.response_joint).1;
            self.extra_cost += response.work(tuning.response_steps).ceil() as i64;
            let changed = now != plan;
            if changed {
                this.state.update_payload(response.model.plan(&plan));
            }
            self.settled[index] = (this.state.hash, rest[index]);
            if changed || kicked {
                return true;
            }
        }
    }

    pub fn neighbour(&mut self) -> bool {
        if self.chest_move() {
            return false;
        }
        let tuning = &self.state_bundle.tuning;
        let (special_floor, order_reach) = (tuning.special_floor, tuning.order_reach);
        let reached = 1.0 - self.state_bundle.special_probs()[0];
        // a first upgrade that is out of reach says nothing of the others
        let floor = if self.state_bundle.affords_a_free_tap() {
            special_floor
        } else {
            0.0
        };
        let mutate_special = self
            .rng
            .random_bool(reached.max(floor) * self.special_affinity);
        if mutate_special && order_reach {
            // any upgrade, to a place a free tap can get to
            let u_len = self.state_bundle.special_state.len();
            let reach = self
                .state_bundle
                .special_probs()
                .iter()
                .rposition(|&chance| chance >= SPECIAL_TOL)
                .unwrap_or(0);
            let idx1: usize = self.rng.random_range(0..u_len);
            let idx2: usize = self.rng.random_range(0..=reach.min(u_len - 1));
            let elem = self.state_bundle.special_state.remove(idx1);
            self.state_bundle.special_state.insert(idx2, elem);
        } else if mutate_special {
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
            let response_rate = self.state_bundle.tuning.response_rate;
            if response_rate > 0.0 && self.rng.random_bool(response_rate) && self.respond(progress)
            {
                return false;
            }
            let non_impact_weight = self.state_bundle.tuning.non_impact_weight;
            // let resolution = self.current_resolution();
            let num_upgrades = self.state_bundle.upgrade_arr.len();
            let block = self.state_bundle.block();
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
                    block,
                    &self.state_bundle.prep_output.juice_info,
                    &mut self.rng,
                );
            }
        }
        mutate_special
    }
}

impl Upgrade {
    fn perturb(&mut self, progress: f64, block: usize, juice_info: &JuiceInfo, rng: &mut SmallRng) {
        if self.is_normal_honing {
            // hashed by apply_new_streak_info
            self.perturb_normal(progress, block, juice_info, rng);
        } else {
            self.perturb_adv(progress, juice_info, rng);
            self.state.update_hash();
        }
    }
    fn perturb_normal(
        &mut self,
        progress: f64,
        block: usize,
        juice_info: &JuiceInfo,
        rng: &mut SmallRng,
    ) {
        let piece_type = self.piece_type_usize;
        let upgrade_index = self.upgrade_index;
        let len = self.state.len();

        // a change shorter than a block would mostly be rounded away
        let max_change_len = ((1.0 - progress).powi(2) * len as f64)
            .ceil()
            .max(3.0)
            .max(2.0 * block as f64) as i64;

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

            // both streaks start and end where a block does, but for the end of the state
            new_front_len -= new_front_len % block;
            new_back_len = len.saturating_sub((len - new_back_len).next_multiple_of(block));

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
