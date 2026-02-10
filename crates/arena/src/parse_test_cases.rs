use hf_core::payload::Payload;
use hf_core::state_bundle::StateBundle;
use std::fs;
use std::path::Path;

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
