// The scanner for a PC of its own, controlled through the relay (cloudflare/relay): connects with data/relay.json
// ({"url", "machine", "token"}), then once started (Enter here, /start on Discord, or a remote command) scans
// round after round with no gap until paused (F10 here, /pause). Remote commands also take screenshots, run
// market_ctl scripts while paused, launch the game through Steam, and switch setup mode (a screenshot after
// every unit of work). Portable layout (profile/ and data/ beside the exe) unless --dev (the repo's folders).
// A round that breaks or reads nothing (lost connection, game closed or updated) closes the game, waits for the
// network, relaunches it and goes on; three in a row and it stops. It replaces itself with the relay's newest
// build at the start and at every round, restarting in the same console (--resume: scanning without asking);
// only a newer build signed by the publisher's key (update_key.hex) is taken.
// market_agent [--dev] [--setup] [--quick] [--strips] [--only materials,gems,exchange] [--regions NAE,EUC]
use base64::Engine;
use hf_market::{
    assets, ctl, db,
    game::{self, Game, Stopped, check_abort, ms, panic_text, quiet_stops},
    region, report, run,
    ui::{self, log, looks},
    upload,
};
use serde_json::{Value, json};
use std::{
    io::BufRead,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use tungstenite::{Message, client::IntoClientRequest, stream::MaybeTlsStream};

const STEAM_APP: &str = "steam://rungameid/1599340";

#[derive(serde::Deserialize, Clone, Debug)]
struct Relay {
    url: String,
    machine: String,
    token: String,
}

static RELAY: OnceLock<Relay> = OnceLock::new();
static STATE: Mutex<&str> = Mutex::new("idle");
static LAST_PASS: Mutex<String> = Mutex::new(String::new());
static CONNECTED: AtomicBool = AtomicBool::new(false);
// messages for the relay, sent by its thread
static OUT: OnceLock<Mutex<mpsc::Sender<String>>> = OnceLock::new();
// "start" from the console or the relay, taken by the main loop while idle
static START: OnceLock<Mutex<mpsc::Sender<()>>> = OnceLock::new();

fn relay() -> &'static Relay {
    RELAY.get().unwrap()
}

fn state() -> &'static str {
    *STATE.lock().unwrap()
}

fn set_state(s: &'static str) {
    *STATE.lock().unwrap() = s;
    send(status());
}

fn send(v: Value) {
    OUT.get()
        .unwrap()
        .lock()
        .unwrap()
        .send(v.to_string())
        .unwrap();
}

fn status() -> Value {
    let (log, log_at) = ui::LAST_LOG.lock().unwrap().clone();
    json!({"type": "status", "status": {
        "state": state(),
        "game": game::running(),
        "last_pass": *LAST_PASS.lock().unwrap(),
        "log": log,
        "log_at": log_at,
        "setup": report::SETUP.load(Ordering::Relaxed),
        "version": build(),
    }})
}

fn png(img: &image::RgbaImage) -> Vec<u8> {
    let mut b = std::io::Cursor::new(vec![]);
    img.write_to(&mut b, image::ImageFormat::Png).unwrap();
    b.into_inner()
}

/// Puts a file in the relay's store under this machine; its key, None when that failed.
fn put_file(path: &str, body: &[u8]) -> Option<String> {
    let r = relay();
    let mut res = ureq::put(&format!("{}/files/{}/{}", r.url, r.machine, encode(path)))
        .header("Authorization", &format!("Bearer {}", r.token))
        .send(body)
        .ok()?;
    let v: Value = serde_json::from_str(&res.body_mut().read_to_string().ok()?).ok()?;
    Some(v["key"].as_str()?.to_string())
}

/// The profile's templates, to the relay's store (files/<machine>/profile/...).
fn sync_profile() -> String {
    let mut n = 0;
    for sub in ["refs", "names", "gems"] {
        for e in std::fs::read_dir(assets::profile().join(sub))
            .into_iter()
            .flatten()
        {
            let p = e.unwrap().path();
            let name = p.file_name().unwrap().to_str().unwrap();
            put_file(
                &format!("profile/{sub}/{name}"),
                &std::fs::read(&p).unwrap(),
            )
            .unwrap();
            n += 1;
        }
    }
    format!("{n} profile files uploaded")
}

fn tail_log(n: usize) -> String {
    let text = std::fs::read_to_string(assets::data().join("scan.log")).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

fn sha_hex(b: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}

/// This build's number (unix seconds, set by cloudflare/relay/publish.mjs); 0 for a local build.
fn build() -> u64 {
    option_env!("MARKET_BUILD").map_or(0, |s| s.parse().unwrap())
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap_or(0))
        .collect()
}

