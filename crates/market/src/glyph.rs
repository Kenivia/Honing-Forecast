// Numbers read glyph by glyph against the game's font (ExuberanceF, public/fonts/English.ttf), drawn at
// quarter-pixel offsets as the slot numbers are: the general text recogniser misreads digits this small.
use ab_glyph::{Font, FontVec, PxScale, point};
use image::RgbaImage;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

#[derive(Debug, Clone)]
pub struct Glyph {
    pub x0: u32,
    pub x1: u32,
    pub y0: u32,
    pub y1: u32,
}

fn value(img: &RgbaImage, x: u32, y: u32) -> u8 {
    let p = img.get_pixel(x, y);
    p[0].max(p[1]).max(p[2])
}

/// Glyphs inside a rectangle, left to right, split at columns with no ink.
pub fn segment(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> (Vec<Glyph>, u8) {
    let most = (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| value(img, x, y))
        .max()
        .unwrap_or(0);
    let th = (most as u32 * 55 / 100).max(90) as u8;
    let on = |x: u32, y: u32| value(img, x, y) >= th;
    let mut out = vec![];
    let mut start = None;
    for x in x0..=x1 {
        let col = x < x1 && (y0..y1).any(|y| on(x, y));
        match (col, start) {
            (true, None) => start = Some(x),
            (false, Some(s)) => {
                let ys: Vec<u32> = (y0..y1).filter(|&y| (s..x).any(|xx| on(xx, y))).collect();
                out.push(Glyph {
                    x0: s,
                    x1: x,
                    y0: ys[0],
                    y1: ys[ys.len() - 1] + 1,
                });
                start = None;
            }
            _ => {}
        }
    }
    (out, th)
}

fn font() -> &'static FontVec {
    static F: OnceLock<FontVec> = OnceLock::new();
    F.get_or_init(|| FontVec::try_from_vec(crate::assets::FONT.to_vec()).unwrap())
}

// coverage of a digit drawn at `px` with its pen at (fx, baseline + fy); returned with the top-left of its ink box
struct Drawing {
    w: usize,
    h: usize,
    cover: Vec<f32>,
}

fn draw(c: char, px: f32, fx: f32, fy: f32) -> Drawing {
    let f = font();
    let g = f
        .glyph_id(c)
        .with_scale_and_position(PxScale::from(px), point(fx, px + fy));
    let o = f.outline_glyph(g).unwrap();
    let b = o.px_bounds();
    let (w, h) = (b.width() as usize, b.height() as usize);
    let mut cover = vec![0.0; w * h];
    o.draw(|x, y, v| cover[y as usize * w + x as usize] = v);
    // trim empty rows and columns so the ink box starts at (0, 0)
    let used_x: Vec<usize> = (0..w)
        .filter(|&x| (0..h).any(|y| cover[y * w + x] > 0.25))
        .collect();
    let used_y: Vec<usize> = (0..h)
        .filter(|&y| (0..w).any(|x| cover[y * w + x] > 0.25))
        .collect();
    let (x0, x1, y0, y1) = (
        used_x[0],
        used_x[used_x.len() - 1] + 1,
        used_y[0],
        used_y[used_y.len() - 1] + 1,
    );
    let (tw, th) = (x1 - x0, y1 - y0);
    let mut t = vec![0.0; tw * th];
    for y in 0..th {
        for x in 0..tw {
            t[y * tw + x] = cover[(y + y0) * w + x + x0];
        }
    }
    Drawing {
        w: tw,
        h: th,
        cover: t,
    }
}

// digit ink height per px of size
fn digit_ratio() -> f32 {
    static R: OnceLock<f32> = OnceLock::new();
    *R.get_or_init(|| draw('0', 200.0, 0.0, 0.0).h as f32 / 200.0)
}

type Key = (char, u32, u8, u8);
fn drawing(c: char, px: f32, fx: u8, fy: u8) -> Arc<Drawing> {
    static CACHE: OnceLock<Mutex<HashMap<Key, Arc<Drawing>>>> = OnceLock::new();
    let key = (c, (px * 100.0) as u32, fx, fy);
    let cache = CACHE.get_or_init(Default::default);
    if let Some(d) = cache.lock().unwrap().get(&key) {
        return d.clone();
    }
    let d = Arc::new(draw(c, px, fx as f32 / 4.0, fy as f32 / 4.0));
    cache.lock().unwrap().insert(key, d.clone());
    d
}

