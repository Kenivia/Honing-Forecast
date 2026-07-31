use crate::advanced_honing::utils::{AdvConfig, AdvDistTriplet};

use crate::constants::juice_info::JuiceInfo;
use crate::state::{OneState, State};
use crate::support::{ProbDist, Support};
use crate::upgrade::PieceType::{Armor, Vambrace, Weapon};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;

use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy, Eq, Hash)]
pub enum PieceType {
    Armor,
    Weapon,
    Vambrace,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upgrade {
    pub is_normal_honing: bool,
    pub normal_dist: ProbDist,
    pub base_chance: f64,
    pub costs: Vec<f64>,

    pub special_cost: i64,
    pub artisan_rate: f64,

    pub upgrade_index: usize,

    pub clean_prob_dist_len: usize,
    // pub juice_arr: Vec<f64>,
    pub state: State,
    pub cost_dist: Vec<Support>,

    pub name_string: String,
    pub piece_index: usize,
    pub piece_type: PieceType,
    pub piece_type_usize: usize,

    pub starting_artisan: f64,
    pub starting_num_taps: usize, // this is only used to calculate the starting base chance, not cost (we don't distinguish between manual artisan input vs slider artisan input on the rust side, cost calculation happens in js)
    pub unlocked: bool,
    pub unlock_costs: [f64; 7],

    pub extra_chance: f64,

    pub adv_config: AdvConfig,
    pub adv_dists: Vec<ProbDist>,
}

pub fn piece_type_to_prefix(piece_type: PieceType) -> String {
    match piece_type {
        Armor => "armor_".to_string(),
        Weapon => "weap_".to_string(),
        Vambrace => "vamb_".to_string(),
    }
}
pub fn piece_index_to_type(piece_index: usize) -> PieceType {
    return match piece_index {
        0..=4 => Armor,
        5 => Weapon,
        6 => Vambrace,
        _ => panic!("invalid piece type"),
    };
}

pub fn piece_type_to_usize(piece_type: PieceType) -> usize {
    match piece_type {
        Armor => 0,
        Weapon => 1,
        Vambrace => 2,
    }
}

impl Upgrade {
    pub fn new_normal(
        base_chance: f64,
        costs: &[f64],
        special_cost: i64,
        piece_index: usize,
        artisan_rate: f64,
        upgrade_index: usize,
        juice_info: &JuiceInfo,
        starting_artisan: f64,
        starting_num_taps: usize,
        state_given: Vec<OneState>,
        unlocked: bool,
        unlock_costs: &[f64],

        extra_chance: f64,
    ) -> Self {
        let state: State = State::new(state_given);
        let piece_type: PieceType = piece_index_to_type(piece_index);
        let mut out = Self {
            is_normal_honing: true,
            normal_dist: ProbDist::default(),
            base_chance,
            costs: costs.try_into().unwrap(),
            special_cost,
            clean_prob_dist_len: 0,
            piece_index,
            piece_type,
            piece_type_usize: piece_type_to_usize(piece_type),
            artisan_rate,
            upgrade_index,
            state, // initialize state with default values
            cost_dist: vec![Support::default(); juice_info.total_num_avail],
            // weap_juice_costs: vec![Support::default(); juice_info.num_juice_avail],
            // armor_juice_costs: vec![Support::default(); juice_info.num_juice_avail],
            name_string: {
                let mut string: String = "".to_owned();
                string += &piece_type_to_prefix(piece_type);
                string += &upgrade_index.to_string();
                string
            },
            starting_artisan,
            starting_num_taps,
            unlocked, // THIS IS IGNORED RN just assuming alr_failed > 0 <==> ulocked
            unlock_costs: unlock_costs.try_into().unwrap(),

            extra_chance,
            adv_config: AdvConfig::default(),
            adv_dists: Vec::new(),
        };

        let mut clean_upgrade = out.clone();
        clean_upgrade.state = State::new_empty(0);
        clean_upgrade.update_dist_normal(juice_info);

        out.clean_prob_dist_len = clean_upgrade.normal_dist.len();
        while out.state.len() < out.clean_prob_dist_len {
            out.state.payload.push(smallvec![]);
        }
        out.state.truncate(out.clean_prob_dist_len);
        out.state.update_hash();

        out.update_dist_normal(juice_info);
        out.update_support_normal(juice_info);
        out
    }

    /// we initialize the support of adv here, and don't update it further (because we aren't doing optimizaiton for adv rn), but that will change in the future
    pub fn new_adv(
        costs: &[f64],
        piece_index: usize,
        upgrade_index: usize,
        unlock_costs: &[f64],

        unlocked: bool,
        (start_xp, start_balls, next_free, next_big): (usize, usize, bool, bool),
        double_balls: bool,
        juice_info: &JuiceInfo,
        adv_cache: &mut AHashMap<AdvConfig, AdvDistTriplet>,
        state_given: Vec<OneState>,
    ) -> Self {
        let state = if state_given.len() == juice_info.adv_uindex_to_id[upgrade_index].len() {
            State::new(state_given)
        } else {
            State::new_empty(juice_info.adv_uindex_to_id[upgrade_index].len())
        };
        let piece_type: PieceType = piece_index_to_type(piece_index);
        let mut out = Self {
            is_normal_honing: false,
            normal_dist: ProbDist::new(Vec::new()),
            base_chance: 0.0,
            costs: costs.try_into().unwrap(),
            special_cost: 0,
            piece_index,
            piece_type,
            piece_type_usize: piece_type_to_usize(piece_type),
            artisan_rate: 0.0,
            upgrade_index,
            clean_prob_dist_len: 0,
            state,
            cost_dist: vec![Support::default(); juice_info.total_num_avail],

            name_string: {
                let mut string: String = "adv_".to_owned();
                string += &piece_type_to_prefix(piece_type);
                string += &upgrade_index.to_string();
                string
            },
            starting_artisan: 0.0,
            starting_num_taps: 0,
            unlocked,
            unlock_costs: unlock_costs.try_into().unwrap(),

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