/// Whether the publisher's key (private key only on the publishing PC) signed this build and hash.
fn signed(build: u64, sha: &str, sig: &str) -> bool {
    let key = unhex(include_str!("../../update_key.hex").trim());
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key)
        .verify(
            format!("hf-market-agent:{build}:{sha}").as_bytes(),
            &unhex(sig),
        )
        .is_ok()
}

fn get(path: &str) -> Option<Vec<u8>> {
    let r = relay();
    let mut res = ureq::get(&format!("{}{path}", r.url))
        .header("Authorization", &format!("Bearer {}", r.token))
        .call()
        .ok()?;
    res.body_mut()
        .with_config()
        .limit(200 << 20)
        .read_to_vec()
        .ok()
}

/// Replaces this exe with the relay's build when it is newer and signed, then restarts in this console, still
/// scanning when `scanning`; otherwise says why not.
fn update(scanning: bool) -> String {
    let Some(v) = get("/update") else {
        return "could not check for an update".into();
    };
    let v: Value = serde_json::from_slice(&v).unwrap();
    let (Some(want), Some(n), Some(sig)) = (
        v["sha256"].as_str(),
        v["build"].as_str().and_then(|s| s.parse::<u64>().ok()),
        v["sig"].as_str(),
    ) else {
        return "no signed build published".into();
    };
    if n <= build() {
        return format!("up to date ({})", build());
    }
    if !signed(n, want, sig) {
        let text = format!("refused build {n}: bad signature");
        report::event("error", &text, None);
        return text;
    }
    let Some(exe) = get("/update/exe").filter(|b| sha_hex(b) == want) else {
        return "the download failed".into();
    };
    let tmp = assets::data().join("update.exe");
    std::fs::write(&tmp, exe).unwrap();
    self_replace::self_replace(&tmp).unwrap();
    std::fs::remove_file(&tmp).unwrap();
    let text = format!("updated {} -> {n}, restarting", build());
    log(&text);
    report::event("info", &text, None);
    // time for the relay thread to send it
    ms(2000);
    let mut args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| a != "--resume")
        .collect();
    if scanning {
        args.push("--resume".into());
    }
    std::process::Command::new(std::env::current_exe().unwrap())
        .args(args)
        .spawn()
        .unwrap();
    std::process::exit(0);
}

fn launch() -> String {
    if game::running() {
        return "the game is already running".into();
    }
    std::process::Command::new("cmd")
        .args(["/C", "start", "", STEAM_APP])
        .spawn()
        .unwrap();
    log("launching the game through Steam");
    "launching the game through Steam".into()
}

/// A command from the relay; its reply.
fn command(cmd: &Value) -> Value {
    let op = cmd["op"].as_str().unwrap_or("");
    let idle = state() == "idle";
    let text = |t: String| json!({"ok": true, "text": t});
    match op {
        "status" => text(status()["status"].to_string()),
        "start" if idle => {
            START.get().unwrap().lock().unwrap().send(()).unwrap();
            text("starting".into())
        }
        "start" => text(format!("already {}", state())),
        "pause" if state() == "scanning" => {
            game::STOP.store(true, Ordering::Relaxed);
            set_state("stopping");
            text("stopping; what this pass read so far is uploaded".into())
        }
        "pause" => text(format!("not scanning ({})", state())),
        "launch" => text(launch()),
        "log" => text(tail_log(cmd["lines"].as_u64().unwrap_or(30) as usize)),
        "setup" => {
            let on = cmd["on"].as_bool().unwrap_or(true);
            report::SETUP.store(on, Ordering::Relaxed);
            text(format!("setup mode {}", if on { "on" } else { "off" }))
        }
        "sync" => text(sync_profile()),
        "update" if idle => {
            std::thread::spawn(|| {
                ms(1000);
                let why = update(false);
                log(&format!("update: {why}"));
                report::event("info", &format!("update: {why}"), None);
            });
            text("checking for an update".into())
        }
        "update" => text("checked at the start of every round".into()),
        "shot" | "run" if !game::running() => {
            json!({"ok": false, "text": "the game is not running"})
        }
        "shot" => {
            let v: Vec<u32> = ["x", "y", "w", "h", "scale"]
                .iter()
                .filter_map(|k| cmd[k].as_u64().map(|n| n as u32))
                .collect();
            let img = ctl::shot(&mut Game::find(), &v);
            let b64 = base64::engine::general_purpose::STANDARD.encode(png(&img));
            json!({"ok": true, "text": "", "png": b64})
        }
        "run" if !idle => json!({"ok": false, "text": format!("{} (pause first)", state())}),
        "run" => {
            let script = cmd["script"].as_str().unwrap_or("");
            let out = ctl::run(&mut Game::find(), script);
            if script.contains("ref ") {
                sync_profile();
            }
            text(if out.is_empty() { "ok".into() } else { out })
        }
        _ => json!({"ok": false, "text": format!("unknown op {op}")}),
    }
}

