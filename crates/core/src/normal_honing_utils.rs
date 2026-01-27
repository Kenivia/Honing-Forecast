use std::f64::NAN;

use crate::constants::{FLOAT_TOL, JuiceInfo};
use crate::parser::PreparationOutput;
use crate::state_bundle::StateBundle;
use crate::upgrade::{Support, Upgrade};

pub fn encode_one_positions(v1: &[(bool, usize)]) -> String {
    v1.iter()
        .map(|(uppercase, num)| {
            let letter: char = if *num == 0 {
                'x' as char
            } else {
                (b'a' + (*num as u8 - 1)) as char
            };

            if *uppercase {
                letter.to_ascii_uppercase()
            } else {
                letter
            }
        })
        .collect()
}

impl StateBundle {
    pub fn one_tap(&self) -> Vec<i64> {
        let mut out = vec![0i64; 7 + self.prep_output.juice_info.num_avail * 2];
        // for upgrade in self.upgrade_arr.iter() {
        for (support_index, cost) in out.iter_mut().enumerate() {
            *cost += self
                .extract_support_with_meta(support_index as i64, 0)
                .map(|support| {
                    support
                        .access_collapsed()
                        .iter()
                        .find(|(_, p)| *p > FLOAT_TOL)
                        .unwrap()
                        .0
                        .ceil() as i64
                })
                .sum::<i64>();
        }
        // }
        out
    }
    pub fn pity(&self) -> Vec<i64> {
        let mut out = vec![0i64; 7 + self.prep_output.juice_info.num_avail * 2];
        for (support_index, cost) in out.iter_mut().enumerate() {
            *cost += self
                .extract_support_with_meta(support_index as i64, 0)
                .map(|support| support.max_value.ceil() as i64)
                .sum::<i64>();
        }

        out
    }
    pub fn get_one_tap_pity(&self) -> (Vec<i64>, Vec<i64>) {
        (self.one_tap(), self.pity())
    }
    pub fn find_min_max(&self, support_index: i64, skip_count: usize) -> (f64, f64) {
        let min_value = self
            .extract_support_with_meta(support_index, skip_count)
            .map(|support| {
                support
                    .access_collapsed()
                    .iter()
                    .find(|(_, p)| *p > FLOAT_TOL)
                    .unwrap_or(&(0.0, 0.0))
                    .0
            })
            .sum();
        let max_value = self
            .extract_support_with_meta(support_index, skip_count)
            .map(|support| support.max_value)
            .sum();
        (min_value, max_value)
    }

    pub fn encode_all(&self) -> String {
        let mut strings = Vec::new();
        strings.push(format!("{:?}", self.special_state));
        for (index, upgrade) in self.upgrade_arr.iter().enumerate() {
            strings.push(
                self.upgrade_arr[index].name_string.clone()
                    + ": "
                    + &encode_one_positions(&upgrade.state),
            );
        }
        strings.join("\n")
    }

