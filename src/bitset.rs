// use std::collections::{HashMap, HashSet};

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
// }

// pub fn generate_bit_sets(
//     cost_data: &[Vec<i64>],
//     mut thresholds: Vec<Vec<i64>>,
//     pity: &Vec<i64>,
//     data_size: usize,
// ) -> BitsetBundle {
//     thresholds.push(pity.clone());
//     thresholds.dedup();

//     let filtered_size: usize = thresholds.len();
//     let mut bitsets: Vec<Vec<Bitset>> = Vec::with_capacity(filtered_size);
//     let mut this_bitset: Vec<Bitset>;

//     let mut transposed_thresholds: Vec<Vec<i64>> = vec![vec![]; 7];
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
//     for (cost_type, i) in input.iter().enumerate() {
//         bitset_bundle.bitsets[*i][cost_type].intersect_into(&mut result_bitset);
//     }
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

// pub fn compute_gold_cost_from_raw(
//     needed: &[i64],
//     input_budget_no_gold: &[i64],
//     price_arr: &[f64],
// ) -> f64 {
//     let mut c: f64 = 0f64;
//     for i in 0..7 {
//         let val = (needed[i] - input_budget_no_gold[i]).max(0) as f64;
//         c += price_arr[i] * val;
//     }
//     c
// }

// struct BeamPrepOut {
//     input_budget_no_gold: Vec<i64>,
//     k: f64,
//     threshold_len: usize,
//     min_indices: Vec<usize>,
//     start_idxs: Vec<usize>,
// }

// fn beam_search_prep(
//     bitset_bundle: &BitsetBundle,
//     price_arr: &[f64],
//     input_budget: &[i64],
//     prev_indices: &Vec<usize>,
// ) -> BeamPrepOut {
//     let mut input_budget_no_gold = input_budget.to_vec();
//     input_budget_no_gold[5] = 0;
//     let k = input_budget[5] as f64;
//     let thresholds = &bitset_bundle.transposed_thresholds;
//     let threshold_len = thresholds[0].len();

//     // Calculate min_indices
//     let mut min_indices = vec![];
//     for i in 0..7 {
//         if i == 5 {
//             min_indices.push(0);
//             continue;
//         }
//         let mut yes = false;
//         for (index, thresh) in thresholds[i].iter().enumerate() {
//             if *thresh >= input_budget[i] {
//                 min_indices.push(index.saturating_sub(1));
//                 yes = true;
//                 break;
//             }
//         }
//         if !yes {
//             min_indices.push(threshold_len - 1);
//         }
//     }

//     // Calculate uniform_index and start_idxs
//     let mut uniform_index = threshold_len - 1;
//     let mut cur_index = vec![0; 7];
//     for thresh_index in 0..threshold_len {
//         debug_my_assert!(cur_index[0], thresh_index);
//         let w = compute_gold_cost_from_indices(
//             thresholds,
//             &cur_index,
//             &input_budget_no_gold,
//             price_arr,
//         );
//         if w > k {
//             uniform_index = thresh_index.saturating_sub(1);
//             break;
//         }
//         for i in 0..7 {
//             cur_index[i] += 1;
//         }
//     }

//     let mut start_idxs = vec![uniform_index; 7];
//     for i in 0..7 {
//         start_idxs[i] = start_idxs[i].max(min_indices[i]);
//         if prev_indices.len() == 7 {
//             start_idxs[i] = start_idxs[i].max(prev_indices[i]);
//         }
//     }

//     BeamPrepOut {
//         input_budget_no_gold,
//         k,
//         threshold_len,
//         min_indices,
//         start_idxs,
//     }
// }

// /// Splits indices into "up" and "down" groups based on ranking.
// /// The lowest-ranked x indices (by cand values) go to "up", rest to "down".
// /// x is randomly chosen between 1 and 6.
// fn split_by_rank<R: rand::Rng>(cand: &[usize], rng: &mut R) -> (Vec<usize>, Vec<usize>) {
//     // Create indices sorted by cand values (lowest first)
//     let mut indexed: Vec<(usize, usize)> = cand.iter().copied().enumerate().collect();
//     indexed.sort_by_key(|(_, val)| *val);

//     // Choose how many go to "up" (1-6, ensuring at least 1)
//     let up_count = rng.random_range(1..=6.min(cand.len()));

//     let mut up_indices = Vec::new();
//     let mut down_indices = Vec::new();

//     for (i, (idx, _)) in indexed.iter().enumerate() {
//         if i < up_count {
//             up_indices.push(*idx);
//         } else {
//             down_indices.push(*idx);
//         }
//     }

//     (up_indices, down_indices)
// }

