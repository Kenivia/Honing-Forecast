use std::collections::HashMap;

use itertools::Itertools;

use crate::constants::FLOAT_TOL;
use crate::helpers::F64_2d;
use crate::normal_honing_utils::{add_juice_gold_cost, new_prob_dist};
use crate::parser::PreparationOutput;
use crate::performance::Performance;
use crate::saddlepoint_approximation::success_prob::honing_sa_wrapper;
#[derive(Clone, Debug)]
pub struct StateBundle {
    pub names: Vec<String>,

    // the above entries are tied to each upgrade, so arr[upgrade_index] correspond to the appropriate info for a particular upgrade
    // probably should make another struct for this at some point
    pub special_state: Vec<usize>, // arbitrary length
    pub metric: f64,
    pub state_index: Vec<Vec<Vec<i64>>>, // i pre-added this for caching but havnt implemented anything
    pub prep_output: PreparationOutput,
    pub special_cache: HashMap<Vec<usize>, Vec<f64>>,
}
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

pub fn find_non_zero<'a, I>(support_arr: I, log_prob_dist_arr: I, biased: bool) -> f64
where
    I: F64_2d<'a>,
{
    support_arr
        .into_iter()
        .zip(log_prob_dist_arr)
        .map(|(support, log_prob_dist)| {
            support
                .iter()
                .zip(log_prob_dist)
                .find(|(support, lp)| {
                    if biased {
                        **lp > f64::NEG_INFINITY && support.abs() > FLOAT_TOL
                    } else {
                        **lp > f64::NEG_INFINITY
                    }
                })
                .unwrap_or((&0.0, &0.0))
                .0
        })
        .sum()
}

impl StateBundle {
    pub fn new(prep_output: PreparationOutput) -> StateBundle {
        let mut starting_special: Vec<usize> =
            Vec::with_capacity(prep_output.upgrade_arr.len() * 2);
        for (index, _upgrade) in prep_output.upgrade_arr.iter().enumerate() {
            starting_special.push(index); //, (1.0 / upgrade.base_chance).round() as usize));
        }

        let state_bundle: StateBundle = StateBundle {
            names: prep_output
                .upgrade_arr
                .iter()
                .map(|x| {
                    let mut string: String = if x.is_normal_honing {
                        "".to_owned()
                    } else {
                        "adv_".to_owned()
                    };
                    string += if x.is_weapon { "weap_" } else { "armor_" };
                    string += &x.upgrade_index.to_string();
                    string
                })
                .collect::<Vec<String>>(),
            state_index: vec![],
            metric: -1.0,
            special_state: starting_special,
            prep_output,
            special_cache: HashMap::new(),
        };
        return state_bundle;
    }
    pub fn find_min_max(&self, support_index: i64, skip_count: usize, biased: bool) -> (f64, f64) {
        let min_value = find_non_zero(
            self.extract_support(support_index, skip_count),
            self.extract_log_prob(skip_count),
            biased,
        );
        let max_value = self
            .extract_support(support_index, skip_count)
            .into_iter()
            .map(|x| x.last().unwrap())
            .sum();
        (min_value, max_value)
    }

    pub fn encode_all(&self) -> String {
        let mut strings = Vec::new();
        strings.push(format!("{:?}", self.special_state));
        for (index, upgrade) in self.prep_output.upgrade_arr.iter().enumerate() {
            strings.push(self.names[index].clone() + ": " + &encode_one_positions(&upgrade.state));
        }
        strings.join("\n")
    }

