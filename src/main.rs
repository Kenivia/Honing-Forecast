use honing_forecast::*;
use js_sys::JSON;
use wasm_bindgen::JsValue;

pub fn main() {
    let js_str: &str = r#"{"normal_hone_ticks":[[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false],[false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,false,false,false,false]],"adv_hone_ticks":[[false,false,true,false],[false,false,true,false],[false,false,true,false],[false,false,true,false],[false,false,true,false],[false,false,true,false]],"budget":[431777,1064398,23748,9010948,15125,1803792,4294967295,0,0,0],"desired_chance":0.69,"adv_hone_strategy":"No juice"}"#;
    let parsed: JsValue = JSON::parse(js_str).unwrap();
    // let my_data: Payload = serde_json::from_str(js_str).unwrap();

    let out: JsValue = chance_to_cost_wrapper(parsed);
    let z: String = JSON::stringify(&out).unwrap().into();
    assert_eq!("a", z);
}
