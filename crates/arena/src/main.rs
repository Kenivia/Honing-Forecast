//! Test harness for hf-core. See `AI notes/Calculator core/Arena.md`.
//!
//! This binary is both the driver, which schedules jobs and writes the database, and the
//! worker, which runs one job. A snapshot is a copy of the binary, so a variant's worker
//! is the code as it was when the snapshot was taken.

mod db;
mod monte_carlo;
mod one_adv_sim;
mod report;
mod scenario;
mod worker;

use clap::{Parser, Subcommand, ValueEnum};
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use monte_carlo::McSpec;
use rayon::prelude::*;
use report::Selection;
use rusqlite::Connection;
use scenario::Scenario;
use serde_json::{Map, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use worker::{Job, Kind, StepResult};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
/// the binary that is running, as opposed to a snapshot
const CURRENT: &str = "current";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
    #[arg(long, global = true, default_value = "test_cases/arena.sqlite")]
    db: PathBuf,
    #[arg(long, global = true, default_value = "test_cases/scenarios")]
    scenarios: PathBuf,
}

#[derive(Clone, Copy, ValueEnum, PartialEq)]
enum Mode {
    Cold,
    Chained,
    Both,
}

#[derive(clap::Args)]
struct McArgs {
    #[arg(long, default_value_t = 500_000)]
    samples: usize,
    #[arg(long, default_value_t = 4.0)]
    z: f64,
    #[arg(long, default_value_t = 1e-3)]
    rel_tol: f64,
}

impl McArgs {
    fn spec(&self) -> McSpec {
        McSpec {
            samples: self.samples,
            z: self.z,
            rel_tol: self.rel_tol,
        }
    }
}

#[derive(clap::Args)]
struct Common {
    /// `label` or `label:key=value,key=value`; `current` is the working tree as built
    variants: Vec<String>,
    /// only scenarios whose name contains this
    #[arg(long, default_value = "")]
    filter: String,
    /// worker processes at once; use 1 when the cpu time matters
    #[arg(long)]
    jobs: Option<usize>,
    /// record per iteration rows
    #[arg(long)]
    detail: bool,
    #[command(flatten)]
    mc: McArgs,
}

#[derive(Subcommand)]
enum Cmd {
    /// keep the working tree, as built, under a label
    Snap { label: String },
    /// run the optimizer on every scenario
    Run {
        #[command(flatten)]
        common: Common,
        #[arg(long, default_value_t = 5)]
        seeds: u64,
        #[arg(long, value_enum, default_value = "both")]
        mode: Mode,
        /// monte carlo check the result of the first seed
        #[arg(long)]
        mc: bool,
        #[arg(long, default_value_t = 1e-4)]
        eps: f64,
    },
    /// check the evaluator against monte carlo, and time it
    Verify {
        #[command(flatten)]
        common: Common,
        /// random states per step, on top of the given one
        #[arg(long, default_value_t = 5)]
        scrambles: usize,
    },
    /// tables for a session (the latest by default) or for variants across sessions
    Compare {
        variants: Vec<String>,
        #[arg(long)]
        session: Option<i64>,
        #[arg(long, default_value_t = 1e-4)]
        eps: f64,
    },
    /// a run's trace rows as csv
    Trace { run_id: i64, key: Option<String> },
    /// write the placeholder scenario
    Dummy,
    #[command(hide = true)]
    Worker,
}

