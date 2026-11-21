// A pass as one JSON for the site's Worker (cloudflare/worker.js, POST /scans), and sending it.
use crate::{auction::Search, db, exchange::Exchange, market::Material, ui::log};
use chrono::{DateTime, Days, SecondsFormat, Utc};
use rusqlite::Connection;
use serde_json::{Map, Value, json};

/// When an item was read (UTC, to the second): each material, search and exchange read carries one.
pub fn stamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// `{"url": "<Worker>", "token": "<MARKET_TOKEN>"}`; without it nothing is uploaded.
pub const CONFIG: &str = concat!(env!("HF_ROOT"), "/ignored/market/upload.json");

/// Everything read, by section, plus `points` [key, day, value] that the Worker merges into its daily history.
/// The 5-day averages are of complete days: the first is taken as the day before the scan (UTC), unconfirmed.
pub fn payload(
    region: &str,
    started: DateTime<Utc>,
    finished: DateTime<Utc>,
    materials: &[Material],
    searches: &[Search],
    exchange: Option<&Exchange>,
) -> Value {
    let iso = |t: DateTime<Utc>| t.to_rfc3339_opts(SecondsFormat::Secs, true);
    let day = |ago: usize| (started.date_naive() - Days::new(ago as u64)).to_string();
    let mut points = vec![];
    for m in materials {
        for (k, a) in m.history.iter().enumerate() {
            if let Some(a) = a {
                points.push(json!([format!("material/{}/avg", m.name), day(k + 1), a]));
            }
        }
        if let Some((p, _)) = m.depth.first() {
            points.push(json!([format!("material/{}/lowest", m.name), day(0), p]));
        }
        if let Some(r) = m.recent {
            points.push(json!([format!("material/{}/recent", m.name), day(0), r]));
        }
    }
    if let Some(ex) = exchange {
        let best = |v: &[crate::exchange::Offer], high: bool| {
            v.iter()
                .map(|o| o.price)
                .reduce(if high { f64::max } else { f64::min })
        };
        for (key, v) in [
            ("royal_sale/lowest", best(&ex.royal_sale, false)),
            ("crystal_purchase/highest", best(&ex.crystal_purchase, true)),
            ("recent/latest", ex.recent.first().map(|t| t.price)),
        ] {
            if let Some(v) = v {
                points.push(json!([format!("exchange/{key}"), day(0), v]));
            }
        }
    }
    let mut sections = Map::new();
    sections.insert("material".into(), json!(materials));
    if let Some(ex) = exchange {
        sections.insert("exchange".into(), json!(ex));
    }
    for s in searches {
        let key = match (s.gem_level, s.level_a, s.level_b) {
            (Some(l), ..) => format!("Lv. {l} {}", s.item),
            (_, Some(a), Some(b)) => format!("{} {} {a}/{b}", s.item, s.category),
            _ => s.item.clone(),
        };
        if let Some(b) = s.rows.iter().filter_map(|r| r.buy_now).reduce(f64::min) {
            points.push(json!([format!("{}/{key}/lowest", s.kind), day(0), b]));
        }
        let list = sections.entry(s.kind).or_insert(json!([]));
        list.as_array_mut().unwrap().push(json!(s));
    }
    json!({
        "version": 1,
        "region": region.to_lowercase(),
        "started": iso(started),
        "finished": iso(finished),
        "sections": sections,
        "points": points,
    })
}

/// Sends every pass not sent yet, oldest first; stops at the first failure, to try again next pass.
pub fn send_pending(c: &Connection) {
    let Ok(text) = std::fs::read_to_string(CONFIG) else {
        log("no ignored/market/upload.json, not uploading");
        return;
    };
    let cfg: Value = serde_json::from_str(&text).unwrap();
    let url = format!("{}/scans", cfg["url"].as_str().unwrap());
    let auth = format!("Bearer {}", cfg["token"].as_str().unwrap());
    for (id, body) in db::unsent(c) {
        let sent = ureq::post(&url)
            .header("Authorization", &auth)
            .content_type("application/json")
            .send(&body);
        match sent {
            Ok(mut r) => {
                let reply = r.body_mut().read_to_string().unwrap_or_default();
                log(&format!(
                    "scan {id} uploaded ({} KB): {reply}",
                    body.len() / 1000
                ));
                db::uploaded(c, id);
            }
            Err(e) => {
                log(&format!(
                    "upload of scan {id} failed, again after the next pass: {e}"
                ));
                return;
            }
        }
    }
}
