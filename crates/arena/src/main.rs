//! Test harness for hf-core. See `AI notes/Calculator core/Arena.md`.
//!
//! This binary is both the driver, which schedules jobs and writes the database, and the
//! worker, which runs one job. A snapshot is a copy of the binary, so a variant's worker
//! is the code as it was when the snapshot was taken.

mod adv_table;
mod bench;
mod carry;
mod db;
mod generate;
mod monte_carlo;
mod replan;
mod report;
mod scenario;
mod space;
mod stress;
mod worker;

use clap::{Parser, Subcommand, ValueEnum};
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use monte_carlo::McSpec;
use report::Selection;
use rusqlite::Connection;
use scenario::Scenario;
use serde_json::{Map, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use worker::{Essence, Job, Kind, StepResult};

const ROOT: &str = env!("HF_ROOT");
/// the binary that is running, as opposed to a snapshot
const CURRENT: &str = "current";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
    #[arg(long, global = true, default_value = "arena_data/arena.sqlite")]
    db: PathBuf,
    #[arg(long, global = true, default_value = "arena_data/scenarios")]
    scenarios: PathBuf,
    /// a named list of scenarios from `arena_data/sets`, in place of every scenario of the folder
    #[arg(long, global = true)]
    set: Option<String>,
}

const SETS: &str = "arena_data/sets";