// /// Maximizes up_amounts to consume as much available cash as possible.
// /// Uses binary search to find the maximum affordable increase for each dimension.
// fn maximize_up_amounts<R: rand::Rng>(
//     thresholds: &[Vec<i64>],
//     cand: &[usize],
//     up_indices: &[usize],
//     up_amounts: &mut [usize],
//     threshold_len: usize,
//     left_overs: f64,
//     price_arr: &[f64],
//     rng: &mut R,
// ) {
//     // Calculate current cost from existing up_amounts
//     let mut current_cost = 0.0;
//     for (idx, &up_idx) in up_indices.iter().enumerate() {
//         if up_amounts[idx] > 0 {
//             current_cost += (thresholds[up_idx][cand[up_idx] + up_amounts[idx]]
//                 - thresholds[up_idx][cand[up_idx]]) as f64
//                 * price_arr[up_idx];
//         }
//     }

//     let mut remaining_cash = left_overs - current_cost;

//     // Create randomized order to process indices
//     let mut order: Vec<usize> = (0..up_indices.len()).collect();
//     use rand::seq::SliceRandom;
//     order.shuffle(rng);

//     for &idx in &order {
//         let up_idx = up_indices[idx];
//         let current_pos = cand[up_idx] + up_amounts[idx];

//         if current_pos >= threshold_len - 1 || remaining_cash <= 0.0 {
//             continue;
//         }

//         let max_by_threshold = threshold_len - 1 - current_pos;

//         // Binary search for maximum affordable increase
//         let mut left = 0;
//         let mut right = max_by_threshold;
//         let mut best_increase = 0;

//         while left <= right {
//             let mid = (left + right) / 2;
//             if mid == 0 {
//                 break;
//             }

//             let cost = (thresholds[up_idx][current_pos + mid] - thresholds[up_idx][current_pos])
//                 as f64
//                 * price_arr[up_idx];

//             if cost <= remaining_cash {
//                 best_increase = mid;
//                 left = mid + 1;
//             } else {
//                 right = mid - 1;
//             }
//         }

//         if best_increase > 0 {
//             let actual_cost = (thresholds[up_idx][current_pos + best_increase]
//                 - thresholds[up_idx][current_pos]) as f64
//                 * price_arr[up_idx];
//             up_amounts[idx] += best_increase;
//             remaining_cash -= actual_cost;
//         }
//     }
// }

// fn compute_diff_cost(
//     thresholds: &[Vec<i64>],
//     cur_cand: &[usize],
//     change_indices: &[usize],
//     change_amount: &[usize],
//     price_arr: &[f64],
//     up: bool,
// ) -> f64 {
//     let mut out: f64 = 0.0;
//     for (index, change_index) in change_indices.iter().enumerate() {
//         out += if up {
//             (thresholds[*change_index][cur_cand[*change_index] + change_amount[index]]
//                 - thresholds[*change_index][cur_cand[*change_index]]) as f64
//                 * price_arr[*change_index]
//         } else {
//             (thresholds[*change_index][cur_cand[*change_index]]
//                 - thresholds[*change_index][cur_cand[*change_index] - change_amount[index]])
//                 as f64
//                 * price_arr[*change_index]
//         }
//     }
//     out
// }

// pub fn beam_search<R: rand::Rng>(
//     bitset_bundle: &BitsetBundle,
//     price_arr: &[f64],
//     input_budget: &[i64],
//     rng: &mut R,
//     search_depth: usize,
//     prev_indices: &mut Vec<usize>,
// ) -> (Vec<i64>, f64) {
//     // Preprocessing
//     let prep = beam_search_prep(bitset_bundle, price_arr, input_budget, prev_indices);
//     let BeamPrepOut {
//         input_budget_no_gold,
//         k,
//         threshold_len,
//         min_indices,
//         start_idxs,
//     } = prep;

//     let dims: usize = 7;
//     let beam_width: usize = 8;
//     let perturb_limits: Vec<usize> = vec![
//         512, 512, 512, 512, 512, 512, 256, 256, 256, 256, 256, 256, 128, 128, 128, 128, 128, 128,
//         64, 64, 64, 64, 64, 64, 32, 32, 32, 32, 32, 32, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8,
//         4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1,
//     ];

//     let thresholds = &bitset_bundle.transposed_thresholds;

//     // Initialize oracle cache
//     let mut oracle_cache: HashMap<Vec<usize>, f64> = HashMap::new();

//     let start_score: f64 = oracle(bitset_bundle, &start_idxs, &mut oracle_cache);

//     // Initial beam
//     let mut beam: Vec<State> = vec![State {
//         indices: start_idxs.clone(),
//         score: start_score,
//         mats_cost: compute_gold_cost_from_indices(
//             thresholds,
//             &start_idxs,
//             &input_budget_no_gold,
//             price_arr,
//         ),
//     }];

//     if prev_indices.len() == 7 {
//         beam.push(State {
//             indices: prev_indices.clone(),
//             score: oracle(bitset_bundle, &prev_indices, &mut oracle_cache),
//             mats_cost: compute_gold_cost_from_indices(
//                 thresholds,
//                 &prev_indices,
//                 &input_budget_no_gold,
//                 price_arr,
//             ),
//         });
//     }

