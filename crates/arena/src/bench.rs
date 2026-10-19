//! Times the evaluator alone, on the given state, scrambles of it and states handed in.

use crate::scenario::Scenario;
use crate::worker::Essence;
use hf_core::optimizer::scramble;
use hf_core::probe::{COUNTER_NAMES, Probe};
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[derive(clap::Args)]
pub struct Args {
    /// `key=value,key=value` over the default tuning
    #[arg(long, default_value = "")]
    pub tuning: String,
    /// json of "scenario:step" to a list of essences
    #[arg(long)]
    pub states: Option<PathBuf>,
    /// one line per state: name, metric, seconds per evaluation, flow
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    pub jobs: usize,
    /// seconds each state is repeated for
    #[arg(long, default_value_t = 0.05)]
    pub seconds: f64,
    #[arg(long, default_value = "")]
    pub filter: String,
    /// time whole solves instead, with this seed
    #[arg(long)]
    pub solve: Option<u64>,
}

pub fn run(scenarios: &[Scenario], args: &Args, tuning: &Tuning) {
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
    let rows: Vec<(String, f64, f64, Vec<i64>, f64)> = pool.install(|| {
        steps
            .par_iter()
            .flat_map(|(name, payload)| {
                let mut filled = (*payload).clone();
                crate::adv_table::fill(&mut filled);
                let mut base = StateBundle::init_from_payload(filled);
                base.tuning = tuning.clone();
                if base.upgrade_arr.is_empty() {
                    return Vec::new();
                }
                let mut rng = StdRng::seed_from_u64(name.bytes().fold(7u64, |seed, byte| {
                    seed.wrapping_mul(131).wrapping_add(byte as u64)
                }));
                if let Some(seed) = args.solve {
                    let mut probe = Probe::default();
                    let mut rng = StdRng::seed_from_u64(seed);
                    let start = Instant::now();
                    let solved = hf_core::optimizer::solve(&mut rng, base, None, &mut probe);
                    return vec![(
                        name.clone(),
                        solved.best.metric,
                        start.elapsed().as_secs_f64(),
                        probe.data.counts.to_vec(),
                        f64::NAN,
                    )];
                }
                let mut states: Vec<(String, StateBundle)> =
                    vec![(format!("{name} given"), base.clone())];
                for moves in [20, 200] {
                    states.push((
                        format!("{name} scramble{moves}"),
                        scramble(&mut rng, base.clone(), moves),
                    ));
                }
                for (index, essence) in given.get(name).into_iter().flatten().enumerate() {
                    let mut this = base.clone();
                    essence.apply(&mut this);
                    states.push((format!("{name} best{index}"), this));
                }
                states
                    .into_iter()
                    .map(|(label, mut state)| {
                        let mut probe = Probe::default();
                        let metric = state.metric_router(&mut probe);
                        let start = Instant::now();
                        let mut reps = 0;
                        while start.elapsed().as_secs_f64() < args.seconds {
                            state.metric_router(&mut Probe::default());
                            reps += 1;
                        }
                        // what it uses does not depend on the tuning, so not at an expensive one
                        let mut plain = state.clone();
                        plain.tuning = Tuning::default();
                        let flow = crate::carry::flow(&plain);
                        (
                            label,
                            metric,
                            start.elapsed().as_secs_f64() / reps as f64,
                            probe.data.counts.to_vec(),
                            flow,
                        )
                    })
                    .collect()
            })
            .collect()
    });
    let mut counts = vec![0i64; COUNTER_NAMES.len()];
    let mut total = 0.0;
    for row in &rows {
        total += row.2;
        for (sum, count) in counts.iter_mut().zip(&row.3) {
            *sum += count;
        }
    }
    if let Some(path) = &args.out {
        let mut file = std::fs::File::create(path).unwrap();
        for (label, metric, seconds, _, flow) in &rows {
            writeln!(file, "{label}\t{metric:e}\t{seconds:e}\t{flow:e}").unwrap();
        }
    }
    println!(
        "{} states, {:.3} ms for one evaluation of each, {:.1} us mean",
        rows.len(),
        total * 1e3,
        total / rows.len() as f64 * 1e6
    );
    for (name, count) in COUNTER_NAMES.iter().zip(&counts) {
        println!(
            "{name:16} {count:10} {:8.2} per state",
            *count as f64 / rows.len() as f64
        );
    }
}
