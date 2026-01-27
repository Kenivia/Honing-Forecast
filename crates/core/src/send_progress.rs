use serde::Serialize;
use serde_wasm_bindgen::to_value;
use web_sys::wasm_bindgen::prelude::*;

use crate::state_bundle::StateBundle;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = postMessage)]
    fn post_message(val: JsValue);

}
#[derive(Serialize)]
pub struct WasmProgress {
    state_bundle: StateBundle,
    est_progress_percentage: f64,
    r#type: String,
}

pub fn send_progress(state_bundle: &StateBundle, est_progress_percentage: f64) {
    let msg = to_value(&WasmProgress {
        state_bundle: state_bundle.clone(),
        est_progress_percentage,
        r#type: "intermediate_result".to_owned(),
    })
    .unwrap();

    post_message(msg);
}
