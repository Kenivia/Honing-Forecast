// use core::f64;
// #[cfg(test)]
// use std::time::Instant;

// use crate::histogram::{HistogramOutputs, prep_histogram};
// use crate::success_analysis::{
//     BuyAnalysisOutput, NoBuyAnalysisOutputs, buy_analysis, no_buy_analysis,
// };
// // use hf_core::upgrade::Upgrade;
// use hf_core::state_bundle::StateBundle;
// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug)]
// pub struct CostToChanceOut {
//     pub chance: f64,       // no buy chance
//     pub reasons: Vec<f64>, // 7 failure rates for each cost type(no buy)

//     pub hundred_gold_costs: Vec<i64>, // gold cost for each percentage
//     pub chance_if_buy: f64,
//     pub typical_costs: Vec<[i64; 9]>,

//     pub hist_counts: Vec<Vec<i64>>, // 7 x num_bins
//     pub hist_mins: Vec<i64>,        // 7
//     pub hist_maxs: Vec<i64>,        // 7

//     pub budgets_red_remaining: i64, // budgets[7], these are just here to provide warning for when there's not enough juice for advanced honing
//     pub budgets_blue_remaining: i64, // budgets[8]
// }
// #[derive(Serialize, Deserialize, Debug)]
// pub struct CostToChanceArrOut {
//     pub buy_chance_arr: Vec<f64>,

//     pub no_buy_chance_arr: Vec<f64>,
//     pub typed_fail_counters: Vec<Vec<f64>>,

//     pub budgets_red_remaining: i64,
//     pub budgets_blue_remaining: i64,
// }

// pub fn cost_to_chance(state_bundle: &StateBundle) -> CostToChanceOut {
//     let no_buy_failure_outputs: NoBuyAnalysisOutputs = no_buy_analysis(&state_bundle);

//     let buy_failure_outputs: BuyAnalysisOutput = buy_analysis(&state_bundle);

//     let typical_costs: Vec<[i64; 9]> = vec![[-67; 9]; 101]; // i dont think this is gonna work

//     // Section 4: Histogram preparation
//     let histogram_outputs: HistogramOutputs = prep_histogram(&state_bundle, 100, 100_000);

//     CostToChanceOut {
//         chance: no_buy_failure_outputs.no_buy_chance,
//         reasons: no_buy_failure_outputs.typed_fail_counter_final,
//         hist_counts: histogram_outputs.hist_counts,
//         hist_mins: histogram_outputs.hist_mins,
//         hist_maxs: histogram_outputs.hist_maxs,

//         budgets_red_remaining: 0,
//         budgets_blue_remaining: 0,
//         hundred_gold_costs: buy_failure_outputs.hundred_gold_costs,
//         chance_if_buy: buy_failure_outputs.buy_chance,
//         typical_costs,
//     }
// }

// /// Same as cost_to_chance, but repeats it over projected budgets
// pub fn cost_to_chance_arr(
//     hone_counts: &[Vec<i64>],
//     input_budgets_arr: &[Vec<i64>],
//     adv_counts: &[Vec<i64>],
//     express_event: bool,
//     user_price_arr: &[f64],
//     adv_hone_strategy: String,
//     cost_data: &[[i64; 9]],
// ) -> CostToChanceArrOut {
//     let first_budget: &Vec<i64> = &input_budgets_arr[0];
//     let prep_output: PreparationOutput = PreparationOutput::initialize(
//         hone_counts,
//         first_budget,
//         adv_counts,
//         express_event,
//         user_price_arr,
//         &adv_hone_strategy,
//         &vec![], // TODO fix this later
//         &vec![], // TODO fix this later
//         &vec![], // TODO fix this later
//         &vec![], // TODO fix this later
//     );

//     // No buy analysis
//     let mut no_buy_chance_arr: Vec<f64> = Vec::new();
//     let mut typed_fail_counters: Vec<Vec<f64>> = Vec::new();

//     for input_budgets in input_budgets_arr {
//         let failure_outputs: NoBuyAnalysisOutputs = no_buy_analysis(cost_data, input_budgets);
//         no_buy_chance_arr.push(failure_outputs.no_buy_chance);
//         typed_fail_counters.push(failure_outputs.typed_fail_counter_final);
//     }

//     // Buy analysis
//     let mut buy_chance_arr: Vec<f64> = Vec::with_capacity(input_budgets_arr.len());
//     let mut all_gold_costs: Vec<f64>;
//     for budget in input_budgets_arr {
//         all_gold_costs = compute_all_gold_costs(budget, cost_data, &prep_output);
//         let mut count: f64 = 0.0;
//         for gold in all_gold_costs.iter() {
//             if *gold <= budget[5] as f64 {
//                 count += 1.0;
//             }
//         }

