// Scans the Market and Auction House, then waits a gap and scans again, until F10 or Ctrl+C.
// market_scan [--gap MINUTES] [--once] [--only materials,gems,accessories] [--region NAE] [--db PATH]
use hf_market::{
    auction, db,
    game::{Game, check_abort, ms},
    market,
    ui::log,
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
    let only = arg("--only").unwrap_or("materials,gems,accessories".into());
    let region = arg("--region").unwrap_or("NAE".into());
    let conn = db::open(&arg("--db").unwrap_or(db::PATH.into()));
    let mut g = Game::find();
    loop {
        let scan = db::start(&conn, &region);
        log(&format!("scan {scan} started ({only})"));
        if only.contains("materials") {
            let m = market::scan(&mut g);
            db::materials(&conn, scan, &m);
            log(&format!("{} materials", m.len()));
        }
        if only.contains("gems") {
            let s = auction::gems(&mut g);
            db::searches(&conn, scan, &s);
            log(&format!("{} gem searches", s.len()));
        }
        if only.contains("accessories") {
            let s = auction::accessories(&mut g);
            db::searches(&conn, scan, &s);
            log(&format!("{} accessory searches", s.len()));
        }
        db::finish(&conn, scan);
        g.key("alt+y");
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
