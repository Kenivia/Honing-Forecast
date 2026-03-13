use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::{
    collections::{HashMap, HashSet},
    f64::NAN,
    ops::Deref,
};

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
    pub market_price: (f64, f64),
    pub trade_price: (f64, f64),
    pub left_price: (f64, f64),
    pub id: usize,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub data: HashMap<usize, OneUindexJuice>,
}
impl Default for JuiceType {
    fn default() -> Self {
        Self {
            market_price: (NAN, NAN),
            left_price: (NAN, NAN),
            trade_price: (NAN, NAN),
            id: 0,
            data: HashMap::new(),
        }
    }
}

impl Deref for JuiceType {
    type Target = HashMap<usize, OneUindexJuice>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuiceInfo {
    pub all_juices: Vec<JuiceType>, // all_juices[id][upgrade_index]
    pub normal_uindex_to_id: Vec<Vec<usize>>, // normal_id_to_uindex[upgrade_index] = all id availiable at this u_index for normal honing
    pub adv_uindex_to_id: Vec<Vec<usize>>, // adv_id_to_uindex[upgrade_index] = all id availiable at this u_index for adv honing
    pub total_num_avail: usize,
    pub num_juice_avail: usize,
}
impl JuiceInfo {
    pub fn access(&self, id: usize, upgrade_index: usize) -> OneUindexJuice {
        self.all_juices[id][&upgrade_index]
    }
    pub fn new(
        juice_books_avail: &[(usize, usize, usize, f64, i64, f64, f64)],
        event_multiplier: &[(usize, usize, usize, f64)],
    ) -> JuiceInfo {
        let mut normal_uindex_to_id: Vec<Vec<usize>> = vec![vec![]; 25];
        let mut adv_uindex_to_id: Vec<Vec<usize>> = vec![vec![]; 4];

        let mut all_juices: Vec<JuiceType> = Vec::new();
        let mut all_data: Vec<HashMap<usize, OneUindexJuice>> = Vec::new();
        let mut seen_ids: HashSet<usize> = HashSet::new();

        let mut event_multipliers: HashMap<(usize, usize, usize), f64> = HashMap::new();
        for (id, is_adv, upgrade_index, mult) in event_multiplier {
            event_multipliers.insert((*id, *is_adv, *upgrade_index), *mult);
        }
        for &(id, is_adv, upgrade_index, normal_chance, amt_used, gs_chance, gsx2_chance) in
            juice_books_avail.iter()
        {
            if !seen_ids.contains(&id) {
                assert!(id == all_juices.len());
                all_data.push(HashMap::new());
                all_juices.push(JuiceType::default());
                seen_ids.insert(id);
            }

            let relevant = if is_adv == 0 {
                &mut normal_uindex_to_id
            } else {
                &mut adv_uindex_to_id
            };
            relevant[upgrade_index].push(id);
            let this_event_mult = event_multipliers.get(&(id, is_adv, upgrade_index));
            let this_event_amt = if this_event_mult.is_none() {
                amt_used
            } else {
                (amt_used as f64 * this_event_mult.unwrap()).ceil() as i64
            };
            if !all_data[id].contains_key(&upgrade_index) {
                all_data[id].insert(upgrade_index, OneUindexJuice::default());
            }
            let this = all_data[id].get_mut(&upgrade_index).unwrap();
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
        let total_num_avail = all_juices.len() * 2 + 7;
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
pub fn get_priced_juice_info(
    base: &JuiceInfo,
    left_price: &[f64],
    trade_price: &[f64],
    market_price: &[f64],
    event: bool,
) -> JuiceInfo {
    assert!(base.total_num_avail == market_price.len());
    assert!(base.total_num_avail == trade_price.len());
    assert!(base.total_num_avail == left_price.len());
    let mut out: JuiceInfo = base.clone();
    for (id, juice_type) in out.all_juices.iter_mut().enumerate() {
        juice_type.market_price = (
            market_price[7 + id],
            market_price[7 + base.num_juice_avail + id],
        );
        juice_type.trade_price = (
            trade_price[7 + id],
            trade_price[7 + base.num_juice_avail + id],
        );
        juice_type.left_price = (
            left_price[7 + id],
            left_price[7 + base.num_juice_avail + id],
        );
        for (_, this) in juice_type.data.iter_mut() {
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
    out
}