//         buy_chance_arr.push(count / all_gold_costs.len() as f64);
//     }
//     CostToChanceArrOut {
//         no_buy_chance_arr: no_buy_chance_arr,
//         typed_fail_counters,
//         budgets_red_remaining: prep_output.budgets[7],
//         budgets_blue_remaining: prep_output.budgets[8],
//         buy_chance_arr: buy_chance_arr,
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::constants::RNG_SEED;
//     use crate::monte_carlo::monte_carlo_data;
//     use crate::test_utils::{DEFAULT_GOLD_VALUES, read_cached_data, write_cached_data};
//     use crate::{calculate_hash, my_assert};
//     use rand::prelude::*;

//     #[test]
//     fn cost_to_chance_stress() {
//         let test_name: &str = "cost_to_chance_stress";
//         let hone_counts: Vec<Vec<i64>> =
//             vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];

//         let adv_counts: Vec<Vec<i64>> =
//             vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()];
//         let input_budgets: Vec<i64> = vec![
//             431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
//         ];
//         let express_event: bool = false;
//         let hist_bins: usize = 1000;
//         let user_price_arr: [f64; 9] = DEFAULT_GOLD_VALUES;
//         let adv_hone_strategy: &'static str = "x2 grace";
//         let data_size: usize = 100000;

//         let hash: String = calculate_hash!(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &adv_hone_strategy,
//             data_size
//         );
//         // Run the function to get the full output
//         let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
//         let prep_output: PreparationOutput = PreparationOutput::initialize(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             &user_price_arr,
//             adv_hone_strategy,
//             &vec![],
//             &vec![],
//             &vec![], // TODO fix this later
//             &vec![], // TODO fix this later
//         );
//         let mut cost_data = monte_carlo_data(
//             data_size,
//             &upgrade_arr,
//             &prep_output.unlock_costs,
//             input_budgets[9],
//             &mut rng,
//         );
//         let result: CostToChanceOut = cost_to_chance(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &user_price_arr,
//             adv_hone_strategy.to_owned(),
//             &mut cost_data,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }
//     #[test]
//     fn cost_to_chance_18_demo() {
//         let test_name: &str = "cost_to_chance_18_demo";
//         let hone_counts = vec![
//             (0..25)
//                 .map(|i| if i == 19 || i == 20 || i == 21 { 5 } else { 0 })
//                 .collect(),
//             (0..25).map(|i| if i >= 19 { 1 } else { 0 }).collect(),
//         ];
//         let input_budgets = vec![
//             631777, 1064398, 33748, 12010948, 25125, 3803792, 999999999, 1420, 690, 6767,
//         ];
//         let adv_counts = vec![
//             (0..4).map(|i| if i == 3 { 3 } else { 0 }).collect(),
//             (0..4).map(|i| if i == 2 { 0 } else { 0 }).collect(),
//         ];
//         let express_event = false;
//         let hist_bins: usize = 1000;
//         let user_price_arr = DEFAULT_GOLD_VALUES;
//         let adv_hone_strategy = "x2 grace";
//         let data_size: usize = 100000;

//         let hash = calculate_hash!(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &adv_hone_strategy,
//             data_size
//         );
//         let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
//         let prep_output: PreparationOutput = PreparationOutput::initialize(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             &user_price_arr,
//             adv_hone_strategy,
//             &vec![],
//             &vec![],
//             &vec![], // TODO fix this later
//             &vec![], // TODO fix this later
//         );
//         let mut cost_data = monte_carlo_data(
//             data_size,
//             &upgrade_arr,
//             &prep_output.unlock_costs,
//             input_budgets[9],
//             &mut rng,
//         );
//         let result: CostToChanceOut = cost_to_chance(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &user_price_arr,
//             adv_hone_strategy.to_owned(),
//             &mut cost_data,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }
//     #[test]
//     fn cost_to_chance_50_normal_weapon_25() {
//         let test_name: &str = "cost_to_chance_50_normal_weapon_25";
//         let hone_counts = vec![
//             (0..25).map(|_| 0).collect(),
//             (0..25).map(|i| if i == 24 { 1 } else { 0 }).collect(),
//         ];
//         let input_budgets = vec![324000, 0, 4680, 1774000, 3600, 406800, 10800000, 0, 0, 0];
//         let adv_counts = vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];
//         let express_event = false;
//         let hist_bins: usize = 1000;
//         let user_price_arr: [f64; 9] = DEFAULT_GOLD_VALUES;
//         let adv_hone_strategy = "x2 grace";
//         let data_size: usize = 100000;

