use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::constants::juice_info::JuiceInfo;
use crate::upgrade::Upgrade;

// use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
use super::one_batch::SolverStateBundle;

// use ordered_float::Float;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
// use rand::seq::IteratorRandom;
use rand::{Rng, random_bool, random_range};

// use std::f64::{MAX, MIN};
use super::constants::*;

impl SolverStateBundle {
    pub fn neighbour(&mut self) -> bool {
        let mutate_special = random_bool(
            (1.0 - self.state_bundle.special_cache[&self.state_bundle.special_state][0])
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

                upgrade.perturb(progress, &self.state_bundle.prep_output.juice_info);
                upgrade.state.update_hash();
            }
        }
        mutate_special
    }
}

impl Upgrade {
    fn perturb(&mut self, progress: f64, juice_info: &JuiceInfo) {
        if self.is_normal_honing {
            self.perturb_normal(progress, juice_info);
        } else {
            self.perturb_adv(progress, juice_info);
        }
        self.state.update_hash();
    }
    fn perturb_normal(&mut self, progress: f64, juice_info: &JuiceInfo) {
        let max_change_len = ((1.0 - progress).powi(2) * self.state.len() as f64)
            .ceil()
            .max(4.0) as i64;
        let new_juice_count = self
            .state
            .iter()
            .skip(self.alr_failed)
            .filter(|(j, _)| *j)
            .count()
            .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize);

        let new_juice_streak_len = self
            .state
            .iter()
            .skip(self.alr_failed)
            .take_while(|(j, _)| *j)
            .count()
            .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize)
            .min(new_juice_count);
        let new_book_count = self
            .state
            .iter()
            .skip(self.alr_failed)
            .filter(|(_, b)| *b > 0)
            .count()
            .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize);

        let new_book_streak_len = self
            .state
            .iter()
            .skip(self.alr_failed)
            .take_while(|(_, b)| *b > 0)
            .count()
            .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize)
            .min(new_book_count);

        // ASSUME THAT ONLY ONE TYPE OF BOOK IS AVAILIABLE FOR NOW
        let book_id_result = juice_info.normal_uindex_to_id[self.upgrade_index].last();
        if !book_id_result.is_none() {
            // this check is for below +3 where there's no juice and no books

            let book_id = *book_id_result.unwrap();

            for (i, (juice, book)) in self.state.iter_mut().skip(self.alr_failed).enumerate() {
                // if artisan >= 1.0 {
                //     effective_len = i + 1;
                //     break;
                // }
                // artisan += (46.51_f64 / 100.0)
                //     * artisan_rate
                //     * (base_chance * (1.0 + count)
                if i < new_juice_count {
                    *juice = i < new_juice_streak_len;
                } else {
                    *juice = false;
                }
                if book_id != 0 && i < new_book_count {
                    if i < new_book_streak_len {
                        *book = book_id;
                    } else {
                        *book = 0;
                    }
                } else {
                    *book = 0;
                }
                // if count < 1.0 {
                //     count += 0.1;
                // }
            }

            let state_len = self.state.len();

            for (i, (juice, book)) in self
                .state
                .iter_mut()
                .skip(self.alr_failed)
                .take(state_len - self.alr_failed)
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
        }
    }

    fn perturb_adv(&mut self, progress: f64, juice_info: &JuiceInfo) {
        let max_change_len = ((1.0 - progress).powi(2) * MAX_ADV_STATE as f64)
            .ceil()
            .max(2.0) as i64;

        assert!(self.state.len() == juice_info.adv_uindex_to_id[self.upgrade_index].len());
        for (_, val) in self.state.iter_mut() {
            *val = val
                .saturating_add_signed(random_range(-max_change_len..max_change_len) as isize)
                .min(MAX_ADV_STATE);
        }
    }
}
