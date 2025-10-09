use crate::bitset::{BitsetBundle, beam_search, generate_bit_sets};
use crate::constants::*;
use crate::helpers::average_juice_cost;
use crate::helpers::{calc_unlock, compress_runs, sort_by_indices};
use crate::histogram::histograms_for_all_costs;
use crate::monte_carlo::{generate_budget_data, get_top_bottom, monte_carlo_data};
use crate::parser::{Upgrade, parser};
use crate::value_estimation::{est_juice_value, est_special_honing_value, juice_to_array};
// use assert_float_eq::assert_f64_near;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct CostToChanceOut {
    pub chance: f64,
    pub reasons: Vec<f64>,                 // 7 failure rates for each cost type
    pub hist_counts: Vec<Vec<i64>>,        // 7 x num_bins
    pub hist_mins: Vec<i64>,               // 7
    pub hist_maxs: Vec<i64>,               // 7
    pub upgrade_strings: Vec<String>,      // ordered upgrade descriptions
    pub juice_strings_armor: Vec<String>,  // e.g., ["+14 armor first 10 taps", ...]
    pub juice_strings_weapon: Vec<String>, // e.g., ["+15 weapon first 6 taps", ...]
    pub budgets_red_remaining: i64,        // budgets[7]
    pub budgets_blue_remaining: i64,       // budgets[8]
}
#[derive(Serialize, Debug)]
pub struct CostToChanceOptimizedOut {
    pub chance: f64,
    pub reasons: Vec<f64>,                 // 7 failure rates for each cost type
    pub hist_counts: Vec<Vec<i64>>,        // 7 x num_bins
    pub hist_mins: Vec<i64>,               // 7
    pub hist_maxs: Vec<i64>,               // 7
    pub upgrade_strings: Vec<String>,      // ordered upgrade descriptions
    pub juice_strings_armor: Vec<String>,  // e.g., ["+14 armor first 10 taps", ...]
    pub juice_strings_weapon: Vec<String>, // e.g., ["+15 weapon first 6 taps", ...]
    pub budgets_red_remaining: i64,        // budgets[7]
    pub budgets_blue_remaining: i64,       // budgets[8]
    pub buy_arr: Vec<i64>,
    pub optimized_chance: f64,
    pub optimized_reasons: Vec<f64>,
}
#[derive(serde::Serialize)]
pub struct CostToChanceArrResult {
    pub final_chances: Vec<f64>,
    pub typed_fail_counters: Vec<Vec<f64>>,
    pub budgets_red_remaining: i64,
    pub budgets_blue_remaining: i64,
}
fn extract_upgrade_strings(
    upgrade_arr: &[Upgrade],
    user_gave_weapon: bool,
    user_gave_armor: bool,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for (_index, upgrade) in upgrade_arr.iter().enumerate() {
        if !upgrade.is_normal_honing {
            continue;
        }
        let level_str: String = format!("+{}", upgrade.upgrade_plus_num + 1);
        let type_str: &'static str = if upgrade.is_weapon { "weapon" } else { "armor" };
        // let value_str: &str = &upgrade.special_value.to_string().as_str();
        let is_valid: bool = if upgrade.is_weapon {
            user_gave_weapon
        } else {
            user_gave_armor
        };
        let value_string: String = if !is_valid {
            "".to_owned()
        } else {
            " ".to_owned() + &upgrade.special_value.round().to_string() + "g"
        };
        result.push(format!("{} {}{}", level_str, type_str, value_string));
    }
    result
}

fn fail_count_to_rates(typed_fail_counter: Vec<f64>, data_size: usize) -> Vec<f64> {
    let mut failure_rates: Vec<f64> = Vec::new();
    for z in 0..typed_fail_counter.len() {
        let failure_rate: f64 = 1.0 - typed_fail_counter[z] as f64 / data_size as f64;
        failure_rates.push(failure_rate);
    }
    failure_rates
}

