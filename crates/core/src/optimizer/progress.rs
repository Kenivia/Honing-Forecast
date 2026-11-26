//! Progress of a solve, sent through a hook the caller sets (the wasm crate posts it to JS).
//! Without one, nothing is sent and the annealer skips the work of preparing it.

use crate::honing_utils::state_bundle::StateBundle;
use std::sync::OnceLock;

pub type ProgressFn = fn(Option<&StateBundle>, f64);

static PROGRESS: OnceLock<ProgressFn> = OnceLock::new();

pub fn set_progress(f: ProgressFn) {
    let _ = PROGRESS.set(f);
}

pub fn enabled() -> bool {
    PROGRESS.get().is_some()
}

pub fn report(state_bundle: Option<&StateBundle>, pct: f64) {
    if let Some(f) = PROGRESS.get() {
        f(state_bundle, pct)
    }
}
