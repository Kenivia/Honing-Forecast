//! The state space of normal honing: every plan of a small case against the plans the
//! optimizer's streaks can make, the best plan of one upgrade with the rest as it is, and
//! the metric over a grid of one upgrade's streaks.
//! See `AI notes/Calculator core/State space.md`.

use crate::carry::{flow, step_key};
use crate::db;
use crate::generate::{self, axis};
use crate::scenario::{Scenario, upgrade};
use crate::worker::Essence;
use ahash::AHashSet;
use hf_core::optimizer::response::{Model, Response};
use hf_core::optimizer::solve;
use hf_core::parser::OneUpgradeInput;
use hf_core::payload::Payload;
use hf_core::probe::Probe;
use hf_core::state::{OneState, State};
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

type Plan = Vec<OneState>;

/// What the arena asks of a `Model` and the optimizer does not.
trait Explore {
    fn walk(
        &self,
        count: usize,
        artisan: f64,
        plan: &mut Vec<usize>,
        out: &mut Option<Vec<Vec<usize>>>,
        total: &mut f64,
        cap: f64,
    );
    fn plans(&self) -> Vec<Vec<usize>>;
    fn count(&self, cap: f64) -> f64;
    fn after(&self, taps: usize) -> Model;
    fn streaks_of(&self, pool: usize) -> Vec<Vec<usize>>;
    fn streaks(&self) -> AHashSet<Vec<usize>>;
    fn pattern(&self, indices: &[usize]) -> String;
}

impl Explore for Model {
    fn walk(
        &self,
        count: usize,
        artisan: f64,
        plan: &mut Vec<usize>,
        out: &mut Option<Vec<Vec<usize>>>,
        total: &mut f64,
        cap: f64,
    ) {
        if artisan >= 1.0 || count + 1 >= self.len || *total > cap {
            *total += 1.0;
            if let Some(out) = out {
                out.push(plan.clone());
            }
            return;
        }
        // inside a block a tap takes what the tap before it took
        let options = if count % self.block == 0 {
            0..self.options.len()
        } else {
            plan[count - 1]..plan[count - 1] + 1
        };
        for option in options {
            let chance = self.chance(count, self.extra[option]);
            plan[count] = option;
            if chance == 1.0 {
                *total += 1.0;
                if let Some(out) = out {
                    out.push(plan.clone());
                }
            } else {
                self.walk(count + 1, self.step(artisan, chance), plan, out, total, cap);
            }
        }
        plan[count] = 0;
    }

    /// every distinct plan
    fn plans(&self) -> Vec<Vec<usize>> {
        let mut out = Some(Vec::new());
        self.walk(
            0,
            self.artisan,
            &mut vec![0; self.len],
            &mut out,
            &mut 0.0,
            f64::INFINITY,
        );
        out.unwrap()
    }

    /// how many distinct plans, as far as the cap
    fn count(&self, cap: f64) -> f64 {
        let mut total = 0.0;
        self.walk(
            0,
            self.artisan,
            &mut vec![0; self.len],
            &mut None,
            &mut total,
            cap,
        );
        total
    }

    /// this upgrade some failed taps on, with nothing used on them
    fn after(&self, taps: usize) -> Model {
        let mut out = self.clone();
        for count in 0..taps {
            out.artisan = self.step(out.artisan, self.chance(count, 0.0));
        }
        out.artisan = out.artisan.min(1.0);
        out.taps = (self.taps + taps).min(10);
        out.len = self.len - taps;
        out
    }

    /// What the optimizer's streaks can make of one pool: an id from the first tap on and
    /// an id up to the last. Each is what every tap adds to its option's index.
    fn streaks_of(&self, pool: usize) -> Vec<Vec<usize>> {
        let ids = self.pools[pool].len();
        let ends = |len: usize| -> Vec<usize> {
            if len == 0 || ids == 0 {
                vec![0]
            } else {
                (1..=ids).collect()
            }
        };
        (0..=self.len)
            .flat_map(|front| (0..=self.len - front).map(move |back| (front, back)))
            // a streak starts and ends where a block does, but for the end of the state
            .filter(|(front, back)| {
                front % self.block == 0 && (*back == 0 || (self.len - back) % self.block == 0)
            })
            .flat_map(|(front, back)| {
                ends(front)
                    .into_iter()
                    .cartesian_product(ends(back))
                    .map(move |(first, last)| {
                        (0..self.len)
                            .map(|tap| {
                                self.strides[pool]
                                    * if tap < front {
                                        first
                                    } else if tap >= self.len - back {
                                        last
                                    } else {
                                        0
                                    }
                            })
                            .collect::<Vec<usize>>()
                    })
            })
            .unique()
            .collect()
    }

    /// every plan the streaks can make, with the taps never charged emptied
    fn streaks(&self) -> AHashSet<Vec<usize>> {
        let per_pool: Vec<Vec<Vec<usize>>> = (0..self.pools.len())
            .map(|pool| self.streaks_of(pool))
            .collect();
        per_pool
            .iter()
            .multi_cartesian_product()
            .map(|choice| {
                self.canonical(
                    &(0..self.len)
                        .map(|tap| choice.iter().map(|row| row[tap]).sum())
                        .collect::<Vec<usize>>(),
                )
            })
            .collect()
    }

