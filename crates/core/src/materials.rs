//! Materials are addressed by label on the wire and by row inside the evaluator.
//!
//! Row order is the frontend's `ALL_LABELS[tier]`, sent as `material_labels`. The first
//! NUM_BASE_MATS rows are the cost table's rows, in the order the constants JSON uses;
//! juice id `k` follows at row NUM_BASE_MATS + k.

use crate::constants::FLOAT_TOL;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

pub const NUM_BASE_MATS: usize = 7;

/// What one leftover unit of an ownership band is worth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BandValue {
    Worthless,
    TaxedSell,
    Market,
}

/// Owned amounts and per-unit prices for one material.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OneMaterial {
    pub bound: f64,
    pub roster: f64,
    pub tradable: f64,
    pub taxed_price: f64,
    pub market_price: f64,
}

impl BandValue {
    fn price(self, mat: &OneMaterial) -> f64 {
        match self {
            BandValue::Worthless => 0.0,
            BandValue::TaxedSell => mat.taxed_price,
            BandValue::Market => mat.market_price,
        }
    }
}

/// Leftover value of the char-bound, roster-bound and tradable bands, non-decreasing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValuationPlan(pub [BandValue; 3]);

impl ValuationPlan {
    /// (threshold, marginal value per unit above it). The last entry's price is what a
    /// unit costs to buy; the earlier ones are what an unused unit is worth.
    pub fn bands(&self, mat: &OneMaterial) -> Vec<(f64, f64)> {
        assert!(self.0[0] <= self.0[1] && self.0[1] <= self.0[2]);
        let owned: [f64; 3] = [mat.bound, mat.roster, mat.tradable];

        let mut cumulative: f64 = 0.0;
        let mut out: Vec<(f64, f64)> = Vec::with_capacity(4);
        for (band, value) in self.0.iter().enumerate() {
            // a zero width band's price never applies, so its entry drops out
            if out
                .last()
                .is_some_and(|last| (cumulative - last.0).abs() < FLOAT_TOL)
            {
                out.pop();
            }
            out.push((cumulative, value.price(mat)));
            cumulative += owned[band];
        }

        // equal consecutive credits are one band, so the upper entry is redundant
        out.dedup_by(|upper, lower| (upper.1 - lower.1).abs() < FLOAT_TOL);
        if out
            .last()
            .is_some_and(|last| (cumulative - last.0).abs() < FLOAT_TOL)
        {
            out.pop();
        }
        // The top threshold is where buying starts, so its entry is never a credit and is
        // never merged away: dropping it would move the whole function by a constant, even
        // when the band below happens to be credited at the same price. The one exception
        // is a price of zero, where that constant is zero: a material nobody pays for (a
        // disabled one) collapses to a single entry, which the evaluator answers without
        // any saddlepoint work at all.
        out.push((cumulative, mat.market_price));
        if out.iter().all(|&(_, price)| price.abs() < FLOAT_TOL) {
            return vec![(0.0, 0.0)];
        }
        out
    }
}

/// Row order and the label every row is addressed by.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialTable {
    pub labels: Vec<String>,
    #[serde(skip)]
    row_of: AHashMap<String, usize>,
}

impl MaterialTable {
    pub fn new(labels: Vec<String>, num_juices: usize) -> MaterialTable {
        assert!(labels.len() == NUM_BASE_MATS + num_juices);
        let row_of: AHashMap<String, usize> = labels
            .iter()
            .enumerate()
            .map(|(row, label)| (label.clone(), row))
            .collect();
        assert!(row_of.len() == labels.len()); // no duplicate labels
        MaterialTable { labels, row_of }
    }

