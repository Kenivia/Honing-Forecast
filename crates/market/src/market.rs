// Materials off the Market: every item of Enhancement Material > All at Tier 4 and Trade Skill > All,
// each through its purchase window (price tiers) and the 5-day average hover.
use crate::{
    game::{Game, ms},
    icons,
    ocr::{best_match, lines, numerals, read_box},
    ui::*,
};
use image::{GrayImage, RgbaImage};
use std::sync::OnceLock;

#[derive(Debug, Default, serde::Serialize)]
pub struct Material {
    pub name: String,
    /// when it was read (`upload::stamp`)
    pub at: String,
    /// the Market category it was listed under
    pub category: String,
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

/// The categories read, each through its "All" list.
pub const SECTIONS: [&str; 2] = ["Enhancement Material", "Trade Skill"];

// Items are told by their name block in the purchase window against crops kept in names/ (file name: the
// name without ':'), else by their icon (icons.rs) with the name read deciding among items that share it.
// the icon and the whole name block (long names wrap and are cut, so the icon tells some apart)
const NAME_X: u32 = 795;
const NAME_Y: u32 = 205;
const NAME_W: u32 = 295;
const NAME_H: u32 = 50;

fn name_refs() -> &'static Vec<(String, GrayImage)> {
    static R: OnceLock<Vec<(String, GrayImage)>> = OnceLock::new();
    R.get_or_init(|| {
        let mut v = vec![];
        for (file, bytes) in crate::assets::files("names") {
            let stem = file.rsplit_once('.').unwrap().0.to_string();
            let name = icons::items()
                .iter()
                .find(|i| i.name.replace(':', "") == stem)
                .map_or(stem, |i| i.name.clone());
            v.push((name, image::load_from_memory(&bytes).unwrap().to_luma8()));
        }
        v
    })
}

// the nearest crop, if it is near and clearly nearer than the next
fn by_crop(img: &RgbaImage) -> Option<String> {
    let mut d: Vec<(&String, f64)> = name_refs()
        .iter()
        .map(|(n, r)| (n, ink_distance(img, r, NAME_X as i32, NAME_Y as i32)))
        .collect();
    d.sort_by(|a, b| a.1.total_cmp(&b.1));
    // the same item draws pixel for pixel alike (0.0); the closest two items are 4.6 apart
    (d[0].1 < 2.5 && d.get(1).is_none_or(|x| x.1 > d[0].1 + 2.0)).then(|| d[0].0.clone())
}

/// The item in the open purchase window: its name crop when one is on file, else its icon, with the
/// name read deciding among the items drawn with that icon.
fn identify(img: &RgbaImage, read: &str) -> Option<String> {
    if let Some(n) = by_crop(img) {
        return Some(n);
    }
    let mut names: Vec<&str> = icons::candidates(img)
        .iter()
        .map(|c| c.0.name.as_str())
        .collect();
    names.dedup();
    match names.as_slice() {
        [] => None,
        [one] => Some(one.to_string()),
        many => pick_by_read(read, many),
    }
}

// The numerals have to be the name's ("Level 1" against "Level 4"); a long name the window cuts short is
// compared with as much of it as was read.
fn pick_by_read(read: &str, names: &[&str]) -> Option<String> {
    // the reader writes no spaces
    let bare = |t: &str| t.to_lowercase().replace(' ', "");
    let r = bare(read);
    let mut s: Vec<(f64, &str)> = names
        .iter()
        .filter_map(|n| {
            let full = bare(n);
            let cut: String = full.chars().take(r.chars().count()).collect();
            [full, cut]
                .iter()
                .filter(|t| numerals(t) == numerals(&r))
                .map(|t| strsim::normalized_levenshtein(&r, t))
                .reduce(f64::max)
                .map(|v| (v, *n))
        })
        .collect();
    s.sort_by(|a, b| b.0.total_cmp(&a.0));
    let second = s.get(1).map_or(0.0, |x| x.0);
    let &(best, name) = s.first()?;
    (best >= 0.6 && best - second >= 0.1).then(|| name.to_string())
}

fn keep_unknown_name(img: &RgbaImage, raw: &str) {
    let dir = crate::assets::data_dir("unknown_names");
    let safe: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect();
    grey_crop(img, NAME_X, NAME_Y, NAME_W, NAME_H)
        .save(dir.join(format!(
            "{}_{safe}.png",
            chrono::Local::now().format("%H%M%S")
        )))
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

/// Lists a section's "All" by rank: Enhancement Material at Tier 4, Trade Skill at all tiers (a fresh Market's).
fn open_section(g: &mut Game, section: &str) -> bool {
    if !fresh_market(g) {
        return false;
    }
    // the tree entry, a crop that shows it open, its "All"
    let (tree_y, open_ref, all_y) = match section {
        "Enhancement Material" => (440, "honing_item", 480),
        _ => (560, "trade_open", 600),
    };
    // the entry toggles: clicked only while shut
    let open = |img: &RgbaImage| looks(img, open_ref);
    if !open(&g.shot(None)) && !act(g, 3, 0.0, 3.0, |g| g.click(350, tree_y), open) {
        return false;
    }
    if section == "Enhancement Material"
        && !pick(g, (610, 242), None, (560, 361), |img| looks(img, "tier4"))
    {
        return false;
    }
    g.click(350, all_y);
    ms(500);
    g.click(1600, 242);
    ms(2000);
    // a fresh Market lists by lowest price, which reorders while pages are read; the rank order is fixed
    let sorted = |img: &RgbaImage| looks(img, "rank_sorted");
    let ok = sorted(&g.shot(None))
        || act(
            g,
            3,
            0.0,
            5.0,
            |g| {
                g.click(710, 282);
                ms(300);
                g.move_to(1700, 1000);
            },
            sorted,
        );
    // the list comes back after the header
    ms(1200);
    ok
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
        let tops = coin_rows(img);
        if tops.len() != 5 {
            return Err(format!("{} coins", tops.len()));
        }
        tops.iter()
            .map(|&t| num(img, 1200, 1324, t - 5, t + 19, true))
            .collect::<Result<Vec<_>, _>>()
    });
    m.history = hist.unwrap_or_default();
    g.move_to(1700, 1000);
    ms(200);
}

