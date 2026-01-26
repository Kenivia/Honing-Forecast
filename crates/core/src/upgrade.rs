use crate::constants::{FLOAT_TOL, NORMAL_JUICE_COST};
use crate::normal_honing_utils::{generate_first_deltas, new_prob_dist, probability_distribution};
use crate::parser::PreparationOutput;
use itertools::izip;
use serde::{Deserialize, Serialize};

use std::collections::hash_map::DefaultHasher;

use std::f64::NAN;
use std::ops::{Deref, DerefMut};

use std::hash::{Hash, Hasher};

// the parser function turns a selection of upgrades into an array of Upgrade objects
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upgrade {
    pub is_normal_honing: bool,
    pub prob_dist: ProbDist,
    pub base_chance: f64,
    pub costs: [i64; 7],
    pub one_juice_cost: i64,
    pub adv_juice_cost: Vec<f64>, // array corresponding to column 2 in the ADV_DATA
    pub special_cost: i64,
    pub juice_values: Vec<f64>, // juice values
    pub prob_dist_len: usize,
    pub is_weapon: bool,
    pub artisan_rate: f64,
    pub tap_offset: i64,
    pub upgrade_index: usize,
    pub special_value: f64,
    pub original_prob_dist_len: usize,
    pub eqv_gold_per_tap: f64,
    pub juice_avail: bool,
    pub books_avail: i64,
    // pub juice_arr: Vec<f64>,
    pub state: State, // state for this upgrade - (juice_used? , id) per tap
    pub cost_dist: Vec<Support>,
    pub weap_juice_costs: Vec<Support>,
    pub armor_juice_costs: Vec<Support>,
    pub combined_gold_costs: Support,
    pub name_string: String,
    pub piece_type: usize,
    pub alr_failed: usize,
    pub unlocked: bool,
    pub unlock_costs: Vec<i64>,
    pub succeeded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct State {
    pub payload: Vec<(bool, usize)>,
    #[serde(skip)]
    pub hash: u64,
}
impl State {
    pub fn new(length: usize) -> State {
        let mut out = State {
            payload: vec![(false, 0); length],
            hash: 0,
        };
        out.update_hash();
        out
    }

    pub fn update_hash(&mut self) {
        let mut hasher: DefaultHasher = DefaultHasher::new();
        self.payload.hash(&mut hasher);
        self.hash = hasher.finish();
    }

    pub fn update_payload(&mut self, new_payload: Vec<(bool, usize)>) {
        self.payload = new_payload;
        self.update_hash();
    }
}
impl Deref for State {
    type Target = Vec<(bool, usize)>;
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProbDist {
    pub payload: Vec<f64>,
    #[serde(skip)]
    pub prob_state_hash: u64,
    // logged_payload: Vec<f64>, // force access through the function to do the check
    // pub log_state_hash: u64,
}

impl Default for ProbDist {
    fn default() -> Self {
        Self {
            payload: vec![],
            prob_state_hash: 0,
            // logged_payload: vec![],
            // log_state_hash: 0,
        }
    }
}

impl ProbDist {
    pub fn new(new_payload: Vec<f64>) -> ProbDist {
        ProbDist {
            payload: new_payload,
            prob_state_hash: 0,
            // logged_payload: vec![],
            // log_state_hash: 0,
        }
    }
    pub fn update_payload(&mut self, new_payload: Vec<f64>, new_state_hash: u64) {
        self.payload = new_payload;
        self.prob_state_hash = new_state_hash;
    }
    // pub fn logged_iter(&self) -> impl Iterator<Item = &f64> {
    //     assert!(self.log_state_hash == self.prob_state_hash);
    //     self.logged_payload.iter()
    // }

    // pub fn log_prob_dist(&self) -> &Vec<f64> {
    //     assert!(self.log_state_hash == self.prob_state_hash);
    //     &self.logged_payload
    // }

    // pub fn compute_log(&mut self) {
    //     if self.log_state_hash != self.prob_state_hash {
    //         self.logged_payload = self.payload.iter().map(|x| x.ln()).collect();
    //         self.log_state_hash = self.prob_state_hash;
    //     }
    // }
    // pub fn is_log_valid(&self) -> bool {
    //     self.log_state_hash == self.prob_state_hash
    // }
}

impl Deref for ProbDist {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Support {
    pub support: Vec<f64>,
    #[serde(skip)]
    pub support_state_hash: u64, // i mean even this i dont think is that necessary but whatever
    pub state_invariant: bool, // only talking about the support, prob can always change
    pub linear: bool,          // gap between 0 and 1 = gap between n and n+1

    collapsed_pair: Vec<(f64, f64)>, // (support, prob, logged_prob)
    #[serde(skip)]
    pub collapsed_state_hash: u64,
    pub ignore: bool,
    pub gap_size: f64,
    pub max_value: f64,
    pub first_non_zero_prob_index: usize,
}

impl Support {
    pub fn access_collapsed(&self) -> &Vec<(f64, f64)> {
        assert!(self.collapsed_state_hash == self.support_state_hash);
        &self.collapsed_pair
    }

    pub fn collapse_support(&mut self, prob_dist: &ProbDist) {
        // assert!(prob_dist.payload.len() == self.support.len());
        assert!(prob_dist.prob_state_hash == self.support_state_hash);

        // let valid_log = prob_dist.is_log_valid();
        let mut result: Vec<(f64, f64)> = Vec::with_capacity(self.support.len());

        if self.collapsed_state_hash != self.support_state_hash || true {
            let mut iter = self.support.iter().zip(prob_dist.iter());

            if let Some((&s, &p)) = iter.next() {
                let mut cur_s = s;
                let mut cur_p = p;
                for (&new_s, &new_p) in iter {
                    if new_s == cur_s {
                        cur_p += new_p;
                    } else {
                        if cur_p > FLOAT_TOL {
                            result.push((cur_s, cur_p));
                        }

                        cur_s = new_s;
                        cur_p = new_p;
                    }
                }

                // push the final run
                result.push((cur_s, cur_p));
            }
            self.ignore = result.len() == 1 && result[0].0.abs() < FLOAT_TOL;
            // self.first_non_zero_prob_index = result
            //     .iter()
            //     .take_while(|(_, p)| p.abs() < FLOAT_TOL)
            //     .count();
            self.first_non_zero_prob_index = 0;
            self.collapsed_pair = result;
            self.collapsed_state_hash = self.support_state_hash;
        }
    }

    // pub fn new(
    //     payload: Vec<f64>,
    //     state_hash: u64,
    //     state_invariant: bool,
    //     linear: bool,
    //     gap_size: f64,
    // ) -> Self {
    //     Self {
    //         support: payload.clone(),
    //         support_state_hash: state_hash,
    //         state_invariant,
    //         linear: linear,
    //         collapsed_pair: payload
    //             .iter()
    //             .map(|&x| (x, NAN))
    //             .collect::<Vec<(f64, f64)>>(),
    //         collapsed_state_hash: 0,
    //         ignore: false,
    //         gap_size,
    //     }
    // }

    pub fn update_payload(
        &mut self,
        new_payload: Vec<f64>,
        new_state_hash: u64,
        prob_dist: &ProbDist,
        gap_size: f64,
        max: f64,
    ) {
        self.support = new_payload;
        self.support_state_hash = new_state_hash;
        self.collapse_support(prob_dist);
        self.gap_size = gap_size;
        self.max_value = max;
    }
}

impl Default for Support {
    fn default() -> Self {
        Self {
            support: vec![],
            support_state_hash: 0,
            state_invariant: false,
            // initialized: false,
            linear: false,
            collapsed_state_hash: 0,
            collapsed_pair: vec![],
            ignore: false,
            gap_size: NAN,
            max_value: NAN,
            first_non_zero_prob_index: 0,
        }
    }
}

impl Deref for Support {
    type Target = Vec<f64>;

    fn deref(&self) -> &Self::Target {
        &self.support
    }
}

impl Upgrade {
    pub fn new_normal(
        base_chance: f64,
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        special_cost: i64,
        is_weapon: bool,
        piece_type: usize,
        artisan_rate: f64,
        upgrade_index: usize,
        num_juice_avail: usize,
        alr_failed: usize,
        state_given: Option<Vec<(bool, usize)>>,
        unlocked: bool,
        unlock_costs: Vec<i64>,
        succeeded: bool,
    ) -> Self {
        let prob_dist_len: usize = prob_dist.len();
        // let base_chance: f64 = prob_dist[1];
        // web_sys::console::log_1(&format!("{:?}", alr_failed).into());
        let original_prob_dist_len: usize =
            probability_distribution(base_chance, artisan_rate, &vec![], 0.0, 0, false).len();
        // web_sys::console::log_1(&format!("a").into());
        let mut state = State::new(prob_dist_len);
        // web_sys::console::log_1(&format!("a").into());
        state.update_payload(
            state_given
                .unwrap_or(State::new(original_prob_dist_len).payload)
                .to_owned(),
        );
        // web_sys::console::log_1(&format!("{:?}", state).into());
        Self {
            is_normal_honing: true,
            prob_dist: ProbDist::new(prob_dist),
            base_chance,
            costs,
            one_juice_cost: NORMAL_JUICE_COST[upgrade_index],
            adv_juice_cost: vec![],
            special_cost,
            juice_values: vec![],
            prob_dist_len,
            is_weapon,
            piece_type,
            artisan_rate,
            tap_offset: 0,
            upgrade_index,
            special_value: -1.0_f64,
            original_prob_dist_len,

            // log_prob_dist: vec![], // will change with each arrangement, maybe use a hashmap later
            eqv_gold_per_tap: -1.0_f64, // dummy value
            // gold_cost_record: vec![],
            // juice_arr: vec![],
            // eqv_gold_per_juice: -1.0_f64,
            juice_avail: upgrade_index > 2, // will overwrite this in prep initialization anyway
            books_avail: -1,                // will overwrite in prep

            state, // initialize state with default values
            cost_dist: vec![Support::default(); 7],
            weap_juice_costs: vec![Support::default(); num_juice_avail],
            armor_juice_costs: vec![Support::default(); num_juice_avail],

            combined_gold_costs: Support::default(),
            name_string: {
                let mut string: String = "".to_owned();
                string += if is_weapon { "weap_" } else { "armor_" };
                string += &upgrade_index.to_string();
                string
            },
            alr_failed: alr_failed,
            unlocked,
            unlock_costs,
            succeeded,
        }
    }

    pub fn new_adv(
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        one_juice_cost: i64,
        adv_juice_cost: Vec<f64>,
        is_weapon: bool,
        piece_type: usize,
        adv_cost_start: i64,
        upgrade_index: usize,
        unlock_costs: Vec<i64>,
        succeeded: bool,
    ) -> Self {
        let prob_dist_len: usize = prob_dist.len();
        assert!(prob_dist_len == adv_juice_cost.len());

        Self {
            is_normal_honing: false,
            prob_dist: ProbDist::default(),
            base_chance: 0.0,
            costs,
            one_juice_cost,
            adv_juice_cost,
            special_cost: 0,
            juice_values: vec![],
            prob_dist_len,
            is_weapon,
            piece_type,
            artisan_rate: 0.0,
            tap_offset: adv_cost_start,
            upgrade_index,
            special_value: -1.0_f64,
            original_prob_dist_len: 1, // need to sort this out

            // log_prob_dist: vec![], // will change with each arrangement, maybe use a hashmap later
            eqv_gold_per_tap: -1.0_f64, // dummy value
            // gold_cost_record: vec![],
            // juice_arr: vec![],
            // eqv_gold_per_juice: -1.0_f64,
            // failure_raw_delta: -1,
            // failure_delta_order: -1,
            juice_avail: upgrade_index > 2, // will overwrite this in prep initialization anyway
            books_avail: -1,                // will overwrite in prep

            state: State::new(prob_dist_len), // this is bogus for now, later it'll be a choice between adv hone strategy ig
            cost_dist: vec![],
            weap_juice_costs: vec![],
            armor_juice_costs: vec![],

            combined_gold_costs: Support::default(),
            name_string: {
                let mut string: String = "adv_".to_owned();
                string += if is_weapon { "weap_" } else { "armor_" };
                string += &upgrade_index.to_string();
                string
            },
            alr_failed: 0,
            unlocked: false,
            unlock_costs,
            succeeded,
        }
    }

    pub fn update_this_individual_support(&mut self, prep_output: &PreparationOutput) {
        let l_len: usize = self.prob_dist.len();

        for t_index in 0..7 {
            let mut this_mats_costs: Vec<f64> = Vec::with_capacity(l_len);
            let mut cost_so_far: f64 = 0.0;
            let this_cost: f64 = self.costs[t_index] as f64;
            for (index, _p) in self.prob_dist.iter().enumerate() {
                this_mats_costs.push(cost_so_far);

                if index >= l_len - 1 {
                    break;
                }

                cost_so_far += this_cost;
            }

            self.cost_dist[t_index].update_payload(
                this_mats_costs,
                self.state.hash,
                &self.prob_dist,
                this_cost,
                cost_so_far,
            );
        }

        for id in 0..prep_output.juice_info.num_avail {
            let mut weap_cost: f64 = 0.0;
            let mut armor_cost: f64 = 0.0;
            let mut weap_support: Vec<f64> = Vec::with_capacity(l_len);
            let mut armor_support: Vec<f64> = Vec::with_capacity(l_len);
            let amt = prep_output.juice_info.amt_used_id[id][self.upgrade_index] as f64;
            for (index, (juice, book)) in self.state.iter().take(l_len).enumerate() {
                weap_support.push(weap_cost);
                armor_support.push(armor_cost);
                if index >= l_len - 1 {
                    break;
                }
                if *juice && id == 0 {
                    if self.is_weapon {
                        weap_cost += amt;
                    } else {
                        armor_cost += amt;
                    }
                }
                if *book == id && id > 0 {
                    if self.is_weapon {
                        weap_cost += amt;
                    } else {
                        armor_cost += amt;
                    }
                }
            }
            self.weap_juice_costs[id].update_payload(
                weap_support,
                self.state.hash,
                &self.prob_dist,
                amt,
                weap_cost,
            );

            self.armor_juice_costs[id].update_payload(
                armor_support,
                self.state.hash,
                &self.prob_dist,
                amt,
                armor_cost,
            );
        }
    }

    pub fn update_this_prob_dist(&mut self, prep_output: &PreparationOutput) {
        let prob_dist: Vec<f64> = new_prob_dist(&self.state, &prep_output.juice_info, self, 0.0);

        self.prob_dist.update_payload(prob_dist, self.state.hash);
    }
}
