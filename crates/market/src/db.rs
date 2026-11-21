// Scans kept in ignored/market/market.sqlite. The currency exchange is kept only in the payload.
use crate::{auction::Search, market::Material};
use rusqlite::{Connection, params};

pub const PATH: &str = concat!(env!("HF_ROOT"), "/ignored/market/market.sqlite");

pub fn open(path: &str) -> Connection {
    std::fs::create_dir_all(concat!(env!("HF_ROOT"), "/ignored/market")).unwrap();
    let c = Connection::open(path).unwrap();
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS scans(id INTEGER PRIMARY KEY, started TEXT, finished TEXT, region TEXT,
            payload TEXT, uploaded INTEGER DEFAULT 0);
         CREATE TABLE IF NOT EXISTS materials(scan INTEGER, at TEXT, name TEXT, raw_name TEXT, bundle INTEGER,
            avg_day REAL, recent REAL, available REAL, category TEXT);
         CREATE TABLE IF NOT EXISTS depth(scan INTEGER, name TEXT, tier INTEGER, price REAL, quantity REAL);
         CREATE TABLE IF NOT EXISTS history(scan INTEGER, name TEXT, days_ago INTEGER, avg REAL);
         CREATE TABLE IF NOT EXISTS searches(id INTEGER PRIMARY KEY, scan INTEGER, at TEXT, kind TEXT, item TEXT,
            category TEXT, gem_level INTEGER, level_a TEXT, level_b TEXT, results INTEGER);
         CREATE TABLE IF NOT EXISTS listings(search INTEGER, rank INTEGER, name TEXT, trades TEXT, quality REAL,
            time_left TEXT, min_bid REAL, buy_now REAL);",
    )
    .unwrap();
    c
}

fn now() -> String {
    chrono::Local::now().to_rfc3339()
}

pub fn start(c: &Connection, region: &str) -> i64 {
    c.execute(
        "INSERT INTO scans(started, region) VALUES (?1, ?2)",
        params![now(), region],
    )
    .unwrap();
    c.last_insert_rowid()
}

pub fn finish(c: &Connection, scan: i64) {
    c.execute(
        "UPDATE scans SET finished = ?1 WHERE id = ?2",
        params![now(), scan],
    )
    .unwrap();
}

pub fn materials(c: &Connection, scan: i64, ms: &[Material]) {
    for m in ms {
        c.execute(
            "INSERT INTO materials VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                scan,
                m.at,
                m.name,
                m.raw_name,
                m.bundle,
                m.avg_day,
                m.recent,
                m.available,
                m.category
            ],
        )
        .unwrap();
        for (i, (p, q)) in m.depth.iter().enumerate() {
            c.execute(
                "INSERT INTO depth VALUES (?1, ?2, ?3, ?4, ?5)",
                params![scan, m.name, i as i64, p, q],
            )
            .unwrap();
        }
        for (i, a) in m.history.iter().enumerate() {
            c.execute(
                "INSERT INTO history VALUES (?1, ?2, ?3, ?4)",
                params![scan, m.name, i as i64, a],
            )
            .unwrap();
        }
    }
}

pub fn searches(c: &Connection, scan: i64, ss: &[Search]) {
    for s in ss {
        c.execute(
            "INSERT INTO searches(scan, at, kind, item, category, gem_level, level_a, level_b, results) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![scan, s.at, s.kind, s.item, s.category, s.gem_level, s.level_a, s.level_b, s.rows.len() as i64],
        )
        .unwrap();
        let id = c.last_insert_rowid();
        for (i, r) in s.rows.iter().enumerate() {
            c.execute(
                "INSERT INTO listings VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    i as i64,
                    r.name,
                    r.trades,
                    r.quality,
                    r.time_left,
                    r.min_bid,
                    r.buy_now
                ],
            )
            .unwrap();
        }
    }
}

/// The pass as uploaded (`upload::payload`), kept until sent.
pub fn set_payload(c: &Connection, scan: i64, payload: &serde_json::Value) {
    c.execute(
        "UPDATE scans SET payload = ?1 WHERE id = ?2",
        params![payload.to_string(), scan],
    )
    .unwrap();
}

pub fn unsent(c: &Connection) -> Vec<(i64, String)> {
    let mut q = c
        .prepare(
            "SELECT id, payload FROM scans WHERE uploaded = 0 AND payload IS NOT NULL ORDER BY id",
        )
        .unwrap();
    q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

pub fn uploaded(c: &Connection, scan: i64) {
    c.execute("UPDATE scans SET uploaded = 1 WHERE id = ?1", params![scan])
        .unwrap();
}