// The tops of the hover's rows, by their gold coins: a "-" row is shorter (26 px against 28), so the rows
// below one sit higher.
fn coin_rows(img: &RgbaImage) -> Vec<u32> {
    let yellow = |y: u32| {
        (1328..1348)
            .filter(|&x| {
                let p = img.get_pixel(x, y);
                p[0] > 150 && p[1] > 110 && p[2] < 120
            })
            .count()
            > 2
    };
    let mut tops = vec![];
    let mut start = None;
    for y in 297..=450 {
        match (y < 450 && yellow(y), start) {
            (true, None) => start = Some(y),
            (false, Some(s)) => {
                if y - s >= 10 {
                    tops.push(s);
                }
                start = None;
            }
            _ => {}
        }
    }
    tops
}

fn purchase_open(img: &RgbaImage) -> bool {
    looks(img, "purchase_title")
}

/// Row `i` of the current page, through its purchase window. None if it would not open.
fn read_row(g: &mut Game, i: usize) -> Option<Material> {
    // the Market's Buy only opens the purchase window; never press it anywhere else
    let listed = |img: &RgbaImage| looks(img, "market_header") && looks(img, "buy_button");
    if !act(g, 2, 0.0, 3.0, |g| g.click(820, row_y(i)), listed) {
        log("not on the Market list, or Buy is not there");
        return None;
    }
    g.click(1580, 926);
    if wait(g, 10.0, purchase_open).is_none() {
        log("purchase window did not open");
        return None;
    }
    // the window's contents draw in after its title
    ms(800);
    let img = g.shot(None);
    // the name's lines, then perhaps "[Sold in bundles of N units]"
    let texts: Vec<String> = lines(&img, 855, 1090, 205, 255)
        .iter()
        .map(|&(a, b)| read_box(&img, 855, 1090, a.saturating_sub(2), b + 3))
        .collect();
    let (bundle, name): (Vec<String>, Vec<String>) = texts
        .into_iter()
        .partition(|t| t.to_lowercase().contains("bundle"));
    let mut m = Material {
        at: crate::upload::stamp(),
        raw_name: name.join(" "),
        bundle: bundle.first().and_then(|t| numerals(t).parse().ok()),
        ..Default::default()
    };
    m.name = match identify(&img, &m.raw_name) {
        Some(n) => n,
        None => {
            keep_unknown_name(&img, &m.raw_name);
            log(&format!("unknown item, read as {:?}", m.raw_name));
            let all: Vec<&str> = icons::items().iter().map(|i| i.name.as_str()).collect();
            best_match(&m.raw_name, &all, 0.9).map_or(m.raw_name.clone(), str::to_string)
        }
    };
    crate::strips::context(&m.name);
    read_window(g, &mut m);
    if !act(
        g,
        3,
        0.0,
        3.0,
        |g| g.click(1110, 170),
        |img| !purchase_open(img),
    ) {
        log("purchase window did not close");
        return None;
    }
    Some(m)
}

/// Each section into `out`, item by item, so a stopped pass keeps what was read.
pub fn scan(g: &mut Game, out: &mut Vec<Material>) {
    for section in SECTIONS {
        let n0 = out.len();
        let done = unit(g, section, |g| {
            out.truncate(n0);
            read_section(g, section, out)
        });
        if done.is_none() {
            out.truncate(n0);
        }
    }
}

// every page of a section; None when a step did not take
fn read_section(g: &mut Game, section: &str, out: &mut Vec<Material>) -> Option<()> {
    if !open_section(g, section) {
        log(&format!("{section}: could not list it"));
        return None;
    }
    for page in 1..=10 {
        let img = g.shot(None);
        let names: Vec<String> = (0..10).map_while(|i| row_name(&img, i)).collect();
        let at = page_of(&img);
        if names.is_empty() || at.is_some_and(|(p, _)| p != page) {
            log(&format!("{section}: expected page {page}, see {at:?}"));
            return None;
        }
        log(&format!("{section} page {page}: {} items", names.len()));
        let n = if quick() {
            2.min(names.len())
        } else {
            names.len()
        };
        for i in 0..n {
            crate::strips::context(&format!("{section} page {page} row {i}"));
            let mut m = read_row(g, i)?;
            log(&format!(
                "  {} {:?} {:?} depth {:?} 5d {:?}",
                m.name, m.avg_day, m.recent, m.depth, m.history
            ));
            m.category = section.to_string();
            out.push(m);
        }
        if at.is_none_or(|(p, n)| p >= n) {
            break;
        }
        g.click(1144, 899);
        ms(1500);
        let next = |img: &RgbaImage| page_of(img).is_some_and(|(p, _)| p == page + 1);
        if wait(g, 10.0, next).is_none() {
            log(&format!("{section}: page {} did not come", page + 1));
            return None;
        }
        // the rows come with the page number
        ms(300);
    }
    Some(())
}
