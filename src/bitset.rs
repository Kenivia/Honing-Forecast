// use crate::helpers::budget_is_enough;
// #[cfg(test)]
// use crate::helpers::count_failure;
// #[cfg(test)]
// // use assert_float_eq::assert_f64_near;
// use std::collections::HashMap;
// // use serde::de::IntoDeserializer; // already present in your module
// #[derive(Clone)]
// pub struct Bitset {
//     data: Vec<u64>,
//     actual_size: usize,
// }

// impl Bitset {
//     pub fn new(size: usize, ones: bool) -> Self {
//         let mut out: Self = Self {
//             data: vec![if ones { u64::MAX } else { 0 }; size.div_ceil(64)],
//             actual_size: size,
//         };
//         for i in out.actual_size..size.div_ceil(64) * 64 {
//             out.set_zero(i);
//         }
//         out
//     }

//     #[inline]
//     pub fn set_one(&mut self, idx: usize) {
//         debug_assert!(idx < self.actual_size);
//         let word: usize = idx / 64;
//         let bit: usize = idx % 64;
//         self.data[word] |= 1u64 << bit;
//     }
//     #[inline]
//     pub fn set_zero(&mut self, idx: usize) {
//         let word: usize = idx / 64;
//         let bit: usize = idx % 64;
//         self.data[word] &= !(1u64 << bit);
//     }
//     // #[inline]
//     // pub fn get(&self, idx: usize) -> bool {
//     //     let word = idx / 64;
//     //     let bit = idx % 64;
//     //     (self.data[word] >> bit) & 1 != 0
//     // }

//     #[inline]
//     pub fn get_success(&self) -> f64 {
//         let count: usize = self.data.iter().map(|x| x.count_ones() as usize).sum();
//         count as f64 / self.actual_size as f64
//     }

//     #[inline]
//     pub fn intersect_into(&self, out: &mut Self) {
//         for i in 0..self.data.len() {
//             out.data[i] &= self.data[i];
//         }
//     }
// }

// pub struct BitsetBundle {
//     bitsets: Vec<Vec<Bitset>>,
//     pub transposed_thresholds: Vec<Vec<i64>>,
//     data_size: usize,
//     // filtered_size: usize,
// }
// pub fn generate_bit_sets(
//     cost_data: &[Vec<i64>],
//     mut thresholds: Vec<Vec<i64>>,
//     // initial_arr: &[i64],
//     pity: &Vec<i64>,
//     data_size: usize,
// ) -> BitsetBundle {
//     thresholds.push(pity.clone());
//     thresholds.dedup();

//     let filtered_size: usize = thresholds.len();
//     let mut bitsets: Vec<Vec<Bitset>> = Vec::with_capacity(filtered_size);
//     let mut this_bitset: Vec<Bitset>;

//     let mut transposed_thresholds: Vec<Vec<i64>> = vec![vec![]; 7];
//     // let mut budget: Vec<i64> = vec![0; 7];
//     for thresh in &thresholds {
//         this_bitset = vec![Bitset::new(data_size, false); 7];
//         for (cost_index, cost) in cost_data.iter().enumerate() {
//             for i in 0..7 {
//                 if cost[i] <= thresh[i] {
//                     this_bitset[i].set_one(cost_index);
//                 }
//             }
//         }
//         bitsets.push(this_bitset);
//         for i in 0..7_usize {
//             transposed_thresholds[i].push(thresh[i]);
//         }
//     }
//     BitsetBundle {
//         bitsets,
//         transposed_thresholds,
//         data_size,
//         // filtered_size,
//     }
// }

// fn oracle(
//     bitset_bundle: &BitsetBundle,
//     input: &Vec<usize>,
//     cache: &mut HashMap<Vec<usize>, f64>,
// ) -> f64 {
//     if let Some(&cached_result) = cache.get(input) {
//         return cached_result;
//     }

//     let mut result_bitset: Bitset = Bitset::new(bitset_bundle.data_size, true);
//     // let min: usize = *input.iter().min().unwrap();
//     for (cost_type, i) in input.iter().enumerate() {
//         // if min > 0 {
//         //     dbg!(i, &bitset_bundle.bitsets[*i][cost_type].data);
//         // }
//         // dbg!(&bitset_bundle.bitsets[*i].data);
//         bitset_bundle.bitsets[*i][cost_type].intersect_into(&mut result_bitset);
//     }
//     // if min > 0 {
//     //     panic!();
//     // }
//     let result = result_bitset.get_success();

//     cache.insert(input.clone(), result);
//     result
// }

// #[derive(Clone, Debug)]
// struct State {
//     indices: Vec<usize>,
//     score: f64,
//     mats_cost: f64,
// }

// fn compute_diff_cost(
//     thresholds: &[Vec<i64>],
//     cur_cand: &[usize],
//     change_index: usize,
//     change_amount: isize,
//     price_arr: &[f64],
// ) -> f64 {
//     (thresholds[change_index][(cur_cand[change_index] as isize + change_amount) as usize]
//         - thresholds[change_index][cur_cand[change_index]]) as f64
//         * price_arr[change_index]
// }
// pub fn compute_gold_cost_from_indices(
//     thresholds: &[Vec<i64>],
//     idxs: &[usize],
//     input_budget_no_gold: &[i64],
//     price_arr: &[f64],
// ) -> f64 {
//     let mut c: f64 = 0f64;
//     for i in 0..7 {
//         let val: f64 = (thresholds[i][idxs[i]] - input_budget_no_gold[i]).max(0) as f64;
//         c += price_arr[i] * val;
//     }
//     c
// }
pub fn compute_gold_cost_from_raw(
    needed: &[i64],
    input_budget_no_gold: &[i64],
    price_arr: &[f64],
) -> f64 {
    let mut c: f64 = 0f64;
    for i in 0..7 {
        let val = (needed[i] - input_budget_no_gold[i]).max(0) as f64;
        c += price_arr[i] * val;
    }
    c
}
