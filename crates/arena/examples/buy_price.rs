// Throwaway: below what market price the optimizer uses more of a juice or book, with no
// tradable stock. A dual estimate (marginal cost at the plan, the price a linear best response
// takes more at, how the market price moves the marginal cost) checked by one polish, against
// the polish search of marginal.rs and against full solves carried from the plan.
// `cargo run --release -p hf-arena --example buy_price -- <dir> <name filter> [<name filter> ...]`
use hf_core::advanced_honing::utils::MAX_ADV_STATE;
use hf_core::materials::NUM_BASE_MATS;
use hf_core::optimizer::response::Response;
use hf_core::optimizer::{Previous, polish_only, solve};
use hf_core::payload::Payload;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::Deserialize;
use smallvec::smallvec;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Deserialize)]
struct Scenario {
    steps: Vec<Payload>,
}

const BISECTIONS: usize = 8;
const POLISHES: usize = 6;

fn more(now: f64, was: f64) -> bool {
    now > was + 1e-6 * was.max(1.0)
}

fn usage(sb: &mut StateBundle) -> Vec<f64> {
    sb.ui_average_gold_metric(&mut Probe::default()).1
}

fn usable(sb: &StateBundle, id: usize) -> bool {
    let ji = &sb.prep_output.juice_info;
    sb.upgrade_arr.iter().any(|u| {
        let ids = if u.is_normal_honing {
            &ji.normal_uindex_to_id
        } else {
            &ji.adv_uindex_to_id
        };
        ids[u.piece_type_usize][u.upgrade_index].contains(&id)
    })
}

fn priced(base: &StateBundle, row: usize, price: f64) -> StateBundle {
    let mut sb = base.clone();
    let m = &mut sb.prep_output.base_materials[row];
    m.taxed_price *= price / m.market_price;
    m.market_price = price;
    sb.apply_chest_state(false);
    sb
}

fn full_plan(base: &StateBundle, row: usize, price: f64) -> StateBundle {
    let previous = Previous::of(base, &[]);
    let mut rng = StdRng::seed_from_u64(0);
    solve(
        &mut rng,
        priced(base, row, price),
        Some(&previous),
        &mut Probe::default(),
    )
    .best
}

fn full(base: &StateBundle, row: usize, price: f64) -> f64 {
    usage(&mut full_plan(base, row, price))[row]
}

// how much better than the base a plan solved at another price is, at market: above 0 where the
// base solve missed it
fn missed(base: &StateBundle, plan: &StateBundle) -> f64 {
    let mut probe = Probe::default();
    let mut at_market = base.clone();
    at_market.my_clone_from(plan);
    at_market.metric_router(&mut probe) - base.clone().metric_router(&mut probe)
}

fn polished(base: &StateBundle, row: usize, price: f64) -> f64 {
    let mut rng = StdRng::seed_from_u64(0);
    let mut sb = priced(base, row, price);
    usage(&mut polish_only(&mut rng, &mut sb, &mut Probe::default()))[row]
}

