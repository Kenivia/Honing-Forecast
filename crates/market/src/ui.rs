// Getting around the game's windows, in 1080p client pixels, with checks that each step took, and the units of
// work that are tried again from a reset UI when one does not.
use crate::{
    game::{Game, check_abort, ms},
    glyph,
    ocr::{parse_number, read_box, similar},
};
use image::RgbaImage;
use std::time::{Duration, Instant};

pub const ROW0: i32 = 333;
pub const ROW_STEP: i32 = 57;

pub fn row_y(i: usize) -> i32 {
    ROW0 + ROW_STEP * i as i32
}

/// Appends to ignored/market/scan.log and prints.
pub fn log(s: &str) {
    use std::io::Write;
    let line = format!("{} {s}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    println!("{line}");
    let dir = concat!(env!("HF_ROOT"), "/ignored/market");
    std::fs::create_dir_all(dir).unwrap();
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/scan.log"))
        .unwrap();
    writeln!(f, "{line}").unwrap();
}

/// Keeps a screenshot of something that could not be read.
pub fn keep(img: &RgbaImage, what: &str) {
    let dir = concat!(env!("HF_ROOT"), "/ignored/market/debug");
    std::fs::create_dir_all(dir).unwrap();
    let name = format!(
        "{}_{}.png",
        chrono::Local::now().format("%Y%m%d-%H%M%S"),
        what.replace([' ', '/', ':'], "_")
    );
    img.save(format!("{dir}/{name}")).unwrap();
}

/// A number of the game's digit font; Err with the glyphs read when it is not one.
pub fn num(
    img: &RgbaImage,
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
    decimal: bool,
) -> Result<Option<f64>, String> {
    let t = glyph::read(img, x0, x1, y0, y1);
    if t.is_empty() {
        return Err(t);
    }
    parse_number(&t, decimal).ok_or(t)
}

pub fn text_like(
    img: &RgbaImage,
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
    want: &str,
    min: f64,
) -> bool {
    similar(&read_box(img, x0, x1, y0, y1), want) >= min
}

/// Quick test pass: a few rows and searches per path (`market_scan --quick`).
pub static QUICK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn quick() -> bool {
    QUICK.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn market_open(img: &RgbaImage) -> bool {
    looks(img, "market_title")
}

/// Polls the screen until `ok` holds, up to `secs`; the screenshot that showed it.
pub fn wait(g: &mut Game, secs: f64, ok: impl Fn(&RgbaImage) -> bool) -> Option<RgbaImage> {
    let end = Instant::now() + Duration::from_secs_f64(secs);
    loop {
        let img = g.shot(None);
        if ok(&img) {
            return Some(img);
        }
        if Instant::now() >= end {
            return None;
        }
        ms(250);
    }
}

/// Does `f` until the screen shows `ok`, waiting up to `secs` after each try. Only for actions that are
/// harmless to repeat once they took.
pub fn act(
    g: &mut Game,
    tries: u32,
    secs: f64,
    mut f: impl FnMut(&mut Game),
    ok: impl Fn(&RgbaImage) -> bool,
) -> bool {
    (0..tries).any(|_| {
        f(g);
        wait(g, secs, &ok).is_some()
    })
}

/// Closes the Market by its X: Alt+Y would type a "y" into a focused search box.
pub fn close_market(g: &mut Game) {
    if market_open(&g.shot(None))
        && !act(g, 3, 3.0, |g| g.click(1625, 111), |img| !market_open(img))
    {
        g.key("esc");
        ms(800);
    }
}

fn shop_up(img: &RgbaImage) -> bool {
    looks(img, "exchange_title") || looks(img, "shop_title")
}

/// Leave closes the Lost Ark Shop from either page; pressed only while one of them is up.
pub fn close_shop(g: &mut Game) {
    if !shop_up(&g.shot(None)) {
        return;
    }
    if !act(g, 3, 5.0, |g| g.click(1814, 1036), |img| !shop_up(img)) {
        g.key("esc");
    }
    ms(1000);
}

/// Back to the bare game screen, where every unit of work can start: the Market and the shop closed.
// Only the windows the scanner opens are closed. Recovering from a lost connection (the game's reconnect
// prompt, the log-in or server screens) belongs here too and is not handled yet: to be investigated.
pub fn reset(g: &mut Game) {
    close_market(g);
    close_shop(g);
}

// seconds waited before each further try of a unit of work
const BACKOFF: [u64; 3] = [5, 20, 60];

/// Runs a unit of work (a Market section, a search, the currency exchange); when it fails (None), keeps a
/// screenshot, resets the UI, waits longer each time and runs it again. None once every try failed.
pub fn unit<T>(g: &mut Game, what: &str, mut f: impl FnMut(&mut Game) -> Option<T>) -> Option<T> {
    for k in 0..=BACKOFF.len() {
        if let Some(v) = f(g) {
            return Some(v);
        }
        keep(&g.shot(None), what);
        reset(g);
        let Some(&s) = BACKOFF.get(k) else { break };
        log(&format!("{what}: failed, again in {s} s"));
        let end = Instant::now() + Duration::from_secs(s);
        while Instant::now() < end {
            check_abort();
            ms(200);
        }
    }
    log(&format!(
        "{what}: failed {} times, skipped",
        BACKOFF.len() + 1
    ));
    None
}

/// Closes whatever is open and opens the Market fresh (Alt+Y always opens it on Search Market, filters reset).
pub fn fresh_market(g: &mut Game) -> bool {
    close_market(g);
    for _ in 0..3 {
        g.key("alt+y");
        if wait(g, 10.0, |img| {
            market_open(img) && looks(img, "market_header")
        })
        .is_some()
        {
            return true;
        }
        // perhaps the game menu is up
        g.key("esc");
        ms(800);
    }
    log("the Market did not open");
    false
}

/// Clicks `open`, then `item`, until `ok` holds of the screen.
pub fn pick(
    g: &mut Game,
    open: (i32, i32),
    scroll: Option<(i32, i32, i32)>,
    item: (i32, i32),
    ok: impl Fn(&RgbaImage) -> bool,
) -> bool {
    for _ in 0..4 {
        g.click(open.0, open.1);
        ms(700);
        if let Some((x, y, n)) = scroll {
            g.scroll(x, y, -40);
            ms(200);
            if n > 0 {
                g.scroll(x, y, n);
                ms(300);
            }
        }
        g.click(item.0, item.1);
        ms(300);
        if wait(g, 2.0, &ok).is_some() {
            return true;
        }
        g.move_to(1700, 1000);
        ms(300);
    }
    false
}

const REFS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/refs");

fn ref_file(name: &str) -> (image::GrayImage, i32, i32) {
    let e = std::fs::read_dir(REFS)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(&format!("{name}@"))
        })
        .unwrap_or_else(|| panic!("no reference {name}"));
    let stem = e.file_stem().unwrap().to_str().unwrap().to_string();
    let (x, y) = stem.split_once('@').unwrap().1.split_once('_').unwrap();
    (
        image::open(&e).unwrap().to_luma8(),
        x.parse().unwrap(),
        y.parse().unwrap(),
    )
}