/// A path for a URL: every byte but letters, digits and -._~/@ percent-encoded.
fn encode(path: &str) -> String {
    path.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b'@' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn ws_url() -> String {
    let r = relay();
    let base = r
        .url
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    format!("{base}/agent/{}", r.machine)
}

/// Keeps a WebSocket to the relay: answers commands, sends the status every 30 s and every queued message.
fn relay_thread(out: mpsc::Receiver<String>) {
    let mut wait = 5;
    loop {
        let mut req = ws_url().into_client_request().unwrap();
        req.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", relay().token).parse().unwrap(),
        );
        let mut ws = match tungstenite::connect(req) {
            Ok((ws, _)) => ws,
            Err(e) => {
                if CONNECTED.swap(false, Ordering::Relaxed) || wait == 5 {
                    log(&format!("relay: cannot connect ({e}), trying again"));
                }
                std::thread::sleep(Duration::from_secs(wait));
                wait = (wait * 2).min(60);
                continue;
            }
        };
        let timeout = Some(Duration::from_millis(500));
        match ws.get_ref() {
            MaybeTlsStream::Plain(s) => s.set_read_timeout(timeout).unwrap(),
            MaybeTlsStream::Rustls(s) => s.get_ref().set_read_timeout(timeout).unwrap(),
            _ => {}
        }
        wait = 5;
        CONNECTED.store(true, Ordering::Relaxed);
        report::OFFLINE.store(false, Ordering::Relaxed);
        log("relay: connected");
        let mut last_status = Instant::now() - Duration::from_secs(60);
        // a dead connection only shows as silence: "ping" every 30 s, the relay answers "pong"
        let mut last_rx = Instant::now();
        loop {
            let read = ws.read();
            if read.is_ok() {
                last_rx = Instant::now();
            }
            match read {
                Ok(Message::Text(t)) if t.as_str() == "pong" => {}
                Ok(Message::Text(t)) => {
                    let cmd: Value = serde_json::from_str(&t).unwrap();
                    let mut reply = catch_unwind(|| command(&cmd))
                        .unwrap_or_else(|e| json!({"ok": false, "text": panic_text(e.as_ref())}));
                    reply["type"] = "reply".into();
                    reply["id"] = cmd["id"].clone();
                    if ws.send(Message::text(reply.to_string())).is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => break,
            }
            let mut ok = last_rx.elapsed() < Duration::from_secs(75);
            if last_status.elapsed() > Duration::from_secs(30) {
                ok &= ws.send(Message::text(status().to_string())).is_ok();
                ok &= ws.send(Message::text("ping")).is_ok();
                last_status = Instant::now();
            }
            while let Ok(m) = out.try_recv() {
                ok &= ws.send(Message::text(m)).is_ok();
            }
            if !ok {
                break;
            }
        }
        CONNECTED.store(false, Ordering::Relaxed);
        report::OFFLINE.store(true, Ordering::Relaxed);
        log("relay: disconnected, reconnecting");
    }
}

/// Starts the game when it is not running and waits for character or server select (Steam may update it first).
fn ensure_game(current: &mut Option<&'static str>) {
    if game::running() {
        return;
    }
    *current = None;
    launch();
    let end = Instant::now() + Duration::from_secs(1800);
    while Instant::now() < end {
        check_abort();
        ms(2000);
        let Some(mut g) = Game::try_find() else {
            continue;
        };
        // the splash window comes first, smaller than the game
        let up = catch_unwind(AssertUnwindSafe(|| {
            let img = g.shot(None);
            img.dimensions() == (1920, 1080)
                && (looks(&img, "char_select") || looks(&img, "server_select"))
        }));
        if up.unwrap_or(false) {
            log("the game is up");
            return;
        }
    }
    panic!("the game did not reach character or server select in 30 minutes");
}

fn wait_secs(s: u64) {
    for _ in 0..s {
        check_abort();
        ms(1000);
    }
}

/// Closes the game and waits for the network; the next round launches it again.
fn recover(current: &mut Option<&'static str>) {
    *current = None;
    log("recovering: closing the game");
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "LOSTARK.exe"])
        .output();
    while game::running() {
        wait_secs(1);
    }
    if !CONNECTED.load(Ordering::Relaxed) {
        log("recovering: waiting for the network");
        while !CONNECTED.load(Ordering::Relaxed) {
            wait_secs(1);
        }
    }
    // Steam needs a moment after the game closed
    wait_secs(30);
}