    pub fn extract_log_prob(&self, skip_count: usize) -> Box<dyn Iterator<Item = &Vec<f64>> + '_> {
        Box::new(
            self.special_state
                .iter()
                .map(move |x| &self.prep_output.upgrade_arr[*x].log_prob_dist)
                .skip(skip_count),
        )
    }

    pub fn extract_prob(&self, skip_count: usize) -> Box<dyn Iterator<Item = &Vec<f64>> + '_> {
        Box::new(
            self.special_state
                .iter()
                .map(move |x| &self.prep_output.upgrade_arr[*x].prob_dist)
                .skip(skip_count),
        )
    }

    pub fn extract_support(
        &self,
        support_index: i64,
        skip_count: usize,
    ) -> Box<dyn Iterator<Item = &Vec<f64>> + '_> {
        let num_avail = self.prep_output.juice_info.num_avail;
        Box::new(
            self.special_state
                .iter()
                .map(move |&u_index| {
                    let upgrade = &self.prep_output.upgrade_arr[u_index];
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

    pub fn update_combined(&mut self) {
        let prep_output = &mut self.prep_output;

        for upgrade in prep_output.upgrade_arr.iter_mut() {
            let mut this_combined = Vec::with_capacity(upgrade.prob_dist.len());
            let mut cost_so_far: f64 = 0.0;
            for (p_index, _) in upgrade.log_prob_dist.iter().enumerate() {
                this_combined.push(cost_so_far);
                cost_so_far += upgrade.eqv_gold_per_tap;
                let (juice, book_index) = upgrade.state[p_index];
                if juice {
                    add_juice_gold_cost(&prep_output.juice_info, upgrade, &mut cost_so_far, 0);
                }
                if book_index > 0 {
                    add_juice_gold_cost(
                        &prep_output.juice_info,
                        upgrade,
                        &mut cost_so_far,
                        book_index,
                    );
                }
            }
            upgrade.combined_gold_costs = this_combined;
        }
    }

    pub fn update_individual_support(&mut self) {
        let prep_output = &mut self.prep_output;

        let j_len: usize = prep_output.juice_info.num_avail;

        for upgrade in prep_output.upgrade_arr.iter_mut() {
            let l_len: usize = upgrade.log_prob_dist.len();
            let mut this_mats_costs: Vec<Vec<f64>> = vec![Vec::with_capacity(l_len); 7];
            let mut this_weap_juices_costs: Vec<Vec<f64>> = vec![Vec::with_capacity(l_len); j_len];
            let mut this_armor_juices_costs: Vec<Vec<f64>> = vec![Vec::with_capacity(l_len); j_len];

            for t_index in 0..7 {
                let mut cost_so_far = 0.0;
                for _ in upgrade.prob_dist.iter() {
                    this_mats_costs[t_index].push(cost_so_far);
                    // means[t_index] += cost_so_far * p;
                    cost_so_far += upgrade.costs[t_index] as f64;
                }

                // mats_costs[t_index][u_index].push(cost_so_far);
            }
            upgrade.cost_dist = this_mats_costs;

            // ts so weird but idk if theres a better way, i think i just designed this special state poorly maybe
            for id_to_match in 0..j_len {
                let mut this_weap: Vec<f64> = Vec::with_capacity(l_len);
                let mut this_armor: Vec<f64> = Vec::with_capacity(l_len);
                for (bit_index, _) in prep_output.juice_info.gold_costs[upgrade.upgrade_index]
                    .iter()
                    .enumerate()
                {
                    let id: usize = prep_output.juice_info.ids[upgrade.upgrade_index][bit_index];
                    if id_to_match != id {
                        continue;
                    }
                    let mut costs_so_far: (f64, f64) = (0.0, 0.0);

                    for (p_index, _) in upgrade.prob_dist.iter().enumerate() {
                        this_weap.push(costs_so_far.0);
                        this_armor.push(costs_so_far.1);
                        // means[7 + id] += costs_so_far.0 * p;
                        // means[7 + j_len + id] += costs_so_far.1 * p;
                        let (juice, book_index) = upgrade.state[p_index];
                        if juice {
                            if upgrade.is_weapon {
                                costs_so_far.0 += prep_output.juice_info.amt_used
                                    [upgrade.upgrade_index][0]
                                    as f64;
                            } else {
                                costs_so_far.1 += prep_output.juice_info.amt_used
                                    [upgrade.upgrade_index][0]
                                    as f64;
                            }
                        }
                        if book_index > 0 {
                            if upgrade.is_weapon {
                                costs_so_far.0 += prep_output.juice_info.amt_used
                                    [upgrade.upgrade_index][book_index]
                                    as f64;
                            } else {
                                costs_so_far.1 += prep_output.juice_info.amt_used
                                    [upgrade.upgrade_index][book_index]
                                    as f64;
                            }
                        }
                    }

                    break;
                }
                if this_armor.len() > 0 {
                    this_weap_juices_costs[id_to_match] = this_weap;
                    this_armor_juices_costs[id_to_match] = this_armor;
                } else {
                    this_weap_juices_costs[id_to_match] = vec![0.0; l_len];
                    this_armor_juices_costs[id_to_match] = vec![0.0; l_len];
                }
            }
            upgrade.weap_juice_costs = this_weap_juices_costs;
            upgrade.armor_juice_costs = this_armor_juices_costs;

            // upgrade.biased_prob_dist = upgrade
            //     .cost_dist
            //     .iter()
            //     .chain(upgrade.weap_juice_costs.iter())
            //     .chain(upgrade.armor_juice_costs.iter())
            //     .enumerate()
            //     .map(|(index, s_arr)| {
            //         if means[index].abs() < FLOAT_TOL {
            //             vec![1.0 / upgrade.prob_dist.len() as f64; upgrade.prob_dist.len()]
            //             // i mean we should just change this to [1.0] but then i have to change the support and prob dist and everything so that's a TODO for me
            //         } else {
            //             s_arr
            //                 .iter()
            //                 .zip(upgrade.prob_dist.clone())
            //                 .map(|(s, p)| s * p / means[index])
            //                 .collect()
            //         }
            //     })
            //     .collect();
            // upgrade.biased_log_prob_dist = upgrade
            //     .biased_prob_dist
            //     .iter()
            //     .map(|x| x.iter().map(|y| y.ln()).collect())
            //     .collect();
        }
    }

    pub fn compute_leftover_probs(&mut self) -> Vec<f64> {
        self.update_dist();
        self.update_individual_support();

        let mut prob_leftover: Vec<f64> =
            Vec::with_capacity(self.flattened_effective_budgets().try_len().unwrap());
        let mut dummy_performance = Performance::new();

        for (support_index, effective_budget) in self.flattened_effective_budgets().enumerate() {
            prob_leftover.push(honing_sa_wrapper(
                self,
                support_index as i64,
                effective_budget,
                &mut dummy_performance,
            ));
        }

        prob_leftover
    }

    pub fn update_dist(&mut self) {
        // TODO add a toggle for computing log or not
        // dbg!(&prep_output, &state_bundle);
        // let zero_probs: Vec<f64> = special_probs(prep_output, state_bundle);
        // dbg!(&zero_probs);
        for upgrade in self.prep_output.upgrade_arr.iter_mut() {
            let prob_dist: Vec<f64> =
                new_prob_dist(&upgrade.state, &self.prep_output.juice_info, upgrade, 0.0);
            // let biasted_prob_dist = prob_dist.iter().enumerate().map(|(index, x)| x * )
            let log_prob_dist: Vec<f64> = prob_dist.iter().map(|x| x.ln()).collect();
            upgrade.prob_dist = prob_dist;
            upgrade.log_prob_dist = log_prob_dist;

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

    pub fn gather_prob_dist(&self) -> Vec<Vec<f64>> {
        let mut arr = Vec::with_capacity(self.prep_output.upgrade_arr.len());

        for upgrade in self.prep_output.upgrade_arr.iter() {
            arr.push(upgrade.prob_dist.clone());
        }
        arr
    }
    pub fn gather_log_prob_dist(&self) -> Vec<Vec<f64>> {
        let mut arr = Vec::with_capacity(self.prep_output.upgrade_arr.len());
        for upgrade in self.prep_output.upgrade_arr.iter() {
            arr.push(upgrade.log_prob_dist.clone());
        }
        arr
    }

    pub fn gather_combined_gold_cost(&self) -> Vec<Vec<f64>> {
        let mut arr = Vec::with_capacity(self.prep_output.upgrade_arr.len());
        for upgrade in self.prep_output.upgrade_arr.iter() {
            arr.push(upgrade.combined_gold_costs.clone());
        }
        arr
    }
}
