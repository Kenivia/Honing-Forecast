//! Chests still to be opened, and how each stack of them is opened.

use crate::materials::{MaterialTable, OneMaterial};
use crate::state_bundle::StateBundle;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One stack of identical chests. An option is what one chest gives when it is picked, by
/// label; an obtain-all chest is a chest with one option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chest {
    pub count: i64,
    pub band: usize,
    pub options: Vec<BTreeMap<String, f64>>,
}

/// how many chests of a stack go to each option
pub type ChestState = Vec<i64>;
/// Both keyed by the stack's content, as the frontend builds the key, so a state survives other
/// stacks coming and going.
pub type Chests = BTreeMap<String, Chest>;
pub type ChestStates = BTreeMap<String, ChestState>;

impl Chest {
    pub fn default_state(&self) -> ChestState {
        let mut out: ChestState = vec![0; self.options.len()];
        if let Some(first) = out.first_mut() {
            *first = self.count;
        }
        out
    }

    pub fn is_valid(&self, state: &ChestState) -> bool {
        state.len() == self.options.len()
            && state.iter().all(|x| *x >= 0)
            && state.iter().sum::<i64>() == self.count
    }

    /// a stack with no choice to make draws nothing from the rng
    pub fn random_state<R: Rng>(&self, rng: &mut R) -> ChestState {
        if self.options.len() < 2 {
            return self.default_state();
        }
        let mut cuts: Vec<i64> = (1..self.options.len())
            .map(|_| rng.random_range(0..=self.count))
            .collect();
        cuts.sort();
        cuts.push(self.count);
        let mut prev: i64 = 0;
        cuts.iter()
            .map(|&cut| {
                let out = cut - prev;
                prev = cut;
                out
            })
            .collect()
    }
}

/// What the chests give when opened as the states say: per row and band, and special leaps.
/// Amounts are not rounded. Every label is a row or the special leap: the frontend sends nothing
/// else.
pub fn resolve_chests(
    chests: &Chests,
    chest_states: &ChestStates,
    table: &MaterialTable,
    special_label: &str,
) -> (Vec<[f64; 3]>, f64) {
    let mut gained: Vec<[f64; 3]> = vec![[0.0; 3]; table.len()];
    let mut special: f64 = 0.0;
    for (key, chest) in chests {
        for (option, &picked) in chest.options.iter().zip(&chest_states[key]) {
            for (label, amount) in option {
                let total = amount * picked as f64;
                if label == special_label {
                    special += total;
                } else {
                    gained[table.get_row(label).unwrap()][chest.band] += total;
                }
            }
        }
    }
    (gained, special)
}

// a sum of converted amounts can land a hair under a whole number
fn whole(x: f64) -> f64 {
    (x + 1e-6).floor()
}

impl StateBundle {
    /// Rebuilds the bands and the free tap budget for the current chest states. Only the
    /// histogram reads the materials themselves, so they are updated on request.
    pub fn apply_chest_state(&mut self, update_materials: bool) {
        let prep = &mut self.prep_output;
        let (gained, special) = resolve_chests(
            &prep.chests,
            &self.chest_states,
            &prep.table,
            &prep.special_label,
        );
        let materials: Vec<OneMaterial> = prep
            .base_materials
            .iter()
            .zip(gained)
            .map(|(base, gain)| {
                OneMaterial {
                    bound: base.bound + whole(gain[0]),
                    roster: base.roster + whole(gain[1]),
                    tradable: base.tradable + whole(gain[2]),
                    ..*base
                }
                .settled()
            })
            .collect();
        prep.special_budget = prep.base_special_budget + whole(special) as i64;
        prep.bands = prep
            .plans
            .iter()
            .map(|plan| materials.iter().map(|mat| plan.bands(mat)).collect())
            .collect();
        if update_materials {
            prep.materials = materials;
        }
    }

    /// for a state away from any plan, as `scramble` makes them
    pub fn randomize_chest_state<R: Rng>(&mut self, rng: &mut R) {
        self.chest_states = self
            .prep_output
            .chests
            .iter()
            .map(|(key, chest)| (key.clone(), chest.random_state(rng)))
            .collect();
        self.apply_chest_state(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bag(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs
            .iter()
            .map(|(label, amount)| (label.to_string(), *amount))
            .collect()
    }

    #[test]
    fn chests_resolve_to_materials() {
        let labels: Vec<String> = ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let table = MaterialTable::new(labels, 0);
        let choice = Chest {
            count: 10,
            band: 1,
            options: vec![bag(&[("Red", 0.5), ("Blue", 3.0)]), bag(&[("Leaps", 2.0)])],
        };
        let all = Chest {
            count: 4,
            band: 0,
            options: vec![bag(&[("Gold", 100.0), ("Special", 5.0)])],
        };
        let states: ChestStates = [
            ("choice".to_string(), vec![3, 7]),
            ("all".to_string(), all.default_state()),
        ]
        .into();
        let chests: Chests = [("choice".to_string(), choice), ("all".to_string(), all)].into();
        let (gained, special) = resolve_chests(&chests, &states, &table, "Special");
        assert_eq!(gained[0], [0.0, 1.5, 0.0]);
        assert_eq!(gained[1], [0.0, 9.0, 0.0]);
        assert_eq!(gained[2], [0.0, 14.0, 0.0]);
        assert_eq!(gained[5], [400.0, 0.0, 0.0]);
        assert_eq!(special, 20.0);

        let mut rng = rand::rng();
        for chest in chests.values() {
            assert!(chest.is_valid(&chest.random_state(&mut rng)));
        }
    }

    /// a choice to buy: nothing, or several things for gold
    #[test]
    fn a_bought_chest_costs_gold() {
        let labels: Vec<String> = ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let table = MaterialTable::new(labels, 0);
        let buy = Chest {
            count: 1,
            band: 0,
            options: vec![
                bag(&[]),
                bag(&[("Gold", -300.0), ("Red", 7.0), ("Special", 2.0)]),
            ],
        };
        let chests: Chests = [("buy".to_string(), buy)].into();
        let skipped: ChestStates = [("buy".to_string(), vec![1, 0])].into();
        let (gained, special) = resolve_chests(&chests, &skipped, &table, "Special");
        assert!(gained.iter().all(|row| *row == [0.0; 3]) && special == 0.0);
        let bought: ChestStates = [("buy".to_string(), vec![0, 1])].into();
        let (gained, special) = resolve_chests(&chests, &bought, &table, "Special");
        assert_eq!(gained[0], [7.0, 0.0, 0.0]);
        assert_eq!(gained[5], [-300.0, 0.0, 0.0]);
        assert_eq!(special, 2.0);
        assert_eq!(whole(-300.0 + 1e-9), -300.0);
    }
}