fn git(args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(ROOT).output().unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// the rust side of the working tree against HEAD, new files included
fn working_tree_diff() -> String {
    const PATHS: [&str; 3] = ["crates", "Cargo.toml", "Cargo.lock"];
    let mut diff = git(&[&["diff", "HEAD", "--"][..], &PATHS[..]].concat());
    for file in git(&[&["ls-files", "--others", "--exclude-standard", "--"][..], &PATHS[..]].concat()).lines() {
        diff += &git(&["diff", "--no-index", "--", "/dev/null", file]);
    }
    diff
}

fn snapshot_exe(label: &str) -> PathBuf {
    Path::new(ROOT).join("target/arena/bin").join(format!("{label}.exe"))
}

struct Variant {
    spec: String,
    overrides: Map<String, Value>,
    exe: PathBuf,
    snapshot_id: i64,
}

fn parse_variant(conn: &Connection, spec: &str) -> Variant {
    let (label, settings) = spec.split_once(':').unwrap_or((spec, ""));
    let overrides = settings
        .split(',')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap();
            let value = serde_json::from_str(value).unwrap_or(Value::String(value.to_string()));
            (key.to_string(), value)
        })
        .collect();
    worker::tuning_with(&overrides); // fail on a bad key now, not in every worker
    let (exe, snapshot_id) = if label == CURRENT {
        let sha = git(&["rev-parse", "HEAD"]);
        let id = db::current_snapshot(conn, sha.trim(), &working_tree_diff());
        (std::env::current_exe().unwrap(), id)
    } else {
        let id = db::latest_snapshot(conn, label).unwrap_or_else(|| panic!("no snapshot {label}"));
        (snapshot_exe(label), id)
    };
    Variant {
        spec: spec.to_string(),
        overrides,
        exe,
        snapshot_id,
    }
}

/// what the driver's own evaluator, at default tuning, makes of a worker's state
fn score(reference: &StateBundle, out: &StepResult) -> f64 {
    let mut state_bundle = reference.clone();
    out.essence.apply(&mut state_bundle);
    state_bundle.metric_router(&mut Probe::default())
}

fn run_jobs(cli: &Cli, common: &Common, kind: Kind, seeds: u64, modes: &[bool], mc_first_seed: bool) -> i64 {
    let conn = db::open(&cli.db);
    let scenarios: Vec<Scenario> = scenario::load(&cli.scenarios, &common.filter);
    assert!(!scenarios.is_empty(), "no scenarios in {:?}", cli.scenarios);
    let specs = if common.variants.is_empty() {
        vec![CURRENT.to_string()]
    } else {
        common.variants.clone()
    };
    let variants: Vec<Variant> = specs.iter().map(|spec| parse_variant(&conn, spec)).collect();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let session_id = db::add_session(&conn, git(&["rev-parse", "HEAD"]).trim(), &args.join(" "));

    let step_ids: Vec<Vec<i64>> = scenarios
        .iter()
        .map(|scenario| {
            scenario
                .steps
                .iter()
                .enumerate()
                .map(|(step, payload)| {
                    db::step_id(&conn, &scenario.name, step, &serde_json::to_string(payload).unwrap())
                })
                .collect()
        })
        .collect();
    let references: Vec<Vec<StateBundle>> = scenarios
        .iter()
        .map(|scenario| {
            scenario.steps.iter().map(|payload| StateBundle::init_from_payload(payload.clone())).collect()
        })
        .collect();

    // variants innermost, so that with one job at a time they take turns
    let mut jobs: Vec<(usize, usize, Job)> = Vec::new();
    for (scenario_index, scenario) in scenarios.iter().enumerate() {
        for seed in 0..seeds {
            for &chained in modes {
                for (variant_index, variant) in variants.iter().enumerate() {
                    let wants_mc = matches!(kind, Kind::Verify { .. }) || (mc_first_seed && seed == 0);
                    jobs.push((
                        scenario_index,
                        variant_index,
                        Job {
                            scenario: scenario.clone(),
                            kind: kind.clone(),
                            seed,
                            chained,
                            overrides: variant.overrides.clone(),
                            detail: common.detail,
                            mc: wants_mc.then(|| common.mc.spec()),
                        },
                    ));
                }
            }
        }
    }

    let threads = common.jobs.unwrap_or_else(|| {
        (std::thread::available_parallelism().unwrap().get() / 2).max(1)
    });
    println!("session {session_id}: {} jobs on {threads} threads", jobs.len());
    let conn = Mutex::new(conn);
    let done = AtomicUsize::new(0);
    let kind_name = match kind {
        Kind::Optimize => "optimize",
        Kind::Verify { .. } => "verify",
    };

    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
    pool.install(|| {
        jobs.par_iter().with_max_len(1).for_each(|(scenario_index, variant_index, job)| {
            let variant = &variants[*variant_index];
            let mut child = Command::new(&variant.exe)
                .arg("worker")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap_or_else(|_| panic!("could not start {:?}", variant.exe));
            let input = serde_json::to_string(job).unwrap();
            child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
            let output = child.wait_with_output().unwrap();

            let results: Vec<StepResult> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let scored: Vec<f64> = results
                .iter()
                .map(|out| score(&references[*scenario_index][out.step], out))
                .collect();

            let mut conn = conn.lock().unwrap();
            let transaction = conn.transaction().unwrap();
            let mut key = db::RunKey {
                session_id,
                snapshot_id: variant.snapshot_id,
                variant: &variant.spec,
                step_id: 0,
                kind: kind_name,
                seed: job.seed,
                chained: job.chained,
            };
            for (out, metric_ref) in results.iter().zip(scored) {
                key.step_id = step_ids[*scenario_index][out.step];
                db::add_run(&transaction, &key, out, metric_ref, common.mc.samples);
            }
            if !output.status.success() {
                let last_step = results.last().map_or(0, |out| out.step);
                key.step_id = step_ids[*scenario_index][last_step];
                let stderr = String::from_utf8_lossy(&output.stderr);
                let tail = stderr.len().saturating_sub(4000);
                db::add_error(&transaction, &key, stderr.get(tail..).unwrap_or(&stderr));
            }
            transaction.commit().unwrap();

            println!(
                "[{}/{}] {} {} seed {} {}{}",
                done.fetch_add(1, Ordering::Relaxed) + 1,
                jobs.len(),
                variant.spec,
                job.scenario.name,
                job.seed,
                if job.chained { "chained" } else { "cold" },
                if output.status.success() { "" } else { " DIED" },
            );
        })
    });
    session_id
}

