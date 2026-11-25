// Items told by their icon in the purchase window, against the game's own art (items/, written by
// tools/market_items.py), compared only where the art is opaque so the rarity background does not count.
// Several items share an icon; the name read then picks among them (`market::identify`).
use image::{RgbaImage, imageops};
use std::sync::OnceLock;

#[derive(serde::Deserialize)]
pub struct Item {
    pub name: String,
    pub grade: i64,
    pub icon: String,
    pub category: i64,
    pub tier: i64,
}

// the icon is drawn 46 px square from here (Great Destiny Leapstone: 0.976, the next icon 0.864)
const X: i32 = 802;
const Y: i32 = 205;
const SIZE: u32 = 46;
/// an icon below this is not the item's
pub const PASS: f64 = 0.9;
/// icons this close to the best are kept for the name to decide
const NEAR: f64 = 0.03;

struct Icon {
    name: String,
    // opaque pixels: offset into the square, colour
    px: Vec<(u32, u32, [f64; 3])>,
}

pub fn items() -> &'static Vec<Item> {
    static I: OnceLock<Vec<Item>> = OnceLock::new();
    I.get_or_init(|| {
        serde_json::from_slice(&crate::assets::find("items", "items").unwrap().1).unwrap()
    })
}

fn icons() -> &'static Vec<Icon> {
    static I: OnceLock<Vec<Icon>> = OnceLock::new();
    I.get_or_init(|| {
        let mut names: Vec<&str> = items().iter().map(|i| i.icon.as_str()).collect();
        names.sort();
        names.dedup();
        names
            .into_iter()
            .map(|n| {
                let art = image::load_from_memory(&crate::assets::find("items", n).unwrap().1)
                    .unwrap()
                    .to_rgba8();
                let art = imageops::resize(&art, SIZE, SIZE, imageops::FilterType::Lanczos3);
                let px = art
                    .enumerate_pixels()
                    .filter(|p| p.2[3] >= 230)
                    .map(|(x, y, p)| (x, y, [p[0] as f64, p[1] as f64, p[2] as f64]))
                    .collect();
                Icon {
                    name: n.to_string(),
                    px,
                }
            })
            .collect()
    })
}

// normalised cross-correlation over the art's opaque pixels, with the square at (x, y)
fn ncc(img: &RgbaImage, icon: &Icon, x: i32, y: i32) -> f64 {
    let n = icon.px.len() as f64;
    let screen: Vec<[f64; 3]> = icon
        .px
        .iter()
        .map(|(dx, dy, _)| {
            let p = img.get_pixel((x + *dx as i32) as u32, (y + *dy as i32) as u32);
            [p[0] as f64, p[1] as f64, p[2] as f64]
        })
        .collect();
    let mut num = 0.0;
    let (mut ss, mut aa) = (0.0, 0.0);
    for c in 0..3 {
        let ma = icon.px.iter().map(|p| p.2[c]).sum::<f64>() / n;
        let ms = screen.iter().map(|p| p[c]).sum::<f64>() / n;
        for (p, s) in icon.px.iter().zip(&screen) {
            let (a, b) = (p.2[c] - ma, s[c] - ms);
            num += a * b;
            aa += a * a;
            ss += b * b;
        }
    }
    num / (aa * ss).sqrt().max(1e-9)
}

/// The best icon's first item and its score, for reports.
pub fn best(img: &RgbaImage) -> (&'static str, f64) {
    let (icon, s) = scores(img)[0];
    let item = items().iter().find(|i| i.icon == icon).unwrap();
    (&item.name, s)
}

/// The items whose icon is drawn in the open purchase window, best first: every item of each icon
/// that passes and is within NEAR of the best. Empty when no icon passes.
pub fn candidates(img: &RgbaImage) -> Vec<(&'static Item, f64)> {
    let scores = scores(img);
    let Some(&(_, top)) = scores.first() else {
        return vec![];
    };
    scores
        .into_iter()
        .filter(|s| s.1 >= PASS && s.1 >= top - NEAR)
        .flat_map(|(icon, s)| {
            items()
                .iter()
                .filter(move |i| i.icon == icon)
                .map(move |i| (i, s))
        })
        .collect()
}

// every icon's score, best first
fn scores(img: &RgbaImage) -> Vec<(&'static str, f64)> {
    let mut scores: Vec<(&'static str, f64)> = icons()
        .iter()
        .map(|ic| {
            let best = (-1..=1)
                .flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
                .map(|(dx, dy)| ncc(img, ic, X + dx, Y + dy))
                .fold(f64::MIN, f64::max);
            (ic.name.as_str(), best)
        })
        .collect();
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    scores
}
