use std::path::Path;

use hf_core::parser::{PreparationOutputs, preparation};
use paste::paste;
use seq_macro::seq;
use serde::Deserialize;
seq!(N in 1..=25 {
#[derive(Debug, Deserialize)]
    struct Row {
        #( armor_~N: i64, )*
        #( weap_~N: i64, )*
        red_owned: i64,
        blue_owned: i64,
        leaps_owned: i64,
        shards_owned: i64,
        oreha_owned: i64,
        gold_owned: i64,
        silver_owned: i64,
        red_juice_owned: i64,
        blue_juice_owned: i64,
        special_owned: i64,
        red_price: f64,
        blue_price: f64,
        leaps_price: f64,
        shards_price: f64,
        oreha_price: f64,
        silver_price: f64,
        red_juice_price: f64,
        blue_juice_price: f64,
        adv_armor_10: i64,
        adv_armor_20: i64,
        adv_armor_30: i64,
        adv_armor_40: i64,
        adv_weap_10: i64,
        adv_weap_20: i64,
        adv_weap_30: i64,
        adv_weap_40: i64,
        express: bool,
    }

});
macro_rules! row_to_vec {
    ($instance:expr, $prefix:ident, $start:literal, $end:literal) => {

        seq!(N in $start..=$end {
            paste! { vec![
                #( $instance.[<$prefix~N>].clone(), )*
            ]
        }
        })

    };
}
macro_rules! row_to_vec_zero {
    ($instance:expr, $prefix:ident, $start:literal, $end:literal) => {

        seq!(N in $start..=$end {
            paste! { vec![
                #( $instance.[<$prefix~N 0>].clone(), )*
            ]
        }
        })

    };
}
pub fn parse_csv(path: &Path) -> Result<Vec<PreparationOutputs>, csv::Error> {
    let mut rdr = csv::Reader::from_path(path)?;

    let mut out: Vec<PreparationOutputs> = Vec::new();

    for result in rdr.deserialize() {
        let row: Row = result?;
        let hone_counts: Vec<Vec<i64>> = vec![
            row_to_vec!(row, armor_, 1, 25),
            row_to_vec!(row, weap_, 1, 25),
        ];
        let adv_counts: Vec<Vec<i64>> = vec![
            row_to_vec_zero!(row, adv_armor_, 1, 4),
            row_to_vec_zero!(row, adv_weap_, 1, 4),
        ];
        let budget: Vec<i64> = vec![
            row.red_owned,
            row.blue_owned,
            row.leaps_owned,
            row.shards_owned,
            row.oreha_owned,
            row.gold_owned,
            row.silver_owned,
            row.red_juice_owned,
            row.blue_juice_owned,
            row.special_owned,
        ];

        let prices: Vec<f64> = vec![
            row.red_price,
            row.blue_price,
            row.leaps_price,
            row.shards_price,
            row.oreha_price,
            1.0,
            row.silver_price,
            row.red_juice_price,
            row.blue_juice_price,
        ];
        out.push(preparation(
            &hone_counts,
            &budget,
            &adv_counts,
            row.express,
            &prices,
            "No juice",
        ));
    }
    Ok(out)
}