/// Rounds until paused, recovering from a round that breaks or reads nothing; panics after three in a row.
fn scan(conn: &rusqlite::Connection, o: &run::Opts, current: &mut Option<&'static str>) {
    let mut strikes = 0;
    loop {
        update(true);
        let n = catch_unwind(AssertUnwindSafe(|| {
            ensure_game(current);
            run::round(&mut Game::find(), conn, o, current)
        }));
        // a screenshot after every unit is a few MB each: one round of it at most
        if report::SETUP.swap(false, Ordering::Relaxed) {
            log("setup mode off after a round");
            report::event("info", "setup mode off after a round", None);
        }
        let why = match n {
            Ok(n) if n > 0 => {
                strikes = 0;
                continue;
            }
            Ok(_) => "a round read nothing".to_string(),
            Err(e) if e.is::<Stopped>() => std::panic::resume_unwind(e),
            Err(e) => panic_text(e.as_ref()),
        };
        strikes += 1;
        if strikes > 3 {
            panic!("gave up after 3 recoveries in a row: {why}");
        }
        let img = catch_unwind(|| Game::find().shot(None)).ok();
        log(&format!("{why}; recovering ({strikes}/3)"));
        report::event(
            "error",
            &format!("{why}; restarting the game ({strikes}/3)"),
            img.as_ref(),
        );
        recover(current);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let has = |k: &str| args.iter().any(|a| a == k);
    let arg = |k: &str| {
        args.iter()
            .position(|a| a == k)
            .map(|i| args[i + 1].clone())
    };
    if !has("--dev") {
        assets::portable();
    }
    report::SETUP.store(has("--setup"), Ordering::Relaxed);
    ui::QUICK.store(has("--quick"), Ordering::Relaxed);
    let o = run::Opts {
        only: arg("--only").unwrap_or("materials,gems,exchange".into()),
        strips: has("--strips"),
        regions: arg("--regions")
            .unwrap_or("NAE,EUC".into())
            .split(',')
            .map(region::get)
            .collect(),
    };
    let cfg_path = assets::data().join("relay.json");
    let Ok(cfg) = std::fs::read_to_string(&cfg_path) else {
        println!("{} is missing", cfg_path.display());
        std::io::stdin().lines().next();
        return;
    };
    let cfg: Relay = serde_json::from_str(&cfg).unwrap();
    upload::MACHINE.set(cfg.machine.clone()).unwrap();
    RELAY.set(cfg).unwrap();
    quiet_stops();

    let (out_tx, out_rx) = mpsc::channel();
    OUT.set(Mutex::new(out_tx)).unwrap();
    let (start_tx, start_rx) = mpsc::channel();
    START.set(Mutex::new(start_tx.clone())).unwrap();
    report::set(|kind, text, img| {
        if kind == "pass" {
            *LAST_PASS.lock().unwrap() = text.to_string();
        }
        let file = img.and_then(|img| {
            let folder = if kind == "step" { "steps" } else { "shots" };
            let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
            let what: String = text
                .chars()
                .take(40)
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            put_file(&format!("{folder}/{stamp}_{what}.png"), &png(img))
        });
        send(json!({"type": "event", "kind": kind, "text": text, "file": file}));
    });
    std::thread::spawn(move || relay_thread(out_rx));
    // console: Enter or y starts while idle
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let l = line.unwrap().trim().to_lowercase();
            if (l.is_empty() || l == "y") && state() == "idle" && CONNECTED.load(Ordering::Relaxed)
            {
                start_tx.send(()).unwrap();
            }
        }
    });

    println!(
        "market_agent: machine {} via {}",
        relay().machine,
        relay().url
    );
    while !CONNECTED.load(Ordering::Relaxed) {
        ms(200);
    }
    let resume = has("--resume");
    if !resume {
        log(&format!("update: {}", update(false)));
    }
    let conn = db::open(&db::path());
    upload::send_pending(&conn);
    let mut current: Option<&'static str> = None;
    let mut first = true;
    loop {
        game::STOP.store(false, Ordering::Relaxed);
        set_state("idle");
        if !(first && resume) {
            while start_rx.try_recv().is_ok() {}
            println!(
                "\nStart scanning? [Y/n]  (the game is {}running; F10 or /pause stops)",
                if game::running() { "" } else { "not " }
            );
            start_rx.recv().unwrap();
        }
        first = false;
        set_state("scanning");
        report::event("info", "started", None);
        let r = catch_unwind(AssertUnwindSafe(|| scan(&conn, &o, &mut current)));
        let Err(e) = r else { unreachable!() };
        if e.is::<Stopped>() {
            log("paused");
            report::event("info", "paused", None);
        } else {
            let text = panic_text(e.as_ref());
            log(&format!("failed: {text}"));
            current = None;
            let img = catch_unwind(|| Game::find().shot(None)).ok();
            report::event("error", &format!("failed, now idle: {text}"), img.as_ref());
        }
    }
}
