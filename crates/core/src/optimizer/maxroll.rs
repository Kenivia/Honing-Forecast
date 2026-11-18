//! Maxroll's upgrade calculator's rule, as a comparison strategy: every material at its buy
//! price. Juice and books where they save more than they cost (the pools not asked for kept as
//! given), and free taps by gold saved per special leap. Normal honing only.

use super::response::Response;
use crate::materials::NUM_BASE_MATS;
use crate::state_bundle::StateBundle;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Maxroll {
    pub juice: bool,
    pub book: bool,
    pub special: bool,
}

pub fn apply_maxroll(state_bundle: &mut StateBundle, maxroll: Maxroll) {
    if !(maxroll.juice || maxroll.book || maxroll.special) {
        return;
    }
    // the site plans tap by tap, whatever the x10 setting
    let streak_length = std::mem::replace(&mut state_bundle.tuning.streak_length, 1);
    let prices: Vec<f64> = state_bundle
        .prep_output
        .materials
        .iter()
        .map(|material| material.market_price)
        .collect();
    // gold a free tap saves for each special leap it takes, by upgrade
    let mut worth = vec![f64::NEG_INFINITY; state_bundle.upgrade_arr.len()];
    for index in 0..state_bundle.upgrade_arr.len() {
        if !state_bundle.upgrade_arr[index].is_normal_honing {
            continue;
        }
        let response = Response::linear(state_bundle, index, &prices);
        let pools = response.model.pools.len();
        // the last pool is the books, the rest juices
        let free: Vec<usize> = (0..pools)
            .filter(|&pool| {
                if pool + 1 == pools {
                    maxroll.book
                } else {
                    maxroll.juice
                }
            })
            .collect();
        let upgrade = &state_bundle.upgrade_arr[index];
        let mut indices = response.model.indices(&upgrade.state.payload);
        if !free.is_empty() {
            indices = response.best_free(&indices, &free).1;
            let plan = response.model.plan(&indices);
            state_bundle.upgrade_arr[index].state.update_payload(plan);
        }
        let upgrade = &state_bundle.upgrade_arr[index];
        let unlock: f64 = if upgrade.unlocked {
            0.0
        } else {
            (0..NUM_BASE_MATS)
                .map(|row| prices[row] * upgrade.unlock_costs[row])
                .sum()
        };
        let remaining = -response.score(&indices) - unlock;
        worth[index] = upgrade.base_chance * remaining / upgrade.special_cost.max(1) as f64;
    }
    state_bundle.tuning.streak_length = streak_length;
    if maxroll.special {
        special_order(state_bundle, &worth);
    }
}

/// The best worth first, of each piece's lowest level left; advanced honing after, as it was.
fn special_order(state_bundle: &mut StateBundle, worth: &[f64]) {
    let upgrades = &state_bundle.upgrade_arr;
    let (mut left, rest): (Vec<usize>, Vec<usize>) = state_bundle
        .special_state
        .iter()
        .copied()
        .partition(|&index| upgrades[index].is_normal_honing);
    let mut order = Vec::with_capacity(upgrades.len());
    while !left.is_empty() {
        let at = (0..left.len())
            .filter(|&at| {
                let this = &upgrades[left[at]];
                !left.iter().any(|&other| {
                    upgrades[other].piece_index == this.piece_index
                        && upgrades[other].upgrade_index < this.upgrade_index
                })
            })
            .max_by(|&a, &b| worth[left[a]].total_cmp(&worth[left[b]]))
            .unwrap();
        order.push(left.remove(at));
    }
    order.extend(rest);
    state_bundle.special_state = order;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::OneMaterial;
    use crate::payload::Payload;

    fn scenario() -> Payload {
        let path = "../../arena_data/scenarios/real3/r3_04b85565_09120f_f0_s1.json";
        let file: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        serde_json::from_value(file["steps"][0].clone()).unwrap()
    }

    /// maxroll.gg, T4 Upper weapon +20 with nothing owned, juice at 100: 539,231 gold
    #[test]
    fn matches_the_site() {
        let mut payload = scenario();
        let prices = [0.6, 0.2, 15.0, 0.25, 80.0, 1.0, 0.0, 150.0, 100.0];
        for (at, label) in payload.material_labels.iter().enumerate() {
            payload.materials.insert(
                label.clone(),
                OneMaterial {
                    bound: 0.0,
                    roster: 0.0,
                    tradable: 0.0,
                    taxed_price: 0.0,
                    market_price: *prices.get(at).unwrap_or(&1e6),
                },
            );
        }
        let mut weapon = payload.upgrade_info[0].clone();
        weapon.piece_index = 5;
        weapon.upgrade_index = 19;
        payload.upgrade_info = vec![weapon];
        // x10 on: the rule plans tap by tap anyway
        payload.min_resolution = 10;
        payload.special_budget = 0;
        payload.chests = Default::default();
        let mut bundle = StateBundle::init_from_payload(payload);
        let maxroll = Maxroll {
            juice: true,
            book: true,
            special: false,
        };
        apply_maxroll(&mut bundle, maxroll);
        let all: Vec<f64> = bundle
            .prep_output
            .materials
            .iter()
            .map(|m| m.market_price)
            .collect();
        let response = Response::linear(&bundle, 0, &all);
        let gold = -response.score(&response.model.indices(&bundle.upgrade_arr[0].state.payload));
        // the site uses part of a juice stack on its last juiced tap, worth 35 gold here
        assert!((gold - 539_231.0).abs() < 100.0, "{gold}");
    }

    /// what the Maxroll pools took before does not matter
    #[test]
    fn does_not_depend_on_the_state_before() {
        let run = |full: bool, maxroll: Maxroll| {
            let mut bundle = StateBundle::init_from_payload(scenario());
            for index in 0..bundle.upgrade_arr.len() {
                if !bundle.upgrade_arr[index].is_normal_honing {
                    continue;
                }
                let model = Response::linear(&bundle, index, &vec![0.0; 99]).model;
                let option = if full { model.options.len() - 1 } else { 0 };
                let plan = model.plan(&vec![option; model.len]);
                bundle.upgrade_arr[index].state.update_payload(plan);
            }
            apply_maxroll(&mut bundle, maxroll);
            let plans: Vec<_> = bundle
                .upgrade_arr
                .iter()
                .map(|upgrade| upgrade.state.payload.clone())
                .collect();
            (plans, bundle.special_state.clone())
        };
        let both = Maxroll {
            juice: true,
            book: true,
            special: true,
        };
        assert!(run(true, both) == run(false, both));
        let book = Maxroll {
            juice: false,
            book: true,
            special: false,
        };
        assert!(run(true, book) != run(false, book));
    }
}
