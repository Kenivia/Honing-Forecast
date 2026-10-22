// Throwaway: value of owned juice with buying disabled, by full solves at x and x + delta,
// the second carried from the first. SWEEP=name:label:points:spacing (in uses).
use hf_core::materials::NUM_BASE_MATS;
use hf_core::optimizer::{Previous, solve};
use hf_core::payload::Payload;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_core::tuning::Tuning;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Deserialize)]
struct Scenario {
    steps: Vec<Payload>,
}

const FACTOR: f64 = 1e3;
const MULTS: [f64; 4] = [1.0, 2.0, 4.0, 8.0];

// the largest one use of the juice by any upgrade, times the block
fn one_use(sb: &StateBundle, id: usize) -> f64 {
    let ji = &sb.prep_output.juice_info;
    sb.upgrade_arr
        .iter()
        .map(|u| {
            let ids = if u.is_normal_honing {
                &ji.normal_uindex_to_id
            } else {
                &ji.adv_uindex_to_id
            };
            if !ids[u.piece_type_usize][u.upgrade_index].contains(&id) {
                return 0.0;
            }
            let j = ji.access(id, u.piece_type_usize, u.upgrade_index);
            if u.is_normal_honing {
                j.normal_amt_used as f64
            } else {
                j.adv_amt_used as f64
            }
        })
        .fold(0.0, f64::max)
        * sb.block() as f64
}

struct Run {
    total: f64,
    used: f64,
    best: StateBundle,
}

fn solved(
    _payload: &Payload,
    base: &StateBundle,
    row: usize,
    stock: f64,
    from: Option<&StateBundle>,
    seed: u64,
) -> Run {
    let mut sb = base.clone();
    let mat = &mut sb.prep_output.base_materials[row];
    mat.market_price *= FACTOR;
    mat.bound += stock;
    sb.apply_chest_state(true);
    let previous = from.map(|best| Previous::of(best, &[]));
    let mut probe = Probe::default();
    let mut rng = StdRng::seed_from_u64(seed);
    let best = solve(&mut rng, sb, previous.as_ref(), &mut probe).best;
    let mut scored = best.clone();
    scored.tuning = Tuning::reference();
    let plan = scored.prep_output.optimizer_plan;
    let (_, used, gold) = scored.ui_average_gold_metric(&mut probe);
    Run {
        total: gold[plan].iter().sum(),
        used: used[row],
        best,
    }
}