    pub fn extract_support_with_meta(
        &self,
        support_index: i64,
        skip_count: usize,
    ) -> Box<dyn Iterator<Item = &Support> + '_> {
        let num_avail = self.prep_output.juice_info.num_avail;
        Box::new(
            self.special_state
                .iter()
                .map(move |&u_index| {
                    let upgrade = &self.upgrade_arr[u_index];
                    if support_index < 0 {
                        &upgrade.combined_gold_costs
                    } else if support_index < 7 {
                        &upgrade.cost_dist[support_index as usize]
                    } else if support_index < 7 + num_avail as i64 {
                        &upgrade.weap_juice_costs[support_index as usize - 7]
                    } else {
                        &upgrade.armor_juice_costs[support_index as usize - 7 - num_avail]
                    }
                })
                .skip(skip_count),
        )
    }
    pub fn extract_collapsed_pair(
        &self,
        support_index: i64,
        skip_count: usize,
    ) -> Box<dyn DoubleEndedIterator<Item = &Vec<(f64, f64)>> + '_> {
        let num_avail = self.prep_output.juice_info.num_avail;
        Box::new(
            self.special_state
                .iter()
                .map(move |&u_index| {
                    let upgrade = &self.upgrade_arr[u_index];
                    if support_index < 0 {
                        upgrade.combined_gold_costs.access_collapsed()
                    } else if support_index < 7 {
                        upgrade.cost_dist[support_index as usize].access_collapsed()
                    } else if support_index < 7 + num_avail as i64 {
                        upgrade.weap_juice_costs[support_index as usize - 7].access_collapsed()
                    } else {
                        upgrade.armor_juice_costs[support_index as usize - 7 - num_avail]
                            .access_collapsed()
                    }
                })
                .skip(skip_count),
        )
    }

    pub fn update_combined(&mut self) {
        let prep_output: &mut PreparationOutput = &mut self.prep_output;

        for upgrade in self.upgrade_arr.iter_mut() {
            let l_len = upgrade.prob_dist.len();
            let mut this_combined = Vec::with_capacity(l_len);
            let mut cost_so_far: f64 = 0.0;
            let mut first_gap: f64 = NAN;
            for (p_index, _) in upgrade.prob_dist.iter().enumerate() {
                this_combined.push(cost_so_far);

                if p_index >= l_len - 1 {
                    break;
                }

                cost_so_far += upgrade.eqv_gold_per_tap;
                if p_index < l_len - 2 {
                    // cannot juice pity
                    let (juice, state_id) = upgrade.state[p_index];
                    if juice {
                        add_juice_gold_cost(&prep_output.juice_info, upgrade, &mut cost_so_far, 0);
                    }
                    if state_id > 0 {
                        add_juice_gold_cost(
                            &prep_output.juice_info,
                            upgrade,
                            &mut cost_so_far,
                            state_id,
                        );
                    }
                }

                if first_gap.is_nan() {
                    first_gap = cost_so_far;
                }
            }
            upgrade.combined_gold_costs.update_payload(
                this_combined,
                upgrade.state.hash,
                &mut upgrade.prob_dist,
                first_gap,
                cost_so_far,
            );

            // upgrade.combined_gold_costs.associated_state_hash
        }
    }

    pub fn update_individual_support(&mut self) {
        let prep_output = &mut self.prep_output;

        for upgrade in self.upgrade_arr.iter_mut() {
            upgrade.update_this_individual_support(prep_output);
        }
    }

    pub fn update_dist(&mut self) {
        // TODO add a toggle for computing log or not
        // dbg!(&prep_output, &state_bundle);
        // let zero_probs: Vec<f64> = special_probs(prep_output, state_bundle);
        // dbg!(&zero_probs);
        for upgrade in self.upgrade_arr.iter_mut() {
            upgrade.update_this_prob_dist(&self.prep_output);
            // if compute_log {
            //     upgrade.prob_dist.compute_log();
            // }

            // gold_costs_arr.push(gold_cost_record);
        }
    }
    pub fn flattened_effective_budgets(&self) -> impl Iterator<Item = f64> {
        self.prep_output
            .effective_budgets
            .iter()
            .map(|x| *x as f64)
            .chain(
                self.prep_output
                    .juice_books_owned
                    .iter()
                    .map(|x| x.0 as f64),
            )
            .chain(
                self.prep_output
                    .juice_books_owned
                    .iter()
                    .map(|x| x.1 as f64),
            )
    }

    pub fn flattened_price(&self) -> impl Iterator<Item = f64> {
        self.prep_output
            .price_arr
            .iter()
            .map(|x| *x as f64)
            .chain(
                self.prep_output
                    .juice_info
                    .one_gold_cost_id
                    .iter()
                    .map(|x| x.0 as f64),
            )
            .chain(
                self.prep_output
                    .juice_info
                    .one_gold_cost_id
                    .iter()
                    .map(|x| x.1 as f64),
            )
    }

    pub fn flattened_leftover(&self) -> impl Iterator<Item = f64> {
        self.prep_output
            .leftover_values
            .iter()
            .map(|x| *x as f64)
            .chain(
                self.prep_output
                    .juice_info
                    .one_leftover_value_id
                    .iter()
                    .map(|x| x.0 as f64),
            )
            .chain(
                self.prep_output
                    .juice_info
                    .one_leftover_value_id
                    .iter()
                    .map(|x| x.1 as f64),
            )
    }

    // these are for brute force stuff that expect to be able to index into the array that uh i can't be bothered to rewrite (it's 100% possible)
    // actaully can probably just store a vector of references?
    pub fn gather_prob_dist(&self) -> Vec<Vec<f64>> {
        let mut arr = Vec::with_capacity(self.upgrade_arr.len());
        for upgrade in self.upgrade_arr.iter() {
            arr.push(upgrade.prob_dist.payload.clone());
        }
        arr
    }
    // pub fn gather_log_prob_dist(&self) -> Vec<Vec<f64>> {
    //     let mut arr = Vec::with_capacity(self.upgrade_arr.len());
    //     for upgrade in self.upgrade_arr.iter() {
    //         arr.push(upgrade.prob_dist.log_prob_dist().clone());
    //     }
    //     arr
    // }

    pub fn gather_collapsed(
        &self,
        support_index: i64,
        skip_count: usize,
        field: usize,
    ) -> Vec<Vec<f64>> {
        self.extract_support_with_meta(support_index, skip_count)
            .map(|support| {
                support
                    .access_collapsed()
                    .iter()
                    .map(|(x, y)| match field {
                        0 => *x,
                        _ => *y,
                    })
                    .collect()
            })
            .collect()
    }

    pub fn gather_combined_gold_cost(&self) -> Vec<Vec<f64>> {
        let mut arr = Vec::with_capacity(self.upgrade_arr.len());
        for upgrade in self.upgrade_arr.iter() {
            arr.push(upgrade.combined_gold_costs.support.clone());
        }
        arr
    }

    // pub fn get_one_tap_pity(&self) -> (Vec<i64>, Vec<i64>) {
    //     debug_assert!(self.prep_output.unlock_costs.len() == 2);
    //     const DATA_SIZE: usize = 2;
    //     let mut cost_data: Vec<Vec<i64>> =
    //         vec![vec![0i64; 7 + self.prep_output.juice_info.num_avail * 2]; 2];

    //     for upgrade in self.upgrade_arr.iter() {
    //         cost_data[0]
    //         // let pd_len: f64 = upgrade.prob_dist.len().saturating_sub(1) as f64;
    //         // for trial_num in 0..DATA_SIZE {
    //         //     let rolled_tap =
    //         //         ((pd_len * (trial_num) as f64) / (DATA_SIZE as f64 - 1.0)).floor() as usize;
    //         //     for cost_type in 0..7 {
    //         //         cost_data[trial_num][cost_type] +=
    //         //             upgrade.costs[cost_type] * (rolled_tap as i64 + upgrade.tap_offset);
    //         //     }
    //         //     if !upgrade.is_normal_honing {
    //         //         cost_data[trial_num][if upgrade.is_weapon { 7 } else { 8 }] +=
    //         //             upgrade.adv_juice_cost[rolled_tap].ceil() as i64;
    //         //     }
    //         // }
    //     }
    //     for row in &mut cost_data {
    //         row[3] += self.prep_output.unlock_costs[0];
    //         row[6] += self.prep_output.unlock_costs[1];
    //     }
    //     (cost_data[0].clone(), cost_data[1].clone())
    // }
}

