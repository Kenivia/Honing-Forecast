//! Generated scenarios. A point on a few axes that decide what a case is like becomes a
//! payload, and a chain replays a player following the optimizer one update at a time.
//! See `AI notes/Calculator core/Arena.md`.

use crate::one_adv_sim::one_sim;
use crate::scenario::{Scenario, upgrade};
use ahash::AHashMap;
use hf_core::chest::Chest;
use hf_core::constants::ARTISAN_MULTIPLIER;
use hf_core::materials::{BandValue, NUM_BASE_MATS, OneMaterial, ValuationPlan};
use hf_core::optimizer::solve;
use hf_core::payload::Payload;
use hf_core::probe::Probe;
use hf_core::state_bundle::StateBundle;
use hf_core::upgrade::Upgrade;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use rand_distr::StandardNormal;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub const AXES: [&str; 16] = [
    "tier", "size", "level", "pieces", "jitter", "adv", "progress", "wealth", "imbalance", "split",
    "juice_owned", "juice_price", "free_taps", "plan", "chests", "price_noise",
];

pub fn axis(name: &str) -> usize {
    AXES.iter().position(|x| *x == name).unwrap()
}

/// label, tiers, gold per unit. The frontend's table order and its fallback prices.
const MATS: [(&str, &[usize], f64); 29] = [
    ("Red", &[0], 6.47),
    ("Serca Red", &[1], 34.94),
    ("Blue", &[0], 0.1),
    ("Serca Blue", &[1], 1.96),
    ("Leaps", &[0], 20.0),
    ("Serca Leaps", &[1], 156.0),
    ("Shards", &[0, 1], 0.2),
    ("Fusion", &[0], 180.0),
    ("Serca Fusion", &[1], 226.0),
    ("Gold", &[0, 1], 1.0),
    ("Silver", &[0, 1], 0.0),
    ("Glacier's Breath", &[0, 1], 260.0),
    ("Lava's Breath", &[0, 1], 430.0),
    ("11-14 Armor", &[0], 298.0),
    ("11-14 Weapon", &[0], 737.0),
    ("15-18 Armor", &[0], 19.0),
    ("15-18 Weapon", &[0], 119.0),
    ("19-20 Armor", &[0], 2748.0),
    ("19-20 Weapon", &[0], 3890.0),
    ("Scroll 1 Armor", &[0], 150.0),
    ("Scroll 1 Weapon", &[0], 496.0),
    ("Scroll 2 Armor", &[0], 70.0),
    ("Scroll 2 Weapon", &[0], 50.0),
    ("Scroll 3 Armor", &[0], 1800.0),
    ("Scroll 3 Weapon", &[0], 1933.0),
    ("Scroll 4 Armor", &[0], 3187.0),
    ("Scroll 4 Weapon", &[0], 2369.0),
    ("Enhanced 19-20 Armor", &[0], 15000.0),
    ("Enhanced 19-20 Weapon", &[0], 15000.0),
];
const SPECIAL_LABELS: [&str; 2] = ["Special Leap", "Serca Special Leap"];
const GOLD: usize = 5;

const W: BandValue = BandValue::Worthless;
const T: BandValue = BandValue::TaxedSell;
const M: BandValue = BandValue::Market;

fn pick<X: Copy>(u: f64, options: &[(f64, X)]) -> X {
    let mut cumulative: f64 = 0.0;
    for (weight, option) in options {
        cumulative += weight;
        if u < cumulative {
            return *option;
        }
    }
    options.last().unwrap().1
}

fn log_uniform(u: f64, low: f64, high: f64) -> f64 {
    low * (high / low).powf(u)
}

fn normal<R: Rng>(rng: &mut R) -> f64 {
    rng.sample(StandardNormal)
}

fn apply_tax(price: f64) -> f64 {
    price.min(1.0).max((price * 0.95).floor())
}

fn set_price(mat: &mut OneMaterial, price: f64) {
    mat.market_price = (price * 100.0).round().max(1.0) / 100.0;
    mat.taxed_price = apply_tax(mat.market_price);
}

