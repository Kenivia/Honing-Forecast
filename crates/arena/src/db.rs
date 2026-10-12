//! The results database. Counters and tuning are JSON columns, so adding one needs no
//! schema change; use json_extract to query them.

use crate::worker::StepResult;
use rusqlite::{Connection, params};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS snapshot(
    id INTEGER PRIMARY KEY, label TEXT, git_sha TEXT, diff TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE IF NOT EXISTS session(
    id INTEGER PRIMARY KEY, git_sha TEXT, args TEXT,
    started_at TEXT DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE IF NOT EXISTS scenario_step(
    id INTEGER PRIMARY KEY, scenario TEXT, step INTEGER, payload_hash TEXT, payload TEXT,
    UNIQUE(scenario, step, payload_hash));
CREATE TABLE IF NOT EXISTS run(
    id INTEGER PRIMARY KEY, session_id INTEGER, snapshot_id INTEGER, variant TEXT,
    step_id INTEGER, kind TEXT, source TEXT, seed INTEGER, chained INTEGER,
    tuning TEXT, cpu_s REAL, wall_s REAL, metric REAL, metric_ref REAL, state TEXT,
    blob_bytes INTEGER, counters TEXT, error TEXT);
CREATE TABLE IF NOT EXISTS trace(run_id INTEGER, iter INTEGER, key TEXT, value REAL);
CREATE TABLE IF NOT EXISTS trace_text(run_id INTEGER, iter INTEGER, key TEXT, value TEXT);
CREATE TABLE IF NOT EXISTS mc(
    run_id INTEGER, material TEXT, analytic REAL, mean REAL, se REAL, z REAL,
    pass INTEGER, samples INTEGER);
CREATE INDEX IF NOT EXISTS run_session ON run(session_id);
CREATE INDEX IF NOT EXISTS run_step ON run(step_id);
CREATE INDEX IF NOT EXISTS trace_run ON trace(run_id, key);
CREATE INDEX IF NOT EXISTS mc_run ON mc(run_id);
";

pub fn open(path: &Path) -> Connection {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    conn
}

pub fn add_snapshot(conn: &Connection, label: &str, git_sha: &str, diff: &str) -> i64 {
    conn.execute(
        "INSERT INTO snapshot(label, git_sha, diff) VALUES (?1, ?2, ?3)",
        params![label, git_sha, diff],
    )
    .unwrap();
    conn.last_insert_rowid()
}

/// the working tree's snapshot row, reused while the tree has not changed
pub fn current_snapshot(conn: &Connection, git_sha: &str, diff: &str) -> i64 {
    let same: Option<i64> = conn
        .query_row(
            "SELECT MAX(id) FROM snapshot WHERE label = 'current' AND git_sha = ?1 AND diff = ?2",
            params![git_sha, diff],
            |row| row.get(0),
        )
        .unwrap();
    same.unwrap_or_else(|| add_snapshot(conn, "current", git_sha, diff))
}

pub fn latest_snapshot(conn: &Connection, label: &str) -> Option<i64> {
    conn.query_row(
        "SELECT MAX(id) FROM snapshot WHERE label = ?1",
        [label],
        |row| row.get(0),
    )
    .unwrap()
}

pub fn add_session(conn: &Connection, git_sha: &str, args: &str) -> i64 {
    conn.execute(
        "INSERT INTO session(git_sha, args) VALUES (?1, ?2)",
        params![git_sha, args],
    )
    .unwrap();
    conn.last_insert_rowid()
}

/// the same payload under the same scenario and step is the same row across sessions
pub fn step_id(conn: &Connection, scenario: &str, step: usize, payload: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    payload.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    conn.execute(
        "INSERT OR IGNORE INTO scenario_step(scenario, step, payload_hash, payload) VALUES (?1, ?2, ?3, ?4)",
        params![scenario, step as i64, hash, payload],
    )
    .unwrap();
    conn.query_row(
        "SELECT id FROM scenario_step WHERE scenario = ?1 AND step = ?2 AND payload_hash = ?3",
        params![scenario, step as i64, hash],
        |row| row.get(0),
    )
    .unwrap()
}

pub struct RunKey<'a> {
    pub session_id: i64,
    pub snapshot_id: i64,
    pub variant: &'a str,
    pub step_id: i64,
    pub kind: &'a str,
    pub seed: u64,
    pub chained: bool,
}

pub fn add_run(conn: &Connection, key: &RunKey, out: &StepResult, metric_ref: f64, samples: usize) {
    conn.execute(
        "INSERT INTO run(session_id, snapshot_id, variant, step_id, kind, source, seed, chained,
            tuning, cpu_s, wall_s, metric, metric_ref, state, blob_bytes, counters)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            key.session_id,
            key.snapshot_id,
            key.variant,
            key.step_id,
            key.kind,
            out.source,
            key.seed as i64,
            key.chained,
            out.tuning.to_string(),
            out.cpu_s,
            out.wall_s,
            out.metric,
            metric_ref,
            serde_json::to_string(&out.essence).unwrap(),
            out.blob_bytes as i64,
            serde_json::to_string(&out.counters).unwrap(),
        ],
    )
    .unwrap();
    let run_id = conn.last_insert_rowid();

    let mut trace = conn
        .prepare_cached("INSERT INTO trace(run_id, iter, key, value) VALUES (?1, ?2, ?3, ?4)")
        .unwrap();
    for (iter, name, value) in &out.rows {
        trace.execute(params![run_id, iter, name, value]).unwrap();
    }
    let mut text = conn
        .prepare_cached("INSERT INTO trace_text(run_id, iter, key, value) VALUES (?1, ?2, ?3, ?4)")
        .unwrap();
    for (iter, name, value) in &out.texts {
        text.execute(params![run_id, iter, name, value]).unwrap();
    }
    let mut mc = conn
        .prepare_cached(
            "INSERT INTO mc(run_id, material, analytic, mean, se, z, pass, samples)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .unwrap();
    for row in &out.mc {
        mc.execute(params![
            run_id,
            row.material,
            row.analytic,
            row.mean,
            row.se,
            row.z,
            row.pass,
            samples as i64
        ])
        .unwrap();
    }
}

/// a job that died; whatever steps it finished are already in
pub fn add_error(conn: &Connection, key: &RunKey, error: &str) {
    conn.execute(
        "INSERT INTO run(session_id, snapshot_id, variant, step_id, kind, seed, chained, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            key.session_id,
            key.snapshot_id,
            key.variant,
            key.step_id,
            key.kind,
            key.seed as i64,
            key.chained,
            error
        ],
    )
    .unwrap();
}