pub fn generate_first_deltas(delta: f64, length: usize, non_zeros: usize) -> Vec<f64> {
    [vec![delta; non_zeros], vec![0.0; length - non_zeros]].concat()
}
/// Sums up gold values from materials and juices
pub fn add_up_golds(mats_gold: &Vec<f64>, juice_gold: &Vec<(f64, f64)>) -> f64 {
    mats_gold.iter().fold(0.0, |last, new| last + *new)
        + juice_gold
            .iter()
            .fold(0.0, |last, new| last + new.0 + new.1)
}

/// Applies price/leftover pricing to concrete consumption values.
/// Used by Monte Carlo simulation where we have actual consumption, not expectations.
/// positive diff = leftover (use leftover_value), negative diff = shortage (use price)
pub fn apply_price_leftovers(
    mats: &[f64],
    juice: &[(f64, f64)],
    state_bundle: &StateBundle,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    apply_price_generic(mats, juice, &state_bundle.prep_output, false)
}

/// Applies naive (linear) pricing to concrete consumption values.
/// Used by Monte Carlo simulation. This is equivalent to apply_price_leftovers
/// when leftover_price = price.
pub fn apply_price_naive(
    mats: &[f64],
    juice: &[(f64, f64)],
    state_bundle: &StateBundle,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    apply_price_generic(mats, juice, &state_bundle.prep_output, true)
}