fn evaluated(payload: &Payload) -> StateBundle {
    let mut state_bundle = StateBundle::init_from_payload(payload.clone());
    state_bundle.metric = state_bundle.metric_router(&mut Probe::default());
    state_bundle.set_latest_special_probs();
    state_bundle
}

/// where a normal upgrade stands after this many more failed taps on its state
fn after_failed(upgrade: &Upgrade, state_bundle: &StateBundle, taps: usize) -> (f64, usize) {
    let juice_info = &state_bundle.prep_output.juice_info;
    let mut artisan = upgrade.starting_artisan;
    for count in 0..taps {
        let juiced: f64 = upgrade
            .state
            .get(count)
            .into_iter()
            .flatten()
            .map(|&id| juice_info.access(id, upgrade.piece_type_usize, upgrade.upgrade_index).normal_chance)
            .sum();
        let ramp = (count + upgrade.starting_num_taps).min(10) as f64 * upgrade.base_chance * 0.1;
        let chance = (upgrade.base_chance + upgrade.extra_chance + ramp + juiced).min(1.0);
        artisan += ARTISAN_MULTIPLIER * chance * upgrade.artisan_rate;
    }
    (artisan.min(1.0), (upgrade.starting_num_taps + taps).min(10))
}

fn mean_taps(upgrade: &Upgrade) -> f64 {
    let dist = if upgrade.is_normal_honing { &upgrade.normal_dist } else { &upgrade.adv_dists[0] };
    dist.iter().enumerate().map(|(taps, p)| taps as f64 * p).sum()
}

/// the amount this much of the time is enough, under the given state
fn quantile(state_bundle: &StateBundle, row: usize, q: f64, low: f64, high: f64) -> f64 {
    let (mut low, mut high) = (low, high);
    for _ in 0..40 {
        let mid = (low + high) / 2.0;
        if state_bundle.one_dimension_prob(row as i64, mid, &mut Probe::default()) < q {
            low = mid;
        } else {
            high = mid;
        }
    }
    high
}

/// Typical amounts of a case, which chests and income are sized by.
#[derive(Clone)]
pub struct Refs {
    used: AHashMap<String, f64>,
    special: f64,
}

fn chest_stack<R: Rng>(rng: &mut R, options: &[Vec<&str>], refs: &Refs, special_label: &str) -> (String, Chest) {
    let count = log_uniform(rng.random(), 1.0, 200.0) as i64;
    let share = log_uniform(rng.random(), 0.05, 1.0);
    let band = pick(rng.random(), &[(0.5, 0), (0.3, 1), (0.2, 2)]);
    let options: Vec<BTreeMap<String, f64>> = options
        .iter()
        .map(|option| {
            option
                .iter()
                .map(|&label| {
                    let typical = if label == special_label { refs.special } else { refs.used[label] };
                    let each = typical.max(50.0) * share / count as f64;
                    let each = if each >= 10.0 { each.round() } else { ((each * 10.0).round() / 10.0).max(0.2) };
                    (label.to_string(), each)
                })
                .collect()
        })
        .collect();
    let key = format!(
        "{band}:{}:{}",
        options.iter().map(|option| option.keys().cloned().collect::<Vec<_>>().join("+")).collect::<Vec<_>>().join("|"),
        rng.random_range(0..1000)
    );
    (key, Chest { count, band, options })
}

/// kind 1 has no choice in it, 2 chooses between materials, 3 also between those and free taps
fn chest_templates<'a>(labels: &'a [String], special_label: &'a str, kind: usize) -> Vec<Vec<Vec<&'a str>>> {
    let l = |row: usize| labels[row].as_str();
    let fixed = vec![vec![vec![l(0), l(1), l(2), l(3), l(4)]], vec![vec![l(GOLD), l(4)]]];
    let choice = vec![vec![vec![l(0)], vec![l(1)]], vec![vec![l(2)], vec![l(3)]], vec![vec![l(7)], vec![l(8)]]];
    let special = vec![
        vec![vec![special_label], vec![l(GOLD)], vec![l(8)], vec![l(4)]],
        vec![vec![special_label], vec![l(2)]],
    ];
    match kind {
        1 => fixed,
        2 => choice,
        _ => special,
    }
}

