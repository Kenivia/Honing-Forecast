use std::path::Path;

use hf_core::parser::{PreparationOutputs, preparation};
use paste::paste;
use seq_macro::seq;
use serde::{Deserialize, Deserializer};
seq!(N in 1..=25 {
#[derive(Debug, Deserialize)]
    struct Row {
        test_case: i64,

        #(
            #[serde(deserialize_with = "empty_as_default")]
             armor_~N: i64,
             )*
        #(
            #[serde(deserialize_with = "empty_as_default")]
         weap_~N: i64,
         )*
         #[serde(deserialize_with = "empty_as_default")]
        red_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        blue_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        leaps_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        shards_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        oreha_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        gold_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        silver_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        red_juice_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        blue_juice_owned: i64,
        #[serde(deserialize_with = "empty_as_default")]
        special_owned: i64,
        red_price: f64,
        blue_price: f64,
        leaps_price: f64,
        shards_price: f64,
        oreha_price: f64,
        silver_price: f64,
        red_juice_price: f64,
        blue_juice_price: f64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_armor_10: i64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_armor_20: i64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_armor_30: i64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_armor_40: i64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_weap_10: i64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_weap_20: i64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_weap_30: i64,
        #[serde(deserialize_with = "empty_as_default")]
        adv_weap_40: i64,

        #[serde(deserialize_with = "deserialize_bool")]
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

fn empty_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let opt = Option::<T>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom(format!("invalid boolean: {}", s))),
    }
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
        let mut this = preparation(
            &hone_counts,
            &budget,
            &adv_counts,
            row.express,
            &prices,
            "No juice",
        );
        this.test_case = row.test_case;
        out.push(this);
    }
    Ok(out)
}
