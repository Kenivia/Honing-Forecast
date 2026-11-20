// Honing materials off the Market: every item of the Tier 4 "Honing Materials" and "Additional Honing
// Materials" categories, each through its purchase window (price tiers) and the 5-day average hover.
use crate::{
    game::{Game, ms},
    ocr::{best_match, lines, read_box},
    ui::*,
};
use image::{GrayImage, RgbaImage};
use std::sync::OnceLock;

pub const KNOWN: &[&str] = &[
    "Destiny Leapstone",
    "Great Destiny Leapstone",
    "Destiny Guardian Stone",
    "Destiny Destruction Stone",
    "Destiny Crystallized Guardian Stone",
    "Destiny Crystallized Destruction Stone",
    "Destiny Shard Pouch (S)",
    "Destiny Shard Pouch (M)",
    "Destiny Shard Pouch (L)",
    "Abidos Fusion Material",
    "Superior Abidos Fusion Material",
    "Glacier's Breath",
    "Lava's Breath",
    "Artisan's Metallurgy: Level 1",
    "Artisan's Tailoring: Level 1",
    "Artisan's Metallurgy: Level 2",
    "Artisan's Tailoring: Level 2",
    "Artisan's Metallurgy: Level 3",
    "Artisan's Tailoring: Level 3",
    "Artisan's Metallurgy: Level 4",
    "Artisan's Tailoring: Level 4",
    "Metallurgy: Hellfire [11-14]",
    "Tailoring: Hellfire [11-14]",
    "Metallurgy: Hellfire [15-18]",
    "Tailoring: Hellfire [15-18]",
    "Metallurgy: Hellfire [19-20]",
    "Tailoring: Hellfire [19-20]",
    "Enhanced Metallurgy: Hellfire [19-20]",
    "Enhanced Tailoring: Hellfire [19-20]",
    "Metallurgy: Tremor [12-15]",
    "Tailoring: Tremor [12-15]",
    "Metallurgy: Tremor [16-19]",
    "Tailoring: Tremor [16-19]",
];

#[derive(Debug, Default)]
pub struct Material {
    pub name: String,
    pub raw_name: String,
    pub bundle: Option<i64>,
    pub avg_day: Option<f64>,
    pub recent: Option<f64>,
    pub available: Option<f64>,
    /// (price, quantity), cheapest first, up to 5
    pub depth: Vec<(f64, f64)>,
    /// today first
    pub history: Vec<Option<f64>>,
}

const CATEGORIES: [(i32, &str); 2] = [(520, "Honing Materials"), (560, "Additional Honing Mat")];

// Items are told apart by their name line in the purchase window against crops kept in names/
// (file name: the name without ':'); the recogniser is only a fallback for items not seen before.
const NAMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/names");
// the icon and the whole name block (long names wrap and are cut, so the icon tells some apart)
const NAME_X: u32 = 795;
const NAME_Y: u32 = 205;
const NAME_W: u32 = 295;
const NAME_H: u32 = 50;

fn name_refs() -> &'static Vec<(String, GrayImage)> {
    static R: OnceLock<Vec<(String, GrayImage)>> = OnceLock::new();
    R.get_or_init(|| {
        let mut v = vec![];
        for e in std::fs::read_dir(NAMES).into_iter().flatten() {
            let p = e.unwrap().path();
            let stem = p.file_stem().unwrap().to_str().unwrap().to_string();
            let name = KNOWN
                .iter()
                .find(|k| k.replace(':', "") == stem)
                .map_or(stem, |k| k.to_string());
            v.push((name, image::open(&p).unwrap().to_luma8()));
        }
        v
    })
}

// the nearest crop, if it is near and clearly nearer than the next
fn identify(img: &RgbaImage) -> Option<String> {
    let mut d: Vec<(&String, f64)> = name_refs()
        .iter()
        .map(|(n, r)| (n, ink_distance(img, r, NAME_X as i32, NAME_Y as i32)))
        .collect();
    d.sort_by(|a, b| a.1.total_cmp(&b.1));
    // the same item draws pixel for pixel alike (0.0); the closest two items are 4.6 apart
    (d[0].1 < 2.5 && d.get(1).is_none_or(|x| x.1 > d[0].1 + 2.0)).then(|| d[0].0.clone())
}

fn keep_unknown_name(img: &RgbaImage, raw: &str) {
    let dir = concat!(env!("HF_ROOT"), "/ignored/market/unknown_names");
    std::fs::create_dir_all(dir).unwrap();
    let safe: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect();
    grey_crop(img, NAME_X, NAME_Y, NAME_W, NAME_H)
        .save(format!(
            "{dir}/{}_{safe}.png",
            chrono::Local::now().format("%H%M%S")
        ))
        .unwrap();
}

/// (page, pages) from the "1/2" under the list.
fn page_of(img: &RgbaImage) -> Option<(u32, u32)> {
    let t = crate::glyph::read(img, 1040, 1110, 889, 911);
    let d: Vec<u32> = t.chars().filter_map(|c| c.to_digit(10)).collect();
    (d.len() >= 2).then(|| (d[0], d[d.len() - 1]))
}

// The first line of a table row's name.
fn row_name(img: &RgbaImage, i: usize) -> Option<String> {
    let y = row_y(i) as u32;
    let l = lines(img, 612, 940, y - 25, y + 25);
    let (a, b) = *l.first()?;
    Some(read_box(img, 612, 940, a.saturating_sub(2), b + 3))
}