pub struct Built {
    pub payload: Payload,
    pub refs: Refs,
}

/// `full` is every upgrade a character has, whatever the size axis says
pub fn build(a: &[f64], seed: u64, full: bool) -> Built {
    let mut rng = StdRng::seed_from_u64(seed);
    let (tier, express) = pick(a[axis("tier")], &[(0.45, (0, false)), (0.2, (0, true)), (0.35, (1, false))]);
    let labels: Vec<String> =
        MATS.iter().filter(|mat| mat.1.contains(&tier)).map(|mat| mat.0.to_string()).collect();
    let special_label = SPECIAL_LABELS[tier];

    // which upgrades
    let armour = rng.random_range(1..=5);
    let pieces: Vec<usize> = match pick(a[axis("pieces")], &[(0.2, 0), (0.25, 1), (0.55, 2)]) {
        _ if full => (0..6 + tier).collect(),
        0 => vec![5],
        1 => (0..armour).collect(),
        _ => {
            let mut out: Vec<usize> = (0..armour).collect();
            out.push(5);
            if tier == 1 && rng.random_bool(0.5) {
                out.push(6);
            }
            out
        }
    };
    let adv_pieces: Vec<usize> = pieces.iter().copied().filter(|piece| tier == 0 && *piece < 6).collect();
    let (per_piece, adv_per_piece): (Vec<usize>, Vec<usize>) = if full {
        (vec![25; pieces.len()], vec![4; adv_pieces.len()])
    } else {
        let n = log_uniform(a[axis("size")], 1.0, 41.0) as usize;
        let adv_share = pick(a[axis("adv")], &[(0.5, 0.0), (0.2, 0.25), (0.2, 0.6), (0.1, 1.0)]);
        let n_adv = ((n as f64 * adv_share).round() as usize).min(4 * adv_pieces.len());
        let n_normal = (n - n_adv).max((n_adv == 0) as usize);
        let spread = |total: usize, over: usize, cap: usize| -> Vec<usize> {
            (0..over).map(|slot| (total / over + (slot < total % over) as usize).min(cap)).collect()
        };
        (spread(n_normal, pieces.len(), 25), spread(n_adv, adv_pieces.len().max(1), 4))
    };
    let jitter = 4.0 * a[axis("jitter")].powi(2);
    let mut upgrade_info = Vec::new();
    // the lowest upgrade of each piece, the only one that can have been started
    let mut firsts: Vec<usize> = Vec::new();
    for (&piece, &count) in pieces.iter().zip(&per_piece).filter(|(_, count)| **count > 0) {
        let room = (25 - count) as f64;
        let start = (a[axis("level")] * room + jitter * normal(&mut rng)).round().clamp(0.0, room) as usize;
        firsts.push(upgrade_info.len());
        upgrade_info.extend((start..start + count).map(|level| upgrade(piece, level, false)));
    }
    for (&piece, &count) in adv_pieces.iter().zip(&adv_per_piece).filter(|(_, count)| **count > 0) {
        let start = (a[axis("level")] * (4 - count) as f64).round() as usize;
        firsts.push(upgrade_info.len());
        upgrade_info.extend((start..start + count).map(|block| {
            let mut out = upgrade(piece, block, true);
            out.double_balls = express && block < 2;
            out
        }));
    }

    let mut payload = Payload {
        materials: labels
            .iter()
            .map(|label| {
                (label.clone(), OneMaterial { bound: 0.0, roster: 0.0, tradable: 0.0, taxed_price: 0.0, market_price: 0.0 })
            })
            .collect(),
        material_labels: labels.clone(),
        plans: vec![
            ValuationPlan([W; 3]),
            ValuationPlan(pick(
                a[axis("plan")],
                &[
                    (0.2, [W, W, W]),
                    (0.3, [W, T, M]),
                    (0.1, [W, W, T]),
                    (0.1, [W, W, M]),
                    (0.1, [W, T, T]),
                    (0.05, [W, M, M]),
                    (0.05, [T, T, M]),
                    (0.05, [T, T, T]),
                    (0.05, [M, M, M]),
                ],
            )),
        ],
        optimizer_plan: 1,
        upgrade_info,
        special_budget: 0,
        special_state: None,
        chests: Default::default(),
        chest_states: None,
        special_label: special_label.to_string(),
        tier,
        express_event: express,
        min_resolution: 1,
        num_threads: 0,
        metric_type: 1,
        adv_cache: None,
    };

    // taps already done
    let fresh = evaluated(&payload);
    let started: Vec<usize> = match pick(a[axis("progress")], &[(0.6, 0), (0.25, 1), (0.15, 2)]) {
        _ if full => Vec::new(),
        0 => Vec::new(),
        1 => vec![*firsts.choose(&mut rng).unwrap()],
        _ => firsts.clone(),
    };
    for index in started {
        let this = &fresh.upgrade_arr[index];
        let info = &mut payload.upgrade_info[index];
        info.unlocked = true;
        if !this.is_normal_honing {
            info.adv_progress = Some((rng.random_range(1..90), rng.random_range(0..6), false, false));
            continue;
        }
        let pity = this.normal_dist.len() - 1;
        if pity < 2 {
            info.unlocked = false;
            continue;
        }
        let taps = 1 + (rng.random::<f64>() * 0.7 * (pity - 1) as f64) as usize;
        let (artisan, num_taps) = after_failed(this, &fresh, taps);
        info.starting_artisan = Some(artisan);
        info.starting_num_taps = Some(num_taps);
    }

    // free taps, as a share of what it takes on average to free tap every upgrade
    let special_scale: f64 = fresh
        .upgrade_arr
        .iter()
        .filter(|this| this.is_normal_honing)
        .map(|this| this.special_cost as f64 / (this.base_chance + this.extra_chance))
        .sum();
    let free = a[axis("free_taps")];
    if free >= 0.25 {
        payload.special_budget = (special_scale * log_uniform((free - 0.25) / 0.75, 0.01, 1.5)).round() as i64;
    }

    // what is owned: a quantile of what the default state uses
    let mut state_bundle = evaluated(&payload);
    let (_, avg_used, _) = state_bundle.ui_average_gold_metric(&mut Probe::default());
    let (lowest, highest) = (state_bundle.luckiest_mf(), state_bundle.pity());
    let mut typical: Vec<f64> = avg_used.clone();
    for this in &state_bundle.upgrade_arr {
        let juice_info = &state_bundle.prep_output.juice_info;
        let (ids, taps) = (
            if this.is_normal_honing { &juice_info.normal_uindex_to_id } else { &juice_info.adv_uindex_to_id },
            mean_taps(this),
        );
        for &id in &ids[this.piece_type_usize][this.upgrade_index] {
            let one = juice_info.access(id, this.piece_type_usize, this.upgrade_index);
            let amount = if this.is_normal_honing { one.normal_amt_used } else { one.adv_amt_used };
            typical[NUM_BASE_MATS + id] += amount as f64 * taps;
        }
    }

    let wealth = a[axis("wealth")];
    let centre = 0.02 + 0.96 * ((wealth - 0.08) / 0.84).clamp(0.0, 1.0);
    let centre = (centre / (1.0 - centre)).ln();
    let imbalance = 4.0 * a[axis("imbalance")].powi(2);
    let juice_owned = a[axis("juice_owned")];
    let split = pick(
        a[axis("split")],
        &[
            (0.3, [1.0, 0.0, 0.0]),
            (0.2, [0.6, 0.4, 0.0]),
            (0.25, [0.5, 0.25, 0.25]),
            (0.15, [0.15, 0.25, 0.6]),
            (0.1, [0.0, 0.0, 1.0]),
        ],
    );
    let market = (0.3 * normal(&mut rng)).exp();
    let juice_market = log_uniform(a[axis("juice_price")], 0.2, 5.0);
    for (row, label) in labels.iter().enumerate() {
        let (low, high) = (lowest[row] as f64, highest[row] as f64);
        let total = if row >= NUM_BASE_MATS {
            let noise = (0.5 * normal(&mut rng)).exp();
            if juice_owned < 0.3 { 0.0 } else { typical[row] * noise * log_uniform((juice_owned - 0.3) / 0.7, 0.05, 3.0) }
        } else if wealth < 0.08 || high == 0.0 {
            0.0
        } else if wealth > 0.92 {
            high * 1.2
        } else {
            let q = 1.0 / (1.0 + (-(centre + imbalance * normal(&mut rng))).exp());
            if q < 0.002 {
                low * rng.random::<f64>()
            } else if q > 0.998 {
                high * (1.0 + rng.random::<f64>())
            } else {
                quantile(&state_bundle, row, q, low, high)
            }
        };
        let mut shares = split.map(|share| share * (0.5 * normal(&mut rng)).exp());
        if row == GOLD {
            // gold is never sold
            shares = [1.0, 0.0, 0.0];
        }
        let sum: f64 = shares.iter().sum();
        let mat = payload.materials.get_mut(label).unwrap();
        mat.bound = (total * shares[0] / sum).floor();
        mat.roster = (total * shares[1] / sum).floor();
        mat.tradable = (total * shares[2] / sum).floor();

        let base = MATS.iter().find(|mat| mat.0 == label).unwrap().2;
        let noise = (a[axis("price_noise")] * normal(&mut rng)).exp();
        if row == GOLD {
            set_price(mat, 1.0);
        } else if base > 0.0 {
            set_price(mat, base * market * noise * if row >= NUM_BASE_MATS { juice_market } else { 1.0 });
        }
    }

    let refs = Refs {
        used: labels.iter().cloned().zip(typical).collect(),
        special: special_scale * 0.3,
    };
    let chest_kind = pick(a[axis("chests")], &[(0.45, 0), (0.15, 1), (0.25, 2), (0.15, 3)]);
    if chest_kind > 0 {
        let mut stacks = chest_templates(&labels, special_label, chest_kind);
        if chest_kind == 3 {
            stacks.extend(chest_templates(&labels, special_label, 2));
        }
        let keep = rng.random_range(1..=stacks.len());
        for options in stacks.choose_multiple(&mut rng, keep) {
            let (key, chest) = chest_stack(&mut rng, options, &refs, special_label);
            payload.chests.insert(key, chest);
        }
    }
    Built { payload, refs }
}

