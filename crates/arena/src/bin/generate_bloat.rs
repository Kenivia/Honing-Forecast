use hf_arena::parse_test_cases::{Payload, read_payload_jsons};
use hf_core::state_bundle::StateBundle;
use rand::prelude::*;
use std::fs;
use std::path::Path;

static BLOAT_FACTOR: i64 = 10;
static MULTIPLIER_RANGE: f64 = 5.0;
static INPUT_DIR: &str = "test_payloads";
static OUTPUT_DIR: &str = "test_payloads_bloated";

fn random_multiplier(rng: &mut ThreadRng) -> f64 {
    MULTIPLIER_RANGE.powf(rng.random_range(-1.0..1.0_f64))
}

fn main() {
    let mut rng: ThreadRng = rand::rng();
    let payloads: Vec<(String, Payload)> = read_payload_jsons(Path::new(INPUT_DIR));
    let mut out: Vec<(String, Payload)> = Vec::new();
    let mut count: i64 = 0;

    for (base_name, payload) in payloads {
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
            payload.progress_grid.clone(),
            payload.state_grid.clone(),
            payload.special_state.clone(),
            payload.unlocked_grid.clone(),
            payload.succeeded_grid.clone(),
        );
        let (one_tap_costs, pity_costs) = state_bundle.get_one_tap_pity();

        for i in 0..BLOAT_FACTOR {
            let mut this_payload = payload.clone();
            count += 1;
            if i != 0 {
                for &index in [0_usize, 1, 2, 3, 4, 6].iter() {
                    if index < this_payload.user_price_arr.len() {
                        this_payload.user_price_arr[index] *= random_multiplier(&mut rng);
                    }
                }
                for (weap_price, armor_price) in this_payload.juice_prices.iter_mut() {
                    *weap_price *= random_multiplier(&mut rng);
                    *armor_price *= random_multiplier(&mut rng);
                }

                let mut price_clamps: Vec<f64> = Vec::new();
                for &index in [0_usize, 1, 2, 3, 4, 6].iter() {
                    if index < this_payload.user_price_arr.len() {
                        price_clamps.push(this_payload.user_price_arr[index]);
                    }
                }
                for (weap_price, armor_price) in this_payload.juice_prices.iter() {
                    price_clamps.push(*weap_price);
                    price_clamps.push(*armor_price);
                }

                let mut clamp_index = 0;
                for &index in [0_usize, 1, 2, 3, 4, 6].iter() {
                    if index >= this_payload.inp_leftover_values.len() {
                        continue;
                    }
                    let mut new_val =
                        this_payload.inp_leftover_values[index] * random_multiplier(&mut rng);
                    if clamp_index < price_clamps.len() {
                        new_val = new_val.min(price_clamps[clamp_index]);
                    }
                    this_payload.inp_leftover_values[index] = new_val;
                    clamp_index += 1;
                }
                for (weap_leftover, armor_leftover) in
                    this_payload.inp_leftover_juice_values.iter_mut()
                {
                    let mut new_val = *weap_leftover * random_multiplier(&mut rng);
                    if clamp_index < price_clamps.len() {
                        new_val = new_val.min(price_clamps[clamp_index]);
                    }
                    *weap_leftover = new_val;
                    clamp_index += 1;

                    let mut new_val = *armor_leftover * random_multiplier(&mut rng);
                    if clamp_index < price_clamps.len() {
                        new_val = new_val.min(price_clamps[clamp_index]);
                    }
                    *armor_leftover = new_val;
                    clamp_index += 1;
                }
                if this_payload.inp_leftover_values.len() > 5 {
                    this_payload.inp_leftover_values[5] *= random_multiplier(&mut rng);
                }

                for index in 0..one_tap_costs.len().min(7) {
                    if index >= this_payload.mats_budget.len() {
                        break;
                    }
                    let this_multiplier: f64 = rng.random_range(0.0..1.0_f64);
                    this_payload.mats_budget[index] = (one_tap_costs[index] as f64
                        + this_multiplier * (pity_costs[index] - one_tap_costs[index]) as f64)
                        .max(0.0)
                        .round() as i64;
                }
                this_payload.express_event = rng.random_bool(0.5);

                if this_payload.mats_budget.len() > 7 {
                    this_payload.mats_budget[7] = (this_payload.mats_budget[7] as f64
                        + 3000.0 * random_multiplier(&mut rng))
                    .max(0.0)
                    .round() as i64;
                }
                for (weap_owned, armor_owned) in this_payload.juice_books_budget.iter_mut() {
                    *weap_owned = (*weap_owned as f64 * random_multiplier(&mut rng)).round() as i64;
                    *armor_owned =
                        (*armor_owned as f64 * random_multiplier(&mut rng)).round() as i64;
                }
            }
            let safe_base = if base_name.is_empty() || base_name == "N/A" {
                "payload".to_string()
            } else {
                base_name.clone()
            };
            let file_name = format!("{}_{:05}.json", safe_base, count,);
            out.push((file_name, this_payload));
        }
    }

    fs::create_dir_all(OUTPUT_DIR).unwrap();
    for (file_name, payload) in out {
        let path = Path::new(OUTPUT_DIR).join(file_name);
        let contents = serde_json::to_string_pretty(&payload).unwrap();
        fs::write(path, contents).unwrap();
    }
}
