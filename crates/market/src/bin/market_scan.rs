// Scans the Market, the Auction House and the currency exchange of each region in turn (switching character,
// region.rs), uploads each pass (upload.json), then waits a gap and scans again, until F10 or Ctrl+C. F10 stores
// and uploads what the pass read so far. --strips keeps every crop read (strips.rs). With one region the
// logged-in character is used as is.
// market_scan --help for the switches.
use clap::Parser;
use hf_market::{
    db,
    game::{Game, Stopped, check_abort, ms, quiet_stops},
    run,
    ui::log,
    upload,
};

#[derive(Parser)]
struct Cli {
    /// Minutes between the end of a round and the next
    #[arg(long, default_value_t = 60.0)]
    gap: f64,
    /// One round, then exit
    #[arg(long)]
    once: bool,
    /// Send the passes not sent yet, then exit
    #[arg(long)]
    upload_only: bool,
    #[arg(long)]
    db: Option<String>,
    #[command(flatten)]
    pass: run::Args,
}

fn main() {
    let cli = Cli::parse();
    let (gap, once) = (cli.gap, cli.once);
    let o = cli.pass.opts();
    let conn = db::open(&cli.db.unwrap_or(db::path()));
    if cli.upload_only {
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