fn spend(mat: &mut OneMaterial, mut amount: f64) {
    for band in [&mut mat.bound, &mut mat.roster, &mut mat.tradable] {
        let take = amount.min(*band);
        *band -= take;
        amount -= take;
    }
}

/// The upgrade to do next for the free taps to go where the plan wants them: the lowest level
/// still to do on the piece its free tap order starts with.
pub fn next_up(best: &StateBundle) -> usize {
    let top = &best.upgrade_arr[best.special_state[0]];
    (0..best.upgrade_arr.len())
        .filter(|&index| {
            let this = &best.upgrade_arr[index];
            this.piece_index == top.piece_index && this.is_normal_honing == top.is_normal_honing
        })
        .min_by_key(|&index| best.upgrade_arr[index].upgrade_index)
        .unwrap()
}

/// what a normal upgrade uses over this many paid taps on its state, the unlock included
pub fn normal_used(best: &StateBundle, index: usize, taps: usize) -> Vec<f64> {
    let this = &best.upgrade_arr[index];
    let prep = &best.prep_output;
    let mut used: Vec<f64> = vec![0.0; prep.table.len()];
    if !this.unlocked {
        used[..NUM_BASE_MATS].copy_from_slice(&this.unlock_costs);
    }
    for row in 0..NUM_BASE_MATS {
        used[row] += this.costs[row] * taps as f64;
    }
    // the pity tap takes no juice
    for count in (0..taps).filter(|count| count + 2 < this.normal_dist.len()) {
        for &id in this.state.get(count).into_iter().flatten() {
            used[prep.table.juice_row(id)] +=
                prep.juice_info.access(id, this.piece_type_usize, this.upgrade_index).normal_amt_used as f64;
        }
    }
    used
}

