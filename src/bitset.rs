// use crate::helpers::budget_is_enough;
#[cfg(test)]
use crate::helpers::count_failure;
#[cfg(test)]
use assert_float_eq::assert_f64_near;
use std::collections::{HashMap, HashSet};
// use serde::de::IntoDeserializer; // already present in your module
#[derive(Clone)]
pub struct Bitset {
    data: Vec<u64>,
    actual_size: usize,
}

impl Bitset {
    pub fn new(size: usize, ones: bool) -> Self {
        let mut out: Self = Self {
            data: vec![if ones { u64::MAX } else { 0 }; size.div_ceil(64)],
            actual_size: size,
        };
        for i in out.actual_size..size.div_ceil(64) * 64 {
            out.set_zero(i);
        }
        out
    }

    #[inline]
    pub fn set_one(&mut self, idx: usize) {
        debug_assert!(idx < self.actual_size);
        let word: usize = idx / 64;
        let bit: usize = idx % 64;
        self.data[word] |= 1u64 << bit;
    }
    #[inline]
    pub fn set_zero(&mut self, idx: usize) {
        let word: usize = idx / 64;
        let bit: usize = idx % 64;
        self.data[word] &= !(1u64 << bit);
    }
    // #[inline]
    // pub fn get(&self, idx: usize) -> bool {
    //     let word = idx / 64;
    //     let bit = idx % 64;
    //     (self.data[word] >> bit) & 1 != 0
    // }

    #[inline]
    pub fn get_success(&self) -> f64 {
        let count: usize = self.data.iter().map(|x| x.count_ones() as usize).sum();
        count as f64 / self.actual_size as f64
    }

    #[inline]
    pub fn intersect_into(&self, out: &mut Self) {
        for i in 0..self.data.len() {
            out.data[i] &= self.data[i];
        }
    }
}

pub struct BitsetBundle {
    bitsets: Vec<Vec<Bitset>>,
    pub transposed_thresholds: Vec<Vec<i64>>,
    data_size: usize,
    // filtered_size: usize,
}
pub fn generate_bit_sets(
    cost_data: &[Vec<i64>],
    mut thresholds: Vec<Vec<i64>>,
    // initial_arr: &[i64],
    pity: &Vec<i64>,
    data_size: usize,
) -> BitsetBundle {
    thresholds.push(pity.clone());
    thresholds.dedup();

    let filtered_size: usize = thresholds.len();
    let mut bitsets: Vec<Vec<Bitset>> = Vec::with_capacity(filtered_size);
    let mut this_bitset: Vec<Bitset>;

    let mut transposed_thresholds: Vec<Vec<i64>> = vec![vec![]; 7];
    // let mut budget: Vec<i64> = vec![0; 7];
    for thresh in &thresholds {
        this_bitset = vec![Bitset::new(data_size, false); 7];
        for (cost_index, cost) in cost_data.iter().enumerate() {
            for i in 0..7 {
                if cost[i] <= thresh[i] {
                    this_bitset[i].set_one(cost_index);
                }
            }
        }
        bitsets.push(this_bitset);
        for i in 0..7_usize {
            transposed_thresholds[i].push(thresh[i]);
        }
    }
    BitsetBundle {
        bitsets,
        transposed_thresholds,
        data_size,
        // filtered_size,
    }
}

fn oracle(
    bitset_bundle: &BitsetBundle,
    input: &Vec<usize>,
    cache: &mut HashMap<Vec<usize>, f64>,
) -> f64 {
    if let Some(&cached_result) = cache.get(input) {
        return cached_result;
    }

    let mut result_bitset: Bitset = Bitset::new(bitset_bundle.data_size, true);
    // let min: usize = *input.iter().min().unwrap();
    for (cost_type, i) in input.iter().enumerate() {
        // if min > 0 {
        //     dbg!(i, &bitset_bundle.bitsets[*i][cost_type].data);
        // }
        // dbg!(&bitset_bundle.bitsets[*i].data);
        bitset_bundle.bitsets[*i][cost_type].intersect_into(&mut result_bitset);
    }
    // if min > 0 {
    //     panic!();
    // }
    let result = result_bitset.get_success();

    cache.insert(input.clone(), result);
    result
}

#[derive(Clone, Debug)]
struct State {
    indices: Vec<usize>,
    score: f64,
    mats_cost: f64,
}

fn compute_diff_cost(
    thresholds: &[Vec<i64>],
    cur_cand: &[usize],
    change_index: usize,
    change_amount: isize,
    price_arr: &[f64],
) -> f64 {
    (thresholds[change_index][(cur_cand[change_index] as isize + change_amount) as usize]
        - thresholds[change_index][cur_cand[change_index]]) as f64
        * price_arr[change_index]
}
pub fn compute_gold_cost_from_indices(
    thresholds: &[Vec<i64>],
    idxs: &[usize],
    input_budget_no_gold: &[i64],
    price_arr: &[f64],
) -> f64 {
    let mut c: f64 = 0f64;
    for i in 0..7 {
        let val: f64 = (thresholds[i][idxs[i]] - input_budget_no_gold[i]).max(0) as f64;
        c += price_arr[i] * val;
    }
    c
}
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

