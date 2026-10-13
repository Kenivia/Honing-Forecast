//! Instrumentation handed through the evaluator and the optimizer.
//!
//! Without the `trace` feature the struct is empty and every method does nothing, so
//! production pays for none of it. With it, counters are always kept and rows are tagged
//! with the iteration last given to `at`. Guard anything written per iteration with
//! `detail()`, so that a timed run only pays for the counters.

macro_rules! counters {
    ($($name:ident),* $(,)?) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Counter { $($name),* }
        pub const COUNTER_NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}

counters!(
    StatesEvaluated,
    Trivial,
    Brute,
    Lattice,
    Saddlepoint,
    Lugannani,
    Edgeworth,
    Conditioned,
    Ks,
    RootIter,
    Householder,
    Bisection,
    TaylorSkip,
);

#[cfg(feature = "trace")]
#[derive(Debug, Clone, Default)]
pub struct ProbeData {
    pub detail: bool,
    pub iter: i64,
    pub counts: [i64; COUNTER_NAMES.len()],
    /// wall seconds a solve spent evaluating; the rest of it is the optimizer deciding
    pub eval_s: f64,
    pub rows: Vec<(i64, &'static str, f64)>,
    pub texts: Vec<(i64, &'static str, String)>,
}

#[cfg(feature = "trace")]
pub type Clock = std::time::Instant;
#[cfg(not(feature = "trace"))]
pub type Clock = ();

#[derive(Debug, Clone, Default)]
pub struct Probe {
    #[cfg(feature = "trace")]
    pub data: ProbeData,
}

impl Probe {
    #[cfg(feature = "trace")]
    pub fn detailed() -> Probe {
        let mut out = Probe::default();
        out.data.detail = true;
        out
    }

    #[inline(always)]
    pub fn detail(&self) -> bool {
        #[cfg(feature = "trace")]
        return self.data.detail;
        #[cfg(not(feature = "trace"))]
        return false;
    }

    #[inline(always)]
    pub fn count(&mut self, _counter: Counter) {
        #[cfg(feature = "trace")]
        {
            self.data.counts[_counter as usize] += 1;
        }
    }

    #[inline(always)]
    pub fn clock(&self) -> Clock {
        #[cfg(feature = "trace")]
        return std::time::Instant::now();
    }

    /// what ran since the clock was taken was an evaluation
    #[inline(always)]
    pub fn evaluated(&mut self, _since: Clock) {
        #[cfg(feature = "trace")]
        {
            self.data.eval_s += _since.elapsed().as_secs_f64();
        }
    }

    /// the iteration that rows written from here on belong to
    #[inline(always)]
    pub fn at(&mut self, _iter: i64) {
        #[cfg(feature = "trace")]
        {
            self.data.iter = _iter;
        }
    }

    #[inline(always)]
    pub fn put(&mut self, _key: &'static str, _value: f64) {
        #[cfg(feature = "trace")]
        self.data.rows.push((self.data.iter, _key, _value));
    }

    /// takes a closure so that building the text costs nothing when tracing is off
    #[inline(always)]
    pub fn put_text(&mut self, _key: &'static str, _text: impl FnOnce() -> String) {
        #[cfg(feature = "trace")]
        self.data.texts.push((self.data.iter, _key, _text()));
    }
}
