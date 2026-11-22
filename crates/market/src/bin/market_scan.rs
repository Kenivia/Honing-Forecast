// Scans the Market, the Auction House and the currency exchange of each region in turn (switching character,
// region.rs), uploads each pass (upload.json), then waits a gap and scans again, until F10 or Ctrl+C. F10 stores
// and uploads what the pass read so far. --strips keeps every crop read (strips.rs). With one region the
// logged-in character is used as is.
// market_scan [--upload-only] [--gap MINUTES] [--once] [--quick] [--strips]
//     [--only materials,gems,accessories,exchange] [--regions NAE,EUC] [--db PATH]
use hf_market::{
    db,
    game::{Game, Stopped, check_abort, ms, quiet_stops},
    region, run,
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
    let o = run::Opts {
        only: arg("--only").unwrap_or("materials,gems,accessories,exchange".into()),
        strips: args.iter().any(|a| a == "--strips"),
        regions: arg("--regions")
            .unwrap_or("NAE,EUC".into())
            .split(',')
            .map(region::get)
            .collect(),
    };
    let conn = db::open(&arg("--db").unwrap_or(db::path()));
    if args.iter().any(|a| a == "--upload-only") {
        upload::send_pending(&conn);
        return;
    }
    quiet_stops();
    let mut g = Game::find();
    // the region logged in; unknown at the start unless only one is scanned
    let mut current = (o.regions.len() == 1).then(|| o.regions[0].name);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        loop {
            run::round(&mut g, &conn, &o, &mut current);
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
    }));
    if let Err(e) = r {
        if e.is::<Stopped>() {
            eprintln!("stopped (F10 held)");
            std::process::exit(2);
        }
        std::panic::resume_unwind(e);
    }
}
