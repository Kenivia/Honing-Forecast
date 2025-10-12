use crate::bitset::{
    BitsetBundle, beam_search, compute_gold_cost_from_indices, compute_gold_cost_from_raw,
    generate_bit_sets,
};
use crate::constants::EVENT_ARTISAN_MULTIPLIER;
use crate::helpers::{average_juice_cost, calc_unlock, count_failure};
use crate::histogram::histograms_for_all_costs;
use crate::monte_carlo::{generate_budget_data, get_top_bottom, monte_carlo_data};
use crate::parser::{PreparationOutputs, Upgrade, parser, preparation};
use serde::{Deserialize, Serialize};

// find the budget in budget_data that most closely matches desired_chance
fn find_best_budget_for_this_chance(
    desired_chance: f64,
    cost_size: usize,
    budget_size: usize,
    failure_counts: &[i64],
    budget_data: &[Vec<i64>],
    tiebreak_with_lowest: bool,
) -> (Vec<i64>, f64) {
    let k_i64: i64 = ((1.0 - desired_chance / 100.0) * cost_size as f64).floor() as i64;
    let k_i64_budget: i64 = (((cost_size as f64 - k_i64 as f64) / cost_size as f64)
        * budget_size as f64)
        .round() as i64;

    let diffs: Vec<i64> = failure_counts
        .iter()
        .map(|&ci| (ci - k_i64).abs())
        .collect();

    let best_index: usize = (0..budget_data.len())
        .min_by_key(|&i| {
            (
                diffs[i],
                if tiebreak_with_lowest {
                    i as i64
                } else {
                    (k_i64_budget - i as i64).abs()
                },
            )
        })
        .expect("budget_data should not be empty");

    let best_budget: Vec<i64> = budget_data[best_index].clone();
    let best_chance: f64 = (1.0 - failure_counts[best_index] as f64 / cost_size as f64) * 100.0;

    (best_budget, best_chance)
}

#[derive(Serialize, Deserialize)]
pub struct ChanceToCostOut {
    pub hist_counts: Vec<Vec<i64>>,     // 7 x num_bins
    pub hist_mins: Vec<i64>,            // 7
    pub hist_maxs: Vec<i64>,            // 7
    pub hundred_budgets: Vec<Vec<i64>>, // actually 101 length, 0 to 100%
    pub hundred_chances: Vec<f64>,      // actually 101 length, 0 to 100%
}

