use crate::constants::{FLOAT_TOL, JuiceInfo, NORMAL_JUICE_COST};
use crate::normal_honing_utils::probability_distribution;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::f64::{INFINITY, NAN, NEG_INFINITY};
use std::ops::{Deref, DerefMut};

use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upgrade {
    pub is_normal_honing: bool,
    pub prob_dist: ProbDist,
    pub base_chance: f64,
    pub costs: [i64; 7],
    pub one_juice_cost: i64,
    pub adv_juice_cost: Vec<f64>, // array corresponding to column 2 in the ADV_DATA * how much per grace
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
    pub extra_chance: f64,
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
#[derive(Default)]
pub struct ProbDist {
    pub payload: Vec<f64>,
    #[serde(skip)]
    pub prob_state_hash: u64,
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
    pub support_state_hash: u64,
    pub state_invariant: bool, // only talking about the support, prob can always change
    pub linear: bool,          // gap between 0 and 1 = gap between n and n+1

    collapsed_pair: Vec<(f64, f64)>, // (support, prob, logged_prob)
    #[serde(skip)]
    pub collapsed_state_hash: u64,
    pub ignore: bool,
    pub gap_size: f64,
    pub max_value: f64,
    pub min_value: f64,
    pub first_non_zero_prob_index: usize,
    skipped_pair: Vec<(f64, f64)>,
}

impl Support {
    pub fn access_collapsed(&self, skipped: bool) -> &Vec<(f64, f64)> {
        assert!(self.collapsed_state_hash == self.support_state_hash);
        if skipped {
            &self.skipped_pair
        } else {
            &self.collapsed_pair
        }
    }

    /// Incredibly crucial pre-processing, collapses identical values into 1 thing, and removes values with p = 0.
    /// cumulant.rs makes the assumption that nothing has p = 0
    pub fn collapse_support(&mut self, prob_dist: &ProbDist, alr_failed: usize) {
        // these hash checks are mostly just for preventing me from doing stupid stuff and saving me from debugging
        // it has the added benefit that we don't update if the state didn't change but is like negligible
        assert!(prob_dist.payload.len() == self.support.len());
        assert!(prob_dist.prob_state_hash == self.support_state_hash);

        if self.collapsed_state_hash != self.support_state_hash {
            let mut result: Vec<(f64, f64)> = Vec::with_capacity(self.support.len());
            let mut max_value: f64 = NEG_INFINITY;
            let mut min_value: f64 = INFINITY;
            let mut iter = self.support.iter().zip(prob_dist.iter()).enumerate();

            if let Some((index, (&s, &p))) = iter.next() {
                let mut cur_s = s;
                let mut cur_p = p;
                if index == alr_failed {
                    self.skipped_pair[0].0 = cur_s;
                }
                for (index, (&new_s, &new_p)) in iter {
                    if index == alr_failed {
                        self.skipped_pair[0].0 = cur_s;
                    }
                    if new_s == cur_s {
                        cur_p += new_p;
                    } else {
                        if cur_p > FLOAT_TOL {
                            max_value = cur_s.max(max_value);
                            min_value = cur_s.min(min_value);
                            result.push((cur_s, cur_p));
                        }

                        cur_s = new_s;
                        cur_p = new_p;
                    }
                }

                // push the final run
                if cur_p > FLOAT_TOL {
                    max_value = cur_s.max(max_value);
                    min_value = cur_s.min(min_value);
                    result.push((cur_s, cur_p));
                }
            }
            self.ignore = result.len() == 1 && result[0].0.abs() < FLOAT_TOL;
            self.max_value = max_value;
            self.min_value = min_value;
            self.first_non_zero_prob_index = 0;
            self.collapsed_pair = result;
            self.collapsed_state_hash = self.support_state_hash;
        }
    }

    pub fn update_payload(
        &mut self,
        new_payload: Vec<f64>,
        new_state_hash: u64,
        prob_dist: &ProbDist,
        gap_size: f64,
        linear: bool, // max: f64,
        alr_failed: usize,
    ) {
        self.support = new_payload;
        self.support_state_hash = new_state_hash;
        self.collapse_support(prob_dist, alr_failed);
        self.gap_size = gap_size;
        self.linear = linear;
    }
}

impl Default for Support {
    fn default() -> Self {
        Self {
            support: vec![],
            support_state_hash: 0,
            state_invariant: false,
            linear: false,
            collapsed_state_hash: 0,
            collapsed_pair: vec![],
            ignore: false,
            gap_size: NAN,
            max_value: NAN,
            min_value: NAN,
            first_non_zero_prob_index: 0,
            skipped_pair: vec![(NAN, 1.0)],
        }
    }
}

impl Upgrade {
    pub fn new_normal(
        base_chance: f64,
        costs: [i64; 7],
        special_cost: i64,
        is_weapon: bool,
        piece_type: usize,
        artisan_rate: f64,
        upgrade_index: usize,
        juice_info: &JuiceInfo,
        alr_failed: usize,
        state_given: Option<Vec<(bool, usize)>>,
        unlocked: bool,
        unlock_costs: Vec<i64>,
        succeeded: bool,
        extra_chance: f64,
    ) -> Self {
        let original_prob_dist_len: usize = probability_distribution(
            base_chance,
            artisan_rate,
            &[],
            0.0,
            0,
            false,
            None,
            extra_chance,
        )
        .len();
        let state_payload = state_given.unwrap_or(State::new(original_prob_dist_len).payload);
        let new_extra: Vec<f64> = state_payload
            .iter()
            .map(|(juice, id)| {
                let mut chance: f64 = 0.0;
                if *juice {
                    chance += juice_info.chances_id[0][upgrade_index];
                }
                if *id > 0 {
                    chance += juice_info.chances_id[*id][upgrade_index];
                }
                chance
            })
            .collect();

        let prob_dist = probability_distribution(
            base_chance,
            artisan_rate,
            &new_extra,
            0.0,
            alr_failed,
            succeeded,
            Some(original_prob_dist_len),
            extra_chance,
        );
        let prob_dist_len: usize = prob_dist.len();
        let mut state = State::new(original_prob_dist_len);
        state.update_payload(state_payload.to_owned());
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
            eqv_gold_per_tap: -1.0_f64,     // dummy value
            juice_avail: upgrade_index > 2, // will overwrite this in prep initialization anyway
            books_avail: -1,                // will overwrite in prep
            state,                          // initialize state with default values
            cost_dist: vec![Support::default(); 7],
            weap_juice_costs: vec![Support::default(); juice_info.num_avail],
            armor_juice_costs: vec![Support::default(); juice_info.num_avail],

            combined_gold_costs: Support::default(),
            name_string: {
                let mut string: String = "".to_owned();
                string += if is_weapon { "weap_" } else { "armor_" };
                string += &upgrade_index.to_string();
                string
            },
            alr_failed,
            unlocked,
            unlock_costs,
            succeeded,
            extra_chance,
        }
    }

    /// we initialize the support of adv here, and don't update it further (because we aren't doing optimizaiton for adv rn), but that will change in the future
    pub fn new_adv(
        prob_dist_vec: Vec<f64>,
        costs: [i64; 7],
        one_juice_cost: i64,
        juice_dist: Vec<f64>,
        is_weapon: bool,
        piece_type: usize,
        upgrade_index: usize,
        unlock_costs: Vec<i64>,
        succeeded: bool,
        num_juice_avail: usize,
        unlocked: bool,
    ) -> Self {
        let prob_dist_len: usize = prob_dist_vec.len();
        assert!(prob_dist_len == juice_dist.len());
        let state: State = State::new(1);

        let mut prob_dist = ProbDist::default();
        prob_dist.update_payload(prob_dist_vec, state.hash);
        let prob_dist_len: usize = prob_dist.len();

        let mut cost_dist = vec![Support::default(); 7];
        for t_index in 0..7 {
            let mut this_mats_costs: Vec<f64> = Vec::with_capacity(prob_dist_len);
            let mut cost_so_far: f64 = if t_index == 3 {
                unlock_costs[0] as f64
            } else if t_index == 6 {
                unlock_costs[1] as f64
            } else {
                0.0
            };
            let this_cost: f64 = costs[t_index] as f64;
            for (index, _p) in prob_dist.iter().enumerate() {
                this_mats_costs.push(cost_so_far);
                if index >= prob_dist_len - 1 {
                    break;
                }
                cost_so_far += this_cost;
            }
            cost_dist[t_index].update_payload(
                this_mats_costs,
                state.hash,
                &prob_dist,
                this_cost,
                true,
                0,
            );
        }
        let mut weap_juice_costs = vec![Support::default(); num_juice_avail];
        let mut armor_juice_costs = vec![Support::default(); num_juice_avail];
        for id in 0..num_juice_avail {
            let mut weap_cost: f64 = 0.0;
            let mut armor_cost: f64 = 0.0;
            let mut weap_support: Vec<f64> = Vec::with_capacity(prob_dist_len);
            let mut armor_support: Vec<f64> = Vec::with_capacity(prob_dist_len);

            for this_juice in juice_dist.iter() {
                if id == 0 {
                    if is_weapon {
                        weap_cost = *this_juice;
                    } else {
                        armor_cost = *this_juice;
                    }
                }
                weap_support.push(weap_cost);
                armor_support.push(armor_cost);
            }
            weap_juice_costs[id].update_payload(
                weap_support,
                state.hash,
                &prob_dist,
                1.0,
                false,
                0,
            );
            armor_juice_costs[id].update_payload(
                armor_support,
                state.hash,
                &prob_dist,
                1.0,
                false,
                0,
            );
        }

        Self {
            is_normal_honing: false,
            prob_dist,
            base_chance: NAN,
            costs,
            one_juice_cost,
            adv_juice_cost: juice_dist,
            special_cost: 0,
            juice_values: vec![],
            prob_dist_len,
            is_weapon,
            piece_type,
            artisan_rate: 0.0,
            tap_offset: 0,
            upgrade_index,
            special_value: -1.0_f64,
            original_prob_dist_len: prob_dist_len,
            eqv_gold_per_tap: -1.0_f64, // dummy value
            juice_avail: true,          // will overwrite this in prep initialization anyway
            books_avail: -1,            // will overwrite in prep
            state, // this is bogus for now, later it'll be a choice between adv hone strategies ig
            cost_dist,
            weap_juice_costs,
            armor_juice_costs,
            combined_gold_costs: Support::default(),
            name_string: {
                let mut string: String = "adv_".to_owned();
                string += if is_weapon { "weap_" } else { "armor_" };
                string += &upgrade_index.to_string();
                string
            },
            alr_failed: 0,
            unlocked,
            unlock_costs,
            succeeded,
            extra_chance: NAN,
        }
    }
}