fn scenarios(cli: &Cli, filter: &str) -> Vec<Scenario> {
    let mut out = match &cli.set {
        Some(set) => scenario::load_set(&cli.scenarios, &Path::new(ROOT).join(SETS), set),
        None => scenario::load(&cli.scenarios, ""),
    };
    out.retain(|scenario| scenario.name.contains(filter));
    out
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
        /// carry on the session this same command started, skipping the jobs it finished
        #[arg(long)]
        resume: bool,
    },
    /// write a generated set of scenarios into a folder of its own, under the scenarios folder
    Gen {
        set: String,
        #[arg(long, default_value_t = 0)]
        singles: u32,
        #[arg(long, default_value_t = 0)]
        chains: u32,
        /// payloads per chain
        #[arg(long, default_value_t = 8)]
        steps: usize,
        /// one axis at a time around a few anchors
        #[arg(long)]
        sweeps: bool,
        /// every upgrade a character has
        #[arg(long)]
        stress: bool,
        /// where in the sequence to start, for a set that shares nothing with another
        #[arg(long, default_value_t = 0)]
        offset: u32,
        #[arg(long)]
        chests: bool,
        /// normal honing only
        #[arg(long)]
        no_adv: bool,
        /// chains of a constant weekly income: this many with the player honing and as many without
        #[arg(long, default_value_t = 0)]
        weekly: u32,
    },
    /// check the evaluator against monte carlo, and time it
    Verify {
        #[command(flatten)]
        common: Common,
        /// random states per step, on top of the given one
        #[arg(long, default_value_t = 5)]
        scrambles: usize,
    },
    /// check the probabilities under the metric against an exact convolution, at many thresholds
    Stress {
        /// only scenarios whose name contains this
        #[arg(long, default_value = "")]
        filter: String,
        /// random states per step, on top of the given one, each further from it (at most 3)
        #[arg(long, default_value_t = 3)]
        scrambles: usize,
        /// `key=value,key=value` over the default tuning
        #[arg(long, default_value = "")]
        tuning: String,
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
    /// the sets there are
    Sets,
    /// write a set: the scenarios under the scenarios folder that pass every filter
    Define {
        name: String,
        /// what the set is for
        #[arg(long, default_value = "")]
        note: String,
        /// the scenario's name starts with this: single, chain, sweep, stress
        #[arg(long, default_value = "")]
        kind: String,
        /// `key=low..high`; a key is an axis, `n` (upgrades of the first step) or `steps`
        #[arg(long = "where")]
        wheres: Vec<String>,
        /// only the first steps of each
        #[arg(long)]
        steps: Option<usize>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// how good each chain step's best known plan still is on the step after it
    Carry {
        /// one row per step, before the summary
        #[arg(long)]
        rows: bool,
        #[arg(long, default_value_t = 1e-4)]
        eps: f64,
    },
    /// what running the optimizer again after every upgrade is worth, on each scenario's first step
    Replan {
        #[command(flatten)]
        args: replan::Args,
    },
    /// score the runs recorded before there was `Tuning::reference` with it, and fill in `gross` and `flow`
    Rescore,
    /// time the evaluator alone
    Bench {
        #[command(flatten)]
        args: bench::Args,
    },
    /// write small cases, every plan of which can be tried, into the `tiny` folder
    Tiny {
        #[arg(long, default_value_t = 60)]
        count: u32,
        /// lone upgrades with `lone_taps` taps left
        #[arg(long, default_value_t = 30)]
        lone: u32,
        #[arg(long, default_value_t = 20)]
        lone_taps: usize,
        /// states a case may have
        #[arg(long, default_value_t = 3e6)]
        cap: f64,
    },
    /// every plan of each small case against the plans the optimizer's streaks can make
    Exhaust {
        #[command(flatten)]
        args: space::ExhaustArgs,
    },
    /// what each upgrade taking its best plan in turn reaches, from each step's best known state
    Ascend {
        #[command(flatten)]
        args: space::AscendArgs,
    },
    /// the metric over two streak lengths of an upgrade, the rest as best known
    Landscape {
        #[command(flatten)]
        args: space::LandscapeArgs,
    },
    /// write the placeholder scenario
    Dummy,
    #[command(hide = true)]
    Worker,
}

fn git(args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(ROOT)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// the rust side of the working tree against HEAD, new files included
fn working_tree_diff() -> String {
    const PATHS: [&str; 3] = ["crates", "Cargo.toml", "Cargo.lock"];
    let mut diff = git(&[&["diff", "HEAD", "--"][..], &PATHS[..]].concat());
    for file in git(&[
        &["ls-files", "--others", "--exclude-standard", "--"][..],
        &PATHS[..],
    ]
    .concat())
    .lines()
    {
        diff += &git(&["diff", "--no-index", "--", "/dev/null", file]);
    }
    diff
}

fn snapshot_exe(label: &str) -> PathBuf {
    Path::new(ROOT)
        .join("target/arena/bin")
        .join(format!("{label}.exe"))
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

/// What the reference evaluator (`Tuning::reference`) makes of a worker's state: the metric,
/// the gold spent, and the market value of what it uses.
fn score(reference: &StateBundle, essence: &Essence) -> (f64, f64, f64) {
    let mut state_bundle = reference.clone();
    state_bundle.tuning = Tuning::reference();
    essence.apply(&mut state_bundle);
    let metric = state_bundle.metric_router(&mut Probe::default());
    (
        metric,
        carry::gross(&state_bundle),
        carry::flow(&state_bundle),
    )
}

fn run_jobs(
    cli: &Cli,
    common: &Common,
    kind: Kind,
    seeds: u64,
    modes: &[bool],
    mc_first_seed: bool,
    resume: bool,
) -> i64 {
    let conn = db::open(&cli.db);
    let scenarios: Vec<Scenario> = scenarios(cli, &common.filter);
    assert!(!scenarios.is_empty(), "no scenarios in {:?}", cli.scenarios);
    let specs = if common.variants.is_empty() {
        vec![CURRENT.to_string()]
    } else {
        common.variants.clone()
    };
    let variants: Vec<Variant> = specs
        .iter()
        .map(|spec| parse_variant(&conn, spec))
        .collect();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = args.join(" ");
    let session_id = resume
        .then(|| db::session_with_args(&conn, &args))
        .flatten()
        .unwrap_or_else(|| db::add_session(&conn, git(&["rev-parse", "HEAD"]).trim(), &args));

    let step_ids: Vec<Vec<i64>> = scenarios
        .iter()
        .map(|scenario| {
            scenario
                .steps
                .iter()
                .enumerate()
                .map(|(step, payload)| {
                    db::step_id(&conn, &scenario.name, step, &carry::step_key(payload))
                })
                .collect()
        })
        .collect();
    let references: Vec<Vec<StateBundle>> = scenarios
        .iter()
        .map(|scenario| {
            scenario
                .steps
                .iter()
                .map(|payload| StateBundle::init_from_payload(payload.clone()))
                .collect()
        })
        .collect();

    // Variants innermost, so that with one job at a time they take turns. Seeds outermost, so
    // that a session stopped early has covered every scenario about equally.
    let finished = db::finished_jobs(&conn, session_id);
    let mut jobs: Vec<(usize, usize, Job)> = Vec::new();
    for seed in 0..seeds {
        for (scenario_index, scenario) in scenarios.iter().enumerate() {
            for &chained in modes {
                for (variant_index, variant) in variants.iter().enumerate() {
                    if finished.contains(&(
                        variant.spec.clone(),
                        step_ids[scenario_index][0],
                        seed,
                        chained,
                    )) {
                        continue;
                    }
                    let wants_mc =
                        matches!(kind, Kind::Verify { .. }) || (mc_first_seed && seed == 0);
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

    let threads = common
        .jobs
        .unwrap_or_else(|| (std::thread::available_parallelism().unwrap().get() / 2).max(1));
    println!(
        "session {session_id}: {} jobs on {threads} threads",
        jobs.len()
    );
    let conn = Mutex::new(conn);
    let done = AtomicUsize::new(0);
    let kind_name = match kind {
        Kind::Optimize => "optimize",
        Kind::Verify { .. } => "verify",
    };

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    // ctrl-c reaches the workers too, and what they were doing must not be recorded as dead
    static STOPPED: AtomicBool = AtomicBool::new(false);
    ctrlc::set_handler(|| STOPPED.store(true, Ordering::Relaxed)).unwrap();
    // in order, which a parallel iterator would not keep
    let next = AtomicUsize::new(0);
    pool.broadcast(|_| {
        while let Some((scenario_index, variant_index, job)) =
            jobs.get(next.fetch_add(1, Ordering::Relaxed))
        {
            if STOPPED.load(Ordering::Relaxed) {
                return;
            }
            let variant = &variants[*variant_index];
            let mut child = Command::new(&variant.exe)
                .arg("worker")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap_or_else(|_| panic!("could not start {:?}", variant.exe));
            let input = serde_json::to_string(job).unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
            let output = child.wait_with_output().unwrap();
            if !output.status.success() {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            if STOPPED.load(Ordering::Relaxed) {
                return;
            }

            let results: Vec<StepResult> = String::from_utf8_lossy(&output.stdout)
                .lines()
                // a worker that panics may have printed something else
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            let scored: Vec<(f64, f64, f64)> = results
                .iter()
                .map(|out| score(&references[*scenario_index][out.step], &out.essence))
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
        }
    });
    if STOPPED.load(Ordering::Relaxed) {
        println!("stopped; with --resume the same command carries on from here");
        std::process::exit(0);
    }
    session_id
}

/// `key=value,key=value` over the default tuning
fn overrides(text: &str) -> Map<String, Value> {
    text.split(',')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap();
            (
                key.to_string(),
                serde_json::from_str(value).unwrap_or(Value::String(value.to_string())),
            )
        })
        .collect()
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
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&scenario::dummy()).unwrap(),
            )
            .unwrap();
            println!("wrote {path:?}");
        }
        Cmd::Snap { label } => {
            assert!(label != CURRENT && !label.contains(':'));
            let exe = snapshot_exe(label);
            std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
            std::fs::copy(std::env::current_exe().unwrap(), &exe).unwrap();
            let conn = db::open(&cli.db);
            db::add_snapshot(
                &conn,
                label,
                git(&["rev-parse", "HEAD"]).trim(),
                &working_tree_diff(),
            );
            println!("snapshot {label} at {exe:?}");
        }
        Cmd::Gen {
            set,
            singles,
            chains,
            steps,
            sweeps,
            stress,
            offset,
            chests,
            no_adv,
            weekly,
        } => {
            let dir = cli.scenarios.join(set);
            std::fs::create_dir_all(&dir).unwrap();
            let spec = generate::Spec {
                singles: *singles,
                chains: *chains,
                steps: *steps,
                sweeps: *sweeps,
                stress: *stress,
                offset: *offset,
                chests: *chests,
                no_adv: *no_adv,
                weekly: *weekly,
            };
            let scenarios = generate::generate(&spec);
            for scenario in &scenarios {
                let path = dir.join(format!("{}.json", scenario.name));
                std::fs::write(path, serde_json::to_string(scenario).unwrap()).unwrap();
            }
            println!("wrote {} scenarios to {dir:?}", scenarios.len());
        }
        Cmd::Run {
            common,
            seeds,
            mode,
            mc,
            eps,
            resume,
        } => {
            let modes: &[bool] = match mode {
                Mode::Cold => &[false],
                Mode::Chained => &[true],
                Mode::Both => &[false, true],
            };
            let session = run_jobs(&cli, common, Kind::Optimize, *seeds, modes, *mc, *resume);
            let conn = db::open(&cli.db);
            let selection = Selection {
                session: Some(session),
                variants: Vec::new(),
            };
            report::optimize(&conn, &selection, *eps);
            report::errors(&conn, &selection);
        }
        Cmd::Verify { common, scrambles } => {
            let kind = Kind::Verify {
                scrambles: *scrambles,
            };
            let session = run_jobs(&cli, common, kind, 1, &[false], false, false);
            let conn = db::open(&cli.db);
            let selection = Selection {
                session: Some(session),
                variants: Vec::new(),
            };
            report::verify(&conn, &selection);
            report::errors(&conn, &selection);
        }
        Cmd::Stress {
            filter,
            scrambles,
            tuning,
        } => {
            let overrides = tuning
                .split(',')
                .filter(|pair| !pair.is_empty())
                .map(|pair| {
                    let (key, value) = pair.split_once('=').unwrap();
                    (key.to_string(), serde_json::from_str(value).unwrap())
                })
                .collect();
            let scenarios = scenarios(&cli, filter);
            assert!(!scenarios.is_empty(), "no scenarios in {:?}", cli.scenarios);
            stress::report(&stress::run(
                &scenarios,
                *scrambles,
                &worker::tuning_with(&overrides),
            ));
        }
        Cmd::Compare {
            variants,
            session,
            eps,
        } => {
            let conn = db::open(&cli.db);
            let latest: Option<i64> = conn
                .query_row("SELECT MAX(id) FROM session", [], |row| row.get(0))
                .unwrap();
            let selection = Selection {
                session: session.or(if variants.is_empty() { latest } else { None }),
                variants: variants.clone(),
            };
            report::optimize(&conn, &selection, *eps);
            report::verify(&conn, &selection);
            report::errors(&conn, &selection);
        }
        Cmd::Trace { run_id, key } => report::trace(&db::open(&cli.db), *run_id, key.clone()),
        Cmd::Sets => {
            let dir = Path::new(ROOT).join(SETS);
            let mut names: Vec<String> = std::fs::read_dir(&dir)
                .unwrap()
                .map(|entry| {
                    entry
                        .unwrap()
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                })
                .collect();
            names.sort();
            let known = scenario::all(&cli.scenarios);
            for name in names {
                let text = std::fs::read_to_string(dir.join(format!("{name}.txt"))).unwrap();
                let note = text
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('#')
                    .trim();
                let entries: Vec<&str> = text
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .collect();
                let payloads: usize = entries
                    .iter()
                    .map(|line| {
                        let (scenario, steps) = line.split_once(':').unwrap_or((line, ""));
                        let all = known
                            .get(scenario)
                            .map_or(0, |scenario| scenario.steps.len());
                        steps.parse().map_or(all, |steps: usize| steps.min(all))
                    })
                    .sum();
                println!(
                    "{name:16} {:4} scenarios {payloads:5} payloads  {note}",
                    entries.len()
                );
            }
        }
        Cmd::Define {
            name,
            note,
            kind,
            wheres,
            steps,
            limit,
        } => {
            let ranges: Vec<(&str, f64, f64)> = wheres
                .iter()
                .map(|range| {
                    let (key, range) = range.split_once('=').unwrap();
                    let (low, high) = range.split_once("..").unwrap();
                    (key, low.parse().unwrap(), high.parse().unwrap())
                })
                .collect();
            let args: Vec<String> = std::env::args().skip(1).collect();
            let mut lines = vec![format!("# {note}"), format!("# {}", args.join(" "))];
            let chosen = scenario::all(&cli.scenarios)
                .into_iter()
                .filter(|(name, scenario)| {
                    name.starts_with(kind.as_str())
                        && ranges.iter().all(|&(key, low, high)| {
                            let value = match key {
                                "n" => scenario.steps[0].upgrade_info.len() as f64,
                                "steps" => scenario.steps.len() as f64,
                                _ => scenario
                                    .meta
                                    .get("axes")
                                    .and_then(|axes| axes[key].as_f64())
                                    .unwrap_or(f64::NAN),
                            };
                            (low..=high).contains(&value)
                        })
                });
            for (name, _) in chosen.take(limit.unwrap_or(usize::MAX)) {
                lines.push(steps.map_or(name.clone(), |steps| format!("{name}:{steps}")));
            }
            let path = Path::new(ROOT).join(SETS).join(format!("{name}.txt"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, lines.join("\n") + "\n").unwrap();
            println!("{} scenarios in {path:?}", lines.len() - 2);
        }
        Cmd::Carry { rows, eps } => {
            carry::report(&db::open(&cli.db), &scenarios(&cli, ""), *rows, *eps)
        }
        Cmd::Replan { args } => {
            let conn = db::open(&cli.db);
            let session = args.session.unwrap_or_else(|| {
                let line: Vec<String> = std::env::args().skip(1).collect();
                let line = line.join(" ");
                let old = args
                    .resume
                    .then(|| db::session_with_args(&conn, &line))
                    .flatten();
                let session = old.unwrap_or_else(|| {
                    db::add_session(&conn, git(&["rev-parse", "HEAD"]).trim(), &line)
                });
                replan::run(
                    db::open(&cli.db),
                    session,
                    &scenarios(&cli, &args.filter),
                    args,
                );
                session
            });
            replan::report(&conn, session);
        }
        Cmd::Bench { args } => {
            let overrides = args
                .tuning
                .split(',')
                .filter(|pair| !pair.is_empty())
                .map(|pair| {
                    let (key, value) = pair.split_once('=').unwrap();
                    (
                        key.to_string(),
                        serde_json::from_str(value).unwrap_or(Value::String(value.to_string())),
                    )
                })
                .collect();
            let scenarios = scenarios(&cli, &args.filter);
            bench::run(&scenarios, args, &worker::tuning_with(&overrides));
        }
        Cmd::Tiny {
            count,
            lone,
            lone_taps,
            cap,
        } => {
            let dir = cli.scenarios.join("tiny");
            std::fs::create_dir_all(&dir).unwrap();
            let scenarios = space::tiny(*count, *lone, *lone_taps, *cap);
            for scenario in &scenarios {
                std::fs::write(
                    dir.join(format!("{}.json", scenario.name)),
                    serde_json::to_string(scenario).unwrap(),
                )
                .unwrap();
            }
            println!("wrote {} scenarios to {dir:?}", scenarios.len());
        }
        Cmd::Exhaust { args } => space::exhaust(&scenarios(&cli, ""), args),
        Cmd::Ascend { args } => {
            let tuning = worker::tuning_with(&overrides(&args.tuning));
            space::ascend_all(&db::open(&cli.db), &scenarios(&cli, ""), args, &tuning);
        }
        Cmd::Landscape { args } => {
            let tuning = worker::tuning_with(&overrides(&args.tuning));
            space::landscape(&db::open(&cli.db), &scenarios(&cli, ""), args, &tuning);
        }
        Cmd::Rescore => {
            let conn = db::open(&cli.db);
            let todo: Vec<(i64, i64, String)> = conn
                .prepare("SELECT id, step_id, state FROM run WHERE ref_reference IS NULL AND state IS NOT NULL ORDER BY step_id")
                .unwrap()
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .map(|row| row.unwrap())
                .collect();
            println!("{} runs", todo.len());
            // a payload from before a change to its format cannot be read, and its runs are skipped
            let mut reference: (i64, Option<StateBundle>) = (-1, None);
            let transaction = conn.unchecked_transaction().unwrap();
            for (id, step_id, state) in todo {
                if reference.0 != step_id {
                    let payload: String = conn
                        .query_row(
                            "SELECT payload FROM scenario_step WHERE id = ?1",
                            [step_id],
                            |row| row.get(0),
                        )
                        .unwrap();
                    let payload: Option<hf_core::payload::Payload> =
                        serde_json::from_str(&payload).ok();
                    reference = (step_id, payload.map(StateBundle::init_from_payload));
                }
                let Some(state_bundle) = &reference.1 else {
                    continue;
                };
                let essence: Essence = serde_json::from_str(&state).unwrap();
                let (metric_ref, gross, flow) = score(state_bundle, &essence);
                transaction
                    .execute(
                        "UPDATE run SET metric_ref = ?1, gross = ?2, flow = ?3, ref_reference = 1 WHERE id = ?4",
                        rusqlite::params![metric_ref, gross, flow, id],
                    )
                    .unwrap();
            }
            transaction.commit().unwrap();
        }
    }
}