fn main() {
    let mut cli = Cli::parse();
    // relative paths are from the repo root, wherever this is run from
    cli.db = Path::new(ROOT).join(&cli.db);
    cli.scenarios = Path::new(ROOT).join(&cli.scenarios);
    match &cli.command {
        Cmd::Worker => worker::main(),
        Cmd::Dummy => {
            std::fs::create_dir_all(&cli.scenarios).unwrap();
            let path = cli.scenarios.join("dummy.json");
            std::fs::write(&path, serde_json::to_string_pretty(&scenario::dummy()).unwrap()).unwrap();
            println!("wrote {path:?}");
        }
        Cmd::Snap { label } => {
            assert!(label != CURRENT && !label.contains(':'));
            let exe = snapshot_exe(label);
            std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
            std::fs::copy(std::env::current_exe().unwrap(), &exe).unwrap();
            let conn = db::open(&cli.db);
            db::add_snapshot(&conn, label, git(&["rev-parse", "HEAD"]).trim(), &working_tree_diff());
            println!("snapshot {label} at {exe:?}");
        }
        Cmd::Run { common, seeds, mode, mc, eps } => {
            let modes: &[bool] = match mode {
                Mode::Cold => &[false],
                Mode::Chained => &[true],
                Mode::Both => &[false, true],
            };
            let session = run_jobs(&cli, common, Kind::Optimize, *seeds, modes, *mc);
            let conn = db::open(&cli.db);
            let selection = Selection { session: Some(session), variants: Vec::new() };
            report::optimize(&conn, &selection, *eps);
            report::errors(&conn, &selection);
        }
        Cmd::Verify { common, scrambles } => {
            let kind = Kind::Verify { scrambles: *scrambles };
            let session = run_jobs(&cli, common, kind, 1, &[false], false);
            let conn = db::open(&cli.db);
            let selection = Selection { session: Some(session), variants: Vec::new() };
            report::verify(&conn, &selection);
            report::errors(&conn, &selection);
        }
        Cmd::Compare { variants, session, eps } => {
            let conn = db::open(&cli.db);
            let latest: Option<i64> =
                conn.query_row("SELECT MAX(id) FROM session", [], |row| row.get(0)).unwrap();
            let selection = Selection {
                session: session.or(if variants.is_empty() { latest } else { None }),
                variants: variants.clone(),
            };
            report::optimize(&conn, &selection, *eps);
            report::verify(&conn, &selection);
            report::errors(&conn, &selection);
        }
        Cmd::Trace { run_id, key } => report::trace(&db::open(&cli.db), *run_id, key.clone()),
    }
}
