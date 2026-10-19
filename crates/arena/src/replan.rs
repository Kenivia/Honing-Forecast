//! What running the optimizer again after every upgrade is worth.
//!
//! A path plays a scenario to the end. After each upgrade the plan is carried over and the
//! optimizer starts from it, so the new plan is never worse, and how much better it is is
//! the gain of that re-plan. The gains of a path add up to what following the re-plans is
//! worth over following the first plan, with no noise from the gold itself.
//! See `AI notes/Calculator core/Arena.md`.

use crate::carry::{carried, flow};
use crate::db::{self, ReplanRow};
use crate::generate::{next_up, play};
use crate::scenario::Scenario;
use crate::worker::Essence;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use hf_core::optimizer::solve;
use hf_core::payload::Payload;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use rand::prelude::*;
use rayon::prelude::*;
use rusqlite::Connection;
use std::collections::{BTreeMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

#[derive(clap::Args)]
pub struct Args {
    /// paths per scenario
    #[arg(long, default_value_t = 30)]
    pub paths: i64,
    /// re-plan after every this many upgrades
    #[arg(long, default_value_t = 1)]
    pub every: usize,
    /// optimizer iterations of a re-plan
    #[arg(long, default_value_t = 24000)]
    pub iters: i64,
    /// solves the starting plan is the best of
    #[arg(long, default_value_t = 3)]
    pub seeds: u64,
    #[arg(long)]
    pub jobs: Option<usize>,
    /// only scenarios whose name contains this
    #[arg(long, default_value = "")]
    pub filter: String,
    #[arg(long)]
    pub resume: bool,
    /// print a session's tables and run nothing
    #[arg(long)]
    pub session: Option<i64>,
}

fn evaluated(mut state_bundle: StateBundle) -> StateBundle {
    state_bundle.metric = state_bundle.metric_router(&mut Probe::default());
    state_bundle
}

/// the metric with every probability summed exactly, which is slow
fn exact(state_bundle: &StateBundle) -> f64 {
    let mut state_bundle = state_bundle.clone();
    state_bundle.tuning.max_brute_size = usize::MAX;
    state_bundle.metric_router(&mut Probe::default())
}

/// paths and sizes the exact metric is taken on, as a check that a gain is not the optimizer
/// finding the approximation's errors
const EXACT_PATHS: i64 = 3;
const EXACT_UPGRADES: usize = 8;

/// the best of some solves that start from this state, which is never worse than it
fn polish(start: &StateBundle, iters: i64, seeds: u64, seed: u64) -> StateBundle {
    (0..seeds)
        .map(|index| {
            let mut state_bundle = start.clone();
            state_bundle.tuning.max_iters = iters;
            let mut rng = StdRng::seed_from_u64(seed.wrapping_add(index));
            evaluated(solve(&mut rng, state_bundle, None, &mut Probe::default()).best)
        })
        .chain([start.clone()])
        .max_by(|a, b| a.metric.total_cmp(&b.metric))
        .unwrap()
}

/// The plan every path starts from, and how it was reached: cold solves, a polish, and one
/// more solve of the size of a re-plan. That last one learns nothing new, so what it gains is
/// what a re-plan gains from the optimizer's slack alone.
fn start(payload: &Payload, args: &Args, seed: u64) -> (StateBundle, Vec<ReplanRow>) {
    let default = evaluated(StateBundle::init_from_payload(payload.clone()));
    let cold = polish(&default, args.iters, args.seeds, seed);
    let polished = polish(&cold, args.iters, args.seeds, seed + 100);
    let control = polish(&polished, args.iters, 1, seed + 200);
    let flow = flow(&control);
    let rows = [
        ("cold", &default, &cold),
        ("polish", &cold, &polished),
        ("control", &polished, &control),
    ]
    .into_iter()
    .enumerate()
    .map(|(step, (kind, before, after))| ReplanRow {
        path: -1,
        step,
        upgrades_left: payload.upgrade_info.len(),
        kind: kind.to_string(),
        carried: before.metric,
        resolved: after.metric,
        flow: Some(flow),
        exact: None,
    })
    .collect();
    (control, rows)
}

fn one_path(
    payload: &Payload,
    plan: &StateBundle,
    path: i64,
    args: &Args,
    seed: u64,
) -> Vec<ReplanRow> {
    let mut rng = StdRng::seed_from_u64(seed);
    let (mut payload, mut plan) = (payload.clone(), plan.clone());
    let mut rows: Vec<ReplanRow> = Vec::new();
    for step in 1.. {
        let before = payload.clone();
        let kind = play(&mut payload, &plan, next_up(&plan), false, &mut rng);
        let upgrades_left = payload.upgrade_info.len();
        if upgrades_left == 0 {
            rows.push(ReplanRow {
                path,
                step,
                upgrades_left,
                kind: "end".to_string(),
                carried: 0.0,
                resolved: 0.0,
                flow: None,
                exact: None,
            });
            break;
        }
        payload.adv_cache = Some(plan.adv_cache.clone());
        let kept = evaluated(carried(&before, &Essence::of(&plan), &payload));
        plan = if step % args.every == 0 {
            let new = polish(&kept, args.iters, 1, rng.next_u64());
            rows.push(ReplanRow {
                path,
                step,
                upgrades_left,
                kind: kind.to_string(),
                carried: kept.metric,
                resolved: new.metric,
                flow: None,
                exact: (path < EXACT_PATHS && upgrades_left <= EXACT_UPGRADES)
                    .then(|| (exact(&kept), exact(&new))),
            });
            new
        } else {
            kept
        };
    }
    rows
}

pub fn run(conn: Connection, session_id: i64, scenarios: &[Scenario], args: &Args) {
    let finished: HashSet<(String, i64)> = db::replan_rows(&conn, session_id)
        .into_iter()
        .map(|(scenario, row)| (scenario, row.path))
        .collect();
    let conn = Mutex::new(conn);
    let threads = args
        .jobs
        .unwrap_or_else(|| (std::thread::available_parallelism().unwrap().get() / 2).max(1));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    println!(
        "session {session_id}: {} scenarios, {} paths each, on {threads} threads",
        scenarios.len(),
        args.paths
    );

    pool.install(|| {
        scenarios
            .par_iter()
            .enumerate()
            .for_each(|(index, scenario)| {
                let todo: Vec<i64> = (0..args.paths)
                    .filter(|path| !finished.contains(&(scenario.name.clone(), *path)))
                    .collect();
                if todo.is_empty() {
                    return;
                }
                let payload = &scenario.steps[0];
                let seed = index as u64 * 1_000_003;
                // a seed repeats exactly, so a resumed session gets the plan it had
                let Ok((plan, rows)) =
                    catch_unwind(AssertUnwindSafe(|| start(payload, args, seed)))
                else {
                    println!("{} DIED on its starting plan", scenario.name);
                    return;
                };
                if !finished.contains(&(scenario.name.clone(), -1)) {
                    db::add_replan(&mut conn.lock().unwrap(), session_id, &scenario.name, &rows);
                }
                println!("{} starts", scenario.name);
                todo.par_iter().for_each(|&path| {
                    let seed = seed + 1000 + path as u64;
                    match catch_unwind(AssertUnwindSafe(|| {
                        one_path(payload, &plan, path, args, seed)
                    })) {
                        Ok(rows) => db::add_replan(
                            &mut conn.lock().unwrap(),
                            session_id,
                            &scenario.name,
                            &rows,
                        ),
                        Err(_) => println!("{} path {path} DIED", scenario.name),
                    }
                });
                println!("{} done", scenario.name);
            });
    });
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn median(values: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    sorted.get(sorted.len() / 2).copied().unwrap_or(f64::NAN)
}

/// Per scenario, the gain of a path next to the control, both as a share of the market value
/// of what the plan uses; then where along a path the gain is made.
pub fn report(conn: &Connection, session_id: i64) {
    let mut by_scenario: BTreeMap<String, Vec<ReplanRow>> = BTreeMap::new();
    for (scenario, row) in db::replan_rows(conn, session_id) {
        by_scenario.entry(scenario).or_default().push(row);
    }

    let mut each = Table::new();
    each.load_style(UTF8_BORDERS_ONLY).set_header(vec![
        "scenario",
        "n",
        "plan",
        "flow",
        "optimizer %",
        "paths",
        "gain",
        "se",
        "gain %",
        "control %",
        "of optimizer %",
        "paths gaining %",
        "exact / approx",
    ]);
    let (mut gains, mut controls, mut of_optimizer): (Vec<f64>, Vec<f64>, Vec<f64>) =
        Default::default();
    // per upgrades left: the gain of each re-plan, as a share of flow
    let mut by_left: BTreeMap<usize, Vec<f64>> = BTreeMap::new();
    let mut totals = (0.0, 0.0);
    for (scenario, rows) in &by_scenario {
        let base: Vec<&ReplanRow> = rows.iter().filter(|row| row.path == -1).collect();
        if base.len() < 3 {
            continue;
        }
        let flow = base[0].flow.unwrap().max(1.0);
        let (default, plan) = (base[0].carried, base[2].resolved);
        let control = (base[2].resolved - base[2].carried) / flow;
        let mut per_path: BTreeMap<i64, f64> = BTreeMap::new();
        // the gain of the re-plans that have an exact metric, by it and by the approximation
        let mut checked = (0.0, 0.0);
        for row in rows.iter().filter(|row| row.path >= 0) {
            *per_path.entry(row.path).or_default() += row.resolved - row.carried;
            if let Some((carried, resolved)) = row.exact {
                checked.0 += resolved - carried;
                checked.1 += row.resolved - row.carried;
            }
            if row.kind != "end" {
                by_left
                    .entry(row.upgrades_left)
                    .or_default()
                    .push((row.resolved - row.carried) / flow);
            }
        }
        let per_path: Vec<f64> = per_path.into_values().collect();
        if per_path.is_empty() {
            continue;
        }
        let gain = mean(&per_path);
        let se = (per_path.iter().map(|x| (x - gain).powi(2)).sum::<f64>()
            / (per_path.len() as f64 * (per_path.len() as f64 - 1.0).max(1.0)))
        .sqrt();
        let share = gain / (plan - default).max(1.0);
        each.add_row(vec![
            scenario.clone(),
            base[0].upgrades_left.to_string(),
            format!("{plan:.0}"),
            format!("{flow:.0}"),
            format!("{:.2}", 100.0 * (plan - default) / flow),
            per_path.len().to_string(),
            format!("{gain:.0}"),
            format!("{se:.0}"),
            format!("{:.4}", 100.0 * gain / flow),
            format!("{:.4}", 100.0 * control),
            format!("{:.2}", 100.0 * share),
            format!(
                "{:.0}",
                100.0 * per_path.iter().filter(|x| **x > 1e-6 * flow).count() as f64
                    / per_path.len() as f64
            ),
            format!("{:.2}", checked.0 / checked.1),
        ]);
        totals.0 += checked.0 / flow;
        totals.1 += checked.1 / flow;
        gains.push(100.0 * gain / flow);
        controls.push(100.0 * control);
        of_optimizer.push(100.0 * share);
    }
    println!("{each}");

    let mut all = Table::new();
    all.load_style(UTF8_BORDERS_ONLY)
        .set_header(vec!["over scenarios", "mean", "median", "max"]);
    for (name, values) in [
        ("gain %", &gains),
        ("control %", &controls),
        ("of optimizer %", &of_optimizer),
    ] {
        all.add_row(vec![
            name.to_string(),
            format!("{:.4}", mean(values)),
            format!("{:.4}", median(values)),
            format!("{:.4}", values.iter().copied().fold(f64::NAN, f64::max)),
        ]);
    }
    println!("{all}");
    println!(
        "exact gain / approximate gain, each as a share of flow, over the re-plans checked: {:.3}",
        totals.0 / totals.1
    );

    let mut left = Table::new();
    left.load_style(UTF8_BORDERS_ONLY).set_header(vec![
        "upgrades left",
        "re-plans",
        "mean gain %",
        "gaining %",
    ]);
    for (upgrades_left, values) in &by_left {
        left.add_row(vec![
            upgrades_left.to_string(),
            values.len().to_string(),
            format!("{:.4}", 100.0 * mean(values)),
            format!(
                "{:.0}",
                100.0 * values.iter().filter(|x| **x > 1e-6).count() as f64 / values.len() as f64
            ),
        ]);
    }
    println!("{left}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::{normal_used, pay};
    use crate::monte_carlo::gold;
    use crate::scenario::dummy;

    /// what the plan is worth, summed over every way its next upgrade can go
    fn over_outcomes(payload: &Payload, plan: &StateBundle) -> f64 {
        let index = next_up(plan);
        let this = &plan.upgrade_arr[index];
        let bands = &plan.prep_output.bands[plan.prep_output.optimizer_plan];
        let after = |special_left: i64, taps: usize| -> f64 {
            let mut next = payload.clone();
            next.special_budget = special_left;
            let used = normal_used(plan, index, taps);
            pay(&mut next, plan, &used);
            next.upgrade_info.remove(index);
            let kept = evaluated(carried(payload, &Essence::of(plan), &next));
            let new_bands = &kept.prep_output.bands[kept.prep_output.optimizer_plan];
            let sunk: f64 = (0..used.len())
                .map(|row| gold(used[row], &bands[row]) - gold(0.0, &new_bands[row]))
                .sum();
            kept.metric + sunk
        };

        if index != plan.special_state[0] {
            return this
                .normal_dist
                .iter()
                .enumerate()
                .map(|(taps, p)| p * after(payload.special_budget, taps))
                .sum();
        }
        let mut total = 0.0;
        let mut none_landed = 1.0;
        for tries in 1..=payload.special_budget / this.special_cost {
            total += none_landed
                * this.base_chance
                * after(payload.special_budget - tries * this.special_cost, 0);
            none_landed *= 1.0 - this.base_chance;
        }
        // the evaluator ends the free taps at the first upgrade they run out on
        total
            + none_landed
                * this
                    .normal_dist
                    .iter()
                    .enumerate()
                    .map(|(taps, p)| p * after(0, taps))
                    .sum::<f64>()
    }

    /// Carrying a plan over an upgrade loses nothing: what the carried plan is worth plus the
    /// gold already gone is, on average, what the plan was worth.
    #[test]
    fn carrying_a_plan_keeps_its_value() {
        let mut payload = dummy().steps[0].clone();
        payload.chests.clear();
        payload.upgrade_info.retain(|info| info.is_normal_honing);
        // the second has one level a piece, so its next upgrade is the one free taps start on
        let mut flat = payload.clone();
        flat.upgrade_info.retain(|info| info.upgrade_index == 14);
        for (payload, free_taps_first) in [(payload, false), (flat, true)] {
            let mut start = StateBundle::init_from_payload(payload.clone());
            start.tuning.max_iters = 2000;
            let plan = evaluated(
                solve(
                    &mut StdRng::seed_from_u64(3),
                    start,
                    None,
                    &mut Probe::default(),
                )
                .best,
            );
            assert_eq!(next_up(&plan) == plan.special_state[0], free_taps_first);
            let total = over_outcomes(&payload, &plan);
            assert!(
                (total - plan.metric).abs() < 1e-3 * plan.metric.abs(),
                "{total} {}",
                plan.metric
            );
        }
    }
}
