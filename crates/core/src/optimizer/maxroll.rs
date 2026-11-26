//! Maxroll's upgrade calculator's rule, as a comparison strategy: every material at its buy
//! price. Juice and books where they save more than they cost (the pools not asked for kept as
//! given), plus owned juice and books on the taps they save the most on, as far as the stock
//! covers every tap given them. Free taps by gold saved per special leap. Normal honing only.

use super::response::Response;
use crate::honing_utils::materials::NUM_BASE_MATS;
use crate::honing_utils::state_bundle::StateBundle;
use crate::honing_utils::update_normal::chance as to_chance;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Maxroll {
    pub juice: bool,
    pub book: bool,
    pub special: bool,
}

/// One charged tap taking one id of a pool, and the expected gold that saves.
struct Candidate {
    index: usize,
    tap: usize,
    pool: usize,
    /// what it adds to the tap's option
    option: usize,
    on: bool,
    row: usize,
    amount: f64,
    gain: f64,
}

/// the taps a plan pays juice on, and the chance each is reached
fn charged(response: &Response, indices: &[usize]) -> Vec<(usize, f64)> {
    let model = &response.model;
    let (mut artisan, mut alive, mut out) = (model.artisan, 1.0, Vec::new());
    for count in 0..model.len {
        if model.tap.full(artisan) || count + 1 >= model.len {
            break;
        }
        let rate = model.rate(count, model.extra[indices[count]]);
        let chance = to_chance(rate);
        if chance == 1.0 {
            break;
        }
        out.push((count, alive));
        alive *= 1.0 - chance;
        artisan = model.step(artisan, rate);
    }
    out
}

/// Lab variants of the owned stock rule: whether bound and roster stock is used, and whether
/// taps go by gold saved if reached rather than expected gold saved.
#[derive(Debug, Clone, Copy)]
pub struct Owned {
    pub stock: bool,
    pub if_reached: bool,
}

pub fn apply_maxroll(state_bundle: &mut StateBundle, maxroll: Maxroll) {
    let owned = Owned {
        stock: true,
        if_reached: true,
    };
    apply_owned(state_bundle, maxroll, owned);
}

pub fn apply_owned(state_bundle: &mut StateBundle, maxroll: Maxroll, owned: Owned) {
    if !(maxroll.juice || maxroll.book || maxroll.special) {
        return;
    }
    // the site plans tap by tap, whatever the x10 setting
    let streak_length = std::mem::replace(&mut state_bundle.tuning.streak_length, 1);
    let materials = &state_bundle.prep_output.materials;
    let prices: Vec<f64> = materials.iter().map(|m| m.market_price).collect();
    let mut stock: Vec<f64> = materials
        .iter()
        // tradable stock is worth its sell price, so it is bought like the rest
        .map(|m| {
            if owned.stock {
                (m.bound + m.roster).max(0.0)
            } else {
                0.0
            }
        })
        .collect();
    // the buy price plan of each upgrade, and what juicing each of its charged taps saves
    let mut plans: Vec<Option<Vec<usize>>> = vec![None; state_bundle.upgrade_arr.len()];
    let mut candidates = Vec::new();
    for index in 0..state_bundle.upgrade_arr.len() {
        let upgrade = &state_bundle.upgrade_arr[index];
        if !upgrade.is_normal_honing {
            continue;
        }
        let response = Response::linear(state_bundle, index, &prices);
        let model = &response.model;
        let pools = model.pools.len();
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
        if free.is_empty() {
            continue;
        }
        let indices = response
            .best_free(&model.indices(&upgrade.state.payload), &free)
            .1;
        let base = response.score(&indices);
        for (tap, reach) in charged(&response, &indices) {
            for &pool in &free {
                let (stride, ids) = (model.strides[pool], &model.pools[pool]);
                let now = indices[tap] / stride % (ids.len() + 1);
                for which in (0..ids.len()).filter(|&which| now == 0 || now == which + 1) {
                    let mut other = indices.clone();
                    let gain = if now == 0 {
                        other[tap] += (which + 1) * stride;
                        response.score(&other) - base
                    } else {
                        other[tap] -= now * stride;
                        base - response.score(&other)
                    };
                    let row = state_bundle.prep_output.table.juice_row(ids[which]);
                    let amount = state_bundle
                        .prep_output
                        .juice_info
                        .access(ids[which], upgrade.piece_type_usize, upgrade.upgrade_index)
                        .normal_amt_used as f64;
                    candidates.push(Candidate {
                        index,
                        tap,
                        pool,
                        option: (which + 1) * stride,
                        on: now != 0,
                        row,
                        amount,
                        // with what the juice costs added back
                        gain: (gain + reach * amount * prices[row])
                            / if owned.if_reached { reach } else { 1.0 },
                    });
                }
            }
        }
        plans[index] = Some(indices);
    }
    // owned stock to the most gold saved a unit, as much as is sure to be there
    candidates.sort_by(|a, b| (b.gain / b.amount).total_cmp(&(a.gain / a.amount)));
    let mut taken = std::collections::HashSet::new();
    for c in candidates.iter().filter(|c| c.gain > 0.0) {
        if stock[c.row] < c.amount || !taken.insert((c.index, c.tap, c.pool)) {
            continue;
        }
        stock[c.row] -= c.amount;
        if !c.on {
            plans[c.index].as_mut().unwrap()[c.tap] += c.option;
        }
    }
    // gold a free tap saves for each special leap it takes, by upgrade
    let mut worth = vec![f64::NEG_INFINITY; state_bundle.upgrade_arr.len()];
    for index in 0..state_bundle.upgrade_arr.len() {
        if !state_bundle.upgrade_arr[index].is_normal_honing {
            continue;
        }
        let response = Response::linear(state_bundle, index, &prices);
        let indices = match plans[index].take() {
            Some(indices) => {
                let plan = response.model.plan(&indices);
                state_bundle.upgrade_arr[index].state.update_payload(plan);
                indices
            }
            None => response
                .model
                .indices(&state_bundle.upgrade_arr[index].state.payload),
        };
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
    use crate::honing_utils::materials::OneMaterial;
    use crate::payload::Payload;

    fn scenario() -> Payload {
        Payload::load_step(
            "../../arena_data/scenarios/real3/r3_04b85565_09120f_f0_s1.json",
            0,
        )
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
        let mut bundle = StateBundle::from_payload(payload);
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
            let mut bundle = StateBundle::from_payload(scenario());
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

    /// owned juice and books go on more taps than buying them would
    #[test]
    fn owned_stock_is_used() {
        let juiced = |owned: f64| {
            let mut payload = scenario();
            for (at, label) in payload.material_labels.clone().iter().enumerate() {
                if at >= NUM_BASE_MATS {
                    let material = payload.materials.get_mut(label).unwrap();
                    material.bound = owned;
                    material.roster = 0.0;
                    material.tradable = 0.0;
                }
            }
            let mut bundle = StateBundle::from_payload(payload);
            let both = Maxroll {
                juice: true,
                book: true,
                special: false,
            };
            apply_maxroll(&mut bundle, both);
            bundle
                .upgrade_arr
                .iter()
                .map(|upgrade| {
                    upgrade
                        .state
                        .payload
                        .iter()
                        .map(|slot| slot.len())
                        .sum::<usize>()
                })
                .sum::<usize>()
        };
        assert!(juiced(1e9) > juiced(0.0));
    }
}
