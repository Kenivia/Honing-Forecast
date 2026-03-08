use serde::{Deserialize, Serialize};

use crate::upgrade::Upgrade;

pub const GRACE_FIRST_N: [usize; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255];
pub const NON_GRACE_FIRST_N: [usize; 15] = [1, 2, 3, 4, 5, 8, 10, 12, 15, 20, 30, 40, 50, 60, 255];
// there's an overlap so (255,0) gets included in grace_first_n)

pub const MAX_ADV_STATE: usize = GRACE_FIRST_N.len() + NON_GRACE_FIRST_N.len() - 1;

/// theres only 31 possible options so this outpus 0 to 30
pub fn tuple_to_index((a, b): (usize, usize)) -> usize {
    if a < 255 {
        return a;
    } else if b == 0 {
        return 15;
    }
    {
        return 16 + NON_GRACE_FIRST_N.iter().position(|x| *x == b).unwrap();
    }
}
pub fn index_to_tuple(index: usize) -> (usize, usize) {
    if index <= 15 {
        // Indices 0–15: a = index, b = 0
        (GRACE_FIRST_N[index], 0)
    } else {
        // Indices 16–30: a = 255, b = NON_GRACE_FIRST_N[index - 16]
        (255, NON_GRACE_FIRST_N[index - 16])
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Copy)]
pub struct InvariantAdvConfig {
    pub double_balls: bool,
    pub is_30_40: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Copy, Default)]
pub struct AdvConfig {
    pub start_xp: usize,
    pub start_balls: usize,
    pub next_free: bool,
    pub next_big: bool,

    pub double_balls: bool,
    pub is_30_40: bool,

    pub grace_juice_target: u8,
    pub non_grace_juice_target: u8,
    pub grace_scroll_target: u8,
    pub non_grace_scroll_target: u8,
}
impl AdvConfig {
    pub fn new(
        start_xp: usize,
        start_balls: usize,
        next_free: bool,
        next_big: bool,
        double_balls: bool,
        is_30_40: bool,
    ) -> Self {
        Self {
            start_xp,
            start_balls,
            next_free,
            next_big,
            double_balls,
            is_30_40,
            grace_juice_target: 0,
            non_grace_juice_target: 0,
            grace_scroll_target: 0,
            non_grace_scroll_target: 0,
        }
    }
}
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct SmallAdvState {
    pub cur_xp: u8,
    pub cur_balls: u8,
    pub next_free: bool,
    pub next_big: bool,
    pub non_grace_juice_count: u8,
    pub non_grace_scroll_count: u8,
    pub grace_juice_count: u8,
    pub grace_scroll_count: u8,
}

impl Upgrade {
    pub fn update_adv_config(&mut self) {
        let juice = index_to_tuple(self.state[0].1);
        let scroll = index_to_tuple(self.state[1].1);
        self.adv_config.grace_juice_target = juice.0 as u8;
        self.adv_config.non_grace_juice_target = juice.1 as u8;
        self.adv_config.grace_scroll_target = scroll.0 as u8;
        self.adv_config.non_grace_scroll_target = scroll.1 as u8;
    }
}
#[derive(Clone, Debug)]
pub struct AdvDistTriplet {
    pub cost: Vec<f64>,
    pub juice: Vec<f64>,
    pub scroll: Vec<f64>,
}