    /// per pool, which of its ids each tap up to the pity takes
    fn pattern(&self, indices: &[usize]) -> String {
        let pity = self.pity(indices).0;
        self.pools
            .iter()
            .zip(&self.strides)
            .filter(|(ids, _)| !ids.is_empty())
            .map(|(ids, stride)| {
                (0..pity)
                    .map(|tap| match indices[tap] / stride % (ids.len() + 1) {
                        0 => '.',
                        which => char::from_digit(which as u32 - 1, 10).unwrap(),
                    })
                    .collect::<String>()
            })
            .join("/")
    }
}

/// an upgrade's response to the state as it is, which is left as it was
fn response_of(state_bundle: &StateBundle, index: usize) -> Response {
    Response::of(&mut state_bundle.clone(), index, &mut Probe::default())
}

/// the best of the plans the streaks can make
fn best_streak(response: &Response) -> (f64, Vec<usize>) {
    let model = &response.model;
    let per_pool: Vec<Vec<Vec<usize>>> = (0..model.pools.len())
        .map(|pool| model.streaks_of(pool))
        .collect();
    per_pool[0]
        .par_iter()
        .map(|first| {
            let mut best = (f64::NEG_INFINITY, Vec::new());
            let mut indices = vec![0; model.len];
            for rest in per_pool[1..].iter().multi_cartesian_product() {
                for (tap, option) in indices.iter_mut().enumerate() {
                    *option = first[tap] + rest.iter().map(|row| row[tap]).sum::<usize>();
                }
                let score = response.score(&indices);
                if score > best.0 {
                    best = (score, indices.clone());
                }
            }
            best
        })
        .reduce(
            || (f64::NEG_INFINITY, Vec::new()),
            |a, b| if b.0 > a.0 { b } else { a },
        )
}

/// the tuning that sums every probability
fn exact_tuning() -> Tuning {
    Tuning {
        max_brute_size: usize::MAX,
        response_points: usize::MAX,
        ..Default::default()
    }
}

fn models(state_bundle: &StateBundle) -> Vec<Model> {
    state_bundle
        .upgrade_arr
        .iter()
        .map(|this| {
            Model::of(
                this,
                &state_bundle.prep_output.juice_info,
                state_bundle.block(),
            )
        })
        .collect()
}

/// the state with these plans and this free tap order, evaluated under a tuning
fn with_plans(
    state_bundle: &StateBundle,
    plans: &[Vec<usize>],
    order: &[usize],
    tuning: &Tuning,
) -> StateBundle {
    let mut state_bundle = state_bundle.clone();
    state_bundle.tuning = tuning.clone();
    state_bundle.gold_memo.clear();
    let models = models(&state_bundle);
    for ((this, model), plan) in state_bundle.upgrade_arr.iter_mut().zip(models).zip(plans) {
        this.state.update_payload(model.plan(plan));
    }
    state_bundle.special_state = order.to_vec();
    state_bundle.metric = state_bundle.metric_router(&mut Probe::default());
    state_bundle
}

/// a state under another tuning
fn retuned(state_bundle: &StateBundle, tuning: &Tuning) -> StateBundle {
    let mut state_bundle = state_bundle.clone();
    state_bundle.tuning = tuning.clone();
    state_bundle.gold_memo.clear();
    state_bundle.metric = state_bundle.metric_router(&mut Probe::default());
    state_bundle
}

/// (metric, each upgrade's plan, free tap order)
type Found = (f64, Vec<Vec<usize>>, Vec<usize>);

const KEPT: usize = 6;

/// the best few of every plan, and the best few the streaks can make
#[derive(Clone, Default)]
struct Tops {
    any: Vec<(f64, usize, Vec<usize>)>,
    streak: Vec<(f64, usize, Vec<usize>)>,
    states: f64,
}

fn keep(list: &mut Vec<(f64, usize, Vec<usize>)>, found: (f64, usize, Vec<usize>)) {
    if list.len() == KEPT && found.0 <= list[KEPT - 1].0 {
        return;
    }
    let at = list.partition_point(|x| x.0 >= found.0);
    list.insert(at, found);
    list.truncate(KEPT);
}

fn descend(
    state_bundle: &mut StateBundle,
    plans: &[Vec<Plan>],
    is_streak: &[Vec<bool>],
    depth: usize,
    index: &mut Vec<usize>,
    order: usize,
    tops: &mut Tops,
) {
    if depth == plans.len() {
        let metric = state_bundle.metric_router(&mut Probe::default());
        tops.states += 1.0;
        let worst = |list: &Vec<(f64, usize, Vec<usize>)>| {
            if list.len() == KEPT {
                list[KEPT - 1].0
            } else {
                f64::NEG_INFINITY
            }
        };
        if metric > worst(&tops.any) {
            keep(&mut tops.any, (metric, order, index.clone()));
        }
        if metric > worst(&tops.streak)
            && index
                .iter()
                .zip(is_streak)
                .all(|(&plan, flags)| flags[plan])
        {
            keep(&mut tops.streak, (metric, order, index.clone()));
        }
        return;
    }
    for plan in 0..plans[depth].len() {
        state_bundle.upgrade_arr[depth]
            .state
            .update_payload(plans[depth][plan].clone());
        index[depth] = plan;
        descend(
            state_bundle,
            plans,
            is_streak,
            depth + 1,
            index,
            order,
            tops,
        );
    }
}