#[derive(Debug)]
struct PreparationOutputs {
    upgrade_arr: Vec<Upgrade>,
    unlock_costs: Vec<i64>,
    budgets: Vec<i64>,
    valid_armor_values: bool,
    valid_weapon_values: bool,
    juice_strings_armor: Vec<String>,
    juice_strings_weapon: Vec<String>,
    mats_value: Vec<f64>,
}

fn preparation(
    hone_counts: &[Vec<i64>],
    input_budgets: &[i64],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    user_mats_value: &[f64],
    adv_hone_strategy: &str,
) -> PreparationOutputs {
    let mut mats_value: Vec<f64> = user_mats_value.to_vec();
    let unlock_costs: Vec<i64> = calc_unlock(hone_counts, adv_counts, express_event);

    let aritsan_arr: Vec<f64>;
    if express_event {
        aritsan_arr = EVENT_ARTISAN_MULTIPLIER.to_vec();
    } else {
        aritsan_arr = vec![1.0; 25];
    }

    let mut upgrade_arr: Vec<Upgrade> = parser(
        hone_counts,
        adv_counts,
        &adv_hone_strategy.to_string(),
        &aritsan_arr,
        &vec![0.0; 25],
        &vec![0; 25],
        express_event,
    );
    let mut budgets: Vec<i64> = input_budgets.to_vec();

    // Add average juice costs to budgets for all upgrades
    if adv_hone_strategy == "Juice on grace" {
        let (avg_red_juice, avg_blue_juice): (i64, i64) = average_juice_cost(&upgrade_arr);
        budgets[7] -= avg_red_juice;
        budgets[8] -= avg_blue_juice;
    }

    let valid_armor_values: bool =
        mats_value.iter().skip(1).any(|&x| x != 0.0) || upgrade_arr.iter().all(|x| x.is_weapon);

    let valid_weapon_values: bool = mats_value
        .iter()
        .enumerate()
        .any(|(index, &x)| index != 1 && x != 0.0)
        || upgrade_arr.iter().all(|x| !x.is_weapon);
    let both_valid: bool = valid_armor_values && valid_weapon_values;

    if !both_valid {
        mats_value = DEFAULT_GOLD_VALUES.to_vec();
    };

    est_juice_value(&mut upgrade_arr, &mats_value);
    let (juice_strings_armor, juice_strings_weapon): (Vec<String>, Vec<String>) = juice_to_array(
        &mut upgrade_arr,
        budgets[8],
        budgets[7],
        valid_armor_values,
        valid_weapon_values,
    );
    let value_per_special_leap: Vec<f64> = est_special_honing_value(&mut upgrade_arr, &mats_value);
    let mut special_indices: Vec<usize> = (0..value_per_special_leap.len()).collect();
    special_indices
        .sort_by(|&a, &b| value_per_special_leap[b].total_cmp(&value_per_special_leap[a]));
    sort_by_indices(&mut upgrade_arr, special_indices.clone());

    PreparationOutputs {
        upgrade_arr,
        unlock_costs,
        budgets,
        valid_armor_values,
        valid_weapon_values,
        juice_strings_armor,
        juice_strings_weapon,
        mats_value,
    }
}

#[derive(Debug)]
struct HistogramOutputs {
    upgrade_strings: Vec<String>,
    // top_bottom: Vec<Vec<i64>>,
    hist_counts: Vec<Vec<i64>>,
    hist_mins: Vec<i64>,
    hist_maxs: Vec<i64>,
}

#[derive(Debug)]
struct FailureAnalysisOutputs {
    typed_fail_counter_final: Vec<f64>,
    final_chance: f64,
}

