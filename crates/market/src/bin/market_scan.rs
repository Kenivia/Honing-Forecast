// Scans the Market, the Auction House and the currency exchange, uploads the pass (ignored/market/upload.json),
// then waits a gap and scans again, until F10 or Ctrl+C. --strips keeps every crop read (strips.rs).
// market_scan [--upload-only] [--gap MINUTES] [--once] [--quick] [--strips]
//     [--only materials,gems,accessories,exchange] [--region NAE] [--db PATH]
use hf_market::{
    auction, db, exchange,
    game::{Game, check_abort, ms},
    market, strips,
    ui::{self, log},
    upload,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| {
        args.iter()
            .position(|a| a == k)
            .map(|i| args[i + 1].clone())
    };
    let gap: f64 = arg("--gap").map_or(60.0, |v| v.parse().unwrap());
    let once = args.iter().any(|a| a == "--once");
    ui::QUICK.store(
        args.iter().any(|a| a == "--quick"),
        std::sync::atomic::Ordering::Relaxed,
    );
    let only = arg("--only").unwrap_or("materials,gems,accessories,exchange".into());
    let keep_strips = args.iter().any(|a| a == "--strips");
    let region = arg("--region").unwrap_or("NAE".into());
    let conn = db::open(&arg("--db").unwrap_or(db::PATH.into()));
    if args.iter().any(|a| a == "--upload-only") {
        upload::send_pending(&conn);
        return;
    }
    let mut g = Game::find();
    loop {
        let scan = db::start(&conn, &region);
        let started = chrono::Utc::now();
        let (mut mats, mut found, mut ex) = (vec![], vec![], None);
        if keep_strips {
            strips::start();
        }
        log(&format!("scan {scan} started ({only})"));
        if only.contains("materials") {
            mats = market::scan(&mut g);
            db::materials(&conn, scan, &mats);
            log(&format!("{} materials", mats.len()));
        }
        if only.contains("gems") {
            let s = auction::gems(&mut g);
            db::searches(&conn, scan, &s);
            log(&format!("{} gem searches", s.len()));
            found.extend(s);
        }
        if only.contains("accessories") {
            let s = auction::accessories(&mut g);
            db::searches(&conn, scan, &s);
            log(&format!("{} accessory searches", s.len()));
            found.extend(s);
        }
        if only.contains("exchange") {
            ex = exchange::scan(&mut g);
            log(&format!("currency exchange: {ex:?}"));
        }
        db::finish(&conn, scan);
        ui::reset(&mut g);
        let p = upload::payload(
            &region,
            started,
            chrono::Utc::now(),
            &mats,
            &found,
            ex.as_ref(),
        );
        db::set_payload(&conn, scan, &p);
        upload::send_pending(&conn);
        log(&format!("scan {scan} done"));
        if once {
            break;
        }
        log(&format!("next scan in {gap} minutes; hold F10 to stop"));
        let until = std::time::Instant::now() + std::time::Duration::from_secs_f64(gap * 60.0);
        while std::time::Instant::now() < until {
            check_abort();
            ms(200);
        }
    }
}
