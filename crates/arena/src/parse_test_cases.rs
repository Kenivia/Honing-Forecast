// use csv::Reader;
use hf_core::state_bundle::StateBundle;
// use hf_core::{helpers::naive_count_to_ticks, parser::PreparationOutput, upgrade::Upgrade};
// use paste::paste;
// use seq_macro::seq;
use hf_core::payload::Payload;

use std::fs;
use std::path::Path;
// seq!(N in 1..=25 {
// #[derive(Debug, Deserialize, Serialize, Clone)]
//     pub struct Row {
//         pub test_case: i64,
//         #(
//             #[serde(deserialize_with = "empty_as_default")]
//              armor_~N: i64,
//              )*
//         #(
//             #[serde(deserialize_with = "empty_as_default")]
//          weap_~N: i64,
//          )*
//          #[serde(deserialize_with = "empty_as_default")]
//      pub      red_owned: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//       pub     blue_owned: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//       pub     leaps_owned: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//        pub    shards_owned: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//       pub     oreha_owned: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//       pub     gold_owned: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//       pub     silver_owned: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//      pub   special_owned: i64,
//       pub     red_price: f64,
//       pub     blue_price: f64,
//       pub     leaps_price: f64,
//      pub      shards_price: f64,
//      pub      oreha_price: f64,
//      pub      silver_price: f64,

//      #[serde(deserialize_with="empty_as_negative")]
//            pub     red_leftover: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//       pub     blue_leftover: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//       pub     leaps_leftover: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//      pub      shards_leftover: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//      pub      oreha_leftover: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//      pub gold_leftover:f64,
//      #[serde(deserialize_with="empty_as_negative")]
//      pub      silver_leftover: f64,

//         #[serde(deserialize_with = "empty_as_default")]
//         adv_armor_10: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//         adv_armor_20: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//         adv_armor_30: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//         adv_armor_40: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//         adv_weap_10: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//         adv_weap_20: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//         adv_weap_30: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//         adv_weap_40: i64,

//         #[serde(deserialize_with = "deserialize_bool")]
//         pub express: bool,

//          #[serde(deserialize_with = "empty_as_default")]
//      pub      juice_weap_1: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//     pub       juice_armor_1: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//      pub      juice_weap_2: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//      pub      juice_armor_2: i64,
//          #[serde(deserialize_with = "empty_as_default")]
//       pub     juice_weap_3: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//        pub    juice_armor_3: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//       pub     juice_weap_4: i64,
//         #[serde(deserialize_with = "empty_as_default")]
//        pub    juice_armor_4: i64,

//       pub     juice_weap_price_1: f64,
//       pub     juice_armor_price_1: f64,
//        pub    juice_weap_price_2: f64,
//       pub     juice_armor_price_2: f64,
//       pub     juice_weap_price_3: f64,
//        pub    juice_armor_price_3: f64,
//        pub    juice_weap_price_4: f64,
//        pub    juice_armor_price_4: f64,

//      #[serde(deserialize_with="empty_as_negative")]
//       pub     juice_weap_leftover_1: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//       pub     juice_armor_leftover_1: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//        pub    juice_weap_leftover_2: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//       pub     juice_armor_leftover_2: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//       pub     juice_weap_leftover_3: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//        pub    juice_armor_leftover_3: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//        pub    juice_weap_leftover_4: f64,
//      #[serde(deserialize_with="empty_as_negative")]
//        pub    juice_armor_leftover_4: f64,

//     #[serde(deserialize_with = "deserialize_bool")]
//     pub saddlepoint_approximation : bool ,
//     #[serde(deserialize_with = "deserialize_bool")]
//      pub average: bool,
//      #[serde(deserialize_with = "deserialize_bool")]
//      pub brute: bool
//     }