// the highest price in [0, market] at which `uses_more` holds, by bisection; None where not at 0
fn bisect(market: f64, steps: usize, mut uses_more: impl FnMut(f64) -> bool) -> Option<f64> {
    if !uses_more(0.0) {
        return None;
    }
    let (mut low, mut high) = (0.0, market);
    for _ in 0..steps {
        let mid = (low + high) / 2.0;
        if uses_more(mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    Some((low + high) / 2.0)
}

// the mean use of each row by every plan of an advanced upgrade, from its supports
fn adv_usage(sb: &mut StateBundle, index: usize) -> Vec<Vec<f64>> {
    let slots = sb.upgrade_arr[index].state.payload.len();
    let kept = sb.upgrade_arr[index].state.payload.clone();
    let upgrade = &mut sb.upgrade_arr[index];
    let mut out = Vec::new();
    for mut plan in 0..(MAX_ADV_STATE + 1).pow(slots as u32) {
        let payload = (0..slots)
            .map(|_| {
                let k = plan % (MAX_ADV_STATE + 1);
                plan /= MAX_ADV_STATE + 1;
                smallvec![k]
            })
            .collect();
        upgrade.state.update_payload(payload);
        upgrade.update_this_prob_dist(&mut sb.adv_cache, &sb.prep_output.juice_info);
        upgrade.update_this_support(&sb.prep_output.juice_info);
        out.push(
            upgrade
                .cost_dist
                .iter()
                .map(|support| support.mean)
                .collect(),
        );
    }
    upgrade.state.update_payload(kept);
    upgrade.update_this_prob_dist(&mut sb.adv_cache, &sb.prep_output.juice_info);
    upgrade.update_this_support(&sb.prep_output.juice_info);
    out
}

// The dual estimate: (pi, b, s, estimate). pi is what one more unit used costs at the plan, b the
// highest price of the row at which some upgrade's linear best response, the other rows at their
// marginal costs, uses more than its plan does, s how pi moves with the market price.
fn dual(base: &mut StateBundle, row: usize) -> (f64, Option<f64>, f64, Option<f64>) {
    let mut probe = Probe::default();
    base.metric = base.metric_router(&mut probe);
    let prices = base.shadow_prices(&mut probe);
    let pi = prices[row];
    let id = row - NUM_BASE_MATS;
    let mut b: Option<f64> = None;
    for index in 0..base.upgrade_arr.len() {
        let u = &base.upgrade_arr[index];
        let ji = &base.prep_output.juice_info;
        if !u.is_normal_honing {
            if !ji.adv_uindex_to_id[u.piece_type_usize][u.upgrade_index].contains(&id) {
                continue;
            }
            // an advanced upgrade takes the plan of least priced mean use
            let usage = adv_usage(base, index);
            let use_at = |p: f64| {
                let mut at = prices.clone();
                at[row] = p;
                let cost =
                    |plan: &Vec<f64>| -> f64 { plan.iter().zip(&at).map(|(u, p)| u * p).sum() };
                usage
                    .iter()
                    .min_by(|a, b| cost(a).total_cmp(&cost(b)))
                    .unwrap()[row]
            };
            let actual = base.upgrade_arr[index]
                .state
                .payload
                .iter()
                .rev()
                .fold(0, |id, slot| id * (MAX_ADV_STATE + 1) + slot[0]);
            let was = usage[actual][row];
            if let Some(found) = bisect(pi, 20, |p| more(use_at(p), was)) {
                b = Some(b.map_or(found, |b: f64| b.max(found)));
            }
            continue;
        }
        if !ji.normal_uindex_to_id[u.piece_type_usize][u.upgrade_index].contains(&id) {
            continue;
        }
        // a plan's expected use of the row is its score at a price less its score a unit dearer
        let use_of = |plan: &[usize]| {
            let mut at = prices.clone();
            at[row] = 1.0;
            let dearer = Response::linear(base, index, &at).score(plan);
            at[row] = 0.0;
            Response::linear(base, index, &at).score(plan) - dearer
        };
        let use_at = |p: f64| {
            let mut at = prices.clone();
            at[row] = p;
            use_of(&Response::linear(base, index, &at).best().1)
        };
        let model = &Response::linear(base, index, &prices).model;
        let was = use_of(&model.canonical(&model.indices(&base.upgrade_arr[index].state.payload)));
        if let Some(found) = bisect(pi, 20, |p| more(use_at(p), was)) {
            b = Some(b.map_or(found, |b: f64| b.max(found)));
        }
    }
    let market = base.prep_output.base_materials[row].market_price;
    let delta = 0.01 * market;
    let mut bumped = priced(base, row, market + delta);
    bumped.metric = bumped.metric_router(&mut probe);
    let s = (bumped.shadow_prices(&mut probe)[row] - pi) / delta;
    let estimate = b
        .filter(|_| s > 1e-9)
        .map(|b| (market - (pi - b).max(0.0) / s).clamp(0.0, market));
    (pi, b, s, estimate)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = PathBuf::from(&args[1]);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            args[2..]
                .iter()
                .any(|f| p.to_string_lossy().contains(f.as_str()))
        })
        .collect();
    paths.sort();
    for path in paths {
        let scenario: Scenario =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let mut sb = StateBundle::init_from_payload(scenario.steps[0].clone());
        for m in sb.prep_output.base_materials.iter_mut() {
            m.tradable = 0.0;
        }
        sb.apply_chest_state(true);
        let t = Instant::now();
        // cold, then once more from that, the better kept
        let mut rng = StdRng::seed_from_u64(0);
        let cold = solve(&mut rng, sb.clone(), None, &mut Probe::default()).best;
        let again = solve(
            &mut rng,
            sb,
            Some(&Previous::of(&cold, &[])),
            &mut Probe::default(),
        )
        .best;
        let mut base = if again.metric > cold.metric {
            again
        } else {
            cold
        };
        let used = usage(&mut base);
        println!(
            "{} ({} upgrades, base solves {:.1}s)",
            path.file_stem().unwrap().to_string_lossy(),
            base.upgrade_arr.len(),
            t.elapsed().as_secs_f64()
        );
        println!(
            "  {:<24} {:>8} {:>8} {:>8} {:>8} {:>7} | {:>8} {:>5} {:>6} | {:>8} {:>6} | {:>8} {:>6} | {:>8} {:>6} {:>8}",
            "row",
            "market",
            "owned",
            "used",
            "pi",
            "s",
            "dual",
            "check",
            "s",
            "refined",
            "s",
            "polish",
            "s",
            "full",
            "s",
            "missed"
        );
        for row in NUM_BASE_MATS..base.prep_output.table.len() {
            let m = base.prep_output.base_materials[row];
            if m.market_price <= 0.0 || !usable(&base, row - NUM_BASE_MATS) {
                continue;
            }
            let market = m.market_price;
            let t = Instant::now();
            let (pi, _b, s, estimate) = dual(&mut base, row);
            let check = estimate.map(|p| more(polished(&base, row, p), used[row]));
            let dual_s = t.elapsed().as_secs_f64();
            // three more polishes, bisecting on the side of the estimate the check says
            let t = Instant::now();
            let refined = estimate.zip(check).map(|(p, ok)| {
                let (mut low, mut high) = if ok { (p, market) } else { (0.0, p) };
                for _ in 0..3 {
                    let mid = (low + high) / 2.0;
                    if more(polished(&base, row, mid), used[row]) {
                        low = mid;
                    } else {
                        high = mid;
                    }
                }
                (low + high) / 2.0
            });
            let refined_s = t.elapsed().as_secs_f64();
            let t = Instant::now();
            let polish = bisect(market, POLISHES, |p| {
                more(polished(&base, row, p), used[row])
            });
            let polish_s = t.elapsed().as_secs_f64();
            let t = Instant::now();
            let at_market = full(&base, row, market);
            let reference = bisect(market, BISECTIONS, |p| more(full(&base, row, p), at_market));
            let full_s = t.elapsed().as_secs_f64();
            let miss = reference.map_or(0.0, |p| missed(&base, &full_plan(&base, row, p)));
            let text = |x: Option<f64>| x.map_or("-".to_string(), |x| format!("{x:.1}"));
            println!(
                "  {:<24} {:>8.1} {:>8.0} {:>8.1} {:>8.2} {:>7.3} | {:>8} {:>5} {:>6.2} | {:>8} {:>6.2} | {:>8} {:>6.2} | {:>8} {:>6.1} {:>8.0}",
                base.prep_output.table.labels[row],
                market,
                m.bound + m.roster,
                used[row],
                pi,
                s,
                text(estimate),
                check.map_or("-", |c| if c { "yes" } else { "no" }),
                dual_s,
                text(refined),
                refined_s,
                text(polish),
                polish_s,
                text(reference),
                full_s,
                miss
            );
        }
    }
}
