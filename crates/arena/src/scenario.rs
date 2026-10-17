//! A scenario is a chain of fully specified payloads, one per optimizer run.

use ahash::AHashMap;
use hf_core::chest::Chest;
use hf_core::constants::BASE_JUICE_INFOS;
use hf_core::materials::{BandValue, NUM_BASE_MATS, OneMaterial, ValuationPlan};
use hf_core::parser::OneUpgradeInput;
use hf_core::payload::Payload;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct Scenario {
    #[serde(default)]
    pub name: String,
    pub steps: Vec<Payload>,
    /// what a generated scenario was made from
    #[serde(default)]
    pub meta: serde_json::Map<String, serde_json::Value>,
}

/// every .json in the folder whose name contains the filter
pub fn load(dir: &Path, filter: &str) -> Vec<Scenario> {
    let mut paths: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let mut scenario: Scenario =
                serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
            scenario.name = path.file_stem().unwrap().to_string_lossy().to_string();
            scenario
        })
        .filter(|scenario| scenario.name.contains(filter))
        .collect()
}

/// every scenario of the folder and of the folders in it, by name
pub fn all(dir: &Path) -> BTreeMap<String, Scenario> {
    let mut out: BTreeMap<String, Scenario> = BTreeMap::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(all(&path));
        }
    }
    out.extend(
        load(dir, "")
            .into_iter()
            .map(|scenario| (scenario.name.clone(), scenario)),
    );
    out
}

/// A set is a list of scenario names, one a line; `name:3` is only its first 3 steps.
pub fn load_set(dir: &Path, sets: &Path, set: &str) -> Vec<Scenario> {
    let known = all(dir);
    fs::read_to_string(sets.join(format!("{set}.txt")))
        .unwrap_or_else(|_| panic!("no set {set}"))
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (name, steps) = line.split_once(':').unwrap_or((line, ""));
            let mut scenario = known
                .get(name)
                .unwrap_or_else(|| panic!("no scenario {name}"))
                .clone();
            if let Ok(steps) = steps.parse() {
                scenario.steps.truncate(steps);
            }
            scenario
        })
        .collect()
}

pub fn upgrade(piece_index: usize, upgrade_index: usize, adv: bool) -> OneUpgradeInput {
    OneUpgradeInput {
        piece_index,
        upgrade_index,
        is_normal_honing: !adv,
        starting_artisan: Some(0.0),
        starting_num_taps: Some(0),
        state: None,
        unlocked: false,
        adv_progress: adv.then_some((0, 0, false, false)),
        double_balls: false,
    }
}

fn chest(count: i64, band: usize, options: &[&[(&str, f64)]]) -> Chest {
    Chest {
        count,
        band,
        options: options
            .iter()
            .map(|option| {
                option
                    .iter()
                    .map(|(label, amount)| (label.to_string(), *amount))
                    .collect()
            })
            .collect(),
    }
}

/// A placeholder chain: the same character after spending some materials and finishing
/// two upgrades, then after a price move.
pub fn dummy() -> Scenario {
    const BASE: [(&str, f64, f64); NUM_BASE_MATS] = [
        ("Red", 5.6, 30000.0),
        ("Blue", 0.12, 60000.0),
        ("Leapstones", 22.0, 900.0),
        ("Shards", 0.22, 400000.0),
        ("Fusion", 172.0, 300.0),
        ("Gold", 1.0, 0.0),
        ("Silver", 0.0, 0.0),
    ];
    let mut material_labels: Vec<String> = Vec::new();
    let mut materials: AHashMap<String, OneMaterial> = AHashMap::new();
    let mut add = |label: String, price: f64, owned: f64| {
        materials.insert(
            label.clone(),
            OneMaterial {
                bound: owned,
                roster: (owned / 4.0).floor(),
                tradable: (owned / 8.0).floor(),
                taxed_price: (price * 0.95 * 100.0).floor() / 100.0,
                market_price: price,
            },
        );
        material_labels.push(label);
    };
    for (label, price, owned) in BASE {
        add(label.to_string(), price, owned);
    }
    for id in 0..BASE_JUICE_INFOS[0].num_juice_avail {
        add(
            format!("Juice {id}"),
            300.0,
            if id < 2 { 200.0 } else { 0.0 },
        );
    }

    let mut upgrade_info: Vec<OneUpgradeInput> = Vec::new();
    for piece_index in 0..3 {
        for upgrade_index in 14..17 {
            upgrade_info.push(upgrade(piece_index, upgrade_index, false));
        }
    }
    upgrade_info.push(upgrade(5, 14, false));
    upgrade_info.push(upgrade(5, 15, false));
    upgrade_info.push(upgrade(0, 2, true));

    let first = Payload {
        material_labels,
        materials,
        plans: vec![
            ValuationPlan([BandValue::Worthless; 3]),
            ValuationPlan([
                BandValue::Worthless,
                BandValue::TaxedSell,
                BandValue::Market,
            ]),
        ],
        optimizer_plan: 1,
        upgrade_info,
        special_budget: 600,
        special_state: None,
        // a choice, an obtain-all, fractional amounts as a conversion leaves them, and free taps
        chests: [
            chest(
                120,
                1,
                &[
                    &[("Red", 500.0)],
                    &[("Blue", 1500.0)],
                    &[("Leapstones", 20.0), ("Shards", 3000.0)],
                ],
            ),
            chest(7, 0, &[&[("Fusion", 30.0), ("Gold", 5000.0)]]),
            chest(33, 2, &[&[("Juice 0", 2.5)], &[("Juice 1", 2.5)]]),
            chest(40, 0, &[&[("Special Leap", 12.0)], &[("Leapstones", 6.4)]]),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, chest)| (index.to_string(), chest))
        .collect(),
        chest_states: None,
        special_label: "Special Leap".to_string(),
        tier: 0,
        express_event: false,
        min_resolution: 1,
        num_threads: 0,
        metric_type: 1,
        adv_cache: None,
    };

    let mut second = first.clone();
    second.upgrade_info.drain(0..2);
    second.special_budget = 250;
    for mat in second.materials.values_mut() {
        mat.bound = (mat.bound * 0.7).floor();
    }

    let mut third = second.clone();
    for label in ["Red", "Leapstones", "Fusion"] {
        let mat = third.materials.get_mut(label).unwrap();
        mat.market_price *= 1.25;
        mat.taxed_price *= 1.25;
    }

    Scenario {
        name: "dummy".to_string(),
        steps: vec![first, second, third],
        meta: Default::default(),
    }
}