#[derive(clap::Args)]
pub struct ExhaustArgs {
    /// only scenarios whose name contains this
    #[arg(long, default_value = "")]
    pub filter: String,
    /// states a case may have
    #[arg(long, default_value_t = 2e7)]
    pub cap: f64,
    /// solves of the optimizer to compare with
    #[arg(long, default_value_t = 4)]
    pub seeds: u64,
    /// taps a streak length is a multiple of
    #[arg(long, default_value_t = 1)]
    pub block: usize,
    /// one json line a case
    #[arg(long)]
    pub out: Option<PathBuf>,
}

fn solved(payload: &Payload, seeds: u64, tuning: &Tuning) -> Vec<StateBundle> {
    (0..seeds)
        .into_par_iter()
        .map(|seed| {
            let mut state_bundle = StateBundle::init_from_payload(payload.clone());
            state_bundle.tuning = tuning.clone();
            solve(
                &mut StdRng::seed_from_u64(seed),
                state_bundle,
                None,
                &mut Probe::default(),
            )
            .best
        })
        .collect()
}

fn exhaust_one(scenario: &Scenario, args: &ExhaustArgs) -> Option<Value> {
    let payload = &scenario.steps[0];
    let exact = Tuning {
        streak_length: args.block,
        ..exact_tuning()
    };
    let base = retuned(&StateBundle::init_from_payload(payload.clone()), &exact);
    let n = base.upgrade_arr.len();
    if n == 0
        || base.upgrade_arr.iter().any(|this| !this.is_normal_honing)
        || base
            .prep_output
            .chests
            .values()
            .any(|chest| chest.options.len() > 1)
    {
        return None;
    }
    let models = models(&base);
    let orders: Vec<Vec<usize>> = if base.prep_output.special_budget > 0 && n > 1 {
        (0..n).permutations(n).collect()
    } else {
        vec![(0..n).collect()]
    };
    let states: f64 = models
        .iter()
        .map(|model| model.count(args.cap))
        .product::<f64>()
        * orders.len() as f64;
    if n > 1 && states > args.cap {
        return None;
    }
    let at = |found: &(f64, Vec<usize>)| -> Found {
        (
            with_plans(&base, &[found.1.clone()], &orders[0], &exact).metric,
            vec![found.1.clone()],
            orders[0].clone(),
        )
    };

    let (any, streak, counted): (Vec<Found>, Vec<Found>, f64) = if n == 1 && states > args.cap {
        let response = response_of(&base, 0);
        (
            vec![at(&response.best())],
            vec![at(&best_streak(&response))],
            states,
        )
    } else {
        let plans: Vec<Vec<Vec<usize>>> = models.iter().map(|model| model.plans()).collect();
        let is_streak: Vec<Vec<bool>> = plans
            .iter()
            .zip(&models)
            .map(|(plans, model)| {
                let set = model.streaks();
                plans.iter().map(|plan| set.contains(plan)).collect()
            })
            .collect();
        let payloads: Vec<Vec<Plan>> = plans
            .iter()
            .zip(&models)
            .map(|(plans, model)| plans.iter().map(|plan| model.plan(plan)).collect())
            .collect();
        let firsts: Vec<(usize, usize)> = (0..orders.len())
            .cartesian_product(0..plans[0].len())
            .collect();
        let tops = firsts
            .par_iter()
            .map(|&(order, first)| {
                let mut state_bundle = base.clone();
                state_bundle.special_state = orders[order].clone();
                state_bundle.upgrade_arr[0]
                    .state
                    .update_payload(payloads[0][first].clone());
                let mut tops = Tops::default();
                let mut index = vec![first; n];
                descend(
                    &mut state_bundle,
                    &payloads,
                    &is_streak,
                    1,
                    &mut index,
                    order,
                    &mut tops,
                );
                tops
            })
            .reduce(Tops::default, |mut a, b| {
                for found in b.any {
                    keep(&mut a.any, found);
                }
                for found in b.streak {
                    keep(&mut a.streak, found);
                }
                a.states += b.states;
                a
            });
        let full = |list: &Vec<(f64, usize, Vec<usize>)>| -> Vec<Found> {
            list.iter()
                .map(|found| {
                    (
                        found.0,
                        found
                            .2
                            .iter()
                            .zip(&plans)
                            .map(|(&plan, plans)| plans[plan].clone())
                            .collect(),
                        orders[found.1].clone(),
                    )
                })
                .collect()
        };
        (full(&tops.any), full(&tops.streak), tops.states)
    };

    let (best, best_streak) = (&any[0], &streak[0]);
    let best_bundle = with_plans(&base, &best.1, &best.2, &exact);
    let approx = Tuning {
        streak_length: args.block,
        ..Tuning::default()
    };
    // what the optimizer finds, on the exact metric and on its own
    let scored = |tuning: &Tuning| -> Vec<f64> {
        solved(payload, args.seeds, tuning)
            .iter()
            .map(|found| retuned(found, &exact).metric)
            .collect()
    };
    // each upgrade in turn taking its best plan, from the best the streaks have and from nothing
    let climbed = |from: &StateBundle| ascend(from, false, 20).0.metric;
    let patterns = |found: &Found| -> String {
        found
            .1
            .iter()
            .zip(&models)
            .map(|(plan, model)| model.pattern(plan))
            .join("  ")
    };
    let listed = |list: &[Found]| -> Vec<Value> {
        list.iter()
            .map(|found| json!({"metric": found.0, "pattern": patterns(found), "order": found.2}))
            .collect()
    };
    Some(json!({
        "name": scenario.name,
        "n": n,
        "upgrades": base.upgrade_arr.iter().zip(&models).map(|(this, model)| json!({
            "name": this.name_string, "len": model.len - 1, "options": model.options.len(),
            "extra": model.extra, "base": this.base_chance, "artisan": this.starting_artisan,
        })).collect::<Vec<_>>(),
        "orders": orders.len(),
        "states": counted,
        "best": best.0,
        "best_streak": best_streak.0,
        "default": base.metric,
        "flow": flow(&best_bundle),
        "best_approx": retuned(&best_bundle, &approx).metric,
        "best_streak_approx": with_plans(&base, &best_streak.1, &best_streak.2, &approx).metric,
        "solved_exact": scored(&exact),
        "solved_approx": scored(&approx),
        "climbed_from_streak": climbed(&with_plans(&base, &best_streak.1, &best_streak.2, &exact)),
        "climbed_from_default": climbed(&base),
        "top": listed(&any),
        "top_streak": listed(&streak),
        "axes": scenario.meta.get("axes"),
    }))
}

