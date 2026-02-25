use crate::state_bundle::StateBundle;
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{Request, RequestInit, RequestMode, Response};

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

// #[wasm_bindgen]
// pub async fn fetch_binary(url: String) -> Result<Vec<u8>, JsValue> {
//     let window = web_sys::window().unwrap();

//     let opts = RequestInit::new();
//     opts.set_method("GET");
//     opts.set_mode(RequestMode::Cors);

//     let request = Request::new_with_str_and_init(&url, &opts)?;

//     let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
//     let resp: Response = resp_value.dyn_into()?;

//     let buffer = JsFuture::from(resp.array_buffer()?).await?;
//     let u8_array = js_sys::Uint8Array::new(&buffer);

//     Ok(u8_array.to_vec())
// }
