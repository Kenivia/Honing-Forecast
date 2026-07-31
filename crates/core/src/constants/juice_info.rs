use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::parser::MaterialInput;

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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuiceType {
    pub prices: Vec<(f64, f64)>, // weapon, armor
    pub id: usize,
    // data[piece_type][upgrade_index] = OneUindexJuice
    #[serde_as(as = "Vec<HashMap<DisplayFromStr, _>>")]
    pub data: Vec<HashMap<usize, OneUindexJuice>>,
}
impl Default for JuiceType {
    fn default() -> Self {
        Self {
            prices: Vec::new(),
            id: 0,
            data: Vec::new(),
        }
    }
}

impl Deref for JuiceType {
    type Target = Vec<HashMap<usize, OneUindexJuice>>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuiceInfo {
    pub all_juices: Vec<JuiceType>, // all_juices[id][piece_type][upgrade_index]
    pub normal_uindex_to_id: Vec<Vec<Vec<usize>>>, // normal_uindex_to_id[piece_type][upgrade_index] = all id availiable at this u_index for normal honing
    pub adv_uindex_to_id: Vec<Vec<Vec<usize>>>, // adv_uindex_to_id[piece_type][upgrade_index] = all id availiable at this u_index for adv honing
    // pub total_num_avail: usize,
    pub num_juice_avail: usize,
}
impl JuiceInfo {
    pub fn access(&self, id: usize, piece_type: usize, upgrade_index: usize) -> OneUindexJuice {
        self.all_juices[id][piece_type][&upgrade_index]
    }
    pub fn new(
        juice_books_avail: &[(usize, usize, usize, usize, f64, i64, f64, f64)],
        event_multiplier: &[(usize, usize, usize, f64)],
    ) -> JuiceInfo {
        const NUM_NORMAL_UPGRADES: usize = 25;
        const NUM_ADV_UPGRADES: usize = 4;

        let mut normal_uindex_to_id: Vec<Vec<Vec<usize>>> = Vec::new();
        let mut adv_uindex_to_id: Vec<Vec<Vec<usize>>> = Vec::new();

        let mut all_juices: Vec<JuiceType> = Vec::new();
        let mut all_data: Vec<Vec<HashMap<usize, OneUindexJuice>>> = Vec::new();
        let mut seen_ids: HashSet<usize> = HashSet::new();

        let mut event_multipliers: HashMap<(usize, usize, usize), f64> = HashMap::new();
        for (id, is_adv, upgrade_plus, mult) in event_multiplier {
            event_multipliers.insert((*id, *is_adv, *upgrade_plus - 1), *mult);
        }
        for &(
            id,
            is_adv,
            piece_type,
            upgrade_plus,
            normal_chance,
            amt_used,
            gs_chance,
            gsx2_chance,
        ) in juice_books_avail.iter()
        {
            let upgrade_index = upgrade_plus - 1;
            if !seen_ids.contains(&id) {
                assert!(id == all_juices.len());
                all_data.push(Vec::new());
                all_juices.push(JuiceType::default());
                seen_ids.insert(id);
            }

            if is_adv == 0 {
                if normal_uindex_to_id.len() <= piece_type {
                    normal_uindex_to_id
                        .resize_with(piece_type + 1, || vec![Vec::new(); NUM_NORMAL_UPGRADES]);
                }
                normal_uindex_to_id[piece_type][upgrade_index].push(id);
            } else {
                if adv_uindex_to_id.len() <= piece_type {
                    adv_uindex_to_id
                        .resize_with(piece_type + 1, || vec![Vec::new(); NUM_ADV_UPGRADES]);
                }
                adv_uindex_to_id[piece_type][upgrade_index].push(id);
            }

            let this_event_mult = event_multipliers.get(&(id, is_adv, upgrade_index));
            let this_event_amt = if this_event_mult.is_none() {
                amt_used
            } else {
                (amt_used as f64 * this_event_mult.unwrap()).ceil() as i64
            };

            let this_juice_data = &mut all_data[id];
            if this_juice_data.len() <= piece_type {
                this_juice_data.resize_with(piece_type + 1, HashMap::new);
            }
            let this = this_juice_data[piece_type]
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
            all_juices[id].data = this_data;
        }
        // let total_num_avail = all_juices.len() * 2 + 7;
        let num_juice_avail = all_juices.len();
        JuiceInfo {
            all_juices,
            normal_uindex_to_id,
            adv_uindex_to_id,
            // total_num_avail,
            num_juice_avail,
        }
    }
}
pub fn get_priced_juice_info(
    base: &JuiceInfo,
    material_info: &MaterialInput,
    event: bool,
) -> JuiceInfo {
    // my_dbg!(base.total_num_avail, &market_price);
    // assert!(base.total_num_avail == material_info.len());

    let mut out: JuiceInfo = base.clone();
    for (id, juice_type) in out.all_juices.iter_mut().enumerate() {
        assert!(juice_type.prices.len() == 0);
        for price_pair in material_info[7 + id].iter().map(|x| x.1).zip(
            material_info[7 + base.num_juice_avail + id]
                .iter()
                .map(|x| x.1),
        ) {
            juice_type.prices.push(price_pair)
        }

        for piece_map in juice_type.data.iter_mut() {
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