fn count_failure_typed(cost_data: &[Vec<i64>], input_budgets: &[i64]) -> FailureAnalysisOutputs {
    let mut typed_fail_counter_final: Vec<f64> = vec![0.0_f64; 7];
    let mut overall_fail_counter: i64 = 0;
    let mut failed: bool;
    for (_trail_num, data) in cost_data.iter().enumerate() {
        failed = false;
        for cost_type in 0..7 {
            // Cost to chance does take silver into account
            if input_budgets[cost_type as usize] < data[cost_type] {
                failed = true;
                typed_fail_counter_final[cost_type] += 1.0_f64;
            }
        }
        if failed {
            overall_fail_counter += 1;
        }
    }

    let final_chance: f64 = 1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64;

    FailureAnalysisOutputs {
        typed_fail_counter_final,
        final_chance,
    }
}

fn count_failure_typed_arr(
    cost_data: &[Vec<i64>],
    input_budgets_arr: &[Vec<i64>],
) -> (Vec<f64>, Vec<Vec<f64>>) {
    let mut final_chances: Vec<f64> = Vec::new();
    let mut typed_fail_counters: Vec<Vec<f64>> = Vec::new();

    for input_budgets in input_budgets_arr {
        let failure_outputs: FailureAnalysisOutputs = count_failure_typed(cost_data, input_budgets);
        final_chances.push(failure_outputs.final_chance);
        typed_fail_counters.push(failure_outputs.typed_fail_counter_final);
    }

    (final_chances, typed_fail_counters)
}

fn prep_histogram(
    upgrade_arr: &mut Vec<Upgrade>,
    valid_weapon_values: bool,
    valid_armor_values: bool,
    cost_data: &[Vec<i64>],
    hist_bins: usize,
    unlock_costs: &[i64],
) -> HistogramOutputs {
    let upgrade_strings: Vec<String> = compress_runs(
        extract_upgrade_strings(upgrade_arr, valid_weapon_values, valid_armor_values),
        true,
    );

    let top_bottom: Vec<Vec<i64>> = get_top_bottom(upgrade_arr, unlock_costs);

    let bins: usize = hist_bins.min(BUCKET_COUNT).max(1);
    let hist_maxs: Vec<i64> = top_bottom[1].clone();
    let hist_counts: Vec<Vec<i64>> = histograms_for_all_costs(cost_data, bins, &hist_maxs);

    HistogramOutputs {
        upgrade_strings,
        // top_bottom,
        hist_counts,
        hist_mins: vec![0_i64; 7],
        hist_maxs,
    }
}

pub fn cost_to_chance<R: rand::Rng>(
    hone_counts: &[Vec<i64>],
    input_budgets: &[i64],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    hist_bins: usize,
    user_mats_value: &[f64],
    adv_hone_strategy: String,
    data_size: usize,
    rng: &mut R,
) -> CostToChanceOut {
    // Section 1: Preparation - setup and parsing
    let mut prep_outputs: PreparationOutputs = preparation(
        hone_counts,
        input_budgets,
        adv_counts,
        express_event,
        user_mats_value,
        &adv_hone_strategy,
    );

    // Section 2: Monte Carlo simulation
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &mut prep_outputs.upgrade_arr,
        &prep_outputs.unlock_costs,
        input_budgets[9],
        rng,
    );

    // Section 3: Failure analysis
    let failure_outputs: FailureAnalysisOutputs = count_failure_typed(&cost_data, input_budgets);

    // Section 4: Histogram preparation
    let histogram_outputs: HistogramOutputs = prep_histogram(
        &mut prep_outputs.upgrade_arr,
        prep_outputs.valid_weapon_values,
        prep_outputs.valid_armor_values,
        &cost_data,
        hist_bins,
        &prep_outputs.unlock_costs,
    );

    CostToChanceOut {
        chance: failure_outputs.final_chance,
        reasons: fail_count_to_rates(failure_outputs.typed_fail_counter_final, data_size),
        hist_counts: histogram_outputs.hist_counts,
        hist_mins: histogram_outputs.hist_mins,
        hist_maxs: histogram_outputs.hist_maxs,
        upgrade_strings: histogram_outputs.upgrade_strings,
        juice_strings_armor: prep_outputs.juice_strings_armor,
        juice_strings_weapon: prep_outputs.juice_strings_weapon,
        budgets_red_remaining: prep_outputs.budgets[7],
        budgets_blue_remaining: prep_outputs.budgets[8],
    }
}

