// use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
use super::one_batch::SolverStateBundle;

use ordered_float::Float;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IteratorRandom;
use rand::{Rng, random_bool, random_range};

// use std::f64::{MAX, MIN};
use super::constants::*;

impl SolverStateBundle {
    pub fn neighbour(&mut self) {
        if random_bool(
            (1.0 - self.state_bundle.special_cache[&self.state_bundle.special_state][0]) * 0.3,
        ) {
            let u_len = self.state_bundle.special_state.len();
            let max_dist = u_len; //((u_len as f64) * 0.5).round().max(1.0) as usize;

            // for _ in 0..(0.5 * u_len as f64).ceil() as usize {
            let idx1 = self.rng.random_range(0..u_len - 1);
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
            let resolution = self.current_resolution();
            let num_upgrades = self.state_bundle.upgrade_arr.len();
            let max_mutations = (num_upgrades as f64 * progress).ceil().min(1.0) as usize;
            // let num_to_mutate = rng.random_range(1..=max_mutations.max(1));
            let mut already_mutated: Vec<bool> = vec![false; self.state_bundle.upgrade_arr.len()];

            for _ in 0..max_mutations {
                let u_idx =
                    WeightedIndex::new(self.upgrade_impact.iter().zip(already_mutated.iter()).map(
                        |(x, alr)| {
                            if *alr {
                                0.0
                            } else {
                                x * (1.0 - progress) + NON_IMPACT_WEIGHT * progress
                            }
                        },
                    ))
                    .unwrap()
                    .sample(&mut self.rng);
                already_mutated[u_idx] = true;
                let upgrade = &mut self.state_bundle.upgrade_arr[u_idx];

                if upgrade.succeeded {
                    continue;
                }

                let mut effective_len = 0;

                let mut artisan: f64 = 0.0;
                let mut count: f64 = 0.1;
                let base_chance = upgrade.base_chance;
                let artisan_rate = upgrade.artisan_rate;
                let juice_chances = &self.state_bundle.prep_output.juice_info.chances_id;

                let max_change_len =
                    ((1.0 - progress).powi(2) * upgrade.state.len() as f64).ceil() as i64;
                let new_juice_count = upgrade
                    .state
                    .iter()
                    .filter(|(j, _)| *j)
                    .count()
                    .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize);

                let new_juice_streak_len = upgrade
                    .state
                    .iter()
                    .take_while(|(j, _)| *j)
                    .count()
                    .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize)
                    .min(new_juice_count);
                let new_book_count = upgrade
                    .state
                    .iter()
                    .filter(|(_, b)| *b > 0)
                    .count()
                    .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize);

                let new_book_streak_len = upgrade
                    .state
                    .iter()
                    .take_while(|(_, b)| *b > 0)
                    .count()
                    .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize)
                    .min(new_book_count);
                // ASSUME THAT ONLY ONE TYPE OF BOOK IS AVAILIABLE FOR NOW
                let book_id = *self.state_bundle.prep_output.juice_info.ids[upgrade.upgrade_index]
                    .last()
                    .unwrap();
                for (i, (juice, book)) in upgrade.state.iter_mut().enumerate() {
                    if artisan >= 1.0 {
                        effective_len = i + 1;
                        break;
                    }
                    artisan += (46.51_f64 / 100.0)
                        * artisan_rate
                        * (base_chance * (1.0 + count)
                            + if i < new_juice_count {
                                if i < new_juice_streak_len {
                                    *juice = true;
                                } else {
                                    *juice = false;
                                }
                                juice_chances[0][upgrade.upgrade_index]
                            } else {
                                *juice = false;
                                0.0
                            }
                            + if book_id != 0 && i < new_book_count {
                                if i < new_book_streak_len {
                                    *book = book_id;
                                } else {
                                    *book = 0;
                                }
                                juice_chances[book_id][upgrade.upgrade_index]
                            } else {
                                *book = 0;
                                0.0
                            });
                    if count < 1.0 {
                        count += 0.1;
                    }
                }
                for (i, (juice, book)) in upgrade
                    .state
                    .iter_mut()
                    .take(effective_len)
                    .rev()
                    .enumerate()
                {
                    if i < (new_juice_count - new_juice_streak_len) {
                        *juice = true;
                    }
                    if i < (new_book_count - new_book_streak_len) {
                        *book = book_id;
                    }
                }
                upgrade.state.update_hash();
            }
        }
    }
}
