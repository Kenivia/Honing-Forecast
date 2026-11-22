// Status events for whoever listens (market_agent sends them to the relay, and on to Discord); nothing without a sink.
use crate::game::Game;
use image::RgbaImage;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};

type Sink = Box<dyn Fn(&str, &str, Option<&RgbaImage>) + Send + Sync>;
static SINK: OnceLock<Sink> = OnceLock::new();

/// Setup mode: a screenshot after every unit of work.
pub static SETUP: AtomicBool = AtomicBool::new(false);

/// Set by market_agent while the relay cannot be reached: a unit that fails then ends the round at once.
pub static OFFLINE: AtomicBool = AtomicBool::new(false);

pub fn set(f: impl Fn(&str, &str, Option<&RgbaImage>) + Send + Sync + 'static) {
    let _ = SINK.set(Box::new(f));
}

/// kind: "info", "error" or "step".
pub fn event(kind: &str, text: &str, img: Option<&RgbaImage>) {
    if let Some(f) = SINK.get() {
        f(kind, text, img);
    }
}

/// After a try of a unit of work; only in setup mode.
pub fn step(g: &mut Game, what: &str, ok: bool) {
    if SETUP.load(Ordering::Relaxed) {
        let text = format!("{what}: {}", if ok { "ok" } else { "failed" });
        event("step", &text, Some(&g.shot(None)));
    }
}
