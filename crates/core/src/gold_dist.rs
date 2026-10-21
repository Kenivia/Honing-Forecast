//! The distribution of the gold one plan spends. The materials are spent together, so this
//! needs their joint distribution, which no lattice holds: it is sampled instead.
use crate::advanced_honing::sim::one_sim;
use crate::advanced_honing::utils::AdvConfig;
use crate::constants::{BUCKET_COUNT, IGNORE_PROB_TOL, SPECIAL_TOL, float_tol};
use crate::materials::{NUM_BASE_MATS, gold};
use crate::state_bundle::StateBundle;
use crate::upgrade::Upgrade;
use ahash::AHashMap;
use rand::prelude::*;
use rand::rngs::SmallRng;
use serde::Serialize;
use std::cell::RefCell;
use std::hash::{DefaultHasher, Hash, Hasher};

/// about 0.4% off at worst over the whole distribution, 19 times out of 20
pub const GOLD_SAMPLES: usize = 100_000;
const MAX_ADV_RUNS: usize = 64;

thread_local! {
    // Played out runs of each advanced strategy, kept while the worker lives: playing them
    // out is most of the time otherwise.
    static ADV_RUNS: RefCell<AHashMap<(AdvConfig, usize), Vec<(u8, u8, u8)>>> =
        RefCell::default();
}

/// n runs of one advanced strategy, in an order of their own
fn adv_runs<R: Rng>(config: &AdvConfig, n: usize, rng: &mut R) -> Vec<(u8, u8, u8)> {
    let mut runs = ADV_RUNS.with_borrow_mut(|cache| {
        if cache.len() >= MAX_ADV_RUNS && !cache.contains_key(&(*config, n)) {
            cache.clear();
        }
        cache
            .entry((*config, n))
            .or_insert_with(|| {
                let mut hasher = DefaultHasher::new();
                config.hash(&mut hasher);
                let mut own = SmallRng::seed_from_u64(hasher.finish());
                (0..n).map(|_| one_sim(&mut own, config)).collect()
            })
            .clone()
    });
    runs.shuffle(rng);
    runs
}

#[derive(Debug, Serialize)]
pub struct GoldDist {
    /// (gold spent, chance to spend at most that), from luckiest to pity
    pub cdf: Vec<(f64, f64)>,
    /// (gold spent, log10 of the chance to spend at most that)
    pub luckiest: (f64, f64),
    /// (gold spent, log10 of the chance to spend at least that)
    pub pity: (f64, f64),
}

/// Instead of actually sampling the distribution, we guarantee that every value has exactly the expected number of occurances
pub fn tap_map_generator<R: Rng>(count_limit: usize, prob_dist: &[f64], rng: &mut R) -> Vec<usize> {
    let mut tap_map: Vec<usize> = Vec::with_capacity(count_limit);

    let mut current_point: f64 = rng.random::<f64>();
    let mut cum_prob: f64 = 0.0;

    for (i, &p) in prob_dist.iter().enumerate() {
        cum_prob += p;
        let target_boundary: f64 = cum_prob * (count_limit as f64);
        while current_point < target_boundary && tap_map.len() < count_limit {
            tap_map.push(i);
            current_point += 1.0;
        }
    }

    // the distribution may sum to slightly under 1
    while tap_map.len() < count_limit {
        tap_map.push(prob_dist.len() - 1);
    }
    tap_map.shuffle(rng);
    tap_map
}

/// the rows an upgrade uses anything of
fn live_rows(upgrade: &Upgrade) -> Vec<usize> {
    (0..upgrade.cost_dist.len())
        .filter(|&row| upgrade.cost_dist[row].support.iter().any(|x| *x != 0.0))
        .collect()
}

/// What one outcome uses of every material: taps index the base supports, and for advanced
/// honing juice and scroll index the juice ones.
fn add_outcome(
    upgrade: &Upgrade,
    rows: &[usize],
    (taps, juice, scroll): (usize, usize, usize),
    used: &mut [f64],
) {
    for &row in rows {
        let index = if upgrade.is_normal_honing || row < NUM_BASE_MATS {
            taps
        } else if row - NUM_BASE_MATS <= 1 {
            juice
        } else {
            scroll
        };
        let support = &upgrade.cost_dist[row].support;
        used[row] += support[index.min(support.len() - 1)];
    }
}

fn add_skipped(upgrade: &Upgrade, rows: &[usize], used: &mut [f64]) {
    for &row in rows {
        used[row] += upgrade.cost_dist[row].access_min(true);
    }
}

fn spent(used: &[f64], bands: &[Vec<(f64, f64)>]) -> f64 {
    -used
        .iter()
        .zip(bands)
        .map(|(used, bands)| gold(*used, bands))
        .sum::<f64>()
}

