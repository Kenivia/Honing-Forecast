//! What a variant's binary does: one job in on stdin, one line per result on stdout.
//!
//! The driver may be a newer build than the worker, so fields added here later need a
//! serde default.

use crate::monte_carlo::{McRow, McSpec, check};
use crate::scenario::Scenario;
use cpu_time::ThreadTime;
use hf_core::optimizer::{Previous, scramble, solve};
use hf_core::probe::{COUNTER_NAMES, Probe};
use hf_core::state::OneState;
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::time::Instant;

#[derive(Clone, Serialize, Deserialize)]
pub enum Kind {
    /// run the optimizer on every step
    Optimize,
    /// check the evaluator on every step's given state and on this many scrambled ones
    Verify { scrambles: usize },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Job {
    pub scenario: Scenario,
    pub kind: Kind,
    pub seed: u64,
    /// hand each step what the step before it produced
    pub chained: bool,
    pub overrides: Map<String, Value>,
    pub detail: bool,
    pub mc: Option<McSpec>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Essence {
    pub state_arr: Vec<Vec<OneState>>,
    pub special_state: Vec<usize>,
}

impl Essence {
    pub fn of(state_bundle: &StateBundle) -> Essence {
        Essence {
            state_arr: state_bundle
                .upgrade_arr
                .iter()
                .map(|upgrade| upgrade.state.payload.clone())
                .collect(),
            special_state: state_bundle.special_state.clone(),
        }
    }

    pub fn apply(&self, state_bundle: &mut StateBundle) {
        for (state, upgrade) in self.state_arr.iter().zip(state_bundle.upgrade_arr.iter_mut()) {
            upgrade.state.update_payload(state.clone());
        }
        state_bundle.special_state = self.special_state.clone();
    }
}

#[derive(Serialize, Deserialize)]
pub struct StepResult {
    pub step: usize,
    /// which state this is about: "best", "given" or "scramble N"
    pub source: String,
    pub tuning: Value,
    /// for Optimize the whole solve, for Verify one evaluation
    pub cpu_s: f64,
    pub wall_s: f64,
    pub metric: f64,
    pub essence: Essence,
    pub blob_bytes: usize,
    pub counters: BTreeMap<String, i64>,
    pub rows: Vec<(i64, String, f64)>,
    pub texts: Vec<(i64, String, String)>,
    pub mc: Vec<McRow>,
}

pub fn tuning_with(overrides: &Map<String, Value>) -> Tuning {
    let mut value = serde_json::to_value(Tuning::default()).unwrap();
    for (key, new) in overrides {
        let old = value.as_object_mut().unwrap().get_mut(key);
        *old.unwrap_or_else(|| panic!("no tuning field {key}")) = new.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn result(step: usize, source: String, tuning: &Tuning, state_bundle: &StateBundle, probe: Probe) -> StepResult {
    StepResult {
        step,
        source,
        tuning: serde_json::to_value(tuning).unwrap(),
        cpu_s: 0.0,
        wall_s: 0.0,
        metric: state_bundle.metric,
        essence: Essence::of(state_bundle),
        blob_bytes: 0,
        counters: COUNTER_NAMES
            .iter()
            .zip(probe.data.counts)
            .map(|(name, count)| (name.to_string(), count))
            .collect(),
        rows: probe.data.rows.into_iter().map(|(iter, key, v)| (iter, key.to_string(), v)).collect(),
        texts: probe.data.texts.into_iter().map(|(iter, key, v)| (iter, key.to_string(), v)).collect(),
        mc: Vec::new(),
    }
}

fn emit(out: &StepResult) {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{}", serde_json::to_string(out).unwrap()).unwrap();
    stdout.flush().unwrap();
}

fn new_probe(detail: bool) -> Probe {
    if detail { Probe::detailed() } else { Probe::default() }
}

/// CPU seconds of one evaluation, repeated for long enough to measure
fn time_evaluation(state_bundle: &mut StateBundle) -> f64 {
    let wall = Instant::now();
    let cpu = ThreadTime::now();
    let mut reps: usize = 0;
    while wall.elapsed().as_secs_f64() < 0.3 {
        state_bundle.metric_router(&mut Probe::default());
        reps += 1;
    }
    cpu.elapsed().as_secs_f64() / reps as f64
}

pub fn run_job(job: &Job) {
    let tuning = tuning_with(&job.overrides);
    let mut previous: Option<Previous> = None;

    for (step, payload) in job.scenario.steps.iter().enumerate() {
        // a step gets the same seed whether or not it is run as part of a chain
        let mut rng = StdRng::seed_from_u64(job.seed.wrapping_mul(1_000_003).wrapping_add(step as u64));
        let mut state_bundle = StateBundle::init_from_payload(payload.clone());
        state_bundle.tuning = tuning.clone();

        match job.kind {
            Kind::Optimize => {
                let mut probe = new_probe(job.detail);
                let wall = Instant::now();
                let cpu = ThreadTime::now();
                let solved = solve(
                    &mut rng,
                    state_bundle,
                    previous.as_ref().filter(|_| job.chained),
                    &mut probe,
                );
                let cpu_s = cpu.elapsed().as_secs_f64();
                let wall_s = wall.elapsed().as_secs_f64();

                let mut best = solved.best;
                let mut out = result(step, "best".to_string(), &tuning, &best, probe);
                out.cpu_s = cpu_s;
                out.wall_s = wall_s;
                out.blob_bytes = solved.blob.len();
                if let Some(spec) = &job.mc {
                    out.mc = check(&mut best, spec, &mut rng);
                }
                emit(&out);
                previous = Some(Previous {
                    payload: payload.clone(),
                    best,
                    blob: solved.blob,
                });
            }
            Kind::Verify { scrambles } => {
                for index in 0..=scrambles {
                    let (source, mut this) = if index == 0 {
                        ("given".to_string(), state_bundle.clone())
                    } else {
                        (format!("scramble {index}"), scramble(&mut rng, state_bundle.clone(), 20))
                    };
                    let mut probe = new_probe(job.detail);
                    this.metric = this.metric_router(&mut probe);
                    let mut out = result(step, source, &tuning, &this, probe);
                    out.cpu_s = time_evaluation(&mut this);
                    out.mc = check(&mut this, job.mc.as_ref().unwrap(), &mut rng);
                    emit(&out);
                }
            }
        }
    }
}

pub fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    run_job(&serde_json::from_str(&input).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::dummy;

    fn short_solve(seed: u64) -> (f64, String) {
        let mut state_bundle = StateBundle::init_from_payload(dummy().steps[0].clone());
        state_bundle.tuning.max_iters = 300;
        let mut rng = StdRng::seed_from_u64(seed);
        let best = solve(&mut rng, state_bundle, None, &mut Probe::default()).best;
        (best.metric, serde_json::to_string(&Essence::of(&best)).unwrap())
    }

    /// comparing variants depends on a seed giving the same run every time
    #[test]
    fn a_seed_repeats_exactly() {
        assert_eq!(short_solve(7), short_solve(7));
        assert_ne!(short_solve(7).1, short_solve(8).1);
    }

    #[test]
    fn evaluator_matches_monte_carlo() {
        let spec = McSpec {
            samples: 100_000,
            z: 4.0,
            rel_tol: 1e-3,
        };
        for payload in dummy().steps {
            let state_bundle = StateBundle::init_from_payload(payload);
            let mut rng = StdRng::seed_from_u64(1);
            let mut scrambled = scramble(&mut rng, state_bundle, 20);
            for row in check(&mut scrambled, &spec, &mut rng) {
                assert!(row.pass, "{} {} {} {}", row.material, row.analytic, row.mean, row.se);
            }
        }
    }
}
