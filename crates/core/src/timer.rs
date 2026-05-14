#[cfg(not(feature = "wasm"))]
use std::time::Instant;

#[cfg(feature = "wasm")]
use web_sys::js_sys;
/// A minimal elapsed-time helper that compiles on both native and wasm32 targets.
///
/// - **Native / run_tests**: backed by `std::time::Instant` (monotonic, nanosecond resolution).
/// - **wasm**: backed by `js_sys::Date::now()` (millisecond resolution, which is more than
///   sufficient for the 1-second progress cadence we use in the solver).
pub struct Timer {
    #[cfg(not(feature = "wasm"))]
    start: Instant,

    /// Milliseconds since the Unix epoch, captured at construction time.
    #[cfg(feature = "wasm")]
    start_ms: f64,
}

impl Timer {
    pub fn start() -> Self {
        #[cfg(not(feature = "wasm"))]
        {
            Self {
                start: Instant::now(),
            }
        }

        #[cfg(feature = "wasm")]
        {
            Self {
                start_ms: js_sys::Date::now(),
            }
        }
    }

    /// Returns elapsed time in seconds.
    pub fn elapsed_sec(&self) -> f64 {
        #[cfg(not(feature = "wasm"))]
        {
            self.start.elapsed().as_secs_f64()
        }

        #[cfg(feature = "wasm")]
        {
            (js_sys::Date::now() - self.start_ms) / 1000.0
        }
    }
}