pub fn chance_to_cost<R: rand::Rng>(
    hone_counts: &[Vec<i64>],
    adv_counts: &[Vec<i64>],
    adv_hone_strategy: &str,
    express_event: bool,
    hist_bins: usize,
    data_size: usize,
    rng: &mut R,
) -> ChanceToCostOut {
    let budget_size: usize = 1000;
    let artisan_arr: Vec<f64> = if express_event {
        EVENT_ARTISAN_MULTIPLIER.to_vec()
    } else {
        vec![1.0; 25]
    };
    let upgrade_arr: Vec<Upgrade> = parser(
        hone_counts,
        adv_counts,
        &adv_hone_strategy.to_string(),
        &artisan_arr,
        &[0.0; 25],
        &[0; 25],
        express_event,
    );

    let unlock_cost: Vec<i64> = calc_unlock(hone_counts, adv_counts, express_event);
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(data_size, &upgrade_arr, &unlock_cost, 0, rng);
    let top_bottom: Vec<Vec<i64>> = get_top_bottom(&upgrade_arr, &unlock_cost);

    let mut budget_data: Vec<Vec<i64>> =
        generate_budget_data(&cost_data, &[0_i64; 7], budget_size);
    budget_data.push(top_bottom[1].clone());

    if adv_hone_strategy == "Juice on grace" {
        let (avg_red_juice, avg_blue_juice) = average_juice_cost(&upgrade_arr);

        for budget_row in &mut budget_data {
            budget_row[7] = avg_red_juice;
            budget_row[8] = avg_blue_juice;
        }
    }
    let failure_counts: Vec<i64> = count_failure(&cost_data, &budget_data, false);
    let (hundred_budgets, hundred_chances): (Vec<Vec<i64>>, Vec<f64>) = (0..101)
        .map(|x| {
            find_best_budget_for_this_chance(
                f64::from(x),
                data_size,
                budget_size,
                &failure_counts,
                &budget_data,
                false,
            )
        })
        .collect();
    ChanceToCostOut {
        hundred_budgets,
        hundred_chances,
        hist_counts: histograms_for_all_costs(&cost_data, hist_bins, &top_bottom[1]),
        hist_mins: vec![0_i64; 7],
        hist_maxs: top_bottom[1].clone(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChanceToCostOptimizedOut {
    pub hist_counts: Vec<Vec<i64>>,     // 7 x num_bins
    pub hist_mins: Vec<i64>,            // 7
    pub hist_maxs: Vec<i64>,            // 7
    pub hundred_budgets: Vec<Vec<i64>>, // actually 101 length, 0 to 100%
    pub hundred_chances: Vec<f64>,      // actually 101 length, 0 to 100%
    pub hundred_gold_costs: Vec<i64>,
}
pub fn chance_to_cost_optimized<R: rand::Rng>(
    hone_counts: &[Vec<i64>],
    adv_counts: &[Vec<i64>],
    adv_hone_strategy: &String,
    express_event: bool,
    hist_bins: usize,
    data_size: usize,
    rng: &mut R,
    input_budgets: &[i64],
    mats_value: &[f64],
) -> ChanceToCostOptimizedOut {
    let budget_size: usize = 1000;

    let mut prep_outputs: PreparationOutputs = preparation(
        hone_counts,
        input_budgets,
        adv_counts,
        express_event,
        mats_value,
        adv_hone_strategy,
    );

    // Section 2: Monte Carlo simulation (use outputs from preparation)
    let cost_data: Vec<Vec<i64>> = monte_carlo_data(
        data_size,
        &mut prep_outputs.upgrade_arr,
        &prep_outputs.unlock_costs,
        input_budgets[9],
        rng,
    );

    let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
    input_budget_no_gold[5] = 0;
    let thresholds: Vec<Vec<i64>> = generate_budget_data(&cost_data, &input_budget_no_gold, 1000);
    let top_bottom: Vec<Vec<i64>> =
        get_top_bottom(&prep_outputs.upgrade_arr, &prep_outputs.unlock_costs);
    let bitset_bundle: BitsetBundle = generate_bit_sets(
        &cost_data,
        thresholds,
        &top_bottom[1].clone(),
        data_size,
    );

    let pity_cost: f64 = compute_gold_cost_from_indices(
        &bitset_bundle.transposed_thresholds,
        &[bitset_bundle.transposed_thresholds[0].len() - 1; 7],
        &input_budget_no_gold,
        mats_value,
    );
    let resolution: usize = 300;
    let gap_size: f64 = (pity_cost - input_budgets[5] as f64) / resolution as f64;
    let mut budget_data: Vec<Vec<i64>> = Vec::with_capacity(resolution + 1);

    let mut new_input_budget: Vec<i64>;
    let mut prev_optimized: Vec<usize> = vec![]; // invalid on purpose
    for i in 0..resolution {
        new_input_budget = input_budgets.to_vec().clone();
        new_input_budget[5] = input_budgets[5] + (gap_size * i as f64).round() as i64;
        let (optimized_budget, _optimized_chance): (Vec<i64>, f64) = beam_search(
            &bitset_bundle,
            mats_value,
            &new_input_budget,
            rng,
            if i == 0 { 999 } else { 12 },
            &mut prev_optimized,
        );

        budget_data.push(optimized_budget);
    }

    budget_data.push(
        top_bottom[1]
            .iter()
            .zip(input_budgets.iter())
            .map(|(a, b)| a.max(b))
            .copied()
            .collect(),
    );

    if adv_hone_strategy == "Juice on grace" {
        let (avg_red_juice, avg_blue_juice) = average_juice_cost(&prep_outputs.upgrade_arr);

        for budget_row in &mut budget_data {
            budget_row[7] = avg_red_juice;
            budget_row[8] = avg_blue_juice;
        }
    }
    let failure_counts: Vec<i64> = count_failure(&cost_data, &budget_data, false);

    let (hundred_budgets, hundred_chances): (Vec<Vec<i64>>, Vec<f64>) = (0..101)
        .map(|x| {
            find_best_budget_for_this_chance(
                f64::from(x),
                data_size,
                budget_size,
                &failure_counts,
                &budget_data,
                true,
            )
        })
        .collect();
    let mut hundred_gold_costs: Vec<i64> = Vec::with_capacity(resolution + 1);
    for budget in &hundred_budgets {
        hundred_gold_costs.push(
            compute_gold_cost_from_raw(budget, &input_budget_no_gold, mats_value).ceil() as i64,
        );
    }

    ChanceToCostOptimizedOut {
        hundred_budgets,
        hundred_chances,
        hist_counts: histograms_for_all_costs(&cost_data, hist_bins, &top_bottom[1]),
        hist_mins: vec![0_i64; 7],
        hist_maxs: top_bottom[1].clone(),
        hundred_gold_costs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::test_cache::{read_cached_data, write_cached_data};
    use rand::prelude::*;

    #[test]
    fn chance_to_cost_all_normal() {
        let test_name: &str = "chance_to_cost_all_normal";
        let hone_counts: Vec<Vec<i64>> =
            vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let hist_bins: usize = 1000;
        let data_size: usize = 10000;

        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            hist_bins,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result: ChanceToCostOut = chance_to_cost(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            hist_bins,
            data_size,
            &mut rng,
        );
        let result_of_interst: Vec<Vec<i64>> = result.hundred_budgets.clone();
        if let Some(cached_result) = read_cached_data::<Vec<Vec<i64>>>(test_name, &hash) {
            assert_eq!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
    }

    #[test]
    fn chance_to_cost_25_weap() {
        let test_name: &str = "chance_to_cost_25_weap";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|_| 0).collect(),
            (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let hist_bins: usize = 1000;
        let data_size: usize = 10000;

        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            hist_bins,
            data_size
        );
        // Run the function to get the full output
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let result: ChanceToCostOut = chance_to_cost(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            hist_bins,
            data_size,
            &mut rng,
        );

        let result_of_interst: Vec<Vec<i64>> = result.hundred_budgets;
        if let Some(cached_result) = read_cached_data::<Vec<Vec<i64>>>(test_name, &hash) {
            assert_eq!(result_of_interst, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result_of_interst);
        }
        // implement test here
    }
}