/// Text of a number read by the game's font: digits, '.' for a separator, '-', '?' when unsure.
pub fn read(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> String {
    let t = read_inner(img, x0, x1, y0, y1);
    crate::strips::save(img, x0, x1, y0, y1, "glyph", &t);
    t
}

fn read_inner(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> String {
    let (glyphs, _) = segment(img, x0, x1, y0, y1);
    let height = glyphs.iter().map(|g| g.y1 - g.y0).max().unwrap_or(0);
    // the digits' line: a short mark above or below it is not text (a "Competitive" tag's edge read as '-')
    let tall = glyphs.iter().filter(|g| (g.y1 - g.y0) * 2 >= height);
    let top = tall.clone().map(|g| g.y0).min().unwrap_or(0);
    let bottom = tall.map(|g| g.y1).max().unwrap_or(0);
    let glyphs: Vec<&Glyph> = glyphs
        .iter()
        .filter(|g| height < 6 || (top..=bottom).contains(&((g.y0 + g.y1) / 2)))
        .collect();
    // ink level: background is the darkest, text the brightest of the rectangle
    let mut sorted: Vec<u8> = (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| value(img, x, y))
        .collect();
    sorted.sort();
    let (bg, fg) = (
        sorted[sorted.len() / 5] as f32,
        sorted[sorted.len() - 1 - sorted.len() / 200] as f32,
    );
    let norm = |x: i64, y: i64| -> f32 {
        if x < 0 || y < 0 || x >= img.width() as i64 || y >= img.height() as i64 {
            return 0.0;
        }
        ((value(img, x as u32, y as u32) as f32 - bg) / (fg - bg).max(1.0)).clamp(0.0, 1.0)
    };
    let debug = std::env::var("GLYPH_DEBUG").is_ok();
    let mut s = String::new();
    for g in &glyphs {
        let (w, h) = (g.x1 - g.x0, g.y1 - g.y0);
        if h * 2 < height {
            s.push(if w > h * 2 { '-' } else { '.' });
            continue;
        }
        if height < 6 && w > h {
            s.push('-');
            continue;
        }
        let size0 = height as f32 / digit_ratio();
        let mut per: Vec<(char, f32)> = vec![];
        for c in "0123456789".chars() {
            let mut best = f32::MAX;
            for k in -2..=2 {
                let px = size0 * (1.0 + k as f32 * 0.03);
                for fx in 0..4u8 {
                    for fy in 0..4u8 {
                        let d = drawing(c, px, fx, fy);
                        // place the drawing's ink box on the glyph's, a pixel either way
                        for ox in -1..=1i64 {
                            for oy in -1..=1i64 {
                                let (left, top) = (g.x0 as i64 + ox, g.y0 as i64 + oy);
                                let (rx0, rx1) = (
                                    (g.x0 as i64 - 1).min(left),
                                    (g.x1 as i64 + 1).max(left + d.w as i64),
                                );
                                let (ry0, ry1) = (
                                    (g.y0 as i64 - 1).min(top),
                                    (g.y1 as i64 + 1).max(top + d.h as i64),
                                );
                                let mut sum = 0.0;
                                for y in ry0..ry1 {
                                    for x in rx0..rx1 {
                                        let (dx, dy) = (x - left, y - top);
                                        let want = if dx >= 0
                                            && dy >= 0
                                            && (dx as usize) < d.w
                                            && (dy as usize) < d.h
                                        {
                                            d.cover[dy as usize * d.w + dx as usize]
                                        } else {
                                            0.0
                                        };
                                        sum += (norm(x, y) - want).abs();
                                    }
                                }
                                best = best.min(sum / ((rx1 - rx0) * (ry1 - ry0)) as f32);
                            }
                        }
                    }
                }
            }
            per.push((c, best));
        }
        per.sort_by(|a, b| a.1.total_cmp(&b.1));
        if debug {
            eprintln!("font glyph {w}x{h} at {}: {:?}", g.x0, &per[..3]);
        }
        if per[1].1 > per[0].1 * 1.25 {
            s.push(per[0].0)
        } else {
            s.push('?')
        }
    }
    s
}
