// Gems and accessories off the Auction House: the first page of each search, cheapest buy-now first.
use crate::{
    game::{Game, ms},
    glyph,
    ocr::{is_name, lines, read_box},
    ui::*,
};
use image::RgbaImage;

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
pub struct Listing {
    pub name: String,
    pub trades: String,
    pub quality: Option<f64>,
    pub time_left: String,
    pub min_bid: Option<f64>,
    pub buy_now: Option<f64>,
}

/// One search and what it found.
#[derive(Debug, Default, serde::Serialize)]
pub struct Search {
    pub kind: &'static str,
    /// when it was read (`upload::stamp`)
    pub at: String,
    pub item: String,
    pub category: String,
    pub gem_level: Option<i64>,
    pub level_a: Option<&'static str>,
    pub level_b: Option<&'static str>,
    pub rows: Vec<Listing>,
}

pub const LEVELS: [&str; 3] = ["low", "mid", "high"];
// (option a, option b) levels searched for each accessory group
const COMBOS: [(usize, usize); 6] = [(2, 2), (1, 1), (2, 1), (1, 2), (2, 0), (0, 2)];

pub struct Group {
    pub item: &'static str,
    pub list_index: i32,
    // reference crop of the category once picked
    pub list_label: &'static str,
    pub category: &'static str,
    // option name, its reference crop once typed, its values
    pub a: (&'static str, &'static str, [&'static str; 3]),
    pub b: (&'static str, &'static str, [&'static str; 3]),
}

// the option values are what the min / max lists offer for Ancient Tier 4, low to high
pub const GROUPS: [Group; 5] = [
    Group {
        item: "necklace",
        list_index: 10,
        list_label: "adv_necklace",
        category: "dps",
        a: (
            "Additional Damage",
            "kw_additional_damage",
            ["0.70", "1.60", "2.60"],
        ),
        b: (
            "Outgoing Damage Increase",
            "kw_outgoing_damage",
            ["0.55", "1.20", "2.00"],
        ),
    },
    Group {
        item: "necklace",
        list_index: 10,
        list_label: "adv_necklace",
        category: "support",
        a: ("Brand Power", "kw_brand_power", ["2.15", "4.80", "8.00"]),
        b: (
            "Increased Serenade, Piety, Harmony Meter Gain",
            "kw_meter_gain",
            ["1.60", "3.60", "6.00"],
        ),
    },
    Group {
        item: "earrings",
        list_index: 11,
        list_label: "adv_earrings",
        category: "dps",
        a: ("Atk. Power %", "kw_atk_power", ["0.40", "0.95", "1.55"]),
        b: (
            "Weapon Power %",
            "kw_weapon_power",
            ["0.80", "1.80", "3.00"],
        ),
    },
    Group {
        item: "ring",
        list_index: 12,
        list_label: "adv_ring",
        category: "dps",
        a: ("Crit Rate", "kw_crit_rate", ["0.40", "0.95", "1.55"]),
        b: ("Crit Damage", "kw_crit_damage", ["1.10", "2.40", "4.00"]),
    },
    Group {
        item: "ring",
        list_index: 12,
        list_label: "adv_ring",
        category: "support",
        a: (
            "Ally Atk. Power Enhancement Effect",
            "kw_ally_atk",
            ["1.35", "3.00", "5.00"],
        ),
        b: (
            "Ally Damage Enhancement Effect",
            "kw_ally_damage",
            ["2.00", "4.50", "7.50"],
        ),
    },
];

fn read_rows(img: &RgbaImage, accessory: bool) -> Result<Vec<Listing>, String> {
    let mut out = vec![];
    for i in 0..10 {
        let y = row_y(i) as u32;
        let l = lines(img, 605, 870, y - 25, y + 25);
        let Some(&(a, b)) = l.first() else { break };
        let mut r = Listing {
            name: read_box(img, 605, 870, a.saturating_sub(2), b + 3),
            ..Default::default()
        };
        if accessory {
            if let Some(&(a, b)) = l.get(1) {
                r.trades = read_box(img, 605, 870, a.saturating_sub(2), b + 3);
            }
            r.quality = num(img, 1000, 1140, y + 3, y + 22, false)
                .map_err(|e| format!("row {i} quality {e}"))?;
        }
        r.time_left = read_box(img, 1160, 1290, y - 12, y + 12);
        r.min_bid =
            num(img, 1290, 1428, y - 12, y + 12, false).map_err(|e| format!("row {i} bid {e}"))?;
        r.buy_now =
            num(img, 1460, 1599, y - 12, y + 12, false).map_err(|e| format!("row {i} buy {e}"))?;
        out.push(r);
    }
    Ok(out)
}

fn ascending(rows: &[Listing]) -> bool {
    let v: Vec<f64> = rows.iter().filter_map(|r| r.buy_now).collect();
    v.windows(2).all(|w| w[0] <= w[1])
}

/// Reads the results of a search just made; None when they could not be read or look stale.
fn results(
    g: &mut Game,
    accessory: bool,
    last: &[Listing],
    what: &str,
) -> Option<(Vec<Listing>, RgbaImage)> {
    let mut empty = 0;
    for attempt in 0..4 {
        ms(2500);
        let img = g.shot(None);
        match read_rows(&img, accessory) {
            Ok(rows) => {
                // nothing may only mean the search has not come back yet
                if rows.is_empty() {
                    empty += 1;
                    if empty < 2 {
                        continue;
                    }
                }
                if !rows.is_empty() && rows == last {
                    log(&format!("{what}: same rows as the search before, waiting"));
                    continue;
                }
                if !ascending(&rows) {
                    log(&format!("{what}: not sorted by buy-now, sorting"));
                    g.click(1550, 282);
                    continue;
                }
                return Some((rows, img));
            }
            Err(e) => {
                if attempt == 3 {
                    log(&format!("{what}: could not read: {e}"));
                }
            }
        }
    }
    None
}

// A gem's rows are told by their name line against crops kept in gems/ (drawn pixel for pixel alike);
// a gem with no crop yet is checked by reading and its crop kept in ignored/market/unknown_gems to be filed.
const GEMS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/gems");
const GEM_X: u32 = 612;
const GEM_W: u32 = 150;

fn gem_crop_y(i: usize) -> u32 {
    row_y(i) as u32 - 12
}

fn is_gem(img: &RgbaImage, rows: &[Listing], name: &str) -> bool {
    if rows.is_empty() {
        return true;
    }
    let refs: Vec<(String, image::GrayImage)> = std::fs::read_dir(GEMS)
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().path())
        .map(|p| {
            (
                p.file_stem().unwrap().to_str().unwrap().to_string(),
                image::open(&p).unwrap().to_luma8(),
            )
        })
        .collect();
    if !refs.iter().any(|(n, _)| n == name) {
        let dir = concat!(env!("HF_ROOT"), "/ignored/market/unknown_gems");
        std::fs::create_dir_all(dir).unwrap();
        grey_crop(img, GEM_X, gem_crop_y(0), GEM_W, 24)
            .save(format!("{dir}/{name}.png"))
            .unwrap();
        return rows.iter().filter(|r| is_name(&r.name, name)).count() * 2 > rows.len();
    }
    // every row's nearest crop has to be this gem's
    (0..rows.len()).all(|i| {
        let mut d: Vec<(&String, f64)> = refs
            .iter()
            .map(|(n, r)| (n, ink_distance(img, r, GEM_X as i32, gem_crop_y(i) as i32)))
            .collect();
        d.sort_by(|a, b| a.1.total_cmp(&b.1));
        d[0].0 == name && d[0].1 < 2.5
    })
}

// The Auction House with class All, from wherever the UI is; false when a step did not take.
fn ah_setup(g: &mut Game) -> bool {
    if looks(&g.shot(None), "ah_header") {
        return true;
    }
    if !fresh_market(g) {
        return false;
    }
    if !act(
        g,
        2,
        0.0,
        10.0,
        |g| g.click(486, 154),
        |img| looks(img, "ah_header"),
    ) {
        log("the Auction House did not open");
        return false;
    }
    let ok = pick(g, (394, 240), Some((350, 330, 0)), (350, 274), |img| {
        looks(img, "class_all")
    });
    if !ok {
        log("could not set class All");
    }
    ok
}

fn gem_search(g: &mut Game, name: &str, last: &[Listing]) -> Option<Vec<Listing>> {
    if !ah_setup(g) {
        return None;
    }
    g.click(1462, 240);
    ms(300);
    g.click(1250, 240);
    ms(300);
    g.key("ctrl+a");
    g.type_text(name);
    ms(400);
    g.key("enter");
    let (rows, img) = results(g, false, last, name)?;
    if !is_gem(&img, &rows, name) {
        log(&format!(
            "{name}: rows are not this gem: {:?}",
            rows.first().map(|r| &r.name)
        ));
        return None;
    }
    Some(rows)
}

/// Each gem search, one unit of work; a search that failed every try is left out.
pub fn gems(g: &mut Game) -> Vec<Search> {
    // from a fresh Auction House, its filters and search box reset
    close_market(g);
    let mut out = vec![];
    let mut last: Vec<Listing> = vec![];
    for level in (5..=10).rev().filter(|l| !quick() || *l == 10 || *l == 5) {
        for (kind, category) in [("Doomfire", "damage"), ("Blazing", "cooldown")] {
            let name = format!("Lv. {level} {kind} Gem");
            crate::strips::context(&name);
            let Some(rows) = unit(g, &name, |g| gem_search(g, &name, &last)) else {
                continue;
            };
            log(&format!(
                "{name}: {:?}",
                rows.iter().map(|r| r.buy_now).collect::<Vec<_>>()
            ));
            last = rows.clone();
            out.push(Search {
                at: crate::upload::stamp(),
                kind: "gem",
                item: kind.to_lowercase(),
                category: category.into(),
                gem_level: Some(level),
                rows,
                ..Default::default()
            });
        }
    }
    out
}

fn option_y(row: i32) -> i32 {
    689 + 47 * row
}

fn value_ok(img: &RgbaImage, x0: u32, x1: u32, y: i32, want: &str) -> bool {
    glyph::read(img, x0, x1, y as u32 - 12, y as u32 + 12).starts_with(want)
}

/// Whether the advanced search shows this group with these levels.
fn configured(img: &RgbaImage, grp: &Group, levels: Option<(usize, usize)>) -> bool {
    let mut ok = looks(img, grp.list_label)
        && looks(img, "adv_class_all")
        && looks(img, "adv_ancient")
        && looks(img, "adv_tier4");
    for (r, (_, kw, _)) in [grp.a, grp.b].iter().enumerate() {
        ok &= looks_at(img, "refine_row", 47 * r as i32) && looks_at(img, kw, 47 * r as i32);
    }
    if let Some((la, lb)) = levels {
        for (r, v) in [grp.a.2[la], grp.b.2[lb]].iter().enumerate() {
            let y = option_y(r as i32);
            ok &= value_ok(img, 925, 982, y, v) && value_ok(img, 1043, 1097, y, v);
        }
    }
    ok
}

fn set_levels(g: &mut Game, grp: &Group, (la, lb): (usize, usize)) {
    for (r, (vals, lvl)) in [(grp.a.2, la), (grp.b.2, lb)].into_iter().enumerate() {
        let y = option_y(r as i32);
        let item = y + 34 + 29 * lvl as i32;
        pick(g, (960, y), None, (960, item), |img| {
            value_ok(img, 925, 982, y, vals[lvl])
        });
        pick(g, (1078, y), None, (1078, item), |img| {
            value_ok(img, 1043, 1097, y, vals[lvl])
        });
    }
}

fn configure(g: &mut Game, grp: &Group) {
    g.click(464, 924);
    ms(800);
    let first = grp.list_index.min(10);
    pick(
        g,
        (620, 172),
        Some((600, 260, first)),
        (565, 206 + 29 * (grp.list_index - first)),
        |img| looks(img, grp.list_label),
    );
    pick(g, (620, 227), Some((600, 300, 0)), (565, 261), |img| {
        looks(img, "adv_class_all")
    });
    pick(g, (620, 282), Some((600, 400, 4)), (520, 403), |img| {
        looks(img, "adv_ancient")
    });
    pick(g, (1040, 283), None, (928, 403), |img| {
        looks(img, "adv_tier4")
    });
    for (r, (opt, _, _)) in [grp.a, grp.b].iter().enumerate() {
        let y = option_y(r as i32);
        pick(g, (510, y), None, (453, y + 92), |img| {
            looks_at(img, "refine_row", 47 * r as i32)
        });
        g.click(857, y);
        ms(300);
        g.click(780, y);
        ms(300);
        g.type_text(opt);
        ms(700);
        g.click(720, y + 33);
        ms(600);
    }
}

fn open_advanced(g: &mut Game) -> bool {
    let open = |img: &RgbaImage| looks(img, "adv_title");
    open(&g.shot(None)) || act(g, 3, 0.0, 4.0, |g| g.click(1580, 240), open)
}

fn accessory_search(
    g: &mut Game,
    grp: &Group,
    levels: (usize, usize),
    what: &str,
    last: &[Listing],
) -> Option<Vec<Listing>> {
    if !ah_setup(g) {
        return None;
    }
    if !open_advanced(g) {
        log(&format!("{what}: the advanced search did not open"));
        return None;
    }
    if !configured(&g.shot(None), grp, None) {
        configure(g, grp);
    }
    set_levels(g, grp, levels);
    if !configured(&g.shot(None), grp, Some(levels)) {
        log(&format!("{what}: settings did not take"));
        return None;
    }
    g.click(1366, 924);
    results(g, true, last, what).map(|r| r.0)
}

/// Each accessory search, one unit of work; a search that failed every try is left out.
pub fn accessories(g: &mut Game) -> Vec<Search> {
    // from a fresh Auction House, its filters and search box reset
    close_market(g);
    let mut out = vec![];
    let mut last: Vec<Listing> = vec![];
    for grp in &GROUPS {
        for (la, lb) in COMBOS
            .into_iter()
            .filter(|c| !quick() || *c == (2, 2) || *c == (0, 2))
        {
            let what = format!(
                "{} {} {}/{}",
                grp.item, grp.category, LEVELS[la], LEVELS[lb]
            );
            crate::strips::context(&what);
            let Some(rows) = unit(g, &what, |g| {
                accessory_search(g, grp, (la, lb), &what, &last)
            }) else {
                continue;
            };
            log(&format!(
                "{what}: {:?}",
                rows.iter().map(|r| r.buy_now).collect::<Vec<_>>()
            ));
            last = rows.clone();
            out.push(Search {
                at: crate::upload::stamp(),
                kind: "accessory",
                item: grp.item.into(),
                category: grp.category.into(),
                level_a: Some(LEVELS[la]),
                level_b: Some(LEVELS[lb]),
                rows,
                ..Default::default()
            });
        }
    }
    out
}