//     let mut best_state: State = beam[0].clone();
//     let mut seen: HashSet<Vec<usize>> = HashSet::new();

//     // Beam search loop
//     for perturb_limit in perturb_limits.into_iter().rev().take(search_depth).rev() {
//         let mut candidates: Vec<State> = Vec::new();

//         for parent in &beam {
//             if !seen.contains(&parent.indices) {
//                 candidates.push(parent.clone());
//                 seen.insert(parent.indices.clone());
//             }

//             // Generate multiple candidate perturbations
//             for _ in 0..80 {
//                 let (up_indices, down_indices) = split_by_rank(&parent.indices, rng);

//                 let randomized_perturb: Vec<usize> = (0..7)
//                     .map(|_| (perturb_limit as f64 * rng.random_range(0.0..2.0)).ceil() as usize)
//                     .collect();

//                 let mut up_amounts: Vec<usize> =
//                     up_indices.iter().map(|&i| randomized_perturb[i]).collect();
//                 let mut down_amounts: Vec<usize> = down_indices
//                     .iter()
//                     .map(|&i| randomized_perturb[i])
//                     .collect();

//                 let mut cand = parent.indices.clone();

//                 // Clamp up and down amounts to valid ranges
//                 for (idx, &up_idx) in up_indices.iter().enumerate() {
//                     up_amounts[idx] = up_amounts[idx].min(threshold_len - 1 - cand[up_idx]);
//                 }

//                 for (idx, &down_idx) in down_indices.iter().enumerate() {
//                     down_amounts[idx] =
//                         down_amounts[idx].min(cand[down_idx] - min_indices[down_idx]);
//                 }

//                 if up_amounts.iter().max().copied().unwrap_or(0) == 0 {
//                     continue;
//                 }

//                 let left_overs = k - compute_gold_cost_from_indices(
//                     thresholds,
//                     &cand,
//                     &input_budget_no_gold,
//                     price_arr,
//                 );

//                 let needed_cash =
//                     compute_diff_cost(thresholds, &cand, &up_indices, &up_amounts, price_arr, true);

//                 let freed_cash = compute_diff_cost(
//                     thresholds,
//                     &cand,
//                     &down_indices,
//                     &down_amounts,
//                     price_arr,
//                     false,
//                 );

//                 let avail_cash = left_overs + freed_cash;

//                 if needed_cash > avail_cash {
//                     continue;
//                 }

//                 // Maximize up_amounts to consume as much available cash as possible
//                 maximize_up_amounts(
//                     thresholds,
//                     &cand,
//                     &up_indices,
//                     &mut up_amounts,
//                     threshold_len,
//                     avail_cash,
//                     price_arr,
//                     rng,
//                 );

//                 // Apply changes
//                 for (idx, &up_idx) in up_indices.iter().enumerate() {
//                     cand[up_idx] += up_amounts[idx];
//                 }

//                 for (idx, &down_idx) in down_indices.iter().enumerate() {
//                     cand[down_idx] -= down_amounts[idx];
//                 }

//                 if seen.contains(&cand) {
//                     continue;
//                 }

//                 candidates.push(State {
//                     indices: cand.clone(),
//                     score: oracle(bitset_bundle, &cand, &mut oracle_cache),
//                     mats_cost: compute_gold_cost_from_indices(
//                         thresholds,
//                         &cand,
//                         &input_budget_no_gold,
//                         price_arr,
//                     ) - thresholds[5][cand[5]] as f64,
//                 });

//                 seen.insert(cand);
//             }
//         }

//         // Sort and select top beam_width candidates
//         candidates.sort_unstable_by(|a, b| {
//             b.score
//                 .partial_cmp(&a.score)
//                 .unwrap_or(std::cmp::Ordering::Equal)
//                 .then_with(|| {
//                     a.mats_cost
//                         .partial_cmp(&b.mats_cost)
//                         .unwrap_or(std::cmp::Ordering::Equal)
//                 })
//         });

//         candidates.truncate(beam_width);

//         if let Some(top) = candidates.first() {
//             if top.score > best_state.score
//                 || (top.score == best_state.score && top.mats_cost < best_state.mats_cost)
//             {
//                 best_state = top.clone();
//             }
//         }

//         beam = candidates;
//     }

//     // Convert best_state indices to threshold values
//     let mut best_values: Vec<i64> = vec![0i64; dims];
//     for i in 0..dims {
//         best_values[i] = thresholds[i][best_state.indices[i]].max(input_budget_no_gold[i]);
//         if i == 5 {
//             best_values[5] = (k - best_state.mats_cost).floor() as i64;
//         }
//     }

//     if *best_state.indices.iter().min().unwrap() > 0 {
//         *prev_indices = best_state.indices;
//         dbg!(&prev_indices);
//     }

//     (best_values, best_state.score)
// }