/// Generic price application for concrete consumption values.
/// When `naive` is true, uses price for both leftover and shortage.
pub fn apply_price_generic(
    mats: &[f64],
    juice: &[(f64, f64)],
    prep_output: &PreparationOutput,
    naive: bool,
) -> (Vec<f64>, Vec<(f64, f64)>) {
    let mut mats_gold = vec![0.0; mats.len()];
    let mut juice_gold = vec![(0.0, 0.0); juice.len()];

    for (index, gold) in mats_gold.iter_mut().enumerate() {
        let diff: f64 = prep_output.budgets[index] as f64 - mats[index];
        *gold = diff
            * if naive {
                prep_output.price_arr[index]
            } else if diff > 0.0 {
                prep_output.leftover_values[index]
            } else {
                prep_output.price_arr[index]
            };
    }

    for (id, (weap, armor)) in juice_gold.iter_mut().enumerate() {
        let weap_diff: f64 = prep_output.juice_books_owned[id].0 as f64 - juice[id].0;
        let armor_diff: f64 = prep_output.juice_books_owned[id].1 as f64 - juice[id].1;

        *weap = weap_diff
            * if naive {
                prep_output.juice_info.one_gold_cost_id[id].0
            } else if weap_diff > 0.0 {
                prep_output.juice_info.one_leftover_value_id[id].0
            } else {
                prep_output.juice_info.one_gold_cost_id[id].0
            };

        *armor = armor_diff
            * if naive {
                prep_output.juice_info.one_gold_cost_id[id].1
            } else if armor_diff > 0.0 {
                prep_output.juice_info.one_leftover_value_id[id].1
            } else {
                prep_output.juice_info.one_gold_cost_id[id].1
            };
    }

    (mats_gold, juice_gold)
}

pub fn add_juice_gold_cost(
    juice_info: &JuiceInfo,
    upgrade: &Upgrade,
    cost_so_far: &mut f64,
    id: usize,
) {
    *cost_so_far += juice_info.amt_used_id[id][upgrade.upgrade_index] as f64
        * if upgrade.is_weapon {
            juice_info.one_gold_cost_id[id].0
        } else {
            juice_info.one_gold_cost_id[id].1
        };
}

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
                chance += juice_info.chances_id[0][upgrade.upgrade_index];
            }
            if *id > 0 {
                chance += juice_info.chances_id[*id][upgrade.upgrade_index];
            }
            chance
        }) //if *x > 0 { upgrade.base_chance } else { 0.0 }) //
        .collect();

    let out = probability_distribution(
        upgrade.base_chance,
        upgrade.artisan_rate,
        &new_extra,
        zero,
        upgrade.alr_failed,
        upgrade.succeeded,
    );
    // for o in out.iter() {
    //     if !o.is_finite() || *o < 0.0 {
    //         dbg!(
    //             &out,
    //             &upgrade,
    //             &juice_info.chances[upgrade.upgrade_index],
    //             &new_extra,
    //             zero
    //         );
    //         panic!();
    //     }
    // }

    out
}

// prob distribution of normal honing, adjusting for any juice usage
pub fn probability_distribution(
    base: f64,
    artisan_rate: f64,
    extra_arr: &[f64],
    zero: f64,
    alr_failed: usize,
    succeeded: bool,
) -> Vec<f64> {
    if succeeded {
        let mut v = vec![0.0; alr_failed + 1];
        v[alr_failed] = 1.0;
        // web_sys::console::log_1(&format!("{:?}", v).into());
        return v;
    }
    let mut raw_chances: Vec<f64> = vec![zero];
    let mut artisan: f64 = 0.0_f64;
    let mut count: usize = 0;
    // web_sys::console::log_1(&alr_failed.into());

    // web_sys::console::log_1(&base.into());
    // web_sys::console::log_1(&artisan_rate.into());
    // web_sys::console::log_1(&format!("{:?}", extra_arr).into());
    // web_sys::console::log_1(&format!("a").into());
    loop {
        // web_sys::console::log_1(&format!("{:?}", count).into());

        let min_count: f64 = std::cmp::min(count, 10) as f64;
        // web_sys::console::log_1(&format!("c").into());
        let mut current_chance: f64 =
            base + (min_count * base) * 0.1 + extra_arr.get(count).unwrap_or(&0.0);

        // web_sys::console::log_1(&format!("{:?}", current_chance).into());
        // web_sys::console::log_1(&format!("d").into());
        if artisan >= 1.0 {
            current_chance = 1.0;
            raw_chances.push(current_chance);
            break;
        }
        // web_sys::console::log_1(&format!("e").into());
        raw_chances.push(current_chance);
        count += 1;
        artisan += (46.51_f64 / 100.0) * current_chance * artisan_rate;
        if current_chance == 1.0 {
            break; // for upgrades that have 100% passrate immediately
        }
        // if count > 300 {
        //     panic!();
        // }
    }
    // web_sys::console::log_1(&format!("c").into());
    // convert raw per-try chances into per-tap probability distribution
    let mut chances = vec![0.0_f64; raw_chances.len()];
    let mut cum_chance = 1.0_f64;

    for (idx, &element) in raw_chances.iter().enumerate() {
        chances[idx] = cum_chance * element;
        cum_chance *= 1.0 - element;
    }

    // web_sys::console::log_1(&format!("{:?}", chances).into());
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

    // web_sys::console::log_1(&format!("{:?}", chances).into());
    chances
}
