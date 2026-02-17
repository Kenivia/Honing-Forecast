// use crate::saddlepoint_approximation::average::DEBUG_AVERAGE;
use crate::engine::v16::one_batch::SolverStateBundle;

use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IteratorRandom;

// use std::f64::{MAX, MIN};
use super::constants::*;

impl SolverStateBundle {
    pub fn neighbour(&mut self) {
        let mutate_special = self.rng.random_bool(
            ((SPECIAL_START_CHANCE - MIN_SPECIAL_CHANCE) * (1.0 - self.progress())
                + MIN_SPECIAL_CHANCE)
                * (1.0 - self.state_bundle.special_cache[&self.state_bundle.special_state][0]),
            //
        ) && self.state_bundle.special_state.len() > 1;

        if mutate_special {
            let u_len = self.state_bundle.special_state.len();
            let max_dist = ((u_len as f64) * 0.5).round().max(1.0) as usize;

            for _ in 0..((1.0 - self.progress()) * u_len as f64).ceil() as usize {
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
            }
        } else {
            let progress = self.progress();
            let resolution = self.current_resolution();
            let num_upgrades = self.state_bundle.upgrade_arr.len();
            let max_mutations = (num_upgrades as f64 * progress).ceil() as usize;
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

                for (i, (juice, id)) in upgrade.state.iter().enumerate() {
                    if artisan >= 1.0 {
                        effective_len = i + 1;
                        break;
                    }
                    artisan += (46.51_f64 / 100.0)
                        * artisan_rate
                        * (base_chance * (1.0 + count)
                            + if *juice {
                                juice_chances[0][upgrade.upgrade_index]
                            } else {
                                0.0
                            }
                            + if *id > 0 {
                                juice_chances[*id][upgrade.upgrade_index]
                            } else {
                                0.0
                            });
                    if count < 1.0 {
                        count += 0.1;
                    }
                }

                let block_size = resolution.max(1);
                let num_blocks = (effective_len + block_size - 1) / block_size;

                let blocks_to_mutate = (0..num_blocks).choose_multiple(
                    &mut self.rng,
                    (num_blocks as f64 * progress.round().max(1.0)) as usize,
                );
                for block_idx in blocks_to_mutate {
                    let start_idx = block_idx * block_size;
                    let end_idx = (start_idx + block_size).min(upgrade.state.len());
                    let juice_target = self.rng.random_bool(0.5);
                    let book_target = {
                        let ids =
                            &self.state_bundle.prep_output.juice_info.ids[upgrade.upgrade_index];
                        let ind = self.rng.random_range(0..ids.len());
                        let out = ids[ind];
                        out
                    };
                    for i in start_idx..end_idx {
                        if i < upgrade.alr_failed {
                            continue;
                        }
                        upgrade.state[i].0 = juice_target;
                        upgrade.state[i].1 = book_target;
                    }
                }

                upgrade.state.update_hash();
            }
        }
    }
}
