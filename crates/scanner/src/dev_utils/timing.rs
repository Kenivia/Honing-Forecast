use ahash::AHashMap;
use parking_lot::Mutex;
use std::sync::LazyLock;

// name -> (total, calls) since the last drain; ms for timers
pub static TIMINGS: LazyLock<Mutex<AHashMap<&'static str, (f64, u32)>>> =
    LazyLock::new(|| Mutex::new(AHashMap::new()));

#[cfg(target_arch = "wasm32")]
pub fn now() -> f64 {
    use js_sys::wasm_bindgen::{JsCast, JsValue};
    let performance =
        js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance")).unwrap();
    js_sys::Reflect::get(&performance, &JsValue::from_str("now"))
        .unwrap()
        .unchecked_into::<js_sys::Function>()
        .call0(&performance)
        .unwrap()
        .as_f64()
        .unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> f64 {
    static START: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);
    START.elapsed().as_secs_f64() * 1000.0
}

pub fn record(name: &'static str, value: f64) {
    let mut timings = TIMINGS.lock();
    let entry = timings.entry(name).or_default();
    entry.0 += value;
    entry.1 += 1;
}

pub fn timed<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    let start = now();
    let out = f();
    record(name, now() - start);
    out
}

pub fn take() -> Vec<(&'static str, f64, u32)> {
    TIMINGS.lock().drain().map(|(name, (total, calls))| (name, total, calls)).collect()
}
