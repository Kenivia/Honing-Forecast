// A pass of one region and a round of every region, shared by market_scan and market_agent. A pass that is
// stopped (F10, pause) or breaks still stores and uploads what it read; the panic then carries on.
use crate::{
    auction, db, exchange,
    game::{Game, panic_text},
    market,
    region::{self, Region},
    report, strips,
    ui::{self, log},
    upload,
};
use rusqlite::Connection;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

pub struct Opts {
    /// materials,gems,accessories,exchange
    pub only: String,
    pub strips: bool,
    pub regions: Vec<&'static Region>,
}

/// How many things it read (materials, searches, the exchange).
pub fn pass(g: &mut Game, conn: &Connection, region: &str, o: &Opts) -> usize {
    let scan = db::start(conn, region);
    let started = chrono::Utc::now();
    let (mut mats, mut found, mut ex) = (vec![], vec![], None);
    if o.strips {
        strips::start();
    }
    log(&format!("scan {scan} started ({region}: {})", o.only));
    let r = catch_unwind(AssertUnwindSafe(|| {
        if o.only.contains("materials") {
            market::scan(g, &mut mats);
            log(&format!("{} materials", mats.len()));
        }
        for (kind, f) in [
            (
                "gems",
                auction::gems as fn(&mut Game, &mut Vec<auction::Search>),
            ),
            ("accessories", auction::accessories),
        ] {
            if o.only.contains(kind) {
                let n = found.len();
                f(g, &mut found);
                log(&format!("{} {kind} searches", found.len() - n));
            }
        }
        if o.only.contains("exchange") {
            ex = exchange::scan(g);
            log(&format!("currency exchange: {ex:?}"));
        }
        ui::reset(g);
    }));
    db::materials(conn, scan, &mats);
    db::searches(conn, scan, &found);
    db::finish(conn, scan);
    if !mats.is_empty() || !found.is_empty() || ex.is_some() {
        let p = upload::payload(
            region,
            started,
            chrono::Utc::now(),
            &mats,
            &found,
            ex.as_ref(),
        );
        db::set_payload(conn, scan, &p);
    }
    upload::send_pending(conn);
    let mut text = format!(
        "{region}: {} materials, {} searches, exchange {}",
        mats.len(),
        found.len(),
        if ex.is_some() { "read" } else { "not read" }
    );
    if let Err(e) = &r {
        text += &format!(" ({} part way)", panic_text(e.as_ref()));
    }
    log(&format!("scan {scan} done: {text}"));
    report::event("pass", &text, None);
    if let Err(e) = r {
        resume_unwind(e);
    }
    mats.len() + found.len() + ex.is_some() as usize
}

/// Every region in turn, switching character when another one is logged in (`current`; None when unknown).
/// How many things it read; panics when a switch fails every try.
pub fn round(
    g: &mut Game,
    conn: &Connection,
    o: &Opts,
    current: &mut Option<&'static str>,
) -> usize {
    let mut n = 0;
    for r in &o.regions {
        if *current != Some(r.name) {
            *current = None;
            *current = ui::unit(g, &format!("switch to {}", r.name), |g| {
                region::switch(g, r).then_some(r.name)
            });
            // the client is somewhere the switch does not know: the agent restarts the game
            if current.is_none() {
                panic!("switch to {} failed", r.name);
            }
        }
        n += pass(g, conn, r.name, o);
    }
    n
}