fn grey(img: &RgbaImage, x: i32, y: i32) -> f64 {
    let p = img.get_pixel(x as u32, y as u32);
    p[0].max(p[1]).max(p[2]) as f64
}

/// How far the screen is from a reference crop (mean difference a pixel, best over a pixel's shift), moved down by `dy`.
pub fn ref_distance(img: &RgbaImage, name: &str, dy: i32) -> f64 {
    let (r, x0, y0) = ref_file(name);
    distance(img, &r, x0, y0 + dy)
}

/// Mean difference a pixel between the screen at (x0, y0) and a grey crop, best over a pixel's shift.
pub fn distance(img: &RgbaImage, r: &image::GrayImage, x0: i32, y0: i32) -> f64 {
    let mut best = f64::MAX;
    for (sx, sy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        let mut sum = 0.0;
        for (x, y, p) in r.enumerate_pixels() {
            sum += (grey(img, x0 + x as i32 + sx, y0 + y as i32 + sy) - p[0] as f64).abs();
        }
        best = best.min(sum / (r.width() * r.height()) as f64);
    }
    best
}

/// Whether the screen shows what a reference crop shows.
pub fn looks(img: &RgbaImage, name: &str) -> bool {
    ref_distance(img, name, 0) < 14.0
}

pub fn looks_at(img: &RgbaImage, name: &str, dy: i32) -> bool {
    ref_distance(img, name, dy) < 14.0
}

/// Saves a reference crop as refs/<name>@<x>_<y>.png, as max-channel grey.
pub fn save_ref(img: &RgbaImage, name: &str, x: u32, y: u32, w: u32, h: u32) {
    std::fs::create_dir_all(REFS).unwrap();
    for e in std::fs::read_dir(REFS).unwrap() {
        let p = e.unwrap().path();
        if p.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with(&format!("{name}@"))
        {
            std::fs::remove_file(p).unwrap();
        }
    }
    grey_crop(img, x, y, w, h)
        .save(format!("{REFS}/{name}@{x}_{y}.png"))
        .unwrap();
}

pub fn grey_crop(img: &RgbaImage, x: u32, y: u32, w: u32, h: u32) -> image::GrayImage {
    image::GrayImage::from_fn(w, h, |i, j| {
        image::Luma([grey(img, (x + i) as i32, (y + j) as i32) as u8])
    })
}

/// Like `distance`, but only over pixels that are ink (bright) in either, so small differences in text count.
pub fn ink_distance(img: &RgbaImage, r: &image::GrayImage, x0: i32, y0: i32) -> f64 {
    let mut best = f64::MAX;
    for (sx, sy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        let (mut sum, mut n) = (0.0, 0);
        for (x, y, p) in r.enumerate_pixels() {
            let (a, b) = (
                grey(img, x0 + x as i32 + sx, y0 + y as i32 + sy),
                p[0] as f64,
            );
            if a.max(b) >= 110.0 {
                sum += (a - b).abs();
                n += 1;
            }
        }
        best = best.min(sum / n.max(1) as f64);
    }
    best
}

/// Reads a value that a popup may cover, taking a new screenshot between tries.
pub fn steady<T>(
    g: &mut Game,
    what: &str,
    f: impl Fn(&RgbaImage) -> Result<T, String>,
) -> Option<T> {
    let mut img = g.shot(None);
    for attempt in 0..4 {
        match f(&img) {
            Ok(v) => return Some(v),
            Err(e) => {
                if attempt == 3 {
                    log(&format!("could not read {what}: {e:?}"));
                    keep(&img, what);
                    return None;
                }
                ms(2500);
                img = g.shot(None);
            }
        }
    }
    None
}
