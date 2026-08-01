use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    f64,
};

use crate::{
    constants::{NUM_ADV_UPGRADES, NUM_NORMAL_UPGRADES, NUM_PIECE_TYPES},
    my_dbg,
    parser::MaterialInput,
};

// use crate::my_dbg;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub struct OneUindexJuice {
    pub normal_amt_used: i64,
    normal_base_amt_used: i64,
    normal_event_amt_used: i64,
    pub normal_chance: f64,
    // pub can_adv: bool, this information is deduced from being in the uindex_to_id vector or not
    pub adv_chances: (f64, f64),

    pub adv_amt_used: i64,
    adv_base_amt_used: i64,
    adv_event_amt_used: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuiceInfo {
    pub all_juices: Vec<Vec<HashMap<usize, OneUindexJuice>>>, // all_juices[id][piece_type][upgrade_index]
    pub normal_uindex_to_id: Vec<Vec<Vec<usize>>>, // normal_uindex_to_id[piece_type][upgrade_index] = all id availiable at this u_index for normal honing
    pub adv_uindex_to_id: Vec<Vec<Vec<usize>>>, // adv_uindex_to_id[piece_type][upgrade_index] = all id availiable at this u_index for adv honing
    pub total_num_avail: usize,
    pub num_juice_avail: usize,
}
impl JuiceInfo {
    pub fn access(
        &self,
        id: usize,
        piece_type_usize: usize,
        upgrade_index: usize,
    ) -> OneUindexJuice {
        if self
            .all_juices
            .get(id)
            .and_then(|v| v.get(piece_type_usize))
            .and_then(|v| v.get(&upgrade_index))
            .is_none()
        {
            my_dbg!(&self.all_juices[id], id, piece_type_usize, upgrade_index);
            my_dbg!(&self.normal_uindex_to_id[piece_type_usize][upgrade_index],);
        }

        self.all_juices[id][piece_type_usize][&upgrade_index]
    }
    pub fn new(
        juice_books_avail: &[(usize, usize, usize, usize, f64, i64, f64, f64)],
        event_multiplier: &[(usize, usize, usize, usize, f64)],
    ) -> JuiceInfo {
        let mut normal_uindex_to_id: Vec<Vec<Vec<usize>>> =
            vec![vec![Vec::new(); NUM_NORMAL_UPGRADES]; NUM_PIECE_TYPES];
        let mut adv_uindex_to_id: Vec<Vec<Vec<usize>>> =
            vec![vec![Vec::new(); NUM_ADV_UPGRADES]; NUM_PIECE_TYPES];

        let mut all_juices: Vec<Vec<HashMap<usize, OneUindexJuice>>> = Vec::new();
        let mut all_data: Vec<Vec<HashMap<usize, OneUindexJuice>>> = Vec::new();
        let mut seen_ids: HashSet<usize> = HashSet::new();

        let mut event_multipliers: HashMap<(usize, usize, usize, usize), f64> = HashMap::new();
        for (id, is_adv, piece_type_usize, upgrade_plus, mult) in event_multiplier {
            event_multipliers.insert((*id, *is_adv, *piece_type_usize, *upgrade_plus - 1), *mult);
        }
        for &(
            id,
            is_adv,
            piece_type_usize,
            upgrade_plus,
            normal_chance,
            amt_used,
            gs_chance,
            gsx2_chance,
        ) in juice_books_avail.iter()
        {
            let upgrade_index = upgrade_plus - 1;
            if !seen_ids.contains(&id) {
                // my_dbg!(id, all_juices.len());
                assert!(id == all_juices.len());
                all_data.push(vec![HashMap::new(); NUM_PIECE_TYPES]);
                all_juices.push(Vec::new());
                seen_ids.insert(id);
            }

            (if is_adv == 0 {
                &mut normal_uindex_to_id
            } else {
                &mut adv_uindex_to_id
            })[piece_type_usize][upgrade_index]
                .push(id);

            let this_event_mult =
                event_multipliers.get(&(id, is_adv, piece_type_usize, upgrade_index));
            let this_event_amt = if this_event_mult.is_none() {
                amt_used
            } else {
                (amt_used as f64 * this_event_mult.unwrap()).ceil() as i64
            };

            let this_juice_data = &mut all_data[id];

            let this = this_juice_data[piece_type_usize]
                .entry(upgrade_index)
                .or_insert_with(OneUindexJuice::default);
            if is_adv == 1 {
                this.adv_amt_used = amt_used;
                this.adv_base_amt_used = amt_used;
                this.adv_event_amt_used = this_event_amt;
                this.adv_chances = (gs_chance, gsx2_chance);
            } else {
                this.normal_amt_used = amt_used;
                this.normal_base_amt_used = amt_used;
                this.normal_event_amt_used = this_event_amt;
                this.normal_chance = normal_chance;
            }
        }
        for (id, this_data) in all_data.into_iter().enumerate() {
            all_juices[id] = this_data;
        }
        let total_num_avail = all_juices.len() + 7;
        let num_juice_avail = all_juices.len();
        JuiceInfo {
            all_juices,
            normal_uindex_to_id,
            adv_uindex_to_id,
            total_num_avail,
            num_juice_avail,
        }
    }
}
pub fn get_event_adjusted_juice_info(
    base: &JuiceInfo,
    material_info: &MaterialInput,
    event: bool,
) -> JuiceInfo {
    // my_dbg!(&base.all_juices);
    assert!(base.total_num_avail == material_info.len());

    let mut out: JuiceInfo = base.clone();
    for (_, juice_type) in out.all_juices.iter_mut().enumerate() {
        for piece_map in juice_type.iter_mut() {
            for (_, this) in piece_map.iter_mut() {
                this.normal_amt_used = if event {
                    this.normal_event_amt_used
                } else {
                    this.normal_base_amt_used
                };

                this.adv_amt_used = if event {
                    this.adv_event_amt_used
                } else {
                    this.adv_base_amt_used
                };
            }
        }
    }
    out
}
