// use crate::helpers::budget_is_enough;
use std::collections::HashSet; // already present in your module
#[derive(Clone)]
pub struct Bitset {
    data: Vec<u64>,
    actual_size: usize,
}

impl Bitset {
    pub fn new(size: usize, ones: bool) -> Self {
        let mut out: Bitset = Self {
            data: vec![if ones { u64::MAX } else { 0 }; (size + 63) / 64],
            actual_size: size,
        };
        for i in out.actual_size..((size + 63) / 64) * 64 {
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
        self.data[word] &= !1u64 << bit;
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
    pub fn intersect_into(&self, out: &mut Bitset) {
        for i in 0..self.data.len() {
            out.data[i] = self.data[i] & out.data[i];
        }
    }
}

pub struct BitsetBundle {
    bitsets: Vec<Vec<Bitset>>,
    transposed_thresholds: Vec<Vec<i64>>,
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
    for thresh in thresholds.iter() {
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

fn oracle(bitset_bundle: &BitsetBundle, input: &Vec<usize>) -> f64 {
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
    result_bitset.get_success()
}

#[derive(Clone, Debug)]
struct State {
    idxs: Vec<usize>,
    score: f64,
    cost: f64,
}

/// Beam search implementation.
/// - bitset_bundle: precomputed bitsets + thresholds
/// - initial_arr: currently ignored (we start from zeros) but left in signature for future seeds
/// - price_arr: price per dimension (len==7)
/// - K: budget
pub fn beam_search<R: rand::Rng>(
    bitset_bundle: &BitsetBundle,
    price_arr: &[f64],
    input_budget: &[i64],
    rng: &mut R,
) -> (Vec<i64>, f64) {
    // parameters you can tune
    let mut input_budget_no_gold: Vec<i64> = input_budget.to_vec();
    input_budget_no_gold[5] = 0;
    let k: f64 = input_budget[5] as f64;
    let dims: usize = 7;
    let beam_width: usize = 32; // W
    let children_per_parent: usize = 12; // target children per parent (including greedy child)
    let greedy_step_max: usize = 10; // greedy increments up to 1..=3
    let random_step_max: i64 = 10; // random +/- steps per dim
    let beam_rounds: usize = 20; // T

    // convenience access
    let thresholds = &bitset_bundle.transposed_thresholds;
    let max_idx = if thresholds.is_empty() {
        0
    } else {
        thresholds[0].len() - 1
    };
    // quick helper to compute cost of an index vector
    let compute_cost = |idxs: &Vec<usize>| -> f64 {
        let mut c: f64 = 0f64;
        for i in 0..dims {
            let val = (thresholds[i][idxs[i]] - input_budget_no_gold[i]).max(0) as f64;
            c += price_arr[i] * val;
        }
        c
    };
    // helper to clamp an index
    let clamp_idx = |v: isize| -> usize {
        if v < 0 {
            0
        } else if (v as usize) > max_idx {
            max_idx
        } else {
            v as usize
        }
    };

    let mut start_idxs: Vec<usize> = vec![];
    for i in 0..7_usize {
        for (index, thresh) in thresholds[i].iter().enumerate() {
            if *thresh >= input_budget_no_gold[i] || index == thresholds[i].len() - 1 {
                start_idxs.push(index);
                break;
            }
        }
    }
    dbg!("start", &start_idxs);
    dbg!(&input_budget_no_gold);
    let start_cost: f64 = compute_cost(&start_idxs);
    let start_score: f64 = oracle(bitset_bundle, &start_idxs);

    //oracle_cache.insert(start_idxs.clone(), start_score);
    // try to consume budget from zero via greedy fill
    // let (start_idxs_filled, start_score_filled, start_cost_filled) =
    //     fill_budget(start_idxs, start_score, start_cost);

    // initial beam: single state for now (we can add more seeds later)
    let mut beam: Vec<State> = vec![State {
        idxs: start_idxs.clone(),
        score: start_score,
        cost: start_cost,
    }];

    // track best overall
    let mut best_state: State = beam[0].clone();

    // beam loop
    for _round in 0..beam_rounds {
        // generate candidates
        let mut candidates: Vec<State> = Vec::new();
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        // dbg!(&best_state);
        // Expand each parent in beam
        for parent in &beam {
            // Add parent itself as a candidate (to keep it eligible)
            if !seen.contains(&parent.idxs) {
                candidates.push(parent.clone());
                seen.insert(parent.idxs.clone());
            }

            // --- GREEDY child: increment the most "efficient" index by 1..=greedy_step_max (best marginal per cost)
            {
                let mut best_marginal = std::f64::NEG_INFINITY;
                let mut best_dim: Option<usize> = None;
                let mut best_new_idx: Option<usize> = None;
                // let mut best_new_score: f64 = parent.score;

                for i in 0..dims {
                    let cur_idx = parent.idxs[i];
                    for step in 1..=greedy_step_max {
                        let next_idx = cur_idx.saturating_add(step).min(max_idx);

                        let added_cost = price_arr[i]
                            * (thresholds[i][next_idx] as f64 - thresholds[i][cur_idx] as f64);
                        if parent.cost + added_cost > k + 1e-12 {
                            break;
                        }
                        let mut cand_idxs = parent.idxs.clone();
                        cand_idxs[i] = next_idx;
                        let cand_score: f64 = oracle(bitset_bundle, &cand_idxs);
                        //oracle_cache.insert(cand_idxs.clone(), cand_score);
                        let marginal: f64 = cand_score - parent.score;
                        // let ratio: f64 = marginal / added_cost;
                        if marginal > 0.0 && marginal > best_marginal {
                            best_marginal = marginal;
                            best_dim = Some(i);
                            best_new_idx = Some(next_idx);
                            // best_new_score = cand_score;
                        }
                    }
                }

                if let (Some(dim), Some(nidx)) = (best_dim, best_new_idx) {
                    // create greedy child then fill budget
                    let mut greedy_idxs = parent.idxs.clone();
                    greedy_idxs[dim] = nidx;
                    let greedy_cost = compute_cost(&greedy_idxs);
                    let greedy_score = oracle(bitset_bundle, &greedy_idxs);
                    //oracle_cache.insert(greedy_idxs.clone(), greedy_score);
                    // let (filled_idxs, filled_score, filled_cost) =
                    //     fill_budget(greedy_idxs, greedy_score, greedy_cost);

                    if !seen.contains(&greedy_idxs) {
                        candidates.push(State {
                            idxs: greedy_idxs.clone(),
                            score: greedy_score,
                            cost: greedy_cost,
                        });
                        seen.insert(greedy_idxs);
                    }
                }
            }

            // --- RANDOM children
            let mut child_count = 0usize;
            // dbg!(children_per_parent);
            while child_count < (children_per_parent.saturating_sub(1)) {
                // start from parent
                let mut cand = parent.idxs.clone();

                // randomly perturb several dims (some may decrease)
                // decide how many dims to touch: 1..=dims
                let dims_to_touch: usize = rng.random_range(1..=dims);
                for _ in 0..dims_to_touch {
                    let dim: usize = rng.random_range(0..dims);
                    let delta: i64 = if price_arr[dim] == 0.0 {
                        max_idx as i64
                    } else {
                        rng.random_range(-random_step_max..random_step_max)
                    };

                    let new_idx_isize = cand[dim] as isize + delta as isize;
                    let new_idx = clamp_idx(new_idx_isize);
                    cand[dim] = new_idx;
                }

                // ensure within bounds and cost feasible; if over budget, try to reduce some dims randomly until feasible
                let mut cand_cost = compute_cost(&cand);
                if cand_cost > k + 1e-12 {
                    // attempt to reduce some dims randomly
                    for _attempt in 0..10 {
                        let dim = rng.random_range(0..dims);
                        // reduce by 1..=random_step_max if possible
                        if cand[dim] > 0 {
                            let reduce = 1 + (rng.random_range(0..(random_step_max as usize) + 1));
                            let new_idx = cand[dim].saturating_sub(reduce);
                            cand[dim] = new_idx;
                            cand_cost = compute_cost(&cand);
                            if cand_cost <= k + 1e-12 {
                                break;
                            }
                        }
                    }
                }
                // dbg!(cand_cost, k);
                if cand_cost > k + 1e-12 {
                    // can't make it feasible, skip this random child
                    child_count += 1;
                    continue;
                }

                // fill budget greedily from this perturbed starting point
                let cand_score: f64 = oracle(bitset_bundle, &cand);
                //oracle_cache.insert(cand.clone(), cand_score);
                // dbg!(&cand);
                // let (filled_idxs, filled_score, filled_cost) =
                //     fill_budget(cand, cand_score, cand_cost);
                // dbg!(&filled_idxs);
                if !seen.contains(&cand) {
                    candidates.push(State {
                        idxs: cand.clone(),
                        score: cand_score,
                        cost: cand_cost,
                    });
                    seen.insert(cand);
                }

                child_count += 1;
            } // random children loop
        } // for each parent

        // pick top beam_width candidates by score (descending)
        candidates.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(beam_width);

        // update best overall
        // dbg!(&candidates);
        if let Some(top) = candidates.get(0) {
            dbg!("top", top);
            if top.score > best_state.score {
                best_state = top.clone();
            }
        }

        // new beam = candidates
        beam = candidates;

        // small early stop if beam didn't improve much (optional)
        // (we track best_state, but continue for full rounds for robustness)
    } // beam rounds

    // convert best_state idices -> threshold values Vec<i64>
    let mut best_values: Vec<i64> = vec![0i64; dims];
    for i in 0..dims {
        best_values[i] = thresholds[i][best_state.idxs[i]];
    }
    dbg!("best", &best_state);
    (best_values, best_state.score)
}
