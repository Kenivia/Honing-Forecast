//! How much of a chain step's best known plan is still good on the step after it.

use crate::db;
use crate::scenario::Scenario;
use crate::worker::Essence;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use hf_core::materials::{BandValue, ValuationPlan};
use hf_core::parser::OneUpgradeInput;
use hf_core::payload::Payload;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use rusqlite::Connection;
use std::collections::{BTreeMap, HashMap};

/// the payload with its keys sorted, which is what a step is identified by
pub fn step_key(payload: &Payload) -> String {
    serde_json::to_value(payload).unwrap().to_string()
}

/// what the state costs with every leftover worthless: the gold spent
pub fn gross(state_bundle: &StateBundle) -> f64 {
    let mut state_bundle = state_bundle.clone();
    state_bundle.prep_output.plans = vec![ValuationPlan([BandValue::Worthless; 3])];
    state_bundle.prep_output.optimizer_plan = 0;
    state_bundle.apply_chest_state(false);
    state_bundle.metric_router(&mut Probe::default())
}

/// what the state uses on average, at market price, whoever owns it
pub fn flow(state_bundle: &StateBundle) -> f64 {
    let (_, avg_used, _) = state_bundle
        .clone()
        .ui_average_gold_metric(&mut Probe::default());
    avg_used
        .iter()
        .zip(&state_bundle.prep_output.base_materials)
        .map(|(used, mat)| used * mat.market_price)
        .sum()
}

/// What a gap is a share of: the gold spent plus what the leftovers are credited, of the
/// best known state. Unlike the metric, which is their difference, it is never near zero.
pub fn scale(metric: f64, gross: Option<f64>) -> f64 {
    gross
        .map_or(metric.abs(), |gross| metric - 2.0 * gross)
        .max(1.0)
}

fn key(info: &OneUpgradeInput) -> (usize, usize, bool) {
    (info.piece_index, info.upgrade_index, info.is_normal_honing)
}

/// The plan of the step before, on this step's upgrades. An upgrade keeps its state, less the
/// taps done since, and its place in the free tap order; a new upgrade goes last.
pub fn carried(before: &Payload, plan: &Essence, payload: &Payload) -> StateBundle {
    let mut state_bundle = StateBundle::init_from_payload(payload.clone());
    let was: HashMap<(usize, usize, bool), usize> = before
        .upgrade_info
        .iter()
        .enumerate()
        .map(|(index, info)| (key(info), index))
        .collect();
    let now: HashMap<(usize, usize, bool), usize> = payload
        .upgrade_info
        .iter()
        .enumerate()
        .map(|(index, info)| (key(info), index))
        .collect();

    for (index, info) in payload.upgrade_info.iter().enumerate() {
        let Some(&old) = was.get(&key(info)) else {
            continue;
        };
        let mut state = plan.state_arr[old].clone();
        let len = state_bundle.upgrade_arr[index].state.len();
        if info.is_normal_honing && state.len() > len {
            state.drain(..state.len() - len);
        }
        if state.len() == len {
            state_bundle.upgrade_arr[index].state.update_payload(state);
        }
    }
    let mut order: Vec<usize> = plan
        .special_state
        .iter()
        .filter_map(|&old| now.get(&key(&before.upgrade_info[old])).copied())
        .collect();
    let new: Vec<usize> = (0..payload.upgrade_info.len())
        .filter(|index| !order.contains(index))
        .collect();
    order.extend(new);
    state_bundle.special_state = order;
    state_bundle
}

fn median(values: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    sorted.sort_by(|a, b| a.total_cmp(b));
    sorted.get(sorted.len() / 2).copied().unwrap_or(f64::NAN)
}

/// For every step after the first: the default state, the carried plan and the best known,
/// by what happened before the step.
pub fn report(conn: &Connection, scenarios: &[Scenario], rows: bool, eps: f64) {
    let mut each = Table::new();
    each.load_style(UTF8_BORDERS_ONLY).set_header(vec![
        "scenario",
        "step",
        "after",
        "default",
        "carried",
        "best",
        "recovered %",
        "carried gap %",
        "default gap %",
    ]);
    // per kind: recovered, carried gap, default gap
    let mut by_kind: BTreeMap<String, (Vec<f64>, Vec<f64>, Vec<f64>)> = BTreeMap::new();
    for scenario in scenarios {
        let ids: Vec<i64> = scenario
            .steps
            .iter()
            .enumerate()
            .map(|(step, payload)| db::step_id(conn, &scenario.name, step, &step_key(payload)))
            .collect();
        for step in 1..scenario.steps.len() {
            let (Some(before), Some(best)) = (
                db::best_run(conn, ids[step - 1]),
                db::best_run(conn, ids[step]),
            ) else {
                continue;
            };
            let payload = &scenario.steps[step];
            // as metric_ref is
            let reference = |mut state_bundle: StateBundle| {
                state_bundle.tuning = Tuning::reference();
                state_bundle.metric_router(&mut Probe::default())
            };
            let default = reference(StateBundle::init_from_payload(payload.clone()));
            let kept = reference(carried(&scenario.steps[step - 1], &before.2, payload));
            let scale = scale(best.0, best.1);
            let headroom = best.0 - default;
            let recovered = if headroom > eps * scale {
                (kept - default) / headroom
            } else {
                f64::NAN
            };
            let (kept_gap, default_gap) = ((best.0 - kept) / scale, headroom / scale);
            let kind = scenario
                .meta
                .get("kinds")
                .and_then(|kinds| kinds[step].as_str())
                .unwrap_or("?")
                .to_string();
            each.add_row(vec![
                scenario.name.clone(),
                step.to_string(),
                kind.clone(),
                format!("{default:.0}"),
                format!("{kept:.0}"),
                format!("{:.0}", best.0),
                format!("{:.1}", recovered * 100.0),
                format!("{:.4}", kept_gap * 100.0),
                format!("{:.4}", default_gap * 100.0),
            ]);
            for group in [kind, "all".to_string()] {
                let entry = by_kind.entry(group).or_default();
                entry.0.push(recovered);
                entry.1.push(kept_gap);
                entry.2.push(default_gap);
            }
        }
    }
    if rows {
        println!("{each}");
    }

    let mut out = Table::new();
    out.load_style(UTF8_BORDERS_ONLY).set_header(vec![
        "after",
        "steps",
        "median recovered %",
        "carried hit %",
        "default hit %",
        "median carried gap %",
        "median default gap %",
        "carried beats best %",
    ]);
    for (kind, (recovered, kept, default)) in &by_kind {
        let share = |values: &[f64], test: fn(f64, f64) -> bool| {
            100.0 * values.iter().filter(|x| test(**x, eps)).count() as f64 / values.len() as f64
        };
        out.add_row(vec![
            kind.clone(),
            kept.len().to_string(),
            format!("{:.1}", median(recovered) * 100.0),
            format!("{:.0}", share(kept, |gap, eps| gap <= eps)),
            format!("{:.0}", share(default, |gap, eps| gap <= eps)),
            format!("{:.4}", median(kept) * 100.0),
            format!("{:.4}", median(default) * 100.0),
            format!("{:.0}", share(kept, |gap, eps| gap < -eps)),
        ]);
    }
    println!("{out}");
}
