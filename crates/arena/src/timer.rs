#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

pub struct Timer {
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
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
            Self {}
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
            -6.9
        }
    }
}
