// use crate::cost_to_chance::compute_all_gold_costs;
use crate::helpers::{average_juice_cost, calc_unlock, count_failure};
use crate::histogram::histograms_for_all_costs;
use crate::monte_carlo::{generate_budget_data, get_top_bottom, monte_carlo_data};
use crate::parser::{Upgrade, parser};
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

    let upgrade_arr: Vec<Upgrade> = parser(
        hone_counts,
        adv_counts,
        &adv_hone_strategy.to_string(),
        express_event,
    );

    let unlock_cost: Vec<i64> = calc_unlock(hone_counts, adv_counts, express_event);
    let cost_data: Vec<[i64; 9]> = monte_carlo_data(data_size, &upgrade_arr, &unlock_cost, 0, rng);
    let top_bottom: Vec<Vec<i64>> = get_top_bottom(&upgrade_arr, &unlock_cost);

    let mut budget_data: Vec<Vec<i64>> = generate_budget_data(&cost_data, &[0_i64; 7], budget_size);
    budget_data.push(top_bottom[1].clone());

    if adv_hone_strategy == "Juice on grace" {
        let (avg_red_juice, avg_blue_juice) = average_juice_cost(&upgrade_arr);

        for budget_row in &mut budget_data {
            budget_row[7] = avg_red_juice;
            budget_row[8] = avg_blue_juice;
        }
    }
    // let all_gold_costs: Vec<f64> = compute_all_gold_costs(&vec![0.0; 7], &cost_data, &prep_outputs);
    let failure_counts: Vec<i64> = count_failure(&cost_data, &budget_data, true);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::RNG_SEED;
    use crate::test_utils::*;
    use crate::{calculate_hash, my_assert};
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

        if let Some(cached_result) = read_cached_data::<ChanceToCostOut>(test_name, &hash) {
            my_assert!(result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result);
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
        dbg!(&result.hundred_chances);

        if let Some(cached_result) = read_cached_data::<ChanceToCostOut>(test_name, &hash) {
            my_assert!(result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result);
        }
        // implement test here
    }
}
