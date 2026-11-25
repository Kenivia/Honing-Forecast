// The scanner for a PC of its own, controlled through the relay (cloudflare/relay): connects with data/relay.json
// ({"url", "machine", "token"}), then once started (Enter here, /start on Discord, or a remote command) scans
// a round at once, then one at every multiple of --gap minutes UTC (default 120; 0 = no gap), the game closed
// in between (the screen kept on: once off it wakes to sign-in), until paused (F10 here, /pause). Remote
// commands also take screenshots, run market_ctl scripts while paused, launch the game through Steam, and
// switch setup mode (a screenshot after every unit of work). Portable layout (profile/ and data/ beside the
// exe) unless --dev (the repo's folders).
// A round that breaks or reads nothing (lost connection, game closed or updated) closes the game, waits for the
// network, relaunches it and goes on; three in a row and it stops. It replaces itself with the relay's newest
// build at the start and at every round, restarting in the same console (--resume: scanning without asking);
// only a newer build signed by the publisher's key (update_key.hex) is taken. Commands other than status, start,
// pause and gap must be signed by that key too (sign.rs); `exec` runs a PowerShell script.
// market_agent [--dev] [--setup] [--quick] [--strips] [--only materials,gems,exchange] [--regions NAE,EUC]
//     [--gap MINUTES]
use base64::Engine;
use hf_market::{
    assets, ctl, db,
    game::{self, Closed, Game, Stopped, check_abort, ms, panic_text, quiet_stops},
    input::awake,
    region, report, run, sign,
    ui::{self, log, looks},
    upload,
};
use serde_json::{Value, json};
use std::{
    io::BufRead,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicI64, Ordering},
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
// minutes between round starts (0: none), and the next start while waiting (unix s, 0 otherwise)
static GAP: AtomicI64 = AtomicI64::new(120);
static NEXT: AtomicI64 = AtomicI64::new(0);
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
        "gap": GAP.load(Ordering::Relaxed),
        "next": next_round(),
    }})
}

/// The next round's start (unix s): while scanning, the next slot after now (a round running past it moves it
/// on); 0 when idle or with no gap.
fn next_round() -> i64 {
    let g = GAP.load(Ordering::Relaxed) * 60;
    match state() {
        "waiting" => NEXT.load(Ordering::Relaxed),
        "scanning" if g > 0 => (chrono::Utc::now().timestamp() / g + 1) * g,
        _ => 0,
    }
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
    sign::hex(&sha2::Sha256::digest(b))
}

