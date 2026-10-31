//! Free tap chances and the gold of every outcome, per state, for working out any cut offline.

use crate::scenario::Scenario;
use crate::worker::Essence;
use hf_core::optimizer::scramble;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use rand::prelude::*;
use rayon::prelude::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub states: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, default_value_t = 16)]
    pub jobs: usize,
    /// order moves of the best state
    #[arg(long, default_value_t = 12)]
    pub orders: usize,
    /// single random moves of the best state
    #[arg(long, default_value_t = 6)]
    pub moves: usize,
    #[arg(long, default_value = "")]
    pub filter: String,
    #[arg(long, default_value_t = 1e-12)]
    pub floor: f64,
}

fn golds(state: &mut StateBundle, outcomes: &[usize], tuning: Tuning) -> Vec<f64> {
    state.tuning = tuning;
    state.outcome_golds(outcomes, &mut Probe::default())
}

fn row(name: &str, label: &str, state: &StateBundle, floor: f64) -> serde_json::Value {
    let mut state = state.clone();
    state.tuning = Tuning::default();
    state.update_prob_dist();
    state.update_cost_dist();
    state.compute_special_probs(false);
    let probs = state.special_probs().clone();
    let outcomes: Vec<usize> = (0..probs.len()).filter(|&k| probs[k] >= floor).collect();
    let def = golds(&mut state, &outcomes, Tuning::default());
    let reference = golds(&mut state, &outcomes, Tuning::reference());
    // the free tap chances alone, from nothing
    let mut timed = state.clone();
    let start = Instant::now();
    let mut reps = 0;
    while reps < 3 || start.elapsed().as_secs_f64() < 0.002 {
        timed.special_cache.clear();
        timed.special_done = None;
        timed.compute_special_probs(false);
        reps += 1;
    }
    let dp_s = start.elapsed().as_secs_f64() / reps as f64;
    let mut plain = state.clone();
    plain.tuning = Tuning::default();
    json!({
        "name": name, "label": label,
        "n": state.upgrade_arr.len(),
        "normal": state.upgrade_arr.iter().filter(|u| u.is_normal_honing).count(),
        "budget": state.prep_output.special_budget,
        "costs": state.special_state.iter().map(|&i| state.upgrade_arr[i].special_cost).collect::<Vec<_>>(),
        "chances": state.special_state.iter().map(|&i| state.upgrade_arr[i].base_chance).collect::<Vec<_>>(),
        "flow": crate::carry::flow(&plain),
        "p": probs, "k": outcomes, "g_def": def, "g_ref": reference, "dp_s": dp_s,
    })
}

pub fn run(scenarios: &[Scenario], args: &Args) {
    let given: BTreeMap<String, Vec<Essence>> = args
        .states
        .as_ref()
        .map(|path| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
        .unwrap_or_default();
    let steps: Vec<(String, &hf_core::payload::Payload)> = scenarios
        .iter()
        .flat_map(|scenario| {
            scenario
                .steps
                .iter()
                .enumerate()
                .map(|(step, payload)| (format!("{}:{step}", scenario.name), payload))
        })
        .collect();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build()
        .unwrap();
    let file = std::sync::Mutex::new(std::fs::File::create(&args.out).unwrap());
    pool.install(|| {
        steps.par_iter().for_each(|(name, payload)| {
            let mut filled = (*payload).clone();
            crate::adv_table::fill(&mut filled);
            let base = StateBundle::init_from_payload(filled);
            if base.upgrade_arr.is_empty() {
                return;
            }
            let mut rng = StdRng::seed_from_u64(name.bytes().fold(7u64, |seed, byte| {
                seed.wrapping_mul(131).wrapping_add(byte as u64)
            }));
            let mut states: Vec<(String, StateBundle)> = vec![("given".to_string(), base.clone())];
            for moves in [20, 200] {
                states.push((format!("scr{moves}"), scramble(&mut rng, base.clone(), moves)));
            }
            let mut anchor = states[1].1.clone();
            if let Some(best) = given.get(name).and_then(|list| list.first()) {
                let mut this = base.clone();
                best.apply(&mut this);
                anchor = this.clone();
                states.push(("best".to_string(), this));
            } else {
                states.push(("anchor".to_string(), anchor.clone()));
            }
            if anchor.affords_a_free_tap() {
                anchor.update_prob_dist();
                anchor.update_cost_dist();
                anchor.compute_special_probs(false);
                let reach = anchor
                    .special_probs()
                    .iter()
                    .rposition(|&chance| chance >= 1e-7)
                    .unwrap_or(0);
                let len = anchor.special_state.len();
                for index in 0..args.orders {
                    let mut this = anchor.clone();
                    let elem = this.special_state.remove(rng.random_range(0..len));
                    this.special_state
                        .insert(rng.random_range(0..=reach.min(len - 1)), elem);
                    this.clean_special_state();
                    states.push((format!("ord{index}"), this));
                }
                for index in 0..args.moves {
                    states.push((format!("mov{index}"), scramble(&mut rng, anchor.clone(), 1)));
                }
            }
            let rows: Vec<String> = states
                .iter()
                .map(|(label, state)| row(name, label, state, args.floor).to_string())
                .collect();
            let mut file = file.lock().unwrap();
            for line in rows {
                writeln!(file, "{line}").unwrap();
            }
        });
    });
}