/// Gold spent under `plan` in each of `n` runs, sorted. Free tap outcomes and normal taps are
/// stratified; advanced honing is played out.
pub fn gold_samples<R: Rng>(sb: &StateBundle, plan: usize, n: usize, rng: &mut R) -> Vec<f64> {
    let num_mats = sb.prep_output.table.len();
    let mut used: Vec<f64> = vec![0.0; n * num_mats];
    let skips = tap_map_generator(n, sb.special_probs(), rng);
    for (position, &u_index) in sb.special_state.iter().enumerate() {
        let upgrade = &sb.upgrade_arr[u_index];
        let rows = live_rows(upgrade);
        let (taps, runs) = if upgrade.is_normal_honing {
            (tap_map_generator(n, &upgrade.normal_dist, rng), Vec::new())
        } else {
            (Vec::new(), adv_runs(&upgrade.adv_config, n, rng))
        };
        for (sample, row) in used.chunks_mut(num_mats).enumerate() {
            if position < skips[sample] {
                add_skipped(upgrade, &rows, row);
            } else if upgrade.is_normal_honing {
                add_outcome(upgrade, &rows, (taps[sample], 0, 0), row);
            } else {
                let (taps, juice, scroll) = runs[sample];
                add_outcome(
                    upgrade,
                    &rows,
                    (taps as usize, juice as usize, scroll as usize),
                    row,
                );
            }
        }
    }
    let bands = &sb.prep_output.bands[plan];
    let mut out: Vec<f64> = used.chunks(num_mats).map(|row| spent(row, bands)).collect();
    out.sort_by(f64::total_cmp);
    out
}

fn end(dist: &[f64], high: bool) -> usize {
    let live = |i: &usize| dist[*i] >= IGNORE_PROB_TOL;
    if high {
        (0..dist.len()).rev().find(live)
    } else {
        (0..dist.len()).find(live)
    }
    .unwrap()
}

/// Every upgrade at one end of its outcomes, with the most free taps at the low end and the
/// fewest at the high: (gold spent, log10 chance of exactly that run). Advanced honing's juice and
/// scroll are taken at the same end as its taps.
fn extreme(sb: &StateBundle, plan: usize, high: bool) -> (f64, f64) {
    let probs = sb.special_probs();
    let mut live = (0..probs.len()).filter(|&k| probs[k] >= SPECIAL_TOL);
    let skip_count = if high { live.next() } else { live.last() }.unwrap();
    let mut odds = probs[skip_count].log10();
    let mut used: Vec<f64> = vec![0.0; sb.prep_output.table.len()];
    for (position, &u_index) in sb.special_state.iter().enumerate() {
        let upgrade = &sb.upgrade_arr[u_index];
        let rows = live_rows(upgrade);
        if position < skip_count {
            add_skipped(upgrade, &rows, &mut used);
            continue;
        }
        let dists: Vec<&[f64]> = if upgrade.is_normal_honing {
            vec![&upgrade.normal_dist[..]; 3]
        } else {
            upgrade.adv_dists.iter().map(|dist| &dist[..]).collect()
        };
        let ends: Vec<usize> = dists.iter().map(|dist| end(dist, high)).collect();
        odds += dists[0][ends[0]].log10();
        add_outcome(upgrade, &rows, (ends[0], ends[1], ends[2]), &mut used);
    }
    (spent(&used, &sb.prep_output.bands[plan]), odds)
}

/// Needs the free tap probabilities computed, as the histogram does. Seeded, so the same state
/// draws the same curve.
pub fn gold_distribution(sb: &StateBundle, plan: usize) -> GoldDist {
    let mut rng = SmallRng::seed_from_u64(0);
    let samples = gold_samples(sb, plan, GOLD_SAMPLES, &mut rng);
    let n = samples.len() as f64;
    let at_most = |x: f64| samples.partition_point(|&s| s <= x + float_tol(x)) as f64 / n;
    let at_least = |x: f64| 1.0 - samples.partition_point(|&s| s < x - float_tol(x)) as f64 / n;

    let (low, low_odds) = extreme(sb, plan, false);
    let (high, high_odds) = extreme(sb, plan, true);
    // a whole region can spend the same as an end, which the sampling sees and the one run does not
    let luckiest = (low, low_odds.max(at_most(low).log10()));
    let pity = (high, high_odds.max(at_least(high).log10()));

    let mut cdf: Vec<(f64, f64)> = Vec::with_capacity(BUCKET_COUNT + 1);
    for index in 0..=BUCKET_COUNT {
        let x = low + index as f64 * (high - low) / BUCKET_COUNT as f64;
        let chance = if index == 0 {
            10f64.powf(luckiest.1)
        } else if index == BUCKET_COUNT {
            1.0
        } else {
            at_most(x).max(10f64.powf(luckiest.1))
        };
        cdf.push((x, chance));
    }
    GoldDist {
        cdf,
        luckiest,
        pity,
    }
}
