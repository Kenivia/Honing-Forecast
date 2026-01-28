#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use web_sys::Performance;

pub struct Timer {
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,

    #[cfg(target_arch = "wasm32")]
    start_ms: f64,
}

impl Timer {
    pub fn start() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                start: Instant::now(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let perf = web_sys::window().unwrap().performance().unwrap();

            Self {
                start_ms: perf.now(),
            }
        }
    }

    /// elapsed time in milliseconds
    pub fn elapsed_sec(&self) -> f64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start.elapsed().as_secs_f64()
        }

        #[cfg(target_arch = "wasm32")]
        {
            let perf = web_sys::window().unwrap().performance().unwrap();

            (perf.now() - self.start_ms) * 1000.0
        }
    }
}
