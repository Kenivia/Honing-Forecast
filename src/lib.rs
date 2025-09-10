mod chance_to_cost;
mod constants;
mod cost_to_chance;
mod helpers;
mod monte_carlos;
mod parser;
use crate::chance_to_cost::chance_to_cost;
use crate::cost_to_chance::cost_to_chance;
use crate::helpers::ticks_to_counts;

use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
pub struct Payload {
    normal_hone_ticks: Vec<Vec<bool>>,
    adv_hone_ticks: Vec<Vec<bool>>,
    desired_chance: f32,
    adv_hone_strategy: String,
    budget: Vec<i64>,
}

#[wasm_bindgen]
pub fn chance_to_cost_wrapper(input: JsValue) -> JsValue {
    let payload: Payload = from_value(input).unwrap();
    let normal_hone_ticks: Vec<Vec<bool>> = payload.normal_hone_ticks;
    let adv_hone_ticks: Vec<Vec<bool>> = payload.adv_hone_ticks;
    let desired_chance: f32 = payload.desired_chance;
    let adv_hone_strategy: String = payload.adv_hone_strategy;

    let out: i64 = chance_to_cost(
        ticks_to_counts(normal_hone_ticks),
        ticks_to_counts(adv_hone_ticks),
        desired_chance,
        adv_hone_strategy,
    );

    // 3) return JS array/object
    to_value(&out).unwrap()
}

#[wasm_bindgen]
pub fn cost_to_chance_wrapper(input: JsValue) -> JsValue {
    let payload: Payload = from_value(input).unwrap();
    let normal_hone_ticks: Vec<Vec<bool>> = payload.normal_hone_ticks;
    let adv_hone_ticks: Vec<Vec<bool>> = payload.adv_hone_ticks;
    let budget: Vec<i64> = payload.budget;
    println!("here");
    let (chance, reason): (f64, String) = cost_to_chance(
        ticks_to_counts(normal_hone_ticks),
        budget,
        ticks_to_counts(adv_hone_ticks),
        String::from("No juice"),
    );

    to_value(&(chance, reason)).unwrap()
}

// #[cfg(test)]
// mod tests {
//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     use super::*;
//     use js_sys::JSON;
//     #[test]
//     fn test_demo() {
//         let js_str: &str = r#"{"normal_hone_ticks":[[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false]],"adv_hone_ticks":[[false,false,true,false],[false,false,true,false],[false,false,true,false],[false,false,true,false],[false,false,true,false],[false,false,true,false]],"budget":{"Red":431777,"Blue":1064398,"Leaps":23748,"Shards":9010948,"Oreha":15125,"Gold":1803792,"Silver(WIP)":4294967295,"Red juice":0,"Blue juice":0,"Special leaps":0}}"#;
//         let parsed: JsValue = JSON::parse(js_str).unwrap();
//         // let my_data: Payload = serde_json::from_str(js_str).unwrap();
//         let out: JsValue = chance_to_cost_wrapper(parsed);
//         let z: String = JSON::stringify(&out).unwrap().into();
//         println!("{}", z);
//         assert!(false);
//     }
// }