fn bundle_of(img: &RgbaImage, x0: u32, x1: u32, l: &[(u32, u32)]) -> Option<i64> {
    let (a, b) = *l.get(1)?;
    let t = read_box(img, x0, x1, a.saturating_sub(2), b + 3);
    let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Opens the Market on Tier 4 with the Enhancement Material tree open.
fn setup(g: &mut Game) {
    fresh_market(g);
    for _ in 0..3 {
        if looks(&g.shot(None), "honing_item") {
            break;
        }
        g.click(350, 440);
        ms(700);
    }
    let ok = pick(g, (610, 242), None, (560, 361), |img| looks(img, "tier4"));
    assert!(ok, "could not set Tier 4");
}

/// Reads a value that a popup may cover, taking a new screenshot between tries.
fn steady<T>(g: &mut Game, what: &str, f: impl Fn(&RgbaImage) -> Result<T, String>) -> Option<T> {
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

fn depth_row(img: &RgbaImage, k: u32) -> Result<Option<(f64, f64)>, String> {
    let (y0, y1) = (381 + 23 * k, 399 + 23 * k);
    let p = num(img, 800, 952, y0, y1, false);
    let q = num(img, 980, 1097, y0, y1, false);
    match (p, q) {
        (Err(a), Err(b)) if a.is_empty() && b.is_empty() => Ok(None),
        (Ok(Some(p)), Ok(Some(q))) => Ok(Some((p, q))),
        (p, q) => Err(format!("{p:?} {q:?}")),
    }
}

/// The open purchase window and its hover.
fn read_window(g: &mut Game, m: &mut Material) {
    let head = steady(g, "window head", |img| {
        Ok((
            num(img, 1000, 1095, 265, 284, true)?,
            num(img, 1000, 1095, 293, 312, false)?,
            num(img, 980, 1097, 321, 340, false)?,
        ))
    });
    if let Some((a, r, q)) = head {
        (m.avg_day, m.recent, m.available) = (a, r, q);
    }
    let rows = steady(g, "depth", |img| {
        let mut v = vec![];
        for k in 0..4 {
            match depth_row(img, k)? {
                Some(r) => v.push(r),
                None => break,
            }
        }
        Ok(v)
    })
    .unwrap_or_default();
    m.depth = rows.clone();
    if rows.len() == 4 {
        // the list shows four tiers; a notch down shows the fifth
        g.scroll(900, 420, 1);
        ms(400);
        let more = steady(g, "depth 5", |img| {
            Ok((depth_row(img, 2)?, depth_row(img, 3)?))
        });
        if let Some((Some(third), Some(fifth))) = more
            && third == rows[3]
        {
            m.depth.push(fifth);
        }
    }
    // an item with no trades today has no 5-day hover
    if m.avg_day.is_none() {
        return;
    }
    g.move_to(1075, 274);
    ms(900);
    let hist = steady(g, "5 day hover", |img| {
        if !looks(img, "hover_title") {
            return Err("no hover".into());
        }
        (0..5)
            .map(|k| num(img, 1200, 1324, 296 + 28 * k, 320 + 28 * k, true))
            .collect::<Result<Vec<_>, _>>()
    });
    m.history = hist.unwrap_or_default();
    g.move_to(1700, 1000);
    ms(200);
}

fn purchase_open(img: &RgbaImage) -> bool {
    looks(img, "purchase_title")
}

/// Row `i` of the current page, through its purchase window. None if it would not open.
fn read_row(g: &mut Game, i: usize) -> Option<Material> {
    wait_safe_time();
    g.click(820, row_y(i));
    ms(500);
    let img = g.shot(None);
    // the Market's Buy only opens the purchase window; never press it anywhere else
    if !looks(&img, "market_header") || !looks(&img, "buy_button") {
        log("not on the Market list, or Buy is not there");
        keep(&img, "no_buy");
        return None;
    }
    g.click(1580, 926);
    ms(1300);
    let img = g.shot(None);
    if !purchase_open(&img) {
        log("purchase window did not open");
        keep(&img, "no_window");
        return None;
    }
    let l = lines(&img, 855, 1090, 205, 255);
    let mut m = Material::default();
    if let Some(&(a, b)) = l.first() {
        m.raw_name = read_box(&img, 855, 1090, a.saturating_sub(2), b + 3);
        m.bundle = bundle_of(&img, 855, 1090, &l);
        m.name = match identify(&img) {
            Some(n) => n,
            None => {
                keep_unknown_name(&img, &m.raw_name);
                log(&format!("unknown name, read as {:?}", m.raw_name));
                best_match(&m.raw_name, KNOWN, 0.9).map_or(m.raw_name.clone(), str::to_string)
            }
        };
    }
    read_window(g, &mut m);
    for _ in 0..3 {
        g.click(1110, 170);
        ms(500);
        if !purchase_open(&g.shot(None)) {
            break;
        }
    }
    Some(m)
}

pub fn scan(g: &mut Game) -> Vec<Material> {
    setup(g);
    let mut out: Vec<Material> = vec![];
    for (y, cat) in CATEGORIES {
        g.click(350, y);
        ms(500);
        g.click(1600, 242);
        ms(2000);
        for page in 1..=6 {
            let img = g.shot(None);
            let names: Vec<String> = (0..10).map_while(|i| row_name(&img, i)).collect();
            let at = page_of(&img);
            if names.is_empty() || at.is_some_and(|(p, _)| p != page) {
                log(&format!("{cat}: expected page {page}, see {at:?}"));
                break;
            }
            log(&format!("{cat} page {page}: {} items", names.len()));
            for i in 0..names.len() {
                if let Some(m) = read_row(g, i) {
                    log(&format!(
                        "  {} {:?} {:?} depth {:?} 5d {:?}",
                        m.name, m.avg_day, m.recent, m.depth, m.history
                    ));
                    out.push(m);
                }
            }
            if at.is_none_or(|(p, n)| p >= n) {
                break;
            }
            g.click(1144, 899);
            ms(1500);
        }
    }
    out
}
