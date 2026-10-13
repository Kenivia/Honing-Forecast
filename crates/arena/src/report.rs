//! Tables over what is in the database.

use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use rusqlite::{Connection, params_from_iter};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// which runs a report covers
pub struct Selection {
    pub session: Option<i64>,
    pub variants: Vec<String>,
}

impl Selection {
    fn clause(&self) -> (String, Vec<String>) {
        let mut clause = String::new();
        let mut values: Vec<String> = Vec::new();
        if let Some(session) = self.session {
            clause += " AND r.session_id = ?";
            values.push(session.to_string());
        }
        if !self.variants.is_empty() {
            clause += &format!(" AND r.variant IN ({})", vec!["?"; self.variants.len()].join(","));
            values.extend(self.variants.iter().cloned());
        }
        (clause, values)
    }
}

fn median(values: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted.sort_by(|a, b| a.total_cmp(b));
    sorted[sorted.len() / 2]
}

pub fn table(header: &[&str]) -> Table {
    let mut out = Table::new();
    out.load_style(UTF8_BORDERS_ONLY).set_header(header.to_vec());
    out
}

struct Run {
    gap: f64,
    cpu_s: f64,
    decide_s: f64,
    states: f64,
    ks: f64,
    found_at: f64,
    failed: bool,
    mc_fails: i64,
}