// });

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct Payload {
//     pub normal_hone_ticks: Vec<Vec<bool>>,
//     pub adv_hone_ticks: Vec<Vec<bool>>,
//     pub adv_hone_strategy: String,

//     pub express_event: bool,
//     pub bucket_count: usize,

//     pub data_size: usize,
//     pub mats_budget: Vec<i64>,
//     pub user_price_arr: Vec<f64>,
//     pub inp_leftover_values: Vec<f64>,
//     pub juice_books_budget: Vec<(i64, i64)>,
//     pub juice_prices: Vec<(f64, f64)>,
//     pub inp_leftover_juice_values: Vec<(f64, f64)>,

//     pub progress_grid: Option<Vec<Vec<usize>>>,
//     pub state_grid: Option<Vec<Vec<Vec<(bool, usize)>>>>,
//     pub special_state: Option<Vec<usize>>,
//     pub unlocked_grid: Option<Vec<Vec<bool>>>,
//     pub succeeded_grid: Option<Vec<Vec<bool>>>,
//     pub min_resolution: usize,
// }
// macro_rules! row_to_vec {
//     ($instance:expr, $prefix:ident, $start:literal, $end:literal) => {

//         seq!(N in $start..=$end {
//             paste! { vec![
//                 #( $instance.[<$prefix~N>].clone(), )*
//             ]
//         }
//         })

//     };
// }
// macro_rules! row_to_vec_zero {
//     ($instance:expr, $prefix:ident, $start:literal, $end:literal) => {

//         seq!(N in $start..=$end {
//             paste! { vec![
//                 #( $instance.[<$prefix~N 0>].clone(), )*
//             ]
//         }
//         })

//     };
// }

// fn empty_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
// where
//     D: Deserializer<'de>,
//     T: Deserialize<'de> + Default,
// {
//     let opt = Option::<T>::deserialize(deserializer)?;
//     Ok(opt.unwrap_or_default())
// }

// fn empty_as_negative<'de, D>(deserializer: D) -> Result<f64, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let opt = Option::<f64>::deserialize(deserializer)?;
//     Ok(opt.unwrap_or(-1.0))
// }
// fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s = String::deserialize(deserializer)?;
//     match s.to_lowercase().as_str() {
//         "true" => Ok(true),
//         "false" => Ok(false),
//         _ => Err(serde::de::Error::custom(format!("invalid boolean: {}", s))),
//     }
// }

// pub fn parse_csv(path: &Path) -> Vec<(StateBundle, Vec<bool>)> {
//     let mut rdr = Reader::from_path(path).unwrap();

//     let mut out: Vec<(StateBundle, Vec<bool>)> = Vec::new();

//     for result in rdr.deserialize() {
//         let row: Row = result.unwrap();
//         let hone_counts: Vec<Vec<i64>> = vec![
//             row_to_vec!(row, armor_, 1, 25),
//             row_to_vec!(row, weap_, 1, 25),
//         ];
//         let adv_counts: Vec<Vec<i64>> = vec![
//             row_to_vec_zero!(row, adv_armor_, 1, 4),
//             row_to_vec_zero!(row, adv_weap_, 1, 4),
//         ];
//         let temp_juice_books_owned: Vec<Vec<i64>> = vec![
//             row_to_vec!(row, juice_weap_, 1, 4),
//             row_to_vec!(row, juice_armor_, 1, 4),
//         ];
//         let juice_books_owned: Vec<(i64, i64)> = (0..4_usize)
//             .into_iter()
//             .map(|x| (temp_juice_books_owned[0][x], temp_juice_books_owned[1][x]))
//             .collect();

//         let temp_juice_prices: Vec<Vec<f64>> = vec![
//             row_to_vec!(row, juice_weap_price_, 1, 4),
//             row_to_vec!(row, juice_armor_price_, 1, 4),
//         ];
//         let juice_prices: Vec<(f64, f64)> = (0..4_usize)
//             .into_iter()
//             .map(|x| (temp_juice_prices[0][x], temp_juice_prices[1][x]))
//             .collect();

//         let temp_juice_leftover: Vec<Vec<f64>> = vec![
//             row_to_vec!(row, juice_weap_leftover_, 1, 4),
//             row_to_vec!(row, juice_armor_leftover_, 1, 4),
//         ];
//         let juice_leftover: Vec<(f64, f64)> = (0..4_usize)
//             .into_iter()
//             .map(|x| (temp_juice_leftover[0][x], temp_juice_leftover[1][x]))
//             .collect();
//         let budget: Vec<i64> = vec![
//             row.red_owned,
//             row.blue_owned,
//             row.leaps_owned,
//             row.shards_owned,
//             row.oreha_owned,
//             row.gold_owned,
//             row.silver_owned,
//             row.special_owned,
//         ];

//         let prices: Vec<f64> = vec![
//             row.red_price,
//             row.blue_price,
//             row.leaps_price,
//             row.shards_price,
//             row.oreha_price,
//             1.0,
//             row.silver_price,
//         ];

//         let leftover_values: Vec<f64> = vec![
//             row.red_leftover,
//             row.blue_leftover,
//             row.leaps_leftover,
//             row.shards_leftover,
//             row.oreha_leftover,
//             row.gold_leftover,
//             row.silver_leftover,
//         ];

//         let (mut this, upgrade_arr): (PreparationOutput, Vec<Upgrade>) =
//             PreparationOutput::initialize(
//                 &naive_count_to_ticks(&hone_counts),
//                 &budget,
//                 &naive_count_to_ticks(&adv_counts),
//                 row.express,
//                 &prices,
//                 "x2 balls",
//                 &juice_books_owned,
//                 &juice_prices,
//                 &leftover_values,
//                 &juice_leftover,
//                 None,
//                 None,
//                 None,
//                 None,
//             );

//         this.test_case = row.test_case;

//         let tests_to_run: Vec<bool> = vec![row.saddlepoint_approximation, row.average, row.brute];
//         let minimum: f64 = leftover_values
//             .iter()
//             .fold(f64::INFINITY, |a, &b| a.min(b))
//             .min(
//                 juice_leftover
//                     .iter()
//                     .fold(f64::INFINITY, |a, &b| a.min(b.0.min(b.1))),
//             );
//         if row.average {
//             assert!(minimum >= 0.0);
//         }
//         out.push((StateBundle::new(this, upgrade_arr), tests_to_run));
//     }
//     out
// }

pub fn read_payload_jsons(path: &Path) -> Vec<(String, Payload)> {
    if !path.exists() {
        return Vec::new();
    }
    let mut entries: Vec<_> = fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();
    entries.sort_by_key(|entry| entry.path());

    let mut out: Vec<(String, Payload)> = Vec::new();
    for entry in entries {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }
        if file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            != Some(true)
        {
            continue;
        }
        let test_case_name = file_path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let contents = fs::read_to_string(&file_path).unwrap();
        let payload: Payload = serde_json::from_str(&contents).unwrap();
        out.push((test_case_name, payload));
    }
    out
}

pub fn parse_payload_jsons(path: &Path) -> Vec<(String, StateBundle, Vec<bool>)> {
    let mut out: Vec<(String, StateBundle, Vec<bool>)> = Vec::new();
    for (test_case_name, payload) in read_payload_jsons(path) {
        let state_bundle = StateBundle::init_from_inputs(
            &payload.normal_hone_ticks,
            &payload.mats_budget,
            &payload.adv_hone_ticks,
            payload.express_event,
            &payload.user_price_arr,
            &payload.adv_hone_strategy,
            &payload.juice_books_budget,
            &payload.juice_prices,
            &payload.inp_leftover_values,
            &payload.inp_leftover_juice_values,
            payload.progress_grid,
            payload.state_grid,
            payload.special_state,
            payload.unlocked_grid,
            payload.succeeded_grid,
            payload.min_resolution,
            payload.num_threads,
            payload.metric_type,
        );
        let tests_to_run: Vec<bool> = vec![true, true];
        out.push((test_case_name, state_bundle, tests_to_run));
    }
    out
}
