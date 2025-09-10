use crate::constants::*;

/// Produce the raw chance sequence used by the TS `raw_chance` helper.
/// Mirrors the JS behavior: increasing base chance with soft pity, artisan accumulation,
/// and stops when artisan >= 1 (in which case the chance is set to 1).
fn raw_chance(base: f32, artisan_rate: f32, mut extra: f32, mut extra_num: i64) -> Vec<f32> {
    let mut chances: Vec<f32> = Vec::new();
    let mut artisan = 0.0_f32;
    let mut current_chance: f32;
    let mut count: i64 = 0;

    loop {
        if extra_num <= 0 {
            extra = 0.0;
        }
        // current_chance = base + (min(count,10) * base) / 10 + extra
        let min_count = std::cmp::min(count, 10) as f32;
        current_chance = base + (min_count * base) / 10.0 + extra;
        extra_num -= 1;

        if artisan >= 1.0 {
            current_chance = 1.0;
            chances.push(current_chance);
            break;
        }
        chances.push(current_chance);
        count += 1;
        artisan += (46.51_f32 / 100.0) * current_chance * artisan_rate;
    }

    chances
}

/// Convert a raw chance sequence (per-try success chance) into the per-tap probability distribution,
/// i.e. the probability that success happens exactly on that tap (before starting).
fn probability_distribution(raw: &[f32]) -> Vec<f32> {
    let mut chances = vec![0.0_f32; raw.len()];
    let mut cum_chance = 1.0_f32;
    for (idx, &element) in raw.iter().enumerate() {
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
) -> (
    Vec<Vec<f32>>,
    Vec<Vec<i64>>,
    Vec<Vec<f32>>,
    Vec<Vec<Vec<i64>>>,
    Vec<String>,
) {
    // --- Input assertions that match the TS checks ---
    assert!(normal_counts.len() == 2, "normal_counts must have length 2");
    assert!(
        normal_counts[0].len() == normal_counts[1].len(),
        "normal_counts rows must be same length"
    );

    // ensure integer ranges as in TS: first row values [0..=5], second row values [0..=1]
    let max_normal0 = *normal_counts[0].iter().max().unwrap_or(&0);
    let min_normal0 = *normal_counts[0].iter().min().unwrap_or(&0);
    let max_normal1 = *normal_counts[1].iter().max().unwrap_or(&0);
    let min_normal1 = *normal_counts[1].iter().min().unwrap_or(&0);

    assert!(max_normal0 <= 5, "normal_counts[0] max must be <= 5");
    assert!(min_normal0 >= 0, "normal_counts[0] min must be >= 0");
    assert!(max_normal1 <= 1, "normal_counts[1] max must be <= 1");
    assert!(min_normal1 >= 0, "normal_counts[1] min must be >= 0");

    // each normal_counts entry should already be integers (type i64) so no runtime check required beyond ranges

    // adv_counts checks
    assert!(adv_counts.len() == 2, "adv_counts must have length 2");
    let max_adv0 = *adv_counts[0].iter().max().unwrap_or(&0);
    let min_adv0 = *adv_counts[0].iter().min().unwrap_or(&0);
    let max_adv1 = *adv_counts[1].iter().max().unwrap_or(&0);
    let min_adv1 = *adv_counts[1].iter().min().unwrap_or(&0);
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
    let mut tags: Vec<String> = Vec::new();
    let mut prob_dist_arr: Vec<Vec<f32>> = Vec::new();

    // hone_costs: create vec of empty vectors equal to number of cost types in weap_costs
    // (TS used weap_costs.length to size this)
    let cost_types_count = NORMAL_HONE_WEAPON_COST.len();
    let mut hone_costs: Vec<Vec<i64>> = vec![Vec::new(); cost_types_count];

    // For each piece type (0..normal_counts.len())
    for piece_type in 0..normal_counts.len() {
        let cur_cost = if piece_type == 0 {
            &NORMAL_HONE_ARMOR_COST
        } else {
            &NORMAL_HONE_WEAPON_COST
        };
        let mut current_counter: i64 = 0;

        // iterate over levels i with repetition according to normal_counts[piece_type][i]
        let row_len = normal_counts[piece_type].len();
        let mut i: usize = 0;
        while i < row_len {
            let needed = normal_counts[piece_type][i];
            if current_counter >= needed {
                i += 1;
                current_counter = 0;
                continue;
            }

            // tag e.g. "Normal Armor +X#Y" or "Normal Weapon +X#Y"
            let piece_str = if piece_type == 0 {
                " Armor "
            } else {
                " Weapon "
            };
            let tag = format!("Normal{}+{}#{}", piece_str, i, current_counter);
            tags.push(tag);

            let base = NORMAL_HONE_CHANCES[i];
            // TS used raw_chance(base) with default artisan_rate and extras
            let raw = raw_chance(base, 1.0, 0.0, 0);
            let prob_dist = probability_distribution(&raw);

            // for each cost_type push cur_cost[cost_type][i]
            for cost_type in 0..cost_types_count {
                // cur_cost is matrix [cost_type][level_index]
                // assert bounds
                let val = cur_cost
                    .get(cost_type)
                    .and_then(|r| r.get(i))
                    .copied()
                    .unwrap_or(0);
                hone_costs[cost_type].push(val);
            }

            prob_dist_arr.push(prob_dist);
            current_counter += 1;
        }
    }

    // Advanced hone
    let mut adv_hone_costs: Vec<Vec<Vec<i64>>> = Vec::new(); // Vec of this_cost matrices
    let mut adv_hone_chances: Vec<Vec<f32>> = Vec::new();

    for wep_or_arm in 0..adv_counts.len() {
        let mut current_counter: i64 = 0;
        let row_len = adv_counts[wep_or_arm].len();
        let mut i: usize = 0;
        while i < row_len {
            let needed = adv_counts[wep_or_arm][i];
            if current_counter >= needed {
                i += 1;
                current_counter = 0;
                continue;
            }

            let piece_str = if wep_or_arm == 0 {
                " Armor "
            } else {
                " Weapon "
            };
            // Tag example: "Adv Armor +10Juice on grace#0"  (matches TS pattern)
            let tag = format!(
                "Adv{}+{}{}#{}",
                piece_str,
                i * 10,
                adv_hone_strategy,
                current_counter
            );
            tags.push(tag);

            // pick relevant_data based on strategy and level i (i <= 1 -> 10/20, else 30/40)
            let relevant_data: &Vec<Vec<i64>> = if adv_hone_strategy == "Juice on grace" {
                if i <= 1 {
                    &convert_static_to_vec(&ADV_DATA_10_20_JUICE)
                } else {
                    &convert_static_to_vec(&ADV_DATA_30_40_JUICE)
                }
            } else {
                if i <= 1 {
                    &convert_static_to_vec(&ADV_DATA_10_20)
                } else {
                    &convert_static_to_vec(&ADV_DATA_30_40)
                }
            };

            // this_chances length = relevant_data.len()
            let rows = relevant_data.len();
            let mut this_chances: Vec<f32> = vec![0.0; rows];
            let sum_taps: i64 = relevant_data
                .iter()
                .map(|row| row.get(2).copied().unwrap_or(0))
                .sum();

            // this_cost is 9 x rows matrix of i64 zeros (TS used length 9)
            let mut this_cost: Vec<Vec<i64>> = vec![vec![0_i64; rows]; 9];

            for row_idx in 0..rows {
                // row structure: [something_for_blue_count, something_for_juice_count, taps]
                let row = &relevant_data[row_idx];

                let taps = row.get(2).copied().unwrap_or(0);
                let taps_f = taps as f32;
                let sum_taps_f = if sum_taps == 0 { 1.0 } else { sum_taps as f32 };
                this_chances[row_idx] = taps_f / sum_taps_f;

                // For cost_type 0..6
                for cost_type in 0..7 {
                    // index into adv_costs columns: 2*i + (1 - wep_or_arm)
                    let col_index = 2 * (i as i64) + (1 - wep_or_arm as i64);
                    let cost_val = ADV_HONE_COST
                        .get(cost_type)
                        .and_then(|r| r.get(col_index as usize))
                        .copied()
                        .unwrap_or(0);
                    let multiplier = row.get(0).copied().unwrap_or(0);
                    this_cost[cost_type][row_idx] = cost_val * multiplier;
                }

                // cost_type 7..8: use row[1] and apply juice strategy multiplier
                for cost_type in 7..9 {
                    let col_index = 2 * (i as i64) + (1 - wep_or_arm as i64);
                    let cost_val = ADV_HONE_COST
                        .get(cost_type)
                        .and_then(|r| r.get(col_index as usize))
                        .copied()
                        .unwrap_or(0);
                    let multiplier = row.get(1).copied().unwrap_or(0);
                    let strategy_mult = if adv_hone_strategy == "Juice on grace" {
                        1
                    } else {
                        0
                    };
                    this_cost[cost_type][row_idx] = cost_val * multiplier * strategy_mult;
                }
            }

            adv_hone_chances.push(this_chances);
            adv_hone_costs.push(this_cost);
            current_counter += 1;
        }
    }

    // TS had a couple of "if length == 0, set to []" - redundant in Rust: Vec already empty.
    (
        prob_dist_arr,
        hone_costs,
        adv_hone_chances,
        adv_hone_costs,
        tags,
    )
}