/// Quality and speed of optimizer runs. The gap is to the best metric any run in the
/// database has reached on the same payload, scored by the driver's evaluator.
pub fn optimize(conn: &Connection, selection: &Selection, eps: f64) {
    let mut best_known: HashMap<i64, f64> = HashMap::new();
    let mut statement = conn
        .prepare("SELECT step_id, MAX(metric_ref) FROM run WHERE kind = 'optimize' AND error IS NULL GROUP BY step_id")
        .unwrap();
    for row in statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?))).unwrap() {
        let (step_id, best): (i64, f64) = row.unwrap();
        best_known.insert(step_id, best);
    }

    let (clause, values) = selection.clause();
    let mut statement = conn
        .prepare(&format!(
            "SELECT s.scenario, s.step, r.chained, r.variant, r.step_id, r.cpu_s, r.metric_ref,
                r.counters, r.error,
                (SELECT MAX(iter) FROM trace t WHERE t.run_id = r.id AND t.key = 'best'),
                (SELECT COUNT(*) FROM mc WHERE mc.run_id = r.id AND mc.pass = 0),
                r.decide_s
             FROM run r JOIN scenario_step s ON s.id = r.step_id
             WHERE r.kind = 'optimize'{clause}"
        ))
        .unwrap();
    let mut groups: BTreeMap<(String, i64, bool), BTreeMap<String, Vec<Run>>> = BTreeMap::new();
    let rows = statement
        .query_map(params_from_iter(values), |row| {
            let step_id: i64 = row.get(4)?;
            let metric_ref: Option<f64> = row.get(6)?;
            let counters: Value =
                serde_json::from_str(&row.get::<_, Option<String>>(7)?.unwrap_or("{}".to_string())).unwrap();
            let best = best_known.get(&step_id).copied().unwrap_or(f64::NAN);
            let run = Run {
                gap: (best - metric_ref.unwrap_or(f64::NAN)) / best.abs().max(1.0),
                cpu_s: row.get::<_, Option<f64>>(5)?.unwrap_or(f64::NAN),
                decide_s: row.get::<_, Option<f64>>(11)?.unwrap_or(f64::NAN),
                states: counters["StatesEvaluated"].as_f64().unwrap_or(f64::NAN),
                ks: counters["Ks"].as_f64().unwrap_or(f64::NAN),
                found_at: row.get::<_, Option<f64>>(9)?.unwrap_or(f64::NAN),
                failed: row.get::<_, Option<String>>(8)?.is_some(),
                mc_fails: row.get(10)?,
            };
            Ok(((row.get(0)?, row.get(1)?, row.get(2)?), row.get::<_, String>(3)?, run))
        })
        .unwrap();
    for row in rows {
        let (group, variant, run) = row.unwrap();
        groups.entry(group).or_default().entry(variant).or_default().push(run);
    }
    if groups.is_empty() {
        return;
    }

    let mut out = table(&[
        "scenario", "step", "mode", "variant", "runs", "died", "median gap %", "worst gap %",
        "hit %", "cpu s", "decide s", "states", "ks", "found at", "mc fails",
    ]);
    // per variant: median gaps, hit rates, cpu
    let mut overall: BTreeMap<String, (Vec<f64>, Vec<f64>, f64, usize, i64)> = BTreeMap::new();
    for ((scenario, step, chained), variants) in &groups {
        for (variant, runs) in variants {
            let pick = |f: fn(&Run) -> f64| runs.iter().map(f).collect::<Vec<f64>>();
            let gaps = pick(|run| run.gap);
            let finished = gaps.iter().filter(|gap| gap.is_finite()).count().max(1);
            let hit = gaps.iter().filter(|gap| **gap <= eps).count() as f64 / finished as f64;
            let worst = gaps.iter().copied().filter(|x| x.is_finite()).fold(f64::NAN, f64::max);
            let died = runs.iter().filter(|run| run.failed).count();
            let mc_fails: i64 = runs.iter().map(|run| run.mc_fails).sum();
            let cpu = median(&pick(|run| run.cpu_s));
            out.add_row(vec![
                scenario.clone(),
                step.to_string(),
                if *chained { "chained" } else { "cold" }.to_string(),
                variant.clone(),
                runs.len().to_string(),
                died.to_string(),
                format!("{:.4}", median(&gaps) * 100.0),
                format!("{:.4}", worst * 100.0),
                format!("{:.0}", hit * 100.0),
                format!("{:.3}", cpu),
                format!("{:.3}", median(&pick(|run| run.decide_s))),
                format!("{:.0}", median(&pick(|run| run.states))),
                format!("{:.0}", median(&pick(|run| run.ks))),
                format!("{:.0}", median(&pick(|run| run.found_at))),
                mc_fails.to_string(),
            ]);
            let total = overall.entry(variant.clone()).or_default();
            total.0.push(median(&gaps));
            total.1.push(hit);
            total.2 += cpu;
            total.3 += died;
            total.4 += mc_fails;
        }
    }
    println!("{out}");

    let mut out = table(&["variant", "mean median gap %", "mean hit %", "sum cpu s", "died", "mc fails"]);
    for (variant, (gaps, hits, cpu, died, mc_fails)) in overall {
        let mean = |values: &[f64]| values.iter().sum::<f64>() / values.len() as f64;
        out.add_row(vec![
            variant,
            format!("{:.4}", mean(&gaps) * 100.0),
            format!("{:.0}", mean(&hits) * 100.0),
            format!("{:.3}", cpu),
            died.to_string(),
            mc_fails.to_string(),
        ]);
    }
    println!("{out}");
}

