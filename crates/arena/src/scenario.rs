//! A scenario is a chain of fully specified payloads, one per optimizer run.

use ahash::AHashMap;
use hf_core::constants::BASE_JUICE_INFOS;
use hf_core::materials::{BandValue, NUM_BASE_MATS, OneMaterial, ValuationPlan};
use hf_core::parser::OneUpgradeInput;
use hf_core::payload::Payload;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct Scenario {
    #[serde(default)]
    pub name: String,
    pub steps: Vec<Payload>,
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

fn upgrade(piece_index: usize, upgrade_index: usize, adv: bool) -> OneUpgradeInput {
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
        add(format!("Juice {id}"), 300.0, if id < 2 { 200.0 } else { 0.0 });
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
            ValuationPlan([BandValue::Worthless, BandValue::TaxedSell, BandValue::Market]),
        ],
        optimizer_plan: 1,
        upgrade_info,
        special_budget: 600,
        special_state: None,
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
    }
}
