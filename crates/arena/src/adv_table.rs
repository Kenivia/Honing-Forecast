//! Advanced honing distributions kept between runs, so that a solve never works one out.
//!
//! A row is one start position and holds the distributions of every strategy from it, which
//! is all an optimizer can ask for. A start not seen before is worked out once and kept.
//! See `AI notes/Calculator core/Arena.md`.

use crate::ROOT;
use ahash::AHashSet;
use hf_core::advanced_honing::compute::compute_adv_dist_wrapper;
use hf_core::advanced_honing::utils::{AdvConfig, AdvDistTriplet};
use hf_core::payload::Payload;
use hf_core::state_bundle::StateBundle;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use std::time::Duration;

const PATH: &str = "arena_data/adv_cache.sqlite";

/// puts every distribution the payload's upgrades can ask for into its cache
pub fn fill(payload: &mut Payload) {
    let starts: AHashSet<AdvConfig> = StateBundle::init_from_payload(payload.clone())
        .upgrade_arr
        .iter()
        .filter(|upgrade| !upgrade.is_normal_honing)
        .map(|upgrade| {
            let at = upgrade.adv_config;
            AdvConfig::new(
                at.start_xp,
                at.start_balls,
                at.next_free,
                at.next_big,
                at.double_balls,
                at.is_30_40,
            )
        })
        .collect();
    if starts.is_empty() {
        return;
    }

    // many workers share the file
    let conn = Connection::open(Path::new(ROOT).join(PATH)).unwrap();
    conn.busy_timeout(Duration::from_secs(120)).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS block(start TEXT PRIMARY KEY, dists BLOB)",
        [],
    )
    .unwrap();

    let cache = payload.adv_cache.get_or_insert_default();
    for start in starts {
        let key = serde_json::to_string(&start).unwrap();
        let configs = start.strategies();
        let stored: Option<Vec<u8>> = conn
            .query_row("SELECT dists FROM block WHERE start = ?1", [&key], |row| {
                row.get(0)
            })
            .optional()
            .unwrap();
        let dists: Vec<AdvDistTriplet> = match stored {
            Some(bytes) => postcard::from_bytes(&bytes).unwrap(),
            None => {
                let dists: Vec<AdvDistTriplet> =
                    configs.iter().map(compute_adv_dist_wrapper).collect();
                let bytes = postcard::to_allocvec(&dists).unwrap();
                conn.execute(
                    "INSERT OR IGNORE INTO block VALUES (?1, ?2)",
                    params![key, bytes],
                )
                .unwrap();
                dists
            }
        };
        cache.extend(configs.into_iter().zip(dists));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::dummy;

    /// a cache that has every strategy leaves a solve nothing to work out, and what it holds
    /// is what the solve would have worked out
    #[test]
    fn a_filled_payload_has_every_distribution() {
        let mut payload = dummy().steps[0].clone();
        fill(&mut payload);
        let cache = payload.adv_cache.clone().unwrap();
        assert!(!cache.is_empty() && cache.len() % AdvConfig::default().strategies().len() == 0);
        assert_eq!(
            StateBundle::init_from_payload(payload).adv_cache.len(),
            cache.len()
        );
        for (config, kept) in cache.iter().take(40) {
            let fresh = compute_adv_dist_wrapper(config);
            for (a, b) in [
                (&kept.cost, &fresh.cost),
                (&kept.juice, &fresh.juice),
                (&kept.scroll, &fresh.scroll),
            ] {
                assert!(
                    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
                );
            }
        }
    }
}