/// what one run of an advanced upgrade uses
fn adv_used<R: Rng>(best: &StateBundle, index: usize, rng: &mut R) -> Vec<f64> {
    let this = &best.upgrade_arr[index];
    let prep = &best.prep_output;
    let juice_info = &prep.juice_info;
    let mut used: Vec<f64> = vec![0.0; prep.table.len()];
    if this.adv_config.start_xp == 0 {
        used[..NUM_BASE_MATS].copy_from_slice(&this.unlock_costs);
    }
    let (taps, juice, scroll) = one_sim(rng, &this.adv_config);
    for row in 0..NUM_BASE_MATS {
        used[row] += this.costs[row] * taps as f64;
    }
    for &id in &juice_info.adv_uindex_to_id[this.piece_type_usize][this.upgrade_index] {
        let amount = juice_info.access(id, this.piece_type_usize, this.upgrade_index).adv_amt_used;
        let times = if id <= 1 { juice } else { scroll };
        used[prep.table.juice_row(id)] += (amount * times as i64) as f64;
    }
    used
}

pub fn pay(payload: &mut Payload, best: &StateBundle, used: &[f64]) {
    for (label, amount) in best.prep_output.table.labels.iter().zip(used) {
        spend(payload.materials.get_mut(label).unwrap(), *amount);
    }
}

