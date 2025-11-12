use crate::helpers::generate_first_deltas;
use crate::parser::{PreparationOutputs, Upgrade, probability_distribution};
use crate::value_estimation::explore_one;
use itertools::{Itertools, iproduct};

fn decision_space_iterator(upgrade_arr: &[Upgrade]) -> impl Iterator<Item = (Vec<i64>, Vec<i64>)> {
    let mut max_juice_counts: Vec<i64> = Vec::with_capacity(upgrade_arr.len());
    for upgrade in upgrade_arr {
        max_juice_counts.push(upgrade.full_juice_len as i64);
    }
    let juice_decision_space: Vec<Vec<i64>> = max_juice_counts
        .into_iter()
        .map(|x: i64| (0..x).collect())
        .collect();

    // decision_space.push();
    // dbg!(&juice_decision_space);
    iproduct!(
        juice_decision_space.into_iter().multi_cartesian_product(),
        vec![vec![0]].into_iter() //(0..upgrade_arr.len() as i64).permutations(10.min(upgrade_arr.len()))
    )
}

fn brute(
    input_budgets: &[i64],
    prep_outputs: &PreparationOutputs,
    data_size: usize,
) -> Vec<Vec<f64>> {
    let mut out: Vec<Vec<f64>> = vec![
        vec![0.0; prep_outputs.upgrade_arr[1].full_juice_len];
        prep_outputs.upgrade_arr[0].full_juice_len
    ];

    for decision in decision_space_iterator(&prep_outputs.upgrade_arr) {
        // dbg!(&decision);
        out[decision.0[0] as usize][decision.0[1] as usize] =
            explore_one(&decision, &input_budgets, prep_outputs, data_size)[0];
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parser, preparation};
    use crate::test_utils::*;
    use crate::{calculate_hash, my_assert};

    #[test]
    fn brute_test() {
        let test_name: &str = "brute_test";
        let hone_counts: Vec<Vec<i64>> = vec![
            (0..25).map(|x| if x == 23 { 1 } else { 0 }).collect(),
            (0..25).map(|x| if x == 24 { 1 } else { 0 }).collect(),
        ];
        let adv_counts: Vec<Vec<i64>> =
            vec![(0..4).map(|_| 0).collect(), (0..4).map(|_| 0).collect()];

        let adv_hone_strategy: &str = "No juice";
        let express_event: bool = true;
        let input_budgets = vec![0, 63600, 0, 0, 0, 69, 0, 0, 0, 0];
        let user_mats_value = DEFAULT_GOLD_VALUES;
        let data_size: usize = 10000;
        let hash: String = calculate_hash!(
            &hone_counts,
            &adv_counts,
            adv_hone_strategy,
            express_event,
            &input_budgets,
            &user_mats_value
        );

        let prep_outputs: PreparationOutputs = preparation(
            &hone_counts,
            &input_budgets,
            &adv_counts,
            express_event,
            &user_mats_value,
            adv_hone_strategy,
        );
        let result: Vec<Vec<f64>> = brute(&input_budgets, &prep_outputs, data_size);
        dbg!(result.len());
        // let result: Vec<Vec<i64>> = out.clone();
        if let Some(cached_result) = read_cached_data::<Vec<Vec<f64>>>(test_name, &hash) {
            my_assert!(*result, cached_result);
        } else {
            write_cached_data(test_name, &hash, &result);
        }
        // let result: Vec<(Vec<i64>, Vec<i64>)> = brute(&mut upgrade_arr);
        // dbg!(result.len());
        // // let result: Vec<Vec<i64>> = out.clone();
        // if let Some(cached_result) = read_cached_data::<Vec<(Vec<i64>, Vec<i64>)>>(test_name, &hash)
        // {
        //     my_assert!(*result, cached_result);
        // } else {
        //     write_cached_data(test_name, &hash, &result);
        // }
    }
}
