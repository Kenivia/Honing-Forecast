// use crate::helpers::{compute_gold_cost_from_raw, get_percentile_window};
// use crate::parser::PreparationOutput;
// #[derive(Debug)]
// pub struct NoBuyAnalysisOutputs {
//     pub typed_fail_counter_final: Vec<f64>,
//     pub no_buy_chance: f64,
// }

// #[derive(Debug)]
// pub struct BuyAnalysisOutput {
//     pub hundred_gold_costs: Vec<i64>,
//     pub buy_chance: f64,
// }

// pub fn no_buy_analysis(cost_data: &[[i64; 9]], input_budgets: &[i64]) -> NoBuyAnalysisOutputs {
//     let mut typed_fail_counter_final: Vec<f64> = vec![0.0_f64; 7];
//     let mut overall_fail_counter: i64 = 0;
//     let mut failed: bool;
//     for data in cost_data {
//         failed = false;
//         for cost_type in 0..7 {
//             if input_budgets[cost_type] < data[cost_type] {
//                 failed = true;
//                 typed_fail_counter_final[cost_type] += 1.0_f64;
//             }
//         }
//         if failed {
//             overall_fail_counter += 1;
//         }
//     }

//     let no_buy_chance: f64 = 1.0_f64 - overall_fail_counter as f64 / cost_data.len() as f64;
//     for cost_type in 0..7 {
//         typed_fail_counter_final[cost_type] =
//             1.0 - typed_fail_counter_final[cost_type] / cost_data.len() as f64;
//     }
//     NoBuyAnalysisOutputs {
//         typed_fail_counter_final,
//         no_buy_chance,
//     }
// }

// pub fn buy_analysis(
//     input_budgets: &[i64],
//     cost_data_to_sort: &mut [[i64; 9]],
//     prep_output: &PreparationOutput,
// ) -> BuyAnalysisOutput {
//     let all_gold_costs: Vec<f64> =
//         compute_all_gold_costs_and_sort_data(input_budgets, cost_data_to_sort, &prep_output);
//     let hundred_gold_costs: Vec<i64> = get_hundred_gold_costs(
//         &all_gold_costs,
//         cost_data_to_sort,
//         &prep_output,
//         input_budgets,
//     );
//     let mut buy_chance: f64 = 1.0;
//     for (index, gold) in all_gold_costs.iter().enumerate() {
//         if *gold > input_budgets[5] as f64 {
//             buy_chance = index as f64 / cost_data_to_sort.len() as f64; // intentionally not subtracting by 1 because index starts from 0
//             break;
//         }
//     }
//     BuyAnalysisOutput {
//         hundred_gold_costs,
//         buy_chance,
//     }
// }

// pub fn compute_all_gold_costs(
//     input_budgets: &[i64],
//     cost_data: &[[i64; 9]],
//     prep_output: &PreparationOutput,
// ) -> Vec<f64> {
//     let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
//     input_budget_no_gold[5] = 0;
//     let mut all_gold_costs: Vec<f64> = Vec::with_capacity(cost_data.len());
//     for cost in cost_data.iter() {
//         all_gold_costs.push(compute_gold_cost_from_raw(
//             cost,
//             &input_budget_no_gold,
//             &prep_output.price_arr,
//         ));
//     }
//     all_gold_costs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
//     all_gold_costs
// }
// pub fn compute_all_gold_costs_and_sort_data(
//     input_budgets: &[i64],
//     cost_data: &mut [[i64; 9]],
//     prep_output: &PreparationOutput,
// ) -> Vec<f64> {
//     let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
//     input_budget_no_gold[5] = 0;
//     let mut all_gold_costs: Vec<f64> = Vec::with_capacity(cost_data.len());

//     for cost in cost_data.iter() {
//         all_gold_costs.push(compute_gold_cost_from_raw(
//             cost,
//             &input_budget_no_gold,
//             &prep_output.price_arr,
//         ));
//     }

//     // Create indices paired with gold costs
//     let mut indices: Vec<usize> = (0..cost_data.len()).collect();
//     indices.sort_unstable_by(|&a, &b| {
//         all_gold_costs[a]
//             .partial_cmp(&all_gold_costs[b])
//             .unwrap_or(std::cmp::Ordering::Equal)
//     });