pub fn exhaust(scenarios: &[Scenario], args: &ExhaustArgs) {
    let mut file = args
        .out
        .as_ref()
        .map(|path| std::fs::File::create(path).unwrap());
    println!(
        "{:28} {:>2} {:>10} {:>14} {:>10} {:>10} {:>10} {:>10}  best",
        "scenario", "n", "states", "best", "streak %", "solve %", "climb %", "default %"
    );
    for scenario in scenarios
        .iter()
        .filter(|scenario| scenario.name.contains(&args.filter))
    {
        let Some(row) = exhaust_one(scenario, args) else {
            println!("{:28} left out", scenario.name);
            continue;
        };
        let number = |key: &str| row[key].as_f64().unwrap();
        let share = |value: f64| 100.0 * (number("best") - value) / number("flow").max(1.0);
        let solve_best = row["solved_exact"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .fold(f64::NEG_INFINITY, f64::max);
        println!(
            "{:28} {:>2} {:>10.0} {:>14.1} {:>10.5} {:>10.5} {:>10.5} {:>10.3}  {}",
            scenario.name,
            number("n"),
            number("states"),
            number("best"),
            share(number("best_streak")),
            share(solve_best),
            share(number("climbed_from_default")),
            share(number("default")),
            row["top"][0]["pattern"].as_str().unwrap(),
        );
        if let Some(file) = file.as_mut() {
            writeln!(file, "{row}").unwrap();
            file.flush().unwrap();
        }
    }
}

/// Each normal upgrade in turn takes its best plan with the rest as it is, of every plan or
/// of the streaks only, until a pass moves nothing. The free tap order and the chests stay.
/// Returns the state and how many responses were worked out.
pub fn ascend(start: &StateBundle, streaks_only: bool, sweeps: usize) -> (StateBundle, usize) {
    let mut state_bundle = retuned(start, &start.tuning);
    let mut responses = 0;
    for _ in 0..sweeps {
        let mut moved = false;
        for index in 0..state_bundle.upgrade_arr.len() {
            if !state_bundle.upgrade_arr[index].is_normal_honing {
                continue;
            }
            let response = response_of(&state_bundle, index);
            responses += 1;
            let model = &response.model;
            let now = model.indices(&state_bundle.upgrade_arr[index].state.payload);
            let (score, plan) = if streaks_only {
                best_streak(&response)
            } else {
                response.best()
            };
            if score <= response.score(&now) + 1e-9 * score.abs().max(1.0) {
                continue;
            }
            // the response is exact where the evaluator is; where it is not, the metric decides
            let before = state_bundle.upgrade_arr[index].state.payload.clone();
            state_bundle.upgrade_arr[index]
                .state
                .update_payload(model.plan(&plan));
            let metric = state_bundle.metric_router(&mut Probe::default());
            if metric > state_bundle.metric {
                state_bundle.metric = metric;
                moved = true;
            } else {
                state_bundle.upgrade_arr[index].state.update_payload(before);
            }
        }
        if !moved {
            break;
        }
    }
    state_bundle.metric = state_bundle.metric_router(&mut Probe::default());
    (state_bundle, responses)
}

#[derive(clap::Args)]
pub struct AscendArgs {
    /// only scenarios whose name contains this
    #[arg(long, default_value = "")]
    pub filter: String,
    /// passes over the upgrades at most
    #[arg(long, default_value_t = 10)]
    pub sweeps: usize,
    /// the tuning the responses and the metric are taken under, `key=value,key=value`
    #[arg(
        long,
        default_value = "max_brute_size=2000000,max_lattice_work=20000000,response_points=100000"
    )]
    pub tuning: String,
    /// one json line a step
    #[arg(long)]
    pub out: Option<PathBuf>,
}