/// This build's number (unix seconds, set by cloudflare/relay/publish.mjs); 0 for a local build.
fn build() -> u64 {
    option_env!("MARKET_BUILD").map_or(0, |s| s.parse().unwrap())
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
    if !sign::verify(&format!("hf-market-agent:{n}:{want}"), sig) {
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

/// A program in Windows' System32 by full path: a bare name would be looked for beside this exe first.
fn system32(exe: &str) -> std::process::Command {
    let root = std::env::var("SystemRoot").unwrap_or("C:\\Windows".into());
    std::process::Command::new(format!("{root}\\System32\\{exe}"))
}

fn launch() -> String {
    if game::running() {
        return "the game is already running".into();
    }
    system32("cmd.exe")
        .args(["/C", "start", "", STEAM_APP])
        .spawn()
        .unwrap();
    log("launching the game through Steam");
    "launching the game through Steam".into()
}

/// How old a signed command may be (its counter is the signer's unix ms).
const CMD_MAX_AGE_MS: u64 = 10 * 60 * 1000;

/// The command to run: status, start, pause and gap as sent (Discord), anything else only when signed by the
/// publisher's key for this machine with a counter above the last one taken (no replays) and under
/// CMD_MAX_AGE_MS old (a signed command never delivered does not stay valid).
fn authentic(msg: &Value) -> Result<Value, String> {
    if sign::UNSIGNED.contains(&msg["op"].as_str().unwrap_or("")) {
        return Ok(msg.clone());
    }
    let (Some(body), Some(sig)) = (msg["signed"].as_str(), msg["sig"].as_str()) else {
        return Err("refused: unsigned".into());
    };
    if !sign::verify(&format!("hf-market-cmd:{}:{body}", relay().machine), sig) {
        report::event("error", "refused a command: bad signature", None);
        return Err("refused: bad signature".into());
    }
    let cmd: Value = serde_json::from_str(body).unwrap();
    let n = cmd["n"].as_u64().unwrap();
    let now = chrono::Utc::now().timestamp_millis() as u64;
    if n + CMD_MAX_AGE_MS < now {
        return Err(format!("refused: signed {} s ago", (now - n) / 1000));
    }
    let file = assets::data().join("cmd_counter");
    let last: u64 = std::fs::read_to_string(&file)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    if n <= last {
        return Err(format!("refused: counter {n} not above {last}"));
    }
    std::fs::write(&file, n.to_string()).unwrap();
    Ok(cmd)
}

/// Runs a PowerShell script; its exit code and output as text, every stream merged (the last 200 KB).
fn exec(script: &str) -> String {
    let wrapped = format!(
        "$ProgressPreference = 'SilentlyContinue'\ntry {{ & {{\n{script}\n}} *>&1 | Out-String -Stream -Width 250 }} catch {{ $_ | Out-String -Width 250; exit 1 }}\nexit $LASTEXITCODE"
    );
    let utf16: Vec<u8> = wrapped.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let out = system32("WindowsPowerShell\\v1.0\\powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-EncodedCommand"])
        .arg(base64::engine::general_purpose::STANDARD.encode(utf16))
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    let mut cut = text.len().saturating_sub(200_000);
    while !text.is_char_boundary(cut) {
        cut += 1;
    }
    format!("exit {}\n{}", out.status.code().unwrap_or(-1), &text[cut..])
}

/// A command from the relay; its reply, or Null when it replies later itself.
fn command(msg: &Value) -> Value {
    let cmd = match authentic(msg) {
        Ok(c) => c,
        Err(t) => return json!({"ok": false, "text": t}),
    };
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
        "pause" if state() == "scanning" || state() == "waiting" => {
            game::STOP.store(true, Ordering::Relaxed);
            set_state("stopping");
            text("stopping; what this pass read so far is uploaded".into())
        }
        "pause" => text(format!("not scanning ({})", state())),
        "gap" => match set_gap(&cmd["minutes"]) {
            Ok(m) => {
                log(&format!("gap set to {m} min"));
                send(status());
                text(if m == 0 {
                    "no gap: rounds back to back, the game kept open".into()
                } else {
                    format!("gap set to {m} min (rounds start at multiples of it, UTC)")
                })
            }
            Err(t) => json!({"ok": false, "text": t}),
        },
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
        "exec" => {
            let id = msg["id"].clone();
            let script = cmd["script"].as_str().unwrap_or("").to_string();
            // not the script: the last log line reaches the relay and Discord (/status)
            log("exec: a command was executed");
            // off the relay thread, which must keep pinging
            std::thread::spawn(move || {
                let text =
                    catch_unwind(|| exec(&script)).unwrap_or_else(|e| panic_text(e.as_ref()));
                send(json!({"type": "reply", "id": id, "ok": true, "text": text}));
            });
            Value::Null
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
                    if reply.is_null() {
                        continue;
                    }
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
        let img = catch_unwind(AssertUnwindSafe(|| g.shot(None)))
            .ok()
            .filter(|img| img.dimensions() == (1920, 1080));
        let Some(img) = img else {
            continue;
        };
        ui::check_closed(&img);
        if looks(&img, "char_select") || looks(&img, "server_select") {
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

fn close_game(current: &mut Option<&'static str>) {
    *current = None;
    let _ = system32("taskkill.exe")
        .args(["/F", "/IM", "LOSTARK.exe"])
        .output();
    while game::running() {
        wait_secs(1);
    }
}

/// Closes the game and waits for the next multiple of the gap after now, UTC (with no gap, `secs` from now);
/// a gap changed meanwhile counts.
fn rest(current: &mut Option<&'static str>, secs: i64) {
    close_game(current);
    let end = chrono::Utc::now().timestamp();
    set_state("waiting");
    loop {
        let g = GAP.load(Ordering::Relaxed) * 60;
        let next = if g == 0 {
            end + secs
        } else {
            (end / g + 1) * g
        };
        if NEXT.swap(next, Ordering::Relaxed) != next {
            let at = chrono::DateTime::from_timestamp(next, 0).unwrap();
            log(&format!("next round at {} UTC", at.format("%H:%M")));
            send(status());
        }
        if chrono::Utc::now().timestamp() >= next {
            break;
        }
        wait_secs(1);
    }
    NEXT.store(0, Ordering::Relaxed);
    set_state("scanning");
}

/// Minutes between round starts: whole, 0 to a day (Discord may set it: nothing else is taken unsigned).
fn set_gap(v: &Value) -> Result<i64, String> {
    let m = v
        .as_u64()
        .filter(|m| *m <= 1440)
        .ok_or("refused: minutes must be a whole number from 0 to 1440")?;
    GAP.store(m as i64, Ordering::Relaxed);
    std::fs::write(assets::data().join("gap"), m.to_string()).unwrap();
    Ok(m as i64)
}

/// Closes the game and waits for the network; the next round launches it again.
fn recover(current: &mut Option<&'static str>) {
    log("recovering: closing the game");
    close_game(current);
    if !CONNECTED.load(Ordering::Relaxed) {
        log("recovering: waiting for the network");
        while !CONNECTED.load(Ordering::Relaxed) {
            wait_secs(1);
        }
    }
    // Steam needs a moment after the game closed
    wait_secs(30);
}

/// Rounds until paused, GAP minutes apart, recovering from a round that breaks or reads nothing (at once);
/// panics after three in a row. A closed region (maintenance) skips to the next slot, with no gap 4 hours
/// and then every 2.
fn scan(conn: &rusqlite::Connection, o: &run::Opts, current: &mut Option<&'static str>) {
    let mut strikes = 0;
    let mut closed = false;
    awake(true);
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
                closed = false;
                if GAP.load(Ordering::Relaxed) > 0 {
                    rest(current, 0);
                }
                continue;
            }
            Ok(_) => "a round read nothing".to_string(),
            Err(e) if e.is::<Stopped>() => std::panic::resume_unwind(e),
            Err(e) if e.is::<Closed>() => {
                let img = &e.downcast_ref::<Closed>().unwrap().0;
                report::event(
                    "info",
                    "the region is closed (maintenance?): skipping this round",
                    Some(img),
                );
                strikes = 0;
                rest(current, if closed { 7200 } else { 14400 });
                closed = true;
                continue;
            }
            Err(e) => panic_text(e.as_ref()),
        };
        closed = false;
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
    // --gap, else the last one set remotely (data/gap), else 2 hours
    let saved = std::fs::read_to_string(assets::data().join("gap")).ok();
    let gap = arg("--gap")
        .or(saved)
        .map_or(120, |v| v.trim().parse().unwrap());
    GAP.store(gap, Ordering::Relaxed);
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
        awake(false);
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