//     let temp_cost_data: Vec<[i64; 9]> = cost_data.to_vec();
//     for (index, cost) in cost_data.iter_mut().enumerate() {
//         *cost = temp_cost_data[indices[index]];
//     }
//     // Sort all_gold_costs to match
//     all_gold_costs = indices.iter().map(|&i| all_gold_costs[i]).collect(); //.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
//     all_gold_costs
// }

// fn get_hundred_gold_costs(
//     all_gold_costs: &[f64],
//     cost_data: &[[i64; 9]],
//     prep_output: &PreparationOutput,
//     input_budgets: &[i64],
// ) -> Vec<i64> {
//     let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
//     input_budget_no_gold[5] = 0;
//     let mut hundred_gold_costs: Vec<i64> = Vec::with_capacity(101);
//     for i in 0..100_usize {
//         hundred_gold_costs.push(
//             all_gold_costs[(i as f64 * cost_data.len() as f64 / 100_f64).ceil() as usize].ceil()
//                 as i64,
//         );
//     }
//     hundred_gold_costs.push(
//         compute_gold_cost_from_raw(
//             &prep_output.pity(),
//             &input_budget_no_gold,
//             &prep_output.price_arr,
//         )
//         .ceil() as i64,
//     );
//     hundred_gold_costs
// }

// fn one_typical_cost(
//     cost_data_sorted: &[[i64; 9]],
//     desired_chance: f64,
//     price_arr: &[f64],
//     input_budget_no_gold: &[i64],
//     target_gold: i64,
// ) -> [i64; 9] {
//     let relevant_data: &[[i64; 9]] = get_percentile_window(desired_chance, cost_data_sorted);
//     let mut average: Vec<f64> = vec![0.0; 9];
//     for data in relevant_data {
//         for i in 0..7 {
//             if i != 5 {
//                 average[i] += data[i] as f64;
//             }
//         }
//     }
//     for i in 0..7 {
//         average[i] /= relevant_data.len() as f64;
//     }

//     let mut out: [i64; 9] = average
//         .iter()
//         .map(|x| x.round() as i64)
//         .collect::<Vec<i64>>()
//         .try_into()
//         .unwrap();

//     let gold_cost_of_average: f64 =
//         compute_gold_cost_from_raw(&out, input_budget_no_gold, price_arr);
//     let mut modified_gold_costs: Vec<f64> = Vec::with_capacity(cost_data_sorted.len());
//     for cost in cost_data_sorted {
//         modified_gold_costs.push(compute_gold_cost_from_raw(cost, &out, price_arr));
//     }
//     modified_gold_costs
//         .sort_unstable_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
//     let mut current_success_chance: f64 = 1.0;
//     for (index, g) in modified_gold_costs.iter().enumerate() {
//         if *g > gold_cost_of_average - target_gold as f64 {
//             current_success_chance =
//                 index.saturating_sub(1) as f64 / modified_gold_costs.len() as f64;
//             break;
//         }
//     }

//     if current_success_chance < (desired_chance - 0.005).max(0.0) {
//         let needed_gold_for_modified: f64 = modified_gold_costs
//             [((desired_chance) * modified_gold_costs.len() as f64).ceil() as usize];
//         out[5] += needed_gold_for_modified.round() as i64;
//     }

//     out
// }

// pub fn generate_typical_cost(
//     input_budgets: &[i64],
//     cost_data_sorted: &[[i64; 9]],
//     prep_output: &PreparationOutput,
//     buy_failure_outputs: &BuyAnalysisOutput,
// ) -> Vec<[i64; 9]> {
//     let mut typical_costs: Vec<[i64; 9]> = Vec::with_capacity(101);
//     let mut input_budget_no_gold: Vec<i64> = input_budgets.to_vec();
//     input_budget_no_gold[5] = 0;
//     for i in 0..100 {
//         typical_costs.push(one_typical_cost(
//             cost_data_sorted,
//             i as f64 / 100.0,
//             &prep_output.price_arr,
//             &input_budget_no_gold,
//             buy_failure_outputs.hundred_gold_costs[i],
//         ));
//     }
//     typical_costs.push(prep_output.pity().try_into().unwrap());
//     typical_costs
// }