pub fn cost_to_chance_arr<R: rand::Rng>(
    hone_counts: &[Vec<i64>],
    input_budgets_arr: &[Vec<i64>],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    user_mats_value: &[f64],
    adv_hone_strategy: String,
    data_size: usize,
    rng: &mut R,
) -> (Vec<f64>, Vec<Vec<f64>>, i64, i64) {
    // Section 1: Preparation - setup and parsing (only run once with first budget)
    let first_budget: &Vec<i64> = &input_budgets_arr[0];
    let mut prep_outputs: PreparationOutputs = preparation(
        hone_counts,
        first_budget,
        adv_counts,
        express_event,
        user_mats_value,
        &adv_hone_strategy,
    );

    // Section 2: Monte Carlo simulation (only run once with first budget's special)
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &mut prep_outputs.upgrade_arr,
        &prep_outputs.unlock_costs,
        first_budget[9], // Use first budget's special leap count
        rng,
    );

    // Section 3: Failure analysis for all budgets
    let (final_chances, typed_fail_counters): (Vec<f64>, Vec<Vec<f64>>) =
        count_failure_typed_arr(&cost_data, input_budgets_arr);

    // Return only the required data: chances, failure counters, and remaining budgets
    (
        final_chances,
        typed_fail_counters,
        prep_outputs.budgets[7], // budgets_red_remaining
        prep_outputs.budgets[8], // budgets_blue_remaining
    )
}

