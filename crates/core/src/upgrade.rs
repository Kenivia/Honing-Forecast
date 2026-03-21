use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use crate::constants::juice_info::JuiceInfo;
use crate::support::{ProbDist, Support};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::ops::{Deref, DerefMut};

use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upgrade {
    pub is_normal_honing: bool,
    pub normal_dist: ProbDist,
    pub base_chance: f64,
    pub costs: Vec<f64>,

    pub special_cost: i64,
    pub is_weapon: bool,
    pub artisan_rate: f64,

    pub upgrade_index: usize,

    pub clean_prob_dist_len: usize,
    // pub juice_arr: Vec<f64>,
    pub state: State, // state for this upgrade - (juice_used? , id) per tap
    pub cost_dist: Vec<Support>,

    pub name_string: String,
    pub piece_type: usize,
    pub alr_failed: usize,
    pub unlocked: bool,
    pub unlock_costs: [f64; 7],
    pub succeeded: bool,
    pub extra_chance: f64,

    pub adv_config: AdvConfig,
    pub adv_dists: Vec<ProbDist>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct State {
    pub payload: Vec<(bool, usize)>,
    #[serde(skip)]
    pub hash: u64,
}

impl State {
    pub fn new_empty(length: usize) -> State {
        let mut out = State {
            payload: vec![(false, 0); length],
            hash: 0,
        };
        out.update_hash();
        out
    }
    pub fn new(payload: Vec<(bool, usize)>) -> State {
        let mut out = State { payload, hash: 0 };
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

impl Upgrade {
    pub fn new_normal(
        base_chance: f64,
        costs: &[f64],
        special_cost: i64,
        is_weapon: bool,
        piece_type: usize,
        artisan_rate: f64,
        upgrade_index: usize,
        juice_info: &JuiceInfo,
        alr_failed: usize,
        state_given: Vec<(bool, usize)>,
        unlocked: bool,
        unlock_costs: &[f64],
        succeeded: bool,
        extra_chance: f64,
    ) -> Self {
        let state = State::new(state_given);

        let mut out = Self {
            is_normal_honing: true,
            normal_dist: ProbDist::default(),
            base_chance,
            costs: costs.try_into().unwrap(),
            special_cost,
            clean_prob_dist_len: 0,
            is_weapon,
            piece_type,
            artisan_rate,
            upgrade_index,
            state, // initialize state with default values
            cost_dist: vec![Support::default(); juice_info.total_num_avail],
            // weap_juice_costs: vec![Support::default(); juice_info.num_juice_avail],
            // armor_juice_costs: vec![Support::default(); juice_info.num_juice_avail],
            name_string: {
                let mut string: String = "".to_owned();
                string += if is_weapon { "weap_" } else { "armor_" };
                string += &upgrade_index.to_string();
                string
            },
            alr_failed,
            unlocked, // THIS IS IGNORED RN just assuming alr_failed > 0 <==> ulocked
            unlock_costs: unlock_costs.try_into().unwrap(),
            succeeded,
            extra_chance,
            adv_config: AdvConfig::default(),
            adv_dists: Vec::new(),
        };

        let mut clean_upgrade = out.clone();
        clean_upgrade.state = State::new_empty(0);
        clean_upgrade.update_dist_normal(juice_info);

        out.clean_prob_dist_len = clean_upgrade.normal_dist.len();
        while out.state.len() < out.clean_prob_dist_len {
            out.state.payload.push((false, 0));
        }
        out.state.update_hash();

        out.update_dist_normal(juice_info);
        out.update_support_normal(juice_info);
        out
    }

    /// we initialize the support of adv here, and don't update it further (because we aren't doing optimizaiton for adv rn), but that will change in the future
    pub fn new_adv(
        costs: &[f64],
        is_weapon: bool,
        piece_type: usize,
        upgrade_index: usize,
        unlock_costs: &[f64],
        succeeded: bool,
        unlocked: bool,
        (start_xp, start_balls, next_free, next_big): (usize, usize, bool, bool),
        double_balls: bool,
        juice_info: &JuiceInfo,
        adv_cache: &mut AHashMap<AdvConfig, AdvDistTriplet>,
        state_given: Vec<(bool, usize)>,
    ) -> Self {
        let state = if state_given.len() == juice_info.adv_uindex_to_id[upgrade_index].len() {
            State::new(state_given)
        } else {
            State::new_empty(juice_info.adv_uindex_to_id[upgrade_index].len())
        };

        let mut out = Self {
            is_normal_honing: false,
            normal_dist: ProbDist::new(Vec::new()),
            base_chance: 0.0,
            costs: costs.try_into().unwrap(),
            special_cost: 0,
            is_weapon,
            piece_type,
            artisan_rate: 0.0,
            upgrade_index,
            clean_prob_dist_len: 0,
            state,
            cost_dist: vec![Support::default(); juice_info.total_num_avail],

            name_string: {
                let mut string: String = "adv_".to_owned();
                string += if is_weapon { "weap_" } else { "armor_" };
                string += &upgrade_index.to_string();
                string
            },
            alr_failed: 0,
            unlocked,
            unlock_costs: unlock_costs.try_into().unwrap(),
            succeeded,
            extra_chance: 0.0,
            adv_config: AdvConfig::new(
                start_xp,
                start_balls,
                next_free,
                next_big,
                double_balls && upgrade_index < 2, // i supposed this needs to be part of the json also TODO
                upgrade_index >= 2,
            ),
            adv_dists: vec![ProbDist::default(); 3],
        };
        out.update_dist_adv(adv_cache);
        out.update_support_adv(juice_info);
        out
    }
}