/// From each step's best known state, and from nothing: what taking best responses reaches,
/// over every plan and over the streaks only.
pub fn ascend_all(conn: &Connection, scenarios: &[Scenario], args: &AscendArgs, tuning: &Tuning) {
    let mut file = args
        .out
        .as_ref()
        .map(|path| std::fs::File::create(path).unwrap());
    println!(
        "{:32} {:>3} {:>12} {:>10} {:>10} {:>10} {:>10} {:>6}",
        "step", "n", "flow", "streak %", "any %", "cold s %", "cold a %", "s"
    );
    for scenario in scenarios
        .iter()
        .filter(|scenario| scenario.name.contains(&args.filter))
    {
        for step in 0..scenario.steps.len() {
            let known = retuned(&best_known(conn, scenario, step), tuning);
            if !known.upgrade_arr.iter().any(|this| this.is_normal_honing) {
                continue;
            }
            // nothing on any normal upgrade, the rest as known
            let mut cold = known.clone();
            for this in cold
                .upgrade_arr
                .iter_mut()
                .filter(|this| this.is_normal_honing)
            {
                this.state
                    .update_payload(vec![OneState::new(); this.state.len()]);
            }
            let cold = retuned(&cold, tuning);
            let clock = Instant::now();
            let runs: Vec<(StateBundle, usize)> = [
                (&known, true),
                (&known, false),
                (&cold, true),
                (&cold, false),
            ]
            .into_par_iter()
            .map(|(from, streaks_only)| ascend(from, streaks_only, args.sweeps))
            .collect();
            let seconds = clock.elapsed().as_secs_f64();
            let models = models(&known);
            let flow = flow(&known).max(1.0);
            let default = Tuning::default();
            let moved: Vec<Value> = (0..known.upgrade_arr.len())
                .filter(|&index| known.upgrade_arr[index].is_normal_honing)
                .filter_map(|index| {
                    let pattern = |state_bundle: &StateBundle| -> String {
                        models[index].pattern(&models[index].canonical(&models[index].indices(&state_bundle.upgrade_arr[index].state.payload)))
                    };
                    let (was, streak, any) = (pattern(&known), pattern(&runs[0].0), pattern(&runs[1].0));
                    (was != streak || was != any)
                        .then(|| json!({"name": known.upgrade_arr[index].name_string, "known": was, "streak": streak, "any": any}))
                })
                .collect();
            let row = json!({
                "name": scenario.name,
                "step": step,
                "n": known.upgrade_arr.len(),
                "flow": flow,
                "known": known.metric,
                "known_default": retuned(&known, &default).metric,
                "cold": cold.metric,
                "runs": runs.iter().map(|(found, responses)| json!({
                    "metric": found.metric, "default": retuned(found, &default).metric, "responses": responses,
                })).collect::<Vec<_>>(),
                "seconds": seconds,
                "moved": moved,
                "axes": scenario.meta.get("axes"),
            });
            let gain = |run: usize| 100.0 * (runs[run].0.metric - known.metric) / flow;
            println!(
                "{:32} {:>3} {:>12.0} {:>10.5} {:>10.5} {:>10.5} {:>10.5} {:>6.1}",
                format!("{}:{step}", scenario.name),
                known.upgrade_arr.len(),
                flow,
                gain(0),
                gain(1),
                gain(2),
                gain(3),
                seconds,
            );
            if let Some(file) = file.as_mut() {
                writeln!(file, "{row}").unwrap();
                file.flush().unwrap();
            }
        }
    }
}