pub fn cost_to_chance_optimized<R: rand::Rng>(
    hone_counts: &[Vec<i64>],
    input_budgets: &[i64],
    adv_counts: &[Vec<i64>],
    express_event: bool,
    hist_bins: usize,
    user_mats_value: &[f64],
    adv_hone_strategy: String,
    data_size: usize,
    rng: &mut R,
) -> CostToChanceOptimizedOut {
    // Section 1: Preparation - setup and parsing
    let mut prep_outputs: PreparationOutputs = preparation(
        hone_counts,
        input_budgets,
        adv_counts,
        express_event,
        user_mats_value,
        &adv_hone_strategy,
    );

    // Section 2: Monte Carlo simulation
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &mut prep_outputs.upgrade_arr,
        &prep_outputs.unlock_costs,
        input_budgets[9],
        rng,
    );

    let failure_outputs_initial: FailureAnalysisOutputs =
        count_failure_typed(&cost_data, &input_budgets);

    let thresholds: Vec<Vec<i64>> = generate_budget_data(&cost_data, 1000, data_size);
    let top_bottom: Vec<Vec<i64>> =
        get_top_bottom(&prep_outputs.upgrade_arr, &prep_outputs.unlock_costs);
    let bitset_bundle: BitsetBundle =
        generate_bit_sets(&cost_data, thresholds, &top_bottom[1].clone(), data_size);
    let (optimized_budget, optimized_chance): (Vec<i64>, f64) = beam_search(
        &bitset_bundle,
        &prep_outputs.mats_value,
        &input_budgets,
        rng,
        16,
        12,
        30,
    );

    // Section 3: Failure analysis
    let failure_outputs_optimized: FailureAnalysisOutputs =
        count_failure_typed(&cost_data, &optimized_budget);
    dbg!(optimized_chance, failure_outputs_optimized.final_chance);
    dbg!(&optimized_budget);

    // Section 4: Histogram preparation
    let histogram_outputs: HistogramOutputs = prep_histogram(
        &mut prep_outputs.upgrade_arr,
        prep_outputs.valid_weapon_values,
        prep_outputs.valid_armor_values,
        &cost_data,
        hist_bins,
        &prep_outputs.unlock_costs,
    );

    CostToChanceOptimizedOut {
        chance: failure_outputs_initial.final_chance,
        reasons: fail_count_to_rates(failure_outputs_initial.typed_fail_counter_final, data_size),
        hist_counts: histogram_outputs.hist_counts,
        hist_mins: histogram_outputs.hist_mins,
        hist_maxs: histogram_outputs.hist_maxs,
        upgrade_strings: histogram_outputs.upgrade_strings,
        juice_strings_armor: prep_outputs.juice_strings_armor,
        juice_strings_weapon: prep_outputs.juice_strings_weapon,
        budgets_red_remaining: prep_outputs.budgets[7],
        budgets_blue_remaining: prep_outputs.budgets[8],
        buy_arr: optimized_budget,
        optimized_chance: failure_outputs_optimized.final_chance,
        optimized_reasons: fail_count_to_rates(
            failure_outputs_optimized.typed_fail_counter_final,
            data_size,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::test_cache::{read_cached_data, write_cached_data};
    use rand::prelude::*;

    #[test]
    fn cost_to_chance_18_demo_optimized() {
        let test_name: &str = "cost_to_chance_18_demo_optimized";
        let hone_counts = vec![
            (0..25)
                .map(|i| if i == 19 || i == 20 || i == 21 { 5 } else { 0 })
                .collect(),
            (0..25).map(|i| if i >= 19 { 1 } else { 0 }).collect(),
        ];
        let input_budgets = vec![
            631777, 1064398, 33748, 12010948, 25125, 3803792, 999999999, 1420, 690, 6767,
        ];
        let adv_counts = vec![
            (0..4).map(|i| if i == 3 { 3 } else { 0 }).collect(),
            (0..4).map(|i| if i == 2 { 0 } else { 0 }).collect(),
        ];
        let express_event = false;
        let hist_bins: usize = 1000;
        let user_mats_value = DEFAULT_GOLD_VALUES.to_vec();
        let adv_hone_strategy = "No juice";
        let data_size: usize = 100000;

        let hash = calculate_hash!(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &adv_hone_strategy,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result: CostToChanceOptimizedOut = cost_to_chance_optimized(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &user_mats_value,
            adv_hone_strategy.to_owned(),
            data_size,
            &mut rng,
        );

        let result_of_interst: f64 = result.optimized_chance;
        if let Some(cached_result) = read_cached_data::<f64>(test_name, &hash) {
            assert_float_eq::assert_f64_near!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
    }

    #[test]
    fn cost_to_chance_stress() {
        let test_name: &str = "cost_to_chance_stress";
        let hone_counts: Vec<Vec<i64>> =
            vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];

        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()];
        let input_budgets: Vec<i64> = vec![
            431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
        ];
        let express_event: bool = false;
        let hist_bins: usize = 1000;
        let user_mats_value: Vec<f64> = vec![0.0; 7];
        let adv_hone_strategy: &'static str = "No juice";
        let data_size: usize = 10000;

        let hash = calculate_hash!(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &adv_hone_strategy,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result: CostToChanceOut = cost_to_chance(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &user_mats_value,
            adv_hone_strategy.to_owned(),
            data_size,
            &mut rng,
        );

        let result_of_interst: f64 = result.chance;
        if let Some(cached_result) = read_cached_data::<f64>(test_name, &hash) {
            assert_eq!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
    }
    #[test]
    fn cost_to_chance_18_demo() {
        let test_name: &str = "cost_to_chance_18_demo";
        let hone_counts = vec![
            (0..25)
                .map(|i| if i == 19 || i == 20 || i == 21 { 5 } else { 0 })
                .collect(),
            (0..25).map(|i| if i >= 19 { 1 } else { 0 }).collect(),
        ];
        let input_budgets = vec![
            631777, 1064398, 33748, 12010948, 25125, 3803792, 999999999, 1420, 690, 6767,
        ];
        let adv_counts = vec![
            (0..4).map(|i| if i == 3 { 3 } else { 0 }).collect(),
            (0..4).map(|i| if i == 2 { 0 } else { 0 }).collect(),
        ];
        let express_event = false;
        let hist_bins: usize = 1000;
        let user_mats_value = vec![0.0; 7];
        let adv_hone_strategy = "No juice";
        let data_size: usize = 100000;

        let hash = calculate_hash!(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &adv_hone_strategy,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result: CostToChanceOut = cost_to_chance(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &user_mats_value,
            adv_hone_strategy.to_owned(),
            data_size,
            &mut rng,
        );

        let result_of_interst: f64 = result.chance;
        if let Some(cached_result) = read_cached_data::<f64>(test_name, &hash) {
            assert_eq!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
    }
    #[test]
    fn cost_to_chance_50_normal_weapon_25() {
        let test_name: &str = "cost_to_chance_50_normal_weapon_25";
        let hone_counts = vec![
            (0..25).map(|_| 0).collect(),
            (0..25).map(|i| if i == 24 { 1 } else { 0 }).collect(),
        ];
        let input_budgets = vec![324000, 0, 4680, 1774000, 3600, 406800, 10800000, 0, 0, 0];
        let adv_counts = vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];
        let express_event = false;
        let hist_bins: usize = 1000;
        let user_mats_value = vec![0.0; 7];
        let adv_hone_strategy = "No juice";
        let data_size: usize = 100000;

        let hash = calculate_hash!(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &adv_hone_strategy,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result: CostToChanceOut = cost_to_chance(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &user_mats_value,
            adv_hone_strategy.to_owned(),
            data_size,
            &mut rng,
        );

        let result_of_interst: f64 = result.chance;
        if let Some(cached_result) = read_cached_data::<f64>(test_name, &hash) {
            assert_eq!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
    }
    #[test]
    fn cost_to_chance_53_adv_armor_40() {
        let test_name: &str = "cost_to_chance_53_adv_armor_40";
        let hone_counts = vec![(0..25).map(|_| 0).collect(), (0..25).map(|_| 0).collect()];
        let input_budgets = vec![0, 63600, 1219, 564000, 1007, 127200, 5003000, 0, 0, 0];
        let adv_counts = vec![
            (0..4).map(|x| if x == 3 { 1 } else { 0 }).collect(),
            (0..4).map(|_| 0).collect(),
        ];
        let express_event = false;
        let hist_bins: usize = 1000;
        let user_mats_value = vec![0.0; 7];
        let adv_hone_strategy = "No juice";
        let data_size: usize = 100000;

        let hash = calculate_hash!(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &adv_hone_strategy,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result: CostToChanceOut = cost_to_chance(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            hist_bins,
            &user_mats_value,
            adv_hone_strategy.to_owned(),
            data_size,
            &mut rng,
        );

        let result_of_interst: f64 = result.chance;
        if let Some(cached_result) = read_cached_data::<f64>(test_name, &hash) {
            assert_eq!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
    }

    #[test]
    fn cost_to_chance_arr_test() {
        let test_name: &str = "cost_to_chance_arr_test";
        let budget_arr = vec![
            vec![
                431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
            ],
            vec![
                431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
            ],
        ];

        let hone_counts = vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
        let adv_counts = vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()];
        let express_event = false;
        let user_mats_value = vec![0.0; 7];
        let adv_hone_strategy = "No juice";
        let data_size: usize = 100000;

        let hash = calculate_hash!(
            &hone_counts,
            &budget_arr,
            &adv_counts,
            express_event,
            &adv_hone_strategy,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result = cost_to_chance_arr(
            &hone_counts,
            &budget_arr,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy.to_owned(),
            data_size,
            &mut rng,
        );

        let result_of_interst: Vec<f64> = result.0;
        if let Some(cached_result) = read_cached_data::<Vec<f64>>(test_name, &hash) {
            assert_eq!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
    }
}