fn load(path: &PathBuf) -> (Payload, StateBundle) {
    let scenario: Scenario = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let payload = scenario.steps[0].clone();
    (payload.clone(), StateBundle::init_from_payload(payload))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = PathBuf::from(&args[1]);
    let count: usize = args[2].parse().unwrap();
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();

    if let Ok(sweep) = std::env::var("ENVELOPE") {
        // V(stock) cold, then improved by carrying each neighbour's plan, forwards and back
        let parts: Vec<&str> = sweep.split(':').collect();
        let path = paths
            .iter()
            .find(|p| p.to_string_lossy().contains(parts[0]))
            .unwrap();
        let (payload, mut base) = load(path);
        let row = base.prep_output.table.row(parts[1]);
        let points: usize = parts[2].parse().unwrap();
        let use_ = one_use(&base, row - NUM_BASE_MATS);
        let spacing = use_ * parts[3].parse::<f64>().unwrap();
        let m = &mut base.prep_output.base_materials[row];
        m.bound = 0.0;
        m.roster = 0.0;
        m.tradable = 0.0;
        let stock = |i: usize| spacing * i as f64;
        let cold: Vec<Run> = (0..points)
            .map(|i| solved(&payload, &base, row, stock(i), None, 0))
            .collect();
        let mut best: Vec<(f64, f64, StateBundle)> = cold
            .iter()
            .map(|r| (r.total, r.used, r.best.clone()))
            .collect();
        let order: Vec<usize> = (1..points).chain((0..points - 1).rev()).collect();
        for (k, &i) in order.iter().enumerate() {
            let j = if k < points - 1 { i - 1 } else { i + 1 };
            let r = solved(&payload, &base, row, stock(i), Some(&best[j].2.clone()), 0);
            if r.total > best[i].0 {
                best[i] = (r.total, r.used, r.best);
            }
        }
        println!("  stock | cold used  value | envelope used  value | gain");
        for i in 0..points - 1 {
            println!(
                "  {:>5.0} | {:>9.1} {:>6.1} | {:>13.1} {:>6.1} | {:>6.1}",
                stock(i),
                cold[i].used,
                (cold[i + 1].total - cold[i].total) / spacing,
                best[i].1,
                (best[i + 1].0 - best[i].0) / spacing,
                best[i].0 - cold[i].total
            );
        }
        return;
    }
    if let Ok(sweep) = std::env::var("SWEEP") {
        let parts: Vec<&str> = sweep.split(':').collect();
        let path = paths
            .iter()
            .find(|p| p.to_string_lossy().contains(parts[0]))
            .unwrap();
        let (payload, mut base) = load(path);
        let row = base.prep_output.table.row(parts[1]);
        let points: usize = parts[2].parse().unwrap();
        let use_ = one_use(&base, row - NUM_BASE_MATS);
        let spacing = use_ * parts[3].parse::<f64>().unwrap();
        let m = &mut base.prep_output.base_materials[row];
        println!(
            "market {:.1} one use {} spacing {}",
            m.market_price, use_, spacing
        );
        println!("  stock    used | carried d=1    d=2    d=4    d=8 | cold d=2");
        m.bound = 0.0;
        m.roster = 0.0;
        m.tradable = 0.0;
        for i in 0..points {
            let stock = spacing * i as f64;
            let x = solved(&payload, &base, row, stock, None, 0);
            let carried: Vec<f64> = MULTS
                .iter()
                .map(|mult| {
                    let d = use_ * mult;
                    let y = solved(&payload, &base, row, stock + d, Some(&x.best), 0);
                    (y.total - x.total) / d
                })
                .collect();
            let cold = solved(&payload, &base, row, stock + 2.0 * use_, None, 0);
            println!(
                "  {:>5.0} {:>7.1} | {:>8.1} {:>6.1} {:>6.1} {:>6.1} | {:>8.1}",
                stock,
                x.used,
                carried[0],
                carried[1],
                carried[2],
                carried[3],
                (cold.total - x.total) / (2.0 * use_)
            );
        }
        return;
    }

    // per row at the scenario's own stock: carried values per delta, over two seeds
    for path in paths.iter().take(count) {
        let (payload, base) = load(path);
        println!("{}", path.file_stem().unwrap().to_string_lossy());
        for row in NUM_BASE_MATS..base.prep_output.table.len() {
            let m = base.prep_output.base_materials[row];
            let use_ = one_use(&base, row - NUM_BASE_MATS);
            if use_ <= 0.0 || m.market_price <= 0.0 {
                continue;
            }
            let t = Instant::now();
            let per_seed: Vec<Vec<f64>> = (0..2)
                .map(|seed| {
                    let x = solved(&payload, &base, row, 0.0, None, seed);
                    MULTS
                        .iter()
                        .map(|mult| {
                            let d = use_ * mult;
                            let y = solved(&payload, &base, row, d, Some(&x.best), seed);
                            ((y.total - x.total) / d).round()
                        })
                        .collect()
                })
                .collect();
            println!(
                "  {:<22} market {:>8.1} owned {:>6.0} use {:>3.0} d=1,2,4,8 seed0 {:?} seed1 {:?} {:.1}s",
                base.prep_output.table.labels[row],
                m.market_price,
                m.bound + m.roster + m.tradable,
                use_,
                per_seed[0],
                per_seed[1],
                t.elapsed().as_secs_f64()
            );
        }
    }
}
