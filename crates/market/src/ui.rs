// Getting around the Market window, in 1080p client pixels, with checks that each step took.
use crate::{
    game::{Game, ms},
    glyph,
    ocr::{parse_number, read_box, similar},
};
use image::RgbaImage;

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

/// The game reshows a notice near the hour that can cover the window's top: wait it out.
pub fn wait_safe_time() {
    use chrono::Timelike;
    loop {
        let m = chrono::Local::now().minute();
        if !(56..=59).contains(&m) && m > 1 {
            return;
        }
        log("waiting out the hourly notice");
        ms(30_000);
    }
}

pub fn market_open(img: &RgbaImage) -> bool {
    looks(img, "market_title")
}

/// Closes whatever is open and opens the Market fresh (Alt+Y always opens it on Search Market, filters reset).
pub fn fresh_market(g: &mut Game) {
    for attempt in 0..6 {
        let img = g.shot(None);
        if market_open(&img) {
            g.key("alt+y");
            ms(900);
            if market_open(&g.shot(None)) {
                g.key("esc");
                ms(600);
            }
            continue;
        }
        g.key("alt+y");
        ms(1500);
        let img = g.shot(None);
        if market_open(&img) && looks(&img, "market_header") {
            return;
        }
        if attempt > 1 {
            // perhaps the game menu is up
            g.key("esc");
            ms(800);
        }
    }
    let img = g.shot(None);
    keep(&img, "no_market");
    panic!("could not open the Market");
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
        ms(600);
        if ok(&g.shot(None)) {
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
