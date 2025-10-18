// use crate::calculate_hash;

// use crate::helpers::generate_first_deltas;
// use crate::parser::parser;
// use crate::test_cache::{read_cached_data, write_cached_data};
// use crate::{
//     parser::{Upgrade, probability_distribution},
//     value_estimation::average_tap,
// };
// use core::f64;
// /// Iterator that generates all arrangements of 0s and delta values
// pub struct BinaryArrangements {
//     length: usize,
//     ones: usize,
//     delta: f64,
//     positions: Vec<usize>,
//     done: bool,
// }

// impl BinaryArrangements {
//     /// Creates a new iterator for binary arrangements
//     ///
//     /// # Arguments
//     /// * `length` - Total length of the arrangement
//     /// * `ones` - Number of delta values in the arrangement
//     /// * `delta` - The value to use instead of 1s
//     ///
//     /// # Returns
//     /// An iterator that yields Vec<f64> containing 0.0s and delta values
//     pub fn new(length: usize, ones: usize, delta: f64) -> Self {
//         if ones > length {
//             return Self {
//                 length,
//                 ones,
//                 delta,
//                 positions: Vec::new(),
//                 done: false,
//             };
//         }

//         // Initialize with positions [0, 1, 2, ..., ones-1]
//         let positions: Vec<usize> = (0..ones).collect();

//         Self {
//             length,
//             ones,
//             delta,
//             positions,
//             done: ones == 0,
//         }
//     }

//     /// Generate the current arrangement from position indices
//     fn current_arrangement(&self) -> Vec<f64> {
//         let mut result = vec![0.0; self.length];
//         for &pos in &self.positions {
//             result[pos] = self.delta;
//         }
//         result
//     }

//     /// Advance to the next combination using the "next combination" algorithm
//     /// Returns false when no more combinations exist
//     fn advance(&mut self) -> bool {
//         if self.ones == 0 {
//             return false;
//         }

//         // Find the rightmost position that can be incremented
//         let mut i = self.ones;
//         while i > 0 {
//             i -= 1;
//             if self.positions[i] < self.length - self.ones + i {
//                 // Increment this position and reset all following positions
//                 self.positions[i] += 1;
//                 for j in (i + 1)..self.ones {
//                     self.positions[j] = self.positions[j - 1] + 1;
//                 }
//                 return true;
//             }
//         }

//         false
//     }
// }

// impl Iterator for BinaryArrangements {
//     type Item = Vec<f64>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.done {
//             return None;
//         }

//         let result = self.current_arrangement();

//         if !self.advance() {
//             self.done = true;
//         }

//         Some(result)
//     }
// }

// // Convenience function
// pub fn generate_arrangements(length: usize, ones: usize, delta: f64) -> BinaryArrangements {
//     BinaryArrangements::new(length, ones, delta)
// }
// struct BestJuiceOut {
//     arrangements: Vec<Vec<f64>>,
//     performance: Vec<f64>,
// }
// fn generate_best_juice(upgrade: &Upgrade) -> BestJuiceOut {
//     let mut arrangements: Vec<Vec<f64>> = Vec::new();
//     let mut performance: Vec<f64> = Vec::new();
//     let mut test_prob_dist: Vec<f64>;
//     let initial_avg: f64 = average_tap(&upgrade.prob_dist, upgrade.tap_offset as f64);
//     for i in 1..upgrade.prob_dist_len {
//         test_prob_dist = generate_first_deltas(upgrade.base_chance, upgrade.prob_dist_len, i);
//         let actual_dist_len: usize =
//             probability_distribution(upgrade.base_chance, upgrade.artisan_rate, &test_prob_dist)
//                 .len();
//         if actual_dist_len <= i {
//             break;
//         }
//         let mut best_change: f64 = f64::MAX;
//         let mut best_arr: Vec<f64> = Vec::new();
//         let mut this_arr: Vec<f64>;
//         for arra in generate_arrangements(actual_dist_len, i, upgrade.base_chance) {
//             // dbg!(&arra);
//             this_arr = probability_distribution(upgrade.base_chance, upgrade.artisan_rate, &arra);
//             let change_in_avg: f64 =
//                 average_tap(&this_arr, upgrade.tap_offset as f64) - initial_avg;
//             // dbg!(change_in_avg);
//             // since it's negative
//             if change_in_avg < best_change {
//                 best_arr = arra;
//                 best_change = change_in_avg;
//             }
//         }
//         dbg!(actual_dist_len);
//         arrangements.push(best_arr);
//         performance.push(best_change);
//     }
//     BestJuiceOut {
//         arrangements,
//         performance,
//     }
// }
// pub fn main() {
//     let test_name: &str = "generate_best_juice";
//     let hone_counts: Vec<Vec<i64>> = vec![
//         (0..25).map(|_| 0).collect(),
//         (0..25).map(|x| if x == 16 { 1 } else { 0 }).collect(),
//     ];
//     let adv_counts: Vec<Vec<i64>> = vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

//     let adv_hone_strategy: &str = "No juice";
//     let express_event: bool = false;

//     let hash: String = calculate_hash!(&hone_counts, &adv_counts, adv_hone_strategy, express_event);

//     let mut upgrade_arr = parser(
//         &hone_counts,
//         &adv_counts,
//         &adv_hone_strategy.to_string(),
//         express_event,
//     );