/// The player does this upgrade, and reports how it went: done, free tapped, or with
/// `partials` sometimes only some failed taps in. Follows the rules `monte_carlo::simulate`
/// checks the evaluator against.
pub fn play<R: Rng>(
    payload: &mut Payload,
    best: &StateBundle,
    index: usize,
    partials: bool,
    rng: &mut R,
) -> &'static str {
    let this = &best.upgrade_arr[index];
    let mut kind = "done";

    let used = if !this.is_normal_honing {
        adv_used(best, index, rng)
    } else {
        let affordable = payload.special_budget / this.special_cost;
        let mut free_tapped = false;
        if index == best.special_state[0] && affordable > 0 {
            let tries = (rng.random::<f64>().log(1.0 - this.base_chance).ceil() as i64).max(1);
            free_tapped = tries <= affordable;
            payload.special_budget -= tries.min(affordable) * this.special_cost;
        }
        if free_tapped {
            kind = "free";
            normal_used(best, index, 0)
        } else {
            let succeeds_on = WeightedIndex::new(this.normal_dist.iter()).unwrap().sample(rng);
            let taps = if partials && succeeds_on > 1 && rng.random_bool(0.25) {
                kind = "partial";
                rng.random_range(1..succeeds_on)
            } else {
                succeeds_on
            };
            if kind == "partial" {
                let (artisan, num_taps) = after_failed(this, best, taps);
                let info = &mut payload.upgrade_info[index];
                info.starting_artisan = Some(artisan);
                info.starting_num_taps = Some(num_taps);
                info.unlocked = true;
            }
            normal_used(best, index, taps)
        }
    };

    pay(payload, best, &used);
    if kind != "partial" {
        payload.upgrade_info.remove(index);
    }
    kind
}

/// the player solves the payload and plays the result
fn tap<R: Rng>(payload: &mut Payload, rng: &mut R) -> &'static str {
    let mut state_bundle = StateBundle::init_from_payload(payload.clone());
    state_bundle.tuning.max_iters = 6000;
    // where the optimizer panics the player follows the default state, so the case is kept
    let solved = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        solve(rng, state_bundle, None, &mut Probe::default()).best
    }));
    let mut best = solved.unwrap_or_else(|_| StateBundle::init_from_payload(payload.clone()));
    best.metric = best.metric_router(&mut Probe::default());
    play(payload, &best, next_up(&best), true, rng)
}

