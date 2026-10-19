use crate::constants::FLOAT_TOL;
use crate::constants::{ALLOWED_JUICE_POOLS, NUM_PIECE_TYPES, juice_info::JuiceInfo};
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};
use std::ops::{Deref, DerefMut};

pub type OneState = SmallVec<[usize; NUM_PIECE_TYPES]>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreakInfo {
    pub front: Vec<(Option<usize>, usize)>, // front[pool_index] = (id, streak length)
    pub back: Vec<(Option<usize>, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct State {
    pub payload: Vec<OneState>,
    #[serde(skip)]
    pub hash: u64,
    #[serde(skip)]
    pub streak_cache: Option<StreakInfo>, // kinda aids to calculate (need upgrade and juice info), will compute as needed
    #[serde(skip)]
    pub pool_ids_list: Option<Vec<Vec<usize>>>,
}

pub fn get_pool_ids(
    piece_type: usize,
    upgrade_index: usize,
    juice_info: &JuiceInfo,
) -> Vec<Vec<usize>> {
    let pools = &ALLOWED_JUICE_POOLS[piece_type];
    let usable_ids = &juice_info.normal_uindex_to_id[piece_type][upgrade_index];

    let explicit: std::collections::HashSet<usize> = pools.iter().flatten().copied().collect();

    pools
        .iter()
        .map(|pool| {
            if pool.is_empty() {
                usable_ids
                    .iter()
                    .copied()
                    .filter(|id| !explicit.contains(id))
                    .collect()
            } else {
                pool.iter()
                    .copied()
                    .filter(|id| usable_ids.contains(id))
                    .collect()
            }
        })
        .collect()
}

impl State {
    pub fn new_empty(length: usize) -> State {
        let mut out = State {
            payload: vec![smallvec![]; length],
            hash: 0,
            streak_cache: None,
            pool_ids_list: None,
        };
        out.update_hash();
        out
    }

    pub fn new(payload: Vec<OneState>) -> State {
        let mut out = State {
            payload,
            hash: 0,
            streak_cache: None,
            pool_ids_list: None,
        };
        out.update_hash();
        out
    }

    /// Becomes the other state of the same upgrade, as `clone_from` would, without allocating
    /// and without copying a payload that is already the same. The optimizer does this for
    /// every upgrade on every iteration.
    pub fn copy_from(&mut self, source: &State) {
        if self.hash != source.hash {
            self.payload.clone_from(&source.payload);
            self.hash = source.hash;
        }
        match (&mut self.streak_cache, &source.streak_cache) {
            (Some(to), Some(from)) => {
                to.front.clone_from(&from.front);
                to.back.clone_from(&from.back);
            }
            (to, from) => *to = from.clone(),
        }
        // the same for every state of an upgrade
        if self.pool_ids_list.is_none() {
            self.pool_ids_list.clone_from(&source.pool_ids_list);
        }
    }

    pub fn update_hash(&mut self) {
        // fixed seeds: a hash has to be the same from one run to the next
        self.hash = ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one(&self.payload);
    }

    pub fn update_payload(&mut self, new_payload: Vec<OneState>) {
        self.payload = new_payload;
        self.streak_cache = None;
        self.update_hash();
    }

    // turns state into pairs of (Some(id), length of streak), one anchored front and one anchored back
    pub fn compute_streak_info(
        &mut self,
        piece_type: usize,
        upgrade_index: usize,
        juice_info: &JuiceInfo,
    ) -> StreakInfo {
        let len = self.payload.len();
        let pool_ids_list = self
            .pool_ids_list
            .get_or_insert_with(|| get_pool_ids(piece_type, upgrade_index, juice_info));

        let mut front = Vec::with_capacity(pool_ids_list.len());
        let mut back = Vec::with_capacity(pool_ids_list.len());

        for pool_ids in pool_ids_list {
            let find_id = |slot: &OneState| -> Option<usize> {
                pool_ids.iter().copied().find(|id| slot.contains(id))
            };

            let front_id = if len > 0 {
                find_id(&self.payload[0])
            } else {
                None
            };
            let front_len = match front_id {
                None => 0,
                Some(id) => self
                    .payload
                    .iter()
                    .take_while(|slot| find_id(slot) == Some(id))
                    .count(),
            };

            // what is left after the front streak, from the last tap backwards
            let back_id = if front_len < len {
                find_id(&self.payload[len - 1])
            } else {
                None
            };
            let back_len = match back_id {
                None => 0,
                Some(id) => self.payload[front_len..]
                    .iter()
                    .rev()
                    .take_while(|slot| find_id(slot) == Some(id))
                    .count(),
            };

            front.push((front_id, front_len));
            back.push((back_id, back_len));
        }

        StreakInfo { front, back }
    }

    pub fn apply_new_streak_info(
        &mut self,
        streak_info: &StreakInfo,
        piece_type_usize: usize,
        upgrade_index: usize,
        juice_info: &JuiceInfo,
    ) {
        let pool_ids_list = self
            .pool_ids_list
            .get_or_insert_with(|| get_pool_ids(piece_type_usize, upgrade_index, juice_info));

        let len = self.payload.len();

        for slot in self.payload.iter_mut() {
            slot.retain(|id| !pool_ids_list.iter().flatten().any(|pool_id| pool_id == id));
        }

        for p in 0..pool_ids_list.len() {
            if let (Some(id), front_len) = streak_info.front[p] {
                let end = front_len.min(len);
                for slot in &mut self.payload[..end] {
                    // vambrace is temporarily disabled like this
                    if juice_info
                        .access(id, piece_type_usize, upgrade_index)
                        .normal_chance
                        > FLOAT_TOL
                    {
                        slot.push(id);
                    }
                }
            }
            if let (Some(id), back_len) = streak_info.back[p] {
                let end = back_len.min(len);
                for slot in self.payload[len - end..].iter_mut() {
                    if juice_info
                        .access(id, piece_type_usize, upgrade_index)
                        .normal_chance
                        > FLOAT_TOL
                    {
                        slot.push(id);
                    }
                }
            }
        }

        self.update_hash();
    }
}
impl Deref for State {
    type Target = Vec<OneState>;
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}
