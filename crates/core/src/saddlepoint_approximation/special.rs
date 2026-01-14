use std::f64;

use crate::constants::FLOAT_TOL;
// use super::saddlepoint_approximation::saddlepoint_approximation_wrapper;
// use crate::helpers::find_non_zero_min_vec;
// use crate::parser::Upgrade;
use crate::state::StateBundle;

impl StateBundle {
    pub fn special_probs(&self) -> &Vec<f64> {
        return &self.special_cache[&self.special_state];
    }

    pub fn comput_special_probs(&mut self) {
        if !self.special_cache.contains_key(&self.special_state) {
            let prep_output = &self.prep_output;
            let upgrades = &prep_output.upgrade_arr;
            let m = upgrades.len();

            let budget: usize = prep_output.budgets[7] as usize; // total special budget B
            let mut result = vec![0.0_f64; m + 1];

            // active[b] = probability we are still running and have 'b' special left
            let mut active: Vec<f64> = vec![0.0_f64; budget + 1];
            active[budget] = 1.0;

            // optional: track stopped distribution; not needed if you only care about success probs
            // let mut stopped = vec![0.0_f64; budget + 1];

            // Process streaks in order
            for (attempt_index, u_index) in self.special_state.iter().enumerate() {
                let upgrade = &upgrades[*u_index];
                let p = upgrade.base_chance;
                let one_minus_p = 1.0 - p;
                let this_special_cost = upgrade.special_cost as usize;

                let repeat_count = (budget / this_special_cost).max(0);
                // Precompute geometric probabilities p * (1-p)^(t-1) up to L
                let mut geom = vec![0.0_f64; repeat_count + 1]; // 1-based: geom[t] for t=1..L
                let mut pow = 1.0_f64;
                for t in 1..=repeat_count {
                    geom[t] = p * pow;
                    pow *= one_minus_p;
                }
                // pow now equals (1-p)^L, but we also need (1-p)^(A-1) often, see below.

                let mut next_active = vec![0.0_f64; budget + 1];

                for b in 0..=budget {
                    let mass = active[b];
                    if mass == 0.0 {
                        continue;
                    }

                    if b < this_special_cost {
                        continue; // do not propagate to next_active
                    }

                    // Max attempts allowed by budget and streak length
                    let max_by_budget = b / this_special_cost;
                    let actual_repeated = repeat_count.min(max_by_budget);

                    // Success probability on this upgrade from this starting budget
                    let fail_all_a = one_minus_p.powi(actual_repeated as i32);
                    result[attempt_index + 1] += mass * (1.0 - fail_all_a);

                    for succeed_at in 1..=actual_repeated {
                        let prob_n_t = geom[succeed_at];
                        let b2 = b - succeed_at * this_special_cost;
                        next_active[b2] += mass * prob_n_t;
                    }
                }

                active = next_active;
            }

            result[0] = 1.0 - result[1]; // nothing free tapped
            let mut actual_out = Vec::with_capacity(result.len());

            for (index, &i) in result.iter().enumerate() {
                // if index < 1 {
                //     actual_out.push(cumulative * *i);
                // } else {
                if index == result.len() - 1 || index == 0 {
                    actual_out.push(i);
                } else {
                    actual_out.push(i - result[index + 1]);
                }
            }

            // dbg!(&result, &actual_out, actual_out.iter().sum::<f64>());
            assert!((actual_out.iter().sum::<f64>() - 1.0).abs() < FLOAT_TOL);
            self.special_cache
                .insert(self.special_state.clone(), actual_out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculate_hash;
    use crate::constants::RNG_SEED;

    use crate::parser::PreparationOutput;

    use crate::test_utils::*;

    use std::collections::HashMap;
    use std::time::Instant;
    static DEBUG: bool = true;
    #[test]
    fn special_sa_test() {
        let start = Instant::now();
        let test_name = format!("special_sa_test");
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 24 { 2 } else { 0 }).collect(),
            (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
        ];
        // let hone_counts: Vec<Vec<i64>> =
        //     vec![(0..25).map(|_| 5).collect(), (0..25).map(|_| 1).collect()];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let budget = vec![0, 0, 0, 0, 0, 3333333, 0, 6767];
        let juice_books_owned: Vec<(i64, i64)> = vec![(0, 0), (0, 0), (0, 0), (0, 0)];
        let juice_prices: Vec<(f64, f64)> = vec![
            (123.0, 123.0),
            (123.0, 123.0),
            (123.0, 123.0),
            (123.0, 123.0),
        ];
        let prices = DEFAULT_GOLD_VALUES;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &budget,
            &prices,
            RNG_SEED,
            PROB_MODE
        );

        let prep_output: PreparationOutput = PreparationOutput::initialize(
            &hone_counts,
            &budget,
            &adv_counts,
            express_event,
            &prices,
            adv_hone_strategy,
            &juice_books_owned,
            &juice_prices,
            &prices,
            &juice_prices,
        );

        let mut starting_special: Vec<usize> =
            Vec::with_capacity(prep_output.upgrade_arr.len() * 2);
        for (index, _upgrade) in prep_output.upgrade_arr.iter().enumerate() {
            starting_special.push(index); //, (1.0 / upgrade.base_chance).round() as usize));
        }
        let mut state_bundle: StateBundle = StateBundle {
            names: prep_output
                .upgrade_arr
                .iter()
                .map(|x| {
                    let mut string: String = if x.is_normal_honing {
                        "".to_owned()
                    } else {
                        "adv_".to_owned()
                    };
                    string += if x.is_weapon { "weap_" } else { "armor_" };
                    string += &x.upgrade_index.to_string();
                    string
                })
                .collect::<Vec<String>>(),
            state_index: vec![],
            metric: -1.0,
            special_state: starting_special,
            prep_output,
            special_cache: HashMap::new(),
        };

        // init_dist(&mut state_bundle, &mut prep_output);

        // dbg!(&state_bundle, &prep_output.upgrade_arr);
        state_bundle.comput_special_probs();
        let result: Vec<f64> = state_bundle.special_probs().clone();

        if DEBUG {
            dbg!(&result);
        }
        if let Some(_cached_result) = read_cached_data::<Vec<f64>>(test_name.as_str(), &hash) {
        } else {
            write_cached_data(test_name.as_str(), &hash, &result);
        }
        if DEBUG {
            dbg!(start.elapsed());
        }
    }
}