    pub fn len(&self) -> usize {
        self.labels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    pub fn row(&self, label: &str) -> usize {
        self.row_of[label]
    }

    pub fn juice_row(&self, id: usize) -> usize {
        NUM_BASE_MATS + id
    }

    pub fn is_base(&self, row: usize) -> bool {
        row < NUM_BASE_MATS
    }

    /// The payload's label keyed map, put into row order.
    pub fn in_row_order(&self, keyed: &AHashMap<String, OneMaterial>) -> Vec<OneMaterial> {
        self.labels.iter().map(|label| keyed[label]).collect()
    }

    /// Pairs a row indexed result back up with its label, for the frontend.
    pub fn keyed<T: Clone>(&self, values: &[T]) -> AHashMap<String, T> {
        assert!(values.len() == self.labels.len());
        self.labels
            .iter()
            .cloned()
            .zip(values.iter().cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::BandValue::{Market, TaxedSell, Worthless};
    use super::{FLOAT_TOL, OneMaterial, ValuationPlan};

    const MAT: OneMaterial = OneMaterial {
        bound: 10.0,
        roster: 20.0,
        tradable: 30.0,
        taxed_price: 95.0,
        market_price: 100.0,
    };

    fn bands(values: [super::BandValue; 3]) -> Vec<(f64, f64)> {
        ValuationPlan(values).bands(&MAT)
    }

    #[test]
    fn nothing_is_credited() {
        assert_eq!(bands([Worthless; 3]), vec![(0.0, 0.0), (60.0, 100.0)]);
    }

    /// what the UI sent before this interface existed
    #[test]
    fn only_tradable_is_credited() {
        assert_eq!(
            bands([Worthless, Worthless, TaxedSell]),
            vec![(0.0, 0.0), (30.0, 95.0), (60.0, 100.0)]
        );
    }

    #[test]
    fn roster_credited_too() {
        assert_eq!(
            bands([Worthless, TaxedSell, TaxedSell]),
            vec![(0.0, 0.0), (10.0, 95.0), (60.0, 100.0)]
        );
    }

    /// crediting a band at the buy price still leaves the top threshold in place, because
    /// the stock in that band is worth something whether it gets used or not
    #[test]
    fn the_top_threshold_survives_the_buy_price() {
        assert_eq!(
            bands([Worthless, Worthless, Market]),
            vec![(0.0, 0.0), (30.0, 100.0), (60.0, 100.0)]
        );
        assert_eq!(bands([Market; 3]), vec![(0.0, 100.0), (60.0, 100.0)]);
    }

    /// owning nothing leaves one entry: everything is bought
    #[test]
    fn nothing_owned_is_all_bought() {
        let mat = OneMaterial {
            bound: 0.0,
            roster: 0.0,
            tradable: 0.0,
            ..MAT
        };
        for values in [[Worthless; 3], [Market; 3], [Worthless, Worthless, TaxedSell]] {
            assert_eq!(
                ValuationPlan(values).bands(&mat),
                vec![(0.0, 100.0)],
                "{values:?}"
            );
        }
    }

    /// The old breakpoint merging lost a whole band's credit when its sell price happened
    /// to equal the buy price, because it extended a threshold rather than dropping an
    /// entry. Shards and Silver list at the same price either way.
    #[test]
    fn a_tied_sell_price_still_credits_its_band() {
        let mat = OneMaterial {
            taxed_price: 100.0,
            ..MAT
        };
        assert_eq!(
            ValuationPlan([Worthless, Worthless, TaxedSell]).bands(&mat),
            vec![(0.0, 0.0), (30.0, 100.0), (60.0, 100.0)]
        );
    }

    /// a band of nothing at the top does not move where buying starts
    #[test]
    fn empty_top_band() {
        let mat = OneMaterial {
            tradable: 0.0,
            ..MAT
        };
        assert_eq!(
            ValuationPlan([Worthless, Worthless, TaxedSell]).bands(&mat),
            vec![(0.0, 0.0), (30.0, 100.0)]
        );
    }

    #[test]
    fn empty_bands_drop_out() {
        let mat = OneMaterial {
            bound: 0.0,
            tradable: 0.0,
            ..MAT
        };
        assert_eq!(
            ValuationPlan([Worthless, TaxedSell, TaxedSell]).bands(&mat),
            vec![(0.0, 95.0), (20.0, 100.0)]
        );
    }

    #[test]
    #[should_panic]
    fn values_must_not_decrease() {
        bands([TaxedSell, Worthless, Worthless]);
    }

    /// A disabled material is sent with zero prices and an effectively infinite bound
    /// stock. It must cost one entry, not two, because two would make the evaluator
    /// compute a probability and then multiply it by a zero price gap.
    #[test]
    fn a_material_nobody_pays_for_is_one_entry() {
        let mat = OneMaterial {
            bound: 999999999.0,
            roster: 0.0,
            tradable: 0.0,
            taxed_price: 0.0,
            market_price: 0.0,
        };
        assert_eq!(
            ValuationPlan([Worthless; 3]).bands(&mat),
            vec![(0.0, 0.0)]
        );
    }

    /// one_dimension_average_gold costs two saddlepoint evaluations per entry past the
    /// first, and does not skip an entry whose price gap is zero. So a repeated threshold
    /// or a repeated credit price would be paid for and contribute nothing.
    #[test]
    fn every_plan_produces_a_minimal_list() {
        let levels = [Worthless, TaxedSell, Market];
        let owned_cases = [
            [0.0, 0.0, 0.0],
            [10.0, 20.0, 30.0],
            [0.0, 20.0, 30.0],
            [10.0, 0.0, 30.0],
            [10.0, 20.0, 0.0],
            [10.0, 0.0, 0.0],
        ];
        let price_cases = [(95.0, 100.0), (0.17, 0.18), (1.0, 1.0), (0.0, 0.0)];
        let mut plans = Vec::new();
        for a in 0..3 {
            for b in a..3 {
                for c in b..3 {
                    plans.push([levels[a], levels[b], levels[c]]);
                }
            }
        }
        assert_eq!(plans.len(), 10, "every non decreasing assignment");

        for plan in plans {
            for owned in owned_cases {
                for (taxed, market) in price_cases {
                    let mat = OneMaterial {
                        bound: owned[0],
                        roster: owned[1],
                        tradable: owned[2],
                        taxed_price: taxed,
                        market_price: market,
                    };
                    let bands = ValuationPlan(plan).bands(&mat);
                    let tag = format!("{plan:?} {owned:?} {taxed}/{market} -> {bands:?}");
                    assert!(
                        bands.windows(2).all(|w| w[1].0 > w[0].0),
                        "repeated threshold: {tag}"
                    );
                    // the last entry is a buy price rather than a credit, so it is exempt
                    assert!(
                        bands[..bands.len() - 1]
                            .windows(2)
                            .all(|w| (w[1].1 - w[0].1).abs() >= FLOAT_TOL),
                        "repeated credit price: {tag}"
                    );
                }
            }
        }
    }
}
