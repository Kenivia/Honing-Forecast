//! The market price of a juice or book under which the optimizer would use more of it.
//! Searched downwards from market: steps doubling away from it until the plan, polished at the
//! lower price, uses more, then bisected back towards it, over a fixed number of polishes.

use crate::materials::NUM_BASE_MATS;
use crate::optimizer::polish_only;
use crate::probe::Probe;
use crate::state_bundle::StateBundle;
use ahash::AHashMap;
use rand::Rng;

const POLISHES: usize = 6;

// expected use of each row, which no tuning changes
fn usage(state_bundle: &mut StateBundle, probe: &mut Probe) -> Vec<f64> {
    state_bundle.ui_average_gold_metric(probe).1
}

// whether any upgrade can take the juice
fn usable(state_bundle: &StateBundle, id: usize) -> bool {
    let juice_info = &state_bundle.prep_output.juice_info;
    state_bundle.upgrade_arr.iter().any(|upgrade| {
        let ids = if upgrade.is_normal_honing {
            &juice_info.normal_uindex_to_id
        } else {
            &juice_info.adv_uindex_to_id
        };
        ids[upgrade.piece_type_usize][upgrade.upgrade_index].contains(&id)
    })
}

// the plan polished with the row bought and sold at this price
fn at_price<R: Rng>(
    rng: &mut R,
    base: &StateBundle,
    row: usize,
    price: f64,
    probe: &mut Probe,
) -> StateBundle {
    let mut priced = base.clone();
    let mat = &mut priced.prep_output.base_materials[row];
    mat.taxed_price *= price / mat.market_price;
    mat.market_price = price;
    priced.apply_chest_state(false);
    polish_only(rng, &mut priced, probe)
}

// (price, found): the middle of the last bracket, or the lowest price tried when none used more
fn threshold<R: Rng>(
    rng: &mut R,
    base: &StateBundle,
    row: usize,
    used: f64,
    probe: &mut Probe,
) -> (f64, bool) {
    let market = base.prep_output.base_materials[row].market_price;
    // a tenth of the price's order of magnitude
    let mut step = 10f64.powf(market.log10().floor() - 1.0);
    let mut high = market;
    let mut low: Option<f64> = None;
    for _ in 0..POLISHES {
        let price = match low {
            Some(low) => (low + high) / 2.0,
            None => (high - step).max(0.0),
        };
        let moved = usage(&mut at_price(rng, base, row, price, probe), probe)[row];
        if moved > used + 1e-6 * used.max(1.0) {
            low = Some(price);
        } else if price == 0.0 {
            return (0.0, false);
        } else {
            high = price;
            step *= 2.0;
        }
    }
    match low {
        Some(low) => ((low + high) / 2.0, true),
        None => (high, false),
    }
}

/// Per unit, by label, for each optional label some upgrade takes.
pub fn use_more_prices<R: Rng>(
    rng: &mut R,
    state_bundle: &mut StateBundle,
    labels: &[String],
) -> AHashMap<String, (f64, bool)> {
    let mut probe = Probe::default();
    let mut base = polish_only(rng, state_bundle, &mut probe);
    let used = usage(&mut base, &mut probe);
    labels
        .iter()
        .filter_map(|label| {
            let row = base.prep_output.table.row(label);
            if row < NUM_BASE_MATS
                || base.prep_output.base_materials[row].market_price <= 0.0
                || !usable(&base, row - NUM_BASE_MATS)
            {
                return None;
            }
            let found = threshold(rng, &base, row, used[row], &mut probe);
            Some((label.clone(), found))
        })
        .collect()
}