/// Small cases for `exhaust`: one to three normal upgrades, each far enough in that every
/// plan of the case can be tried, and lone upgrades with `lone_taps` taps left.
pub fn tiny(count: u32, lone: u32, lone_taps: usize, cap: f64) -> Vec<Scenario> {
    let made = |index: u32, lone: bool| -> Scenario {
        let mut rng = StdRng::seed_from_u64(index as u64 + if lone { 50_000 } else { 0 });
        let mut a = generate::sobol(index, if lone { 4 } else { 3 });
        for (name, value) in [("chests", 0.0), ("progress", 0.0), ("adv", 0.0)] {
            a[axis(name)] = value;
        }
        if lone || index % 2 == 0 {
            a[axis("free_taps")] = 0.0;
        }
        let n = if lone {
            1
        } else {
            *[1, 2, 2, 3, 3].choose(&mut rng).unwrap()
        };
        let mut infos: Vec<OneUpgradeInput> = Vec::new();
        while infos.len() < n {
            let piece = if rng.random_bool(0.3) {
                5
            } else {
                rng.random_range(0..5)
            };
            let level = rng.random_range(if lone { 10 } else { 5 }..25);
            if !infos
                .iter()
                .any(|info| info.piece_index == piece && info.upgrade_index == level)
            {
                infos.push(upgrade(piece, level, false));
            }
        }
        infos.sort_by_key(|info| (info.piece_index, info.upgrade_index));

        // fail taps, on the upgrade with the most plans left, until every plan can be tried
        let fresh = StateBundle::init_from_payload(
            generate::build_with(&a, index as u64, false, Some(infos.clone())).payload,
        );
        let models = models(&fresh);
        let mut failed: Vec<usize> = vec![0; n];
        if lone {
            failed[0] = (models[0].len - 1).saturating_sub(lone_taps);
        } else {
            let orders: f64 = if a[axis("free_taps")] >= 0.25 {
                (1..=n).product::<usize>() as f64
            } else {
                1.0
            };
            loop {
                let counts: Vec<f64> = models
                    .iter()
                    .zip(&failed)
                    .map(|(model, &taps)| model.after(taps).count(cap))
                    .collect();
                if counts.iter().product::<f64>() * orders <= cap {
                    break;
                }
                let most = (0..n)
                    .max_by(|&x, &y| counts[x].total_cmp(&counts[y]))
                    .unwrap();
                failed[most] += 1;
            }
        }
        for ((info, model), &taps) in infos.iter_mut().zip(&models).zip(&failed) {
            if taps > 0 {
                let after = model.after(taps);
                info.starting_artisan = Some(after.artisan);
                info.starting_num_taps = Some(after.taps);
                info.unlocked = true;
            }
        }
        let payload = generate::build_with(&a, index as u64, false, Some(infos)).payload;
        Scenario {
            name: format!(
                "{}_{index:03}_{}",
                if lone { "lone" } else { "tiny" },
                generate::tag(&payload)
            ),
            steps: vec![payload],
            meta: generate::meta(&a, &[]),
        }
    };
    let todo: Vec<(u32, bool)> = (0..count)
        .map(|index| (index, false))
        .chain((0..lone).map(|index| (index, true)))
        .collect();
    todo.par_iter()
        .filter_map(|&(index, lone)| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| made(index, lone))).ok()
        })
        .collect()
}

#[derive(clap::Args)]
pub struct LandscapeArgs {
    /// only scenarios whose name contains this
    #[arg(long, default_value = "")]
    pub filter: String,
    #[arg(long, default_value_t = 0)]
    pub step: usize,
    /// one upgrade, where not every normal one
    #[arg(long)]
    pub upgrade: Option<usize>,
    /// a pool and an end of it: `0f` is the streak of pool 0 from the first tap, `1b` of pool 1 from the last
    #[arg(long, default_value = "0f")]
    pub x: String,
    #[arg(long, default_value = "1f")]
    pub y: String,
    /// the tuning of the second metric, `key=value,key=value`
    #[arg(
        long,
        default_value = "max_brute_size=2000000,max_lattice_work=20000000,response_points=100000"
    )]
    pub tuning: String,
    #[arg(long)]
    pub out: PathBuf,
}

/// the best known state of a step, or the best of a few solves where it has no runs
pub fn best_known(conn: &Connection, scenario: &Scenario, step: usize) -> StateBundle {
    let payload = &scenario.steps[step];
    let mut filled = payload.clone();
    crate::adv_table::fill(&mut filled);
    let mut state_bundle = StateBundle::init_from_payload(filled);
    match db::best_run(
        conn,
        db::step_id(conn, &scenario.name, step, &step_key(payload)),
    ) {
        Some((_, _, essence)) => essence.apply(&mut state_bundle),
        None => {
            let found = solved(payload, 4, &Tuning::default());
            let best = found
                .iter()
                .max_by(|a, b| a.metric.total_cmp(&b.metric))
                .unwrap();
            Essence::of(best).apply(&mut state_bundle);
        }
    }
    state_bundle
}