fn other_change<R: Rng>(payload: &mut Payload, refs: &Refs, kind: &str, rng: &mut R) {
    match kind {
        "income" => {
            let rate = rng.random_range(0.05..0.25);
            for label in &payload.material_labels {
                payload.materials.get_mut(label).unwrap().bound +=
                    (refs.used[label] * rate * (0.3 * normal(rng)).exp()).floor();
            }
            payload.special_budget += (refs.special * rate).round() as i64;
        }
        "price" => {
            let market = (0.05 * normal(rng)).exp();
            for label in &payload.material_labels {
                let mat = payload.materials.get_mut(label).unwrap();
                if label != "Gold" && mat.market_price > 0.0 {
                    set_price(mat, mat.market_price * market * (0.08 * normal(rng)).exp());
                }
            }
        }
        "goal" => {
            let normals: Vec<(usize, usize)> = payload
                .upgrade_info
                .iter()
                .filter(|info| info.is_normal_honing)
                .map(|info| (info.piece_index, info.upgrade_index))
                .collect();
            let piece = normals.choose(rng).map_or(5, |x| x.0);
            let top = normals.iter().filter(|x| x.0 == piece).map(|x| x.1).max();
            match top {
                Some(top) if top < 24 && rng.random_bool(0.6) => payload.upgrade_info.push(upgrade(piece, top + 1, false)),
                Some(top) if payload.upgrade_info.len() > 1 => payload
                    .upgrade_info
                    .retain(|info| !(info.is_normal_honing && info.piece_index == piece && info.upgrade_index == top)),
                _ => payload.upgrade_info.push(upgrade(piece, 10, false)),
            }
        }
        _ => {
            let keys: Vec<String> = payload.chests.keys().cloned().collect();
            if let Some(key) = keys.choose(rng).filter(|_| rng.random_bool(0.7)) {
                let chest = payload.chests.get_mut(key).unwrap();
                chest.count = (chest.count + rng.random_range(-chest.count / 2..=chest.count / 2 + 5)).max(1);
            } else {
                let (labels, special_label) = (payload.material_labels.clone(), payload.special_label.clone());
                let stacks = chest_templates(&labels, &special_label, rng.random_range(1..=3));
                let options = stacks.choose(rng).unwrap();
                let (key, chest) = chest_stack(rng, options, refs, &special_label);
                payload.chests.insert(key, chest);
            }
        }
    }
}

/// the steps, and what happened between each and the one before
pub fn chain(built: Built, steps: usize, seed: u64, chests: bool) -> (Vec<Payload>, Vec<String>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut out = vec![built.payload];
    let mut kinds = vec!["base".to_string()];
    while out.len() < steps && !out.last().unwrap().upgrade_info.is_empty() {
        let mut next = out.last().unwrap().clone();
        let kind = pick(
            rng.random(),
            &[(0.7, "tap"), (0.1, "income"), (0.08, "price"), (0.06, "goal"), (0.06, "chest")],
        );
        // disabled until the optimizer has chest moves
        let kind = if kind == "chest" && !chests { "income" } else { kind };
        let kind = if kind == "tap" {
            tap(&mut next, &mut rng)
        } else {
            other_change(&mut next, &built.refs, kind, &mut rng);
            kind
        };
        out.push(next);
        kinds.push(kind.to_string());
    }
    (out, kinds)
}

fn sobol(index: u32, seed: u32) -> Vec<f64> {
    (0..AXES.len() as u32).map(|dimension| sobol_burley::sample(index, dimension, seed) as f64).collect()
}

fn meta(a: &[f64], kinds: &[String]) -> Map<String, Value> {
    let axes: Map<String, Value> = AXES.iter().zip(a).map(|(name, value)| (name.to_string(), json!(value))).collect();
    let mut out = Map::new();
    out.insert("axes".to_string(), Value::Object(axes));
    out.insert("kinds".to_string(), json!(kinds));
    out
}

fn tag(payload: &Payload) -> String {
    format!(
        "{}_n{}",
        ["t4", "serca"][payload.tier],
        payload.upgrade_info.len()
    )
}

pub struct Spec {
    pub singles: u32,
    pub chains: u32,
    pub steps: usize,
    pub sweeps: bool,
    pub stress: bool,
    pub offset: u32,
    /// off until the optimizer has chest moves
    pub chests: bool,
}