//         let hash = calculate_hash!(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &adv_hone_strategy,
//             data_size
//         );
//         let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
//         let prep_output: PreparationOutput = PreparationOutput::initialize(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             &user_price_arr,
//             adv_hone_strategy,
//             &vec![],
//             &vec![],
//             &vec![],
//             &vec![],
//         );
//         let mut cost_data = monte_carlo_data(
//             data_size,
//             &upgrade_arr,
//             &prep_output.unlock_costs,
//             input_budgets[9],
//             &mut rng,
//         );
//         let result: CostToChanceOut = cost_to_chance(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &user_price_arr,
//             adv_hone_strategy.to_owned(),
//             &mut cost_data,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }
//     #[test]
//     fn cost_to_chance_53_adv_armor_40() {
//         let test_name: &str = "cost_to_chance_53_adv_armor_40";
//         let hone_counts = vec![(0..25).map(|_| 0).collect(), (0..25).map(|_| 0).collect()];
//         let input_budgets = vec![0, 63600, 1219, 564000, 1007, 127200, 5003000, 0, 0, 0];
//         let adv_counts = vec![
//             (0..4).map(|x| if x == 3 { 1 } else { 0 }).collect(),
//             (0..4).map(|_| 0).collect(),
//         ];
//         let express_event = false;
//         let hist_bins: usize = 1000;
//         let user_price_arr = DEFAULT_GOLD_VALUES;
//         let adv_hone_strategy = "x2 grace";
//         let data_size: usize = 100000;

//         let hash = calculate_hash!(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &adv_hone_strategy,
//             data_size
//         );
//         let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
//         let prep_output: PreparationOutput = PreparationOutput::initialize(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             &user_price_arr,
//             adv_hone_strategy,
//             &vec![],
//             &vec![],
//             &vec![],
//             &vec![],
//         );
//         let mut cost_data = monte_carlo_data(
//             data_size,
//             &upgrade_arr,
//             &prep_output.unlock_costs,
//             input_budgets[9],
//             &mut rng,
//         );
//         let result: CostToChanceOut = cost_to_chance(
//             &hone_counts,
//             &input_budgets,
//             &adv_counts,
//             express_event,
//             hist_bins,
//             &user_price_arr,
//             adv_hone_strategy.to_owned(),
//             &mut cost_data,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }

//     #[test]
//     fn cost_to_chance_arr_test() {
//         let test_name: &str = "cost_to_chance_arr_test";
//         let budget_arr = vec![
//             vec![
//                 431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
//             ],
//             vec![
//                 431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 420, 690, 6767,
//             ],
//         ];

//         let hone_counts = vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
//         let adv_counts = vec![(0..4).map(|_| 5).collect(), (0..4).map(|_| 1).collect()];
//         let express_event = false;
//         let user_price_arr = DEFAULT_GOLD_VALUES;
//         let adv_hone_strategy = "x2 grace";
//         let data_size: usize = 100000;

//         let hash = calculate_hash!(
//             &hone_counts,
//             &budget_arr,
//             &adv_counts,
//             express_event,
//             &adv_hone_strategy,
//             data_size
//         );
//         // Run the function to get the full output
//         let mut rng: StdRng = StdRng::seed_from_u64(RNG_SEED);
//         let prep_output: PreparationOutput = PreparationOutput::initialize(
//             &hone_counts,
//             &budget_arr[0],
//             &adv_counts,
//             express_event,
//             &user_price_arr,
//             adv_hone_strategy,
//             &vec![],
//             &vec![],
//             &vec![],
//             &vec![],
//         );
//         let mut cost_data = monte_carlo_data(
//             data_size,
//             &upgrade_arr,
//             &prep_output.unlock_costs,
//             budget_arr[0][9],
//             &mut rng,
//         );
//         let result: CostToChanceArrOut = cost_to_chance_arr(
//             &hone_counts,
//             &budget_arr,
//             &adv_counts,
//             express_event,
//             &user_price_arr,
//             adv_hone_strategy.to_owned(),
//             &mut cost_data,
//         );

//         if let Some(cached_result) = read_cached_data::<CostToChanceArrOut>(test_name, &hash) {
//             my_assert!(result, cached_result);
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }
// }