/// The evaluator against monte carlo, and what one evaluation costs.
pub fn verify(conn: &Connection, selection: &Selection) {
    let (clause, values) = selection.clause();
    let mut statement = conn
        .prepare(&format!(
            "SELECT r.variant, r.cpu_s, r.counters, r.error,
                (SELECT COUNT(*) FROM mc WHERE mc.run_id = r.id AND mc.pass = 0),
                (SELECT ABS(z) FROM mc WHERE mc.run_id = r.id AND mc.material = 'total'),
                (SELECT ABS(analytic - mean) / MAX(ABS(analytic), 1) FROM mc WHERE mc.run_id = r.id AND mc.material = 'total')
             FROM run r WHERE r.kind = 'verify'{clause}"
        ))
        .unwrap();
    // per variant: cpu, ks, failing rows, |z|, relative error, died
    let mut variants: BTreeMap<String, (Vec<f64>, Vec<f64>, i64, Vec<f64>, Vec<f64>, usize)> = BTreeMap::new();
    let rows = statement
        .query_map(params_from_iter(values.clone()), |row| {
            let counters: Value =
                serde_json::from_str(&row.get::<_, Option<String>>(2)?.unwrap_or("{}".to_string())).unwrap();
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<f64>>(1)?,
                counters["Ks"].as_f64(),
                row.get::<_, Option<String>>(3)?.is_some(),
                row.get::<_, i64>(4)?,
                row.get::<_, Option<f64>>(5)?,
                row.get::<_, Option<f64>>(6)?,
            ))
        })
        .unwrap();
    for row in rows {
        let (variant, cpu_s, ks, died, fails, z, rel) = row.unwrap();
        let this = variants.entry(variant).or_default();
        this.0.extend(cpu_s);
        this.1.extend(ks);
        this.2 += fails;
        this.3.extend(z);
        this.4.extend(rel);
        this.5 += died as usize;
    }
    if variants.is_empty() {
        return;
    }

    let mut out = table(&[
        "variant", "states", "died", "failing rows", "worst |z| total", "worst rel err total",
        "eval ms", "ks per eval",
    ]);
    for (variant, (cpu, ks, fails, z, rel, died)) in variants {
        let worst = |values: &[f64]| values.iter().copied().fold(0.0, f64::max);
        out.add_row(vec![
            variant,
            cpu.len().to_string(),
            died.to_string(),
            fails.to_string(),
            format!("{:.2}", worst(&z)),
            format!("{:.2e}", worst(&rel)),
            format!("{:.4}", median(&cpu) * 1000.0),
            format!("{:.0}", median(&ks)),
        ]);
    }
    println!("{out}");

    let mut statement = conn
        .prepare(&format!(
            "SELECT r.id, r.variant, s.scenario, s.step, r.source, mc.material, mc.analytic, mc.mean, mc.se, mc.z
             FROM mc JOIN run r ON r.id = mc.run_id JOIN scenario_step s ON s.id = r.step_id
             WHERE mc.pass = 0{clause} ORDER BY ABS(mc.z) DESC LIMIT 30"
        ))
        .unwrap();
    let mut out = table(&["run", "variant", "scenario", "step", "state", "material", "analytic", "monte carlo", "se", "z"]);
    let mut any = false;
    let rows = statement
        .query_map(params_from_iter(values), |row| {
            Ok(vec![
                row.get::<_, i64>(0)?.to_string(),
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?.to_string(),
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                format!("{:.2}", row.get::<_, f64>(6)?),
                format!("{:.2}", row.get::<_, f64>(7)?),
                format!("{:.2}", row.get::<_, f64>(8)?),
                format!("{:.2}", row.get::<_, f64>(9)?),
            ])
        })
        .unwrap();
    for row in rows {
        any = true;
        out.add_row(row.unwrap());
    }
    if any {
        println!("monte carlo disagreements, worst first:\n{out}");
    }
}

pub fn errors(conn: &Connection, selection: &Selection) {
    let (clause, values) = selection.clause();
    let mut statement = conn
        .prepare(&format!("SELECT r.variant, r.error FROM run r WHERE r.error IS NOT NULL{clause} LIMIT 5"))
        .unwrap();
    let rows = statement
        .query_map(params_from_iter(values), |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .unwrap();
    for row in rows {
        let (variant, error) = row.unwrap();
        println!("--- {variant} died:\n{error}");
    }
}

pub fn trace(conn: &Connection, run_id: i64, key: Option<String>) {
    let mut statement = conn
        .prepare("SELECT iter, key, value FROM trace WHERE run_id = ?1 AND (?2 IS NULL OR key = ?2) ORDER BY iter")
        .unwrap();
    let rows = statement
        .query_map((run_id, key), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, f64>(2)?))
        })
        .unwrap();
    println!("iter,key,value");
    for row in rows {
        let (iter, key, value) = row.unwrap();
        println!("{iter},{key},{value}");
    }
}