/// One axis at a time around a few anchors, to see what each axis does on its own.
fn sweeps(chests: bool) -> Vec<(String, Vec<f64>, u64)> {
    const LEVELS: [(&str, &[f64]); 10] = [
        ("size", &[0.0, 0.3, 0.49, 0.67, 0.84, 0.999]),
        ("level", &[0.0, 0.25, 0.5, 0.75, 1.0]),
        ("wealth", &[0.0, 0.15, 0.3, 0.5, 0.7, 0.85, 1.0]),
        ("imbalance", &[0.0, 0.35, 0.7, 1.0]),
        ("split", &[0.15, 0.4, 0.6, 0.8, 0.95]),
        ("juice_owned", &[0.0, 0.4, 0.7, 1.0]),
        ("juice_price", &[0.0, 0.25, 0.5, 0.75, 1.0]),
        ("free_taps", &[0.0, 0.3, 0.55, 0.8, 1.0]),
        ("plan", &[0.1, 0.35, 0.55, 0.65, 0.75, 0.82, 0.87, 0.92, 0.97]),
        ("chests", &[0.2, 0.5, 0.7, 0.9]),
    ];
    let mut out = Vec::new();
    for (anchor, tier) in [0.1, 0.55, 0.8].into_iter().enumerate() {
        let mut base = vec![0.5; AXES.len()];
        base[axis("tier")] = tier;
        base[axis("size")] = 0.67;
        base[axis("adv")] = 0.6;
        base[axis("progress")] = 0.0;
        base[axis("chests")] = 0.0;
        for (name, levels) in LEVELS.into_iter().filter(|(name, _)| chests || *name != "chests") {
            for (level, value) in levels.iter().enumerate() {
                let mut a = base.clone();
                a[axis(name)] = *value;
                out.push((format!("sweep_{anchor}_{name}_{level}"), a, 1000 + anchor as u64));
            }
        }
    }
    out
}

pub fn generate(spec: &Spec) -> Vec<Scenario> {
    use rayon::prelude::*;
    let mut todo: Vec<Box<dyn Fn() -> Scenario + Send + Sync>> = Vec::new();
    let chests = spec.chests;
    let sobol = move |index: u32, seed: u32| -> Vec<f64> {
        let mut a = sobol(index, seed);
        if !chests {
            a[axis("chests")] = 0.0;
        }
        a
    };
    for index in spec.offset..spec.offset + spec.singles {
        todo.push(Box::new(move || {
            let a = sobol(index, 1);
            let payload = build(&a, index as u64, false).payload;
            Scenario { name: format!("single_{index:04}_{}", tag(&payload)), steps: vec![payload], meta: meta(&a, &[]) }
        }));
    }
    let steps = spec.steps;
    for index in spec.offset..spec.offset + spec.chains {
        todo.push(Box::new(move || {
            let mut a = sobol(index, 2);
            // a chain needs upgrades to get through
            a[axis("size")] = 0.37 + 0.63 * a[axis("size")];
            let built = build(&a, 1_000_000 + index as u64, false);
            let name = format!("chain_{index:04}_{}", tag(&built.payload));
            let (steps, kinds) = chain(built, steps, index as u64, chests);
            Scenario { name, steps, meta: meta(&a, &kinds) }
        }));
    }
    if spec.sweeps {
        for (name, a, seed) in sweeps(spec.chests) {
            todo.push(Box::new(move || {
                let payload = build(&a, seed, false).payload;
                Scenario { name: format!("{name}_{}", tag(&payload)), steps: vec![payload], meta: meta(&a, &[]) }
            }));
        }
    }
    if spec.stress {
        for (tier_name, tier) in [("t4", 0.1), ("serca", 0.8)] {
            for (wealth_name, wealth) in [("none", 0.0), ("mid", 0.5)] {
                todo.push(Box::new(move || {
                    let mut a = vec![0.5; AXES.len()];
                    a[axis("tier")] = tier;
                    a[axis("wealth")] = wealth;
                    let payload = build(&a, 7, true).payload;
                    Scenario {
                        name: format!("stress_{tier_name}_{wealth_name}_n{}", payload.upgrade_info.len()),
                        steps: vec![payload],
                        meta: meta(&a, &[]),
                    }
                }));
            }
        }
    }

    // a case the evaluator panics on is left out, and its panic is on stderr
    todo.par_iter()
        .filter_map(|make| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| make())).ok())
        .collect()
}
