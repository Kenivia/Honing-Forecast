use crate::constants::*;

#[derive(Debug)]
pub struct Upgrade {
    pub is_normal_honing: bool,
    pub prob_dist: Vec<f64>,
    pub original_prob_dist: Vec<f64>,
    pub base_chance: f64,
    pub costs: [i64; 7],
    pub one_juice_cost: i64,
    pub adv_juice_cost: Vec<f64>,
    pub special_cost: i64,
    pub values: Vec<f64>,
    pub prob_dist_len: usize,
    pub is_weapon: bool,
    pub artisan_rate: f64,
    pub tap_offset: i64,
    pub upgrade_plus_num: usize,
    pub special_value: f64,
}
impl Upgrade {
    fn new_normal(
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        special_cost: i64,
        is_weapon: bool,
        artisan_rate: f64,
        upgrade_plus_num: usize,
    ) -> Upgrade {
        let prob_dist_len: usize = prob_dist.len();
        let base_chance: f64 = prob_dist[0];
        Upgrade {
            is_normal_honing: true,
            prob_dist: prob_dist.clone(),
            original_prob_dist: prob_dist.clone(),
            base_chance,
            costs,
            one_juice_cost: NORMAL_JUICE_COST[upgrade_plus_num],
            adv_juice_cost: vec![],
            special_cost,
            values: vec![],
            prob_dist_len,
            is_weapon,
            artisan_rate,
            tap_offset: 1,
            upgrade_plus_num,
            special_value: -1.0_f64,
        }
    }
    fn new_adv(
        prob_dist: Vec<f64>,
        costs: [i64; 7],
        one_juice_cost: i64,
        adv_juice_cost: Vec<f64>,
        is_weapon: bool,
        adv_cost_start: i64,
        upgrade_plus_num: usize,
    ) -> Upgrade {
        let prob_dist_len: usize = prob_dist.len();
        assert!(prob_dist_len == adv_juice_cost.len());
        Upgrade {
            is_normal_honing: false,
            prob_dist: prob_dist.clone(),
            original_prob_dist: prob_dist.clone(),
            base_chance: 0.0,
            costs,
            one_juice_cost,
            adv_juice_cost,
            special_cost: 0,
            values: vec![],
            prob_dist_len,
            is_weapon,
            artisan_rate: 0.0,
            tap_offset: adv_cost_start,
            upgrade_plus_num,
            special_value: -1.0_f64,
        }
    }
}
/// Produce the raw chance sequence used by the TS `raw_chance` helper.
/// Mirrors the JS behavior: increasing base chance with soft pity, artisan accumulation,
/// and stops when artisan >= 1 (in which case the chance is set to 1).
/// Produce the per-tap probability distribution used by the TS `raw_chance` + `raw -> distribution` logic.
/// Mirrors the JS behavior: increasing base chance with soft pity, artisan accumulation,
/// stops when artisan >= 1 (in which case the chance is set to 1), and converts the
/// per-try success chances into the probability that success happens exactly on that tap.
pub fn probability_distribution(
    base: f64,
    artisan_rate: f64,
    mut extra: f64,
    mut extra_num: usize,
) -> Vec<f64> {
    let mut raw_chances: Vec<f64> = Vec::new();
    let mut artisan: f64 = 0.0_f64;
    let mut count: i64 = 0;

    loop {
        if extra_num == 0 {
            extra = 0.0;
        } else {
            extra_num -= 1;
        }
        let min_count: f64 = std::cmp::min(count, 10) as f64;
        let mut current_chance: f64 = base + (min_count * base) / 10.0 + extra;

        if artisan >= 1.0 {
            if extra_num == 0 {
                current_chance = 1.0;
                raw_chances.push(current_chance);
                break;
            } else {
                return vec![];
            }
        }
        raw_chances.push(current_chance);
        count += 1;
        artisan += (46.51_f64 / 100.0) * current_chance * artisan_rate;
    }

    // convert raw per-try chances into per-tap probability distribution
    let mut chances = vec![0.0_f64; raw_chances.len()];
    let mut cum_chance = 1.0_f64;
    for (idx, &element) in raw_chances.iter().enumerate() {
        chances[idx] = cum_chance * element;
        cum_chance *= 1.0 - element;
    }
    chances
}