//     let juice_out: BestJuiceOut = generate_best_juice(&mut upgrade_arr[0]);
//     let mut arra: Vec<Vec<f64>> = juice_out.arrangements;
//     arra.extend_from_slice(&[juice_out.performance]);

//     let result: Vec<Vec<f64>> = arra;
//     if let Some(cached_result) = read_cached_data::<Vec<Vec<f64>>>(test_name, &hash) {
//         for (a, i) in result.iter().enumerate() {
//             for (b, z) in i.iter().enumerate() {
//                 my_assert!(*z, cached_result[a][b], 0.000000001);
//             }
//         }
//     } else {
//         write_cached_data(test_name, &hash, &result);
//     }
// }
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::calculate_hash;

//     use crate::parser::parser;
//     use crate::test_cache::{read_cached_data, write_cached_data};

//     #[test]
//     fn generate_best_juice_test() {
//         let test_name: &str = "generate_best_juice";
//         let hone_counts: Vec<Vec<i64>> = vec![
//             (0..25).map(|_| 0).collect(),
//             (0..25).map(|x| if x == 10 { 1 } else { 0 }).collect(),
//         ];
//         let adv_counts: Vec<Vec<i64>> =
//             vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

//         let adv_hone_strategy: &str = "No juice";
//         let express_event: bool = true;

//         let hash: String =
//             calculate_hash!(&hone_counts, &adv_counts, adv_hone_strategy, express_event);

//         let mut upgrade_arr = parser(
//             &hone_counts,
//             &adv_counts,
//             &adv_hone_strategy.to_string(),
//             express_event,
//         );

//         let juice_out: BestJuiceOut = generate_best_juice(&mut upgrade_arr[0]);
//         let mut arra: Vec<Vec<f64>> = juice_out.arrangements;
//         arra.extend_from_slice(&[juice_out.performance]);

//         let result: Vec<Vec<f64>> = arra;
//         if let Some(cached_result) = read_cached_data::<Vec<Vec<f64>>>(test_name, &hash) {
//             for (a, i) in result.iter().enumerate() {
//                 for (b, z) in i.iter().enumerate() {
//                     my_assert!(*z, cached_result[a][b], 0.00000001);
//                 }
//             }
//         } else {
//             write_cached_data(test_name, &hash, &result);
//         }
//     }

//     use assert_float_eq::assert_float_absolute_eq;

//     #[test]
//     fn test_basic_example() {
//         let delta = 0.5;
//         let arrangements: Vec<Vec<f64>> = generate_arrangements(3, 2, delta).collect();
//         my_assert!(arrangements.len(), 3);

//         // Check that we have the expected arrangements
//         let expected = vec![
//             vec![0.5, 0.5, 0.0],
//             vec![0.5, 0.0, 0.5],
//             vec![0.0, 0.5, 0.5],
//         ];

//         for exp in &expected {
//             assert!(arrangements.iter().any(|arr| {
//                 arr.len() == exp.len()
//                     && arr
//                         .iter()
//                         .zip(exp.iter())
//                         .all(|(a, b)| (a - b).abs() < 1e-10)
//             }));
//         }
//     }

//     #[test]
//     fn test_different_delta() {
//         let delta = 1.234;
//         let arrangements: Vec<Vec<f64>> = generate_arrangements(3, 2, delta).collect();

//         for arr in &arrangements {
//             let delta_count = arr.iter().filter(|&&x| (x - delta).abs() < 1e-10).count();
//             let zero_count = arr.iter().filter(|&&x| x.abs() < 1e-10).count();
//             my_assert!(delta_count, 2);
//             my_assert!(zero_count, 1);
//         }
//     }

//     #[test]
//     fn test_edge_cases() {
//         let delta = 0.7;

//         // All zeros
//         let arr: Vec<Vec<f64>> = generate_arrangements(3, 0, delta).collect();
//         my_assert!(arr.len(), 1);
//         for val in &arr[0] {
//             assert_float_absolute_eq!(*val, 0.0, 0.0000000001);
//         }

//         // All deltas
//         let arr: Vec<Vec<f64>> = generate_arrangements(3, 3, delta).collect();
//         my_assert!(arr.len(), 1);
//         for val in &arr[0] {
//             assert_float_absolute_eq!(*val, delta, 0.000000001);
//         }

//         // Invalid case
//         let arr: Vec<Vec<f64>> = generate_arrangements(3, 4, delta).collect();
//         my_assert!(arr.len(), 0);
//     }

//     #[test]
//     fn test_combination_count() {
//         let delta = 0.1;

//         // C(5, 2) = 10
//         let count = generate_arrangements(5, 2, delta).count();
//         my_assert!(count, 10);

//         // C(6, 3) = 20
//         let count = generate_arrangements(6, 3, delta).count();
//         my_assert!(count, 20);
//     }

//     #[test]
//     fn test_large_case() {
//         let delta = 2.5;
//         // Test that it works with larger numbers (not too large to avoid test timeout)
//         let count = generate_arrangements(20, 10, delta).count();
//         my_assert!(count, 184_756); // C(20, 10)
//     }

//     #[test]
//     fn test_negative_delta() {
//         let delta = -0.5;
//         let arrangements: Vec<Vec<f64>> = generate_arrangements(3, 1, delta).collect();
//         my_assert!(arrangements.len(), 3);

//         for arr in &arrangements {
//             let delta_count = arr.iter().filter(|&&x| (x - delta).abs() < 1e-10).count();
//             my_assert!(delta_count, 1);
//         }
//     }
// }
