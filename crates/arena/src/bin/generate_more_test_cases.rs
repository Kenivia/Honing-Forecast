use csv::{Reader, Writer};
use hf_arena::parse_test_cases::{Row, parse_csv};
use hf_core::parser::PreparationOutput;
use rand::prelude::*;
use std::path::Path;

static BLOAT_FACTOR: i64 = 10;
static MULTIPLIER_RANGE: f64 = 5.0;
fn main() {
    let mut rdr = Reader::from_path(Path::new("test_cases.csv")).unwrap();

    let mut rng: ThreadRng = rand::rng();
    let outputs: Vec<(PreparationOutput, Vec<bool>)> = parse_csv(Path::new("test_cases.csv"));
    let mut out: Vec<Row> = Vec::new();

    let mut rows: Vec<Row> = Vec::with_capacity(outputs.len());
    for result in rdr.deserialize() {
        rows.push(result.unwrap());
    }
    let mut count: i64 = 0;
    for (index, (prep_output, _tests_to_run)) in outputs.iter().enumerate() {
        let (one_tap_costs, pity_costs) = prep_output.get_one_tap_pity();

        for i in 0..BLOAT_FACTOR {
            let mut this_row: Row = rows[index].clone();
            count += 1;
            this_row.test_case = count;
            if i != 0 {
                let mut multiply_f64: Vec<&mut f64> = vec![
                    &mut this_row.red_price,
                    &mut this_row.blue_price,
                    &mut this_row.leaps_price,
                    &mut this_row.shards_price,
                    &mut this_row.oreha_price,
                    &mut this_row.silver_price,
                    &mut this_row.juice_weap_price_1,
                    &mut this_row.juice_armor_price_1,
                    &mut this_row.juice_weap_price_2,
                    &mut this_row.juice_armor_price_2,
                    &mut this_row.juice_weap_price_3,
                    &mut this_row.juice_armor_price_3,
                    &mut this_row.juice_weap_price_4,
                    &mut this_row.juice_armor_price_4,
                ];
                let old_len = multiply_f64.len();
                for price in multiply_f64.iter_mut() {
                    let this_multiplier: f64 =
                        MULTIPLIER_RANGE.powf(rng.random_range(-1.0..1.0_f64));
                    **price *= this_multiplier;
                }

                let mut leftovers: Vec<&mut f64> = vec![
                    &mut this_row.red_leftover,
                    &mut this_row.blue_leftover,
                    &mut this_row.leaps_leftover,
                    &mut this_row.shards_leftover,
                    &mut this_row.oreha_leftover,
                    &mut this_row.silver_leftover,
                    &mut this_row.juice_weap_leftover_1,
                    &mut this_row.juice_armor_leftover_1,
                    &mut this_row.juice_weap_leftover_2,
                    &mut this_row.juice_armor_leftover_2,
                    &mut this_row.juice_weap_leftover_3,
                    &mut this_row.juice_armor_leftover_3,
                    &mut this_row.juice_weap_leftover_4,
                    &mut this_row.juice_armor_leftover_4,
                    &mut this_row.gold_leftover,
                ];

                for (index, leftover) in leftovers.iter_mut().enumerate() {
                    let this_multiplier: f64 =
                        MULTIPLIER_RANGE.powf(rng.random_range(-1.0..1.0_f64));
                    **leftover *= this_multiplier;
                    if index < old_len {
                        **leftover = (**leftover).min(*multiply_f64[index]);
                    }
                }

                let pity_i64: Vec<&mut i64> = vec![
                    &mut this_row.red_owned,
                    &mut this_row.blue_owned,
                    &mut this_row.leaps_owned,
                    &mut this_row.shards_owned,
                    &mut this_row.oreha_owned,
                    &mut this_row.gold_owned,
                    &mut this_row.silver_owned,
                ];
                for (index, x) in pity_i64.into_iter().enumerate() {
                    let this_multiplier: f64 = rng.random_range(0.0..1.0_f64);
                    *x = (one_tap_costs[index] as f64
                        + this_multiplier * (pity_costs[index] - one_tap_costs[index]) as f64)
                        .round() as i64;
                }
                this_row.express = rng.random_bool(0.5);

                let multiply_i64: Vec<&mut i64> = vec![
                    &mut this_row.special_owned,
                    &mut this_row.juice_weap_1,
                    &mut this_row.juice_armor_1,
                    &mut this_row.juice_weap_2,
                    &mut this_row.juice_armor_2,
                    &mut this_row.juice_weap_3,
                    &mut this_row.juice_armor_3,
                    &mut this_row.juice_weap_4,
                    &mut this_row.juice_armor_4,
                ];
                for x in multiply_i64 {
                    let this_multiplier: f64 =
                        MULTIPLIER_RANGE.powf(rng.random_range(-1.0..1.0_f64));
                    *x = (*x as f64 * this_multiplier).round() as i64;
                }
            }
            out.push(this_row);
        }
    }

    let mut wtr: Writer<std::fs::File> =
        Writer::from_path(Path::new("bloated_test_cases.csv")).unwrap();

    for row in out {
        wtr.serialize(row).unwrap();
    }

    wtr.flush().unwrap();
}