fn convert_static_to_vec(arr: &[[i64; 3]]) -> Vec<Vec<i64>> {
    arr.iter().map(|inner| inner.to_vec()).collect()
}
/// The main parser converted from the TypeScript version.
///
/// Inputs:
/// - `normal_counts`: 2 x N matrix with counts for normal honing (first row levels, second row ??? - same shape as TS)
/// - `normal_chances`: base rates for normal honing per level (float in (0,1])
/// - `weap_costs`: matrix [cost_type][level_index] for weapon cost types
/// - `armor_costs`: matrix [cost_type][level_index] for armor cost types
/// - `adv_counts`: 2 x M matrix for advanced honing counts (similar shape to normal_counts)
/// - `adv_costs`: matrix [cost_type][col_index] for adv cost types
/// - `adv_data_10_20_juice`, `adv_data_30_40_juice`, `adv_data_10_20`, `adv_data_30_40`:
///      arrays of adv-data rows (each row is an array where indexes 0,1,2 are used like in TS)
/// - `adv_hone_strategy`: either "Juice on grace" or "No juice"
///
/// Returns tuple:
/// (prob_dist_arr, hone_costs, adv_hone_chances, adv_hone_costs, tags)
pub fn parser(
    normal_counts: &Vec<Vec<i64>>,
    adv_counts: &Vec<Vec<i64>>,
    adv_hone_strategy: &String,
    artisan_rate_arr: &Vec<f64>,
    extra_arr: &Vec<f64>,
    extra_num_arr: &Vec<usize>,
    express_event: bool,
) -> Vec<Upgrade> {
    // --- Input assertions that match the TS checks ---
    assert!(normal_counts.len() == 2, "normal_counts must have length 2");
    assert!(
        normal_counts[0].len() == normal_counts[1].len(),
        "normal_counts rows must be same length"
    );

    // ensure integer ranges as in TS: first row values [0..=5], second row values [0..=1]
    let max_normal0: i64 = *normal_counts[0].iter().max().unwrap_or(&0);
    let min_normal0: i64 = *normal_counts[0].iter().min().unwrap_or(&0);
    let max_normal1: i64 = *normal_counts[1].iter().max().unwrap_or(&0);
    let min_normal1: i64 = *normal_counts[1].iter().min().unwrap_or(&0);

    assert!(max_normal0 <= 5, "normal_counts[0] max must be <= 5");
    assert!(min_normal0 >= 0, "normal_counts[0] min must be >= 0");
    assert!(max_normal1 <= 1, "normal_counts[1] max must be <= 1");
    assert!(min_normal1 >= 0, "normal_counts[1] min must be >= 0");

    // each normal_counts entry should already be integers (type i64) so no runtime check required beyond ranges

    // adv_counts checks
    assert!(adv_counts.len() == 2, "adv_counts must have length 2");
    let max_adv0: i64 = *adv_counts[0].iter().max().unwrap_or(&0);
    let min_adv0: i64 = *adv_counts[0].iter().min().unwrap_or(&0);
    let max_adv1: i64 = *adv_counts[1].iter().max().unwrap_or(&0);
    let min_adv1: i64 = *adv_counts[1].iter().min().unwrap_or(&0);
    assert!(max_adv0 <= 5, "adv_counts[0] max must be <= 5");
    assert!(min_adv0 >= 0, "adv_counts[0] min must be >= 0");
    assert!(max_adv1 <= 1, "adv_counts[1] max must be <= 1");
    assert!(min_adv1 >= 0, "adv_counts[1] min must be >= 0");

    // base_rates validation
    for &i in &NORMAL_HONE_CHANCES {
        assert!(i > 0.0 && i <= 1.0, "normal_chances must be in (0,1]");
    }

    assert!(
        adv_hone_strategy == "Juice on grace" || adv_hone_strategy == "No juice",
        "invalid adv_hone_strategy"
    );

    // --- Core logic translation from TS ---
    // let mut tags: Vec<String> = Vec::new();
    // let mut prob_dist_arr: Vec<Vec<f64>> = Vec::new();
    let mut out: Vec<Upgrade> = Vec::new();

    // hone_costs: create vec of empty vectors equal to number of cost types in weap_costs
    // (TS used weap_costs.length to size this)
    // let cost_types_count = NORMAL_HONE_WEAPON_COST.len();
    // let mut hone_costs: Vec<Vec<i64>> = vec![Vec::new(); cost_types_count];
    let mut this_cost: Vec<i64>;
    let mut prob_dist: Vec<f64>;
    // For each piece type (0..normal_counts.len())
    for is_weapon in 0..normal_counts.len() {
        let cur_cost = if is_weapon == 0 {
            get_event_modified_armor_costs(express_event)
        } else {
            get_event_modified_weapon_costs(express_event)
        };

        let mut current_counter: i64 = 0;

        // iterate over levels i with repetition according to normal_counts[piece_type][i]
        let row_len = normal_counts[is_weapon].len(); // 25
        let mut upgrade_plus_num: usize = 0;
        let mut base: f64;
        let mut special_cost: i64;
        while upgrade_plus_num < row_len {
            let needed: i64 = normal_counts[is_weapon][upgrade_plus_num];
            if current_counter >= needed {
                upgrade_plus_num += 1;
                current_counter = 0;
                continue;
            }

            // tag e.g. "Normal Armor +X#Y" or "Normal Weapon +X#Y"
            // let piece_str = if piece_type == 0 {
            //     " Armor "
            // } else {
            //     " Weapon "
            // };
            // let tag = format!("Normal{}+{}#{}", piece_str, i, current_counter);

            // special_costs.push(SPECIAL_LEAPS_COST[piece_type][i]);
            special_cost = SPECIAL_LEAPS_COST[is_weapon][upgrade_plus_num];
            base = NORMAL_HONE_CHANCES[upgrade_plus_num];
            // let event_artisan_rate = if express_event {
            //     artisan_rate_arr[upgrade_plus_num] * EVENT_ARTISAN_MULTIPLIER[upgrade_plus_num]
            // } else {
            let event_artisan_rate: f64 = artisan_rate_arr[upgrade_plus_num];
            // };

            prob_dist = probability_distribution(
                base,
                event_artisan_rate,
                extra_arr[upgrade_plus_num],
                extra_num_arr[upgrade_plus_num],
            );

            this_cost = Vec::with_capacity(7);
            // for each cost_type push cur_cost[cost_type][i]
            for cost_type in 0..7 {
                this_cost.push(cur_cost[cost_type][upgrade_plus_num]);
            }

            out.push(Upgrade::new_normal(
                prob_dist,
                this_cost.try_into().unwrap(),
                special_cost,
                is_weapon == 1,
                event_artisan_rate,
                upgrade_plus_num,
            ));
            current_counter += 1;
        }
    }

    // Advanced hone
    let mut this_juice_cost: Vec<f64>;
    for is_weapon in 0..adv_counts.len() {
        let mut current_counter: i64 = 0;
        let row_len = adv_counts[is_weapon].len();
        let mut upgrade_plus_num: usize = 0;
        while upgrade_plus_num < row_len {
            let needed = adv_counts[is_weapon][upgrade_plus_num];
            if current_counter >= needed {
                upgrade_plus_num += 1;
                current_counter = 0;
                continue;
            }

            // pick relevant_data based on strategy and level i (i <= 1 -> 10/20, else 30/40)
            let relevant_data: &Vec<Vec<i64>> = if adv_hone_strategy == "Juice on grace" {
                if upgrade_plus_num <= 1 {
                    &convert_static_to_vec(&ADV_DATA_10_20_JUICE)
                } else {
                    &convert_static_to_vec(&ADV_DATA_30_40_JUICE)
                }
            } else {
                if upgrade_plus_num <= 1 {
                    &convert_static_to_vec(&ADV_DATA_10_20)
                } else {
                    &convert_static_to_vec(&ADV_DATA_30_40)
                }
            };

            // this_chances length = relevant_data.len()
            let rows = relevant_data.len();
            let sum_taps: i64 = relevant_data
                .iter()
                .map(|row| row.get(2).copied().unwrap_or(0))
                .sum();

            // this_cost is 9 x rows matrix of i64 zeros (TS used length 9)
            let col_index = 2 * (upgrade_plus_num as i64) + (1 - is_weapon as i64);
            this_cost = Vec::with_capacity(7);
            for cost_type in 0..7 {
                // index into adv_costs columns: 2*i + (1 - wep_or_arm)

                let cost_val = ADV_HONE_COST[cost_type][col_index as usize];
                this_cost.push(cost_val);
            }
            prob_dist = Vec::with_capacity(rows);
            this_juice_cost = Vec::with_capacity(rows);
            let cost_val: i64 = ADV_HONE_COST[7][col_index as usize];
            for row_idx in 0..rows {
                // row structure: [something_for_blue_count, something_for_juice_count, taps]
                let row = &relevant_data[row_idx];

                let taps = row.get(2).copied().unwrap_or(0);
                let taps_f = taps as f64;
                let sum_taps_f = if sum_taps == 0 { 1.0 } else { sum_taps as f64 };
                prob_dist.push(taps_f / sum_taps_f);

                this_juice_cost
                    .push(cost_val as f64 * relevant_data[row_idx][1] as f64 / 1000.0_f64);
            }

            // adv_hone_chances.push(this_chances);
            // adv_hone_costs.push(this_cost);
            out.push(Upgrade::new_adv(
                prob_dist,
                this_cost.try_into().unwrap(),
                cost_val,
                this_juice_cost.clone(),
                is_weapon == 1,
                relevant_data[0][0],
                upgrade_plus_num,
            ));
            current_counter += 1;
        }
    }

    out
}