/// The metric over two streak lengths of one upgrade, its other streaks and the rest of the
/// state as best known. `second` is the same under another tuning, and `response` what the
/// upgrade's response says, which is exact where the second tuning is.
pub fn landscape(conn: &Connection, scenarios: &[Scenario], args: &LandscapeArgs, second: &Tuning) {
    let axis_of =
        |text: &str| -> (usize, bool) { (text[..1].parse().unwrap(), text.ends_with('f')) };
    let (x_axis, y_axis) = (axis_of(&args.x), axis_of(&args.y));
    let mut file = std::fs::File::create(&args.out).unwrap();
    writeln!(
        file,
        "scenario\tupgrade\tname\tlen\tx\ty\tpity\tmetric\tsecond\tresponse"
    )
    .unwrap();
    for scenario in scenarios
        .iter()
        .filter(|scenario| scenario.name.contains(&args.filter))
    {
        let base = retuned(&best_known(conn, scenario, args.step), &Tuning::default());
        let other = retuned(&base, second);
        let juice_info = base.prep_output.juice_info.clone();
        let chosen: Vec<usize> = (0..base.upgrade_arr.len())
            .filter(|&index| {
                base.upgrade_arr[index].is_normal_honing
                    && args.upgrade.is_none_or(|only| only == index)
            })
            .collect();
        for index in chosen {
            let this = &base.upgrade_arr[index];
            let model = Model::of(this, &juice_info, base.block());
            let (piece_type, upgrade_index) = (this.piece_type_usize, this.upgrade_index);
            if [x_axis, y_axis]
                .iter()
                .any(|axis| model.pools.get(axis.0).is_none_or(|ids| ids.is_empty()))
            {
                continue;
            }
            let response = response_of(&other, index);
            let start =
                this.state
                    .clone()
                    .compute_streak_info(piece_type, upgrade_index, &juice_info);
            let cells: Vec<(usize, usize)> =
                (0..model.len).cartesian_product(0..model.len).collect();
            let rows: Vec<(usize, usize, usize, [f64; 3])> = cells
                .par_chunks(64)
                .flat_map_iter(|chunk| {
                    let mut bundles = [base.clone(), other.clone()];
                    let mut out = Vec::new();
                    for &(x, y) in chunk {
                        let mut info = start.clone();
                        for ((pool, front), len) in [(x_axis, x), (y_axis, y)] {
                            let entry = if front {
                                &mut info.front[pool]
                            } else {
                                &mut info.back[pool]
                            };
                            *entry = (
                                (len > 0).then(|| entry.0.unwrap_or(model.pools[pool][0])),
                                len,
                            );
                        }
                        if (0..model.pools.len())
                            .any(|pool| info.front[pool].1 + info.back[pool].1 > model.len)
                        {
                            continue;
                        }
                        let mut state = State::new_empty(model.len);
                        state.apply_new_streak_info(&info, piece_type, upgrade_index, &juice_info);
                        let indices = model.indices(&state.payload);
                        let mut metrics = [0.0, 0.0, response.score(&indices)];
                        for (metric, bundle) in metrics.iter_mut().zip(bundles.iter_mut()) {
                            bundle.upgrade_arr[index]
                                .state
                                .update_payload(state.payload.clone());
                            *metric = bundle.metric_router(&mut Probe::default());
                        }
                        out.push((x, y, model.pity(&indices).0, metrics));
                    }
                    out
                })
                .collect();
            for (x, y, pity, metrics) in rows {
                writeln!(
                    file,
                    "{}\t{index}\t{}\t{}\t{x}\t{y}\t{pity}\t{:e}\t{:e}\t{:e}",
                    scenario.name,
                    this.name_string,
                    model.len - 1,
                    metrics[0],
                    metrics[1],
                    metrics[2]
                )
                .unwrap();
            }
            println!("{} {} {}", scenario.name, index, this.name_string);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monte_carlo::{exact_single, single_outcomes};

    /// the evaluator against an exact sum, on the two lone upgrades whose best plan is not a streak plan
    #[test]
    #[ignore]
    fn lone_exact() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../arena_data/scenarios/tiny");
        for name in ["lone_022_t4_n1", "lone_029_t4_n1"] {
            let scenario = &crate::scenario::load(&dir, name)[0];
            let payload = &scenario.steps[0];
            let base = retuned(
                &StateBundle::init_from_payload(payload.clone()),
                &exact_tuning(),
            );
            let model = &models(&base)[0];
            let response = response_of(&base, 0);
            let mut rows = vec![
                ("response", response.best().1),
                ("streak", best_streak(&response).1),
            ];
            for (seed, found) in solved(payload, 4, &Tuning::default()).iter().enumerate() {
                rows.push((
                    ["solve 0", "solve 1", "solve 2", "solve 3"][seed],
                    model.indices(&found.upgrade_arr[0].state.payload),
                ));
            }
            println!("{name}");
            for (label, indices) in rows {
                let at = |tuning: &Tuning| {
                    let mut state_bundle = with_plans(&base, &[indices.clone()], &[0], tuning);
                    let ui = state_bundle.ui_average_gold_metric(&mut Probe::default()).0
                        [state_bundle.prep_output.optimizer_plan];
                    (state_bundle.metric, ui, exact_single(&state_bundle))
                };
                let (default, _, _) = at(&Tuning::default());
                let (brute, ui, exact) = at(&exact_tuning());
                println!(
                    "  {label:9} default {default:12.3} brute {brute:12.3} ui {ui:12.3} exact {exact:12.3}  {}",
                    model.pattern(&model.canonical(&indices))
                );
            }
        }
    }

    /// per tap data of the same two cases for plotting, to the file in DUMP_OUT
    #[test]
    #[ignore]
    fn lone_dump() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../arena_data/scenarios/tiny");
        let mut out = serde_json::Map::new();
        for name in ["lone_022_t4_n1", "lone_029_t4_n1"] {
            let scenario = &crate::scenario::load(&dir, name)[0];
            let base = retuned(
                &StateBundle::init_from_payload(scenario.steps[0].clone()),
                &exact_tuning(),
            );
            let model = &models(&base)[0];
            let response = response_of(&base, 0);
            let plan_of = |indices: &Vec<usize>| {
                let mut state_bundle = with_plans(&base, &[indices.clone()], &[0], &exact_tuning());
                let plan = state_bundle.prep_output.optimizer_plan;
                let breakdown =
                    state_bundle.ui_average_gold_metric(&mut Probe::default()).2[plan].clone();
                let (dist, used): (Vec<f64>, Vec<Vec<f64>>) =
                    single_outcomes(&state_bundle).into_iter().unzip();
                // the metric with each tap of each pool toggled to its first id or to nothing
                let flips: Vec<Vec<f64>> = (0..model.pools.len())
                    .map(|pool| {
                        (0..model.len)
                            .map(|tap| {
                                let mut other = indices.clone();
                                let stride = model.strides[pool];
                                let id = other[tap] / stride % (model.pools[pool].len() + 1);
                                if id == 0 {
                                    other[tap] += stride;
                                } else {
                                    other[tap] -= stride * id;
                                }
                                with_plans(&base, &[other], &[0], &exact_tuning()).metric
                            })
                            .collect()
                    })
                    .collect();
                json!({
                    "metric": state_bundle.metric,
                    "pattern": model.pattern(&model.canonical(indices)),
                    "pity": model.pity(indices).0,
                    "dist": dist, "used": used, "breakdown": breakdown, "flips": flips,
                })
            };
            let prep = &base.prep_output;
            out.insert(
                name.to_string(),
                json!({
                    "labels": prep.table.labels,
                    "bands": prep.bands[prep.optimizer_plan],
                    "costs": base.upgrade_arr[0].costs,
                    "extra": model.extra, "base": model.base, "artisan": model.artisan,
                    "best": plan_of(&response.best().1),
                    "streak": plan_of(&best_streak(&response).1),
                }),
            );
        }
        std::fs::write(
            std::env::var("DUMP_OUT").unwrap(),
            Value::Object(out).to_string(),
        )
        .unwrap();
    }

    /// every placement of the best plan's juices (pool 0), the rest kept, by material, to DUMP_OUT
    #[test]
    #[ignore]
    fn lone_moves() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../arena_data/scenarios/tiny");
        let mut out = serde_json::Map::new();
        for name in ["lone_022_t4_n1", "lone_029_t4_n1"] {
            let scenario = &crate::scenario::load(&dir, name)[0];
            let base = retuned(
                &StateBundle::init_from_payload(scenario.steps[0].clone()),
                &exact_tuning(),
            );
            let model = &models(&base)[0];
            let best = response_of(&base, 0).best().1;
            let stride = model.strides[0];
            let pity = model.pity(&best).0;
            let rest: Vec<usize> = best
                .iter()
                .map(|x| {
                    x % stride
                        + x / stride / (model.pools[0].len() + 1)
                            * stride
                            * (model.pools[0].len() + 1)
                })
                .collect();
            let used: Vec<usize> = (0..model.len)
                .filter(|&tap| best[tap] != rest[tap])
                .collect();
            let row = |taps: &[usize]| {
                let mut indices = rest.clone();
                for &tap in taps {
                    indices[tap] += stride;
                }
                let mut state_bundle = with_plans(&base, &[indices.clone()], &[0], &exact_tuning());
                let plan = state_bundle.prep_output.optimizer_plan;
                let (metric, _, breakdown) =
                    state_bundle.ui_average_gold_metric(&mut Probe::default());
                let (dist, _): (Vec<f64>, Vec<Vec<f64>>) =
                    single_outcomes(&state_bundle).into_iter().unzip();
                json!({"taps": taps, "metric": metric[plan], "breakdown": breakdown[plan], "pity": model.pity(&indices).0, "dist": dist})
            };
            // every k-subset of the taps before the pity, k the best plan's count, where few enough
            let k = used.len();
            let subsets: Vec<Value> = if k <= 3 {
                (0..pity).combinations(k).map(|taps| row(&taps)).collect()
            } else {
                Vec::new()
            };
            // one juice moved from a used tap to an unused one
            let moves: Vec<Value> = used
                .iter()
                .flat_map(|&from| {
                    (0..pity)
                        .filter(|tap| !used.contains(tap))
                        .map(move |to| (from, to))
                })
                .map(|(from, to)| {
                    let taps: Vec<usize> = used
                        .iter()
                        .map(|&tap| if tap == from { to } else { tap })
                        .sorted()
                        .collect();
                    let mut value = row(&taps);
                    value["from"] = json!(from);
                    value["to"] = json!(to);
                    value
                })
                .collect();
            // the same count as one block, slid from the first tap to the last before the pity
            let slides: Vec<Value> = (0..=pity - k)
                .map(|start| row(&(start..start + k).collect::<Vec<usize>>()))
                .collect();
            // a single juice anywhere, and none
            let singles: Vec<Value> = (0..pity).map(|tap| row(&[tap])).collect();
            out.insert(
                name.to_string(),
                json!({
                    "labels": base.prep_output.table.labels,
                    "used": used, "pity": pity,
                    "best": row(&used), "none": row(&[]),
                    "subsets": subsets, "moves": moves, "slides": slides, "singles": singles,
                }),
            );
        }
        std::fs::write(
            std::env::var("DUMP_OUT").unwrap(),
            Value::Object(out).to_string(),
        )
        .unwrap();
    }
}