/// Beam search implementation. INSANELY RUDIMENTARY AND NEEDS A LOT OF WORK
/// - `bitset_bundle`: precomputed bitsets + thresholds
/// - `initial_arr`: currently ignored (we start from zeros) but left in signature for future seeds
/// - `price_arr`: price per dimension (len==7)
/// - K: budget
pub fn beam_search<R: rand::Rng>(
    bitset_bundle: &BitsetBundle,
    price_arr: &[f64],
    input_budget: &[i64],
    _rng: &mut R,
    search_depth: usize,
    prev_indices: &mut Vec<usize>,

    #[cfg(test)] cost_data: &[Vec<i64>],
    // prevent_spend_gold: bool,
) -> (Vec<i64>, f64) {
    // parameters you can tune
    let mut input_budget_no_gold: Vec<i64> = input_budget.to_vec();
    input_budget_no_gold[5] = 0;
    let k: f64 = input_budget[5] as f64;
    let dims: usize = 7;
    let beam_width: usize = 8; // W
    let perturb_limits: Vec<usize> = vec![
        512, 512, 512, 256, 256, 256, 128, 128, 128, 64, 64, 64, 32, 32, 32, 16, 16, 16, 8, 8, 8,
        4, 4, 4, 2, 2, 2, 1, 1, 1, 1,
    ];
    // let perturb_limits: Vec<usize> = vec![
    //     512, 512, 512, 512, 512, 512, 256, 256, 256, 256, 256, 256, 128, 128, 128, 128, 128, 128,
    //     64, 64, 64, 64, 64, 64, 32, 32, 32, 32, 32, 32, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8,
    //     4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1,
    // ];
    // let children_per_parent: usize = 12; // target children per parent (including greedy child)
    // let greedy_step_max: usize = 1500 / beam_rounds; // greedy increments up to 1..=3
    // let random_step_max: i64 = 1500 / beam_rounds as i64; // random +/- steps per dim
    // let beam_rounds: usize = 30; // T

    // convenience access
    let thresholds = &bitset_bundle.transposed_thresholds;
    let threshold_len: usize = thresholds[0].len();

    let mut min_indices: Vec<usize> = vec![];
    for i in 0..7_usize {
        if i == 5 {
            min_indices.push(0);
            continue;
        }
        let mut yes: bool = false;
        for (index, thresh) in thresholds[i].iter().enumerate() {
            if *thresh >= input_budget[i] {
                min_indices.push(index.saturating_sub(1));
                yes = true;
                break;
            }
        }
        if !yes {
            min_indices.push(threshold_len - 1);
        }
    }

    let mut start_idxs: Vec<usize>;

    let mut uniform_index: usize = threshold_len - 1;
    let mut cur_index: Vec<usize> = vec![0; 7];
    for thresh_index in 0..threshold_len {
        debug_assert_eq!(cur_index[0], thresh_index);
        let w: f64 = compute_gold_cost_from_indices(
            thresholds,
            &cur_index,
            &input_budget_no_gold,
            price_arr,
        );
        // dbg!(w);
        if w > k {
            uniform_index = thresh_index.saturating_sub(1);
            break;
        }
        for i in 0..7 {
            cur_index[i] += 1;
        }
    }
    // start_idxs[5] = start_idxs[5].min(thresholds[0].len() / 2);
    start_idxs = vec![uniform_index; 7];
    for i in 0..7 {
        start_idxs[i] = start_idxs[i].max(min_indices[i]);
        if prev_indices.len() == 7 {
            start_idxs[i] = start_idxs[i].max(prev_indices[i]);
        }
    }

    // dbg!(&input_budget);
    // dbg!(&start_idxs);

    // dbg!(&input_budget_no_gold);

    // Initialize oracle cache
    let mut oracle_cache: HashMap<Vec<usize>, f64> = HashMap::new();

    // let start_cost: f64 = compute_cost(thresholds, &start_idxs, &input_budget_no_gold, &price_arr);
    let start_score: f64 = oracle(bitset_bundle, &start_idxs, &mut oracle_cache);

    // initial beam: single state for now (we can add more seeds later)
    let mut beam: Vec<State> = vec![State {
        indices: start_idxs.clone(),
        score: start_score,
        mats_cost: compute_gold_cost_from_indices(
            thresholds,
            &start_idxs,
            &input_budget_no_gold,
            price_arr,
        ),
    }];

    // track best overall
    let mut best_state: State = beam[0].clone();

    // beam loop
    for perturb_limit in perturb_limits.into_iter().rev().take(search_depth).rev() {
        // generate candidates
        let mut candidates: Vec<State> = Vec::new();
        let mut seen: HashSet<Vec<usize>> = HashSet::new();

        // dbg!(&best_state);
        // Expand each parent in beam
        let mut cand: Vec<usize>;
        for parent in &beam {
            // Add parent itself as a candidate (to keep it eligible)
            if !seen.contains(&parent.indices) {
                candidates.push(parent.clone());
                seen.insert(parent.indices.clone());
            }

            for up_index in 0..7_usize {
                for mut down_index in 0..6_usize {
                    cand = parent.indices.clone();
                    down_index += usize::from(down_index >= up_index);

                    let mut max_up_change: usize = perturb_limit;
                    let mut max_down_change: usize = perturb_limit;
                    // for i in 1..=perturb_limit {
                    if cand[up_index] + perturb_limit >= threshold_len.saturating_sub(1) {
                        max_up_change = max_up_change.min(threshold_len - 1 - cand[up_index]);
                    }
                    if cand[down_index] < perturb_limit
                        || down_index.saturating_sub(perturb_limit) < min_indices[down_index]
                    {
                        max_down_change =
                            max_down_change.min(cand[down_index] - min_indices[down_index]);
                    }
                    // }
                    if max_up_change == 0 && max_down_change == 0 {
                        continue;
                    }
                    let left_overs: f64 = k - compute_gold_cost_from_indices(
                        thresholds,
                        &cand,
                        &input_budget_no_gold,
                        price_arr,
                    );

                    let mut needed_cash: f64 = compute_diff_cost(
                        thresholds,
                        &cand,
                        up_index,
                        max_up_change as isize,
                        price_arr,
                    );

                    let mut avail_cash: f64 = left_overs
                        - compute_diff_cost(
                            thresholds,
                            &cand,
                            down_index,
                            -(max_down_change as isize),
                            price_arr,
                        );

                    let mut actual_up_change: usize = max_up_change;
                    let mut actual_down_change: usize = max_down_change;
                    while needed_cash > avail_cash && actual_up_change > 1 {
                        actual_up_change = actual_up_change.saturating_sub(1);
                        needed_cash = compute_diff_cost(
                            thresholds,
                            &cand,
                            up_index,
                            actual_up_change as isize,
                            price_arr,
                        );
                    }
                    if needed_cash > avail_cash {
                        continue;
                    }
                    while actual_down_change > 0 {
                        avail_cash = left_overs
                            - compute_diff_cost(
                                thresholds,
                                &cand,
                                down_index,
                                -((actual_down_change.saturating_sub(1)) as isize),
                                price_arr,
                            );
                        if avail_cash < needed_cash {
                            break;
                        }
                        actual_down_change = actual_down_change.saturating_sub(1);
                    }
                    cand[up_index] += actual_up_change;
                    cand[down_index] = cand[down_index].saturating_sub(actual_down_change);
                    // dbg!(&cand, round);

                    candidates.push(State {
                        indices: cand.clone(),
                        score: oracle(bitset_bundle, &cand, &mut oracle_cache),
                        mats_cost: compute_gold_cost_from_indices(
                            thresholds,
                            &cand,
                            &input_budget_no_gold,
                            price_arr,
                        ) - thresholds[5][cand[5]] as f64,
                    });

                    #[cfg(test)]
                    let mut budget_data: Vec<Vec<i64>> = vec![vec![]];
                    #[cfg(test)]
                    for i in 0..7 {
                        budget_data[0].push(thresholds[i][cand[i]]);
                    }
                    #[cfg(test)]
                    assert!(
                        oracle(bitset_bundle, &cand, &mut oracle_cache)
                            - (1.0
                                - count_failure(cost_data, &budget_data, false)[0] as f64
                                    / cost_data.len() as f64)
                            < 1.0 / 1000000.0 as f64
                    );

                    seen.insert(cand);
                }
            }
        } // for each parent

        candidates.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.mats_cost
                        .partial_cmp(&b.mats_cost)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        candidates.truncate(beam_width);
        if let Some(top) = candidates.first() {
            // dbg!("top", top);

            best_state = top.clone();
        }

        beam = candidates;
    } // beam rounds

    // convert best_state idices -> threshold values Vec<i64>
    let mut best_values: Vec<i64> = vec![0i64; dims];
    for i in 0..dims {
        best_values[i] = thresholds[i][best_state.indices[i]].max(input_budget_no_gold[i]);
        if i == 5 {
            best_values[5] = (k - best_state.mats_cost).floor() as i64;
        }
    }

    // dbg!(&best_state);
    // dbg!(&best_values);
    // dbg!(oracle(
    //     bitset_bundle,
    //     &best_state.indices,
    //     &mut oracle_cache
    // ));
    if *best_state.indices.iter().min_by(|a, b| a.cmp(b)).unwrap() > 0_usize {
        *prev_indices = best_state.indices;
    }

    (best_values, best_state.score)
}
