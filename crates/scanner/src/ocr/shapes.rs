// Reads a slot's number by its shapes: every candidate number is drawn in the game's font, with its
// shadow, over the background expected behind it, and the one that looks most like the crop wins.
// The crop is compared as captured; nothing is cleaned up first.
use super::number::number_background;
use crate::{
    anchor_to_slot::anchor_defs::{ICON_WIDTH, NUMBER_HEIGHT},
    dev_utils::timing::timed,
};
use ab_glyph::{Font, FontVec, PxScale, point};
use image::{ImageBuffer, Luma, RgbaImage};
use imageproc::filter::gaussian_blur_f32;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::LazyLock};

// Where the game draws the number, in 1440p UI pixels within the number crop, measured with the
// English font: right-aligned to RIGHT, on BASELINE, at SIZE px a em (13 px at 1080p).
const RIGHT: f64 = 59.11;
const BASELINE: f64 = 15.125;
const SIZE: f64 = 17.28;
// The digit is drawn this blurred; its shadow is the digit blurred by SHADOW_BLUR at
// SHADOW_STRENGTH times its coverage, in a near-black grey.
const BLUR: f64 = 0.35;
const SHADOW_BLUR: f64 = 3.0;
const SHADOW_STRENGTH: f64 = 4.0;
const SHADOW_GREY: f64 = 8.0;
// drawn either side of a digit's cell, for its shadow
const MARGIN: f64 = 2.0;
// at most this many digits; "9999+" is one more candidate
const CELLS: usize = 5;
// A read that beats its runner-up by less than this (luma a pixel) is left to the OCR.
pub const MIN_MARGIN: f64 = 2.0;

pub struct NumberFont {
    font: FontVec,
    // the candidates drawn for each crop size
    drawn: HashMap<(u32, u32), Drawn>,
}

pub static NUMBER_FONT: LazyLock<RwLock<Option<NumberFont>>> = LazyLock::new(|| RwLock::new(None));

// None if the bytes are not a font with every digit and "+"
pub fn number_font(bytes: Vec<u8>) -> Option<FontVec> {
    FontVec::try_from_vec(bytes)
        .ok()
        .filter(|font| "0123456789+".chars().all(|c| font.glyph_id(c).0 != 0))
}

// the bytes must pass number_font
pub fn load_number_font(bytes: Vec<u8>) {
    *NUMBER_FONT.write() = Some(NumberFont {
        font: number_font(bytes).unwrap(),
        drawn: HashMap::new(),
    });
}

// positions tried, in crop pixels
fn shifts() -> Vec<(f64, f64)> {
    let mut out = vec![];
    for dx in -4..=4 {
        for dy in -3..=3 {
            out.push((dx as f64 / 4.0, dy as f64 / 4.0));
        }
    }
    out
}

// a drawing over a region: observed = a + b * background
#[derive(Clone)]
struct Layer {
    a: Vec<f32>,
    b: Vec<f32>,
}

struct Drawn {
    cols: Vec<(usize, usize)>, // each cell's columns, the rightmost cell first
    cells: Vec<Vec<Vec<Layer>>>, // [cell][digit][shift], over the cell's columns
    plus: Vec<Layer>,          // [shift], "9999+" over the whole crop
}

fn blur(image: &[f32], w: usize, h: usize, sigma: f64) -> Vec<f32> {
    let image = ImageBuffer::<Luma<f32>, _>::from_raw(w as u32, h as u32, image.to_vec()).unwrap();
    gaussian_blur_f32(&image, sigma as f32).into_raw()
}

// These characters at these pen positions (crop pixels) drawn over columns from..to, as a layer.
fn layer(
    font: &FontVec,
    chars: &[(char, f64)],
    scale: (f64, f64),
    baseline: f64,
    (from, to): (usize, usize),
    h: usize,
) -> Layer {
    // drawn wider than kept, so the shadow has room
    let pad = (MARGIN * scale.0).ceil() as i64 + 1;
    let left = from as i64 - pad;
    let w = (to - from) + 2 * pad as usize;
    let em = font.units_per_em().unwrap() as f64 / font.height_unscaled() as f64;
    let px = PxScale {
        x: (SIZE * scale.0 / em) as f32,
        y: (SIZE * scale.1 / em) as f32,
    };
    let mut cover = vec![0.0f32; w * h];
    for (c, pen) in chars {
        let glyph = font
            .glyph_id(*c)
            .with_scale_and_position(px, point((*pen - left as f64) as f32, baseline as f32));
        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();
            outline.draw(|x, y, v| {
                let (x, y) = (
                    bounds.min.x as i64 + x as i64,
                    bounds.min.y as i64 + y as i64,
                );
                if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                    let at = &mut cover[y as usize * w + x as usize];
                    *at = (*at + v).min(1.0);
                }
            });
        }
    }
    let cover = blur(&cover, w, h, BLUR * scale.0);
    let shadow = blur(&cover, w, h, SHADOW_BLUR * scale.0);
    let (mut a, mut b) = (vec![], vec![]);
    for y in 0..h {
        for x in pad as usize..pad as usize + (to - from) {
            let al = cover[y * w + x] as f64;
            let sh = if al < 0.9 {
                (SHADOW_STRENGTH * shadow[y * w + x] as f64).min(1.0)
            } else {
                0.0
            };
            a.push((255.0 * al + (1.0 - al) * sh * SHADOW_GREY) as f32);
            b.push(((1.0 - al) * (1.0 - sh)) as f32);
        }
    }
    Layer { a, b }
}

fn draw(font: &FontVec, w: usize, h: usize) -> Drawn {
    let scale = (w as f64 / ICON_WIDTH, h as f64 / NUMBER_HEIGHT);
    let em = font.units_per_em().unwrap() as f64;
    let advance = |c: char| font.h_advance_unscaled(font.glyph_id(c)) as f64 / em * SIZE * scale.0;
    let (digit, plus) = (advance('0'), advance('+'));
    let right = RIGHT * scale.0;
    let cols: Vec<(usize, usize)> = (0..CELLS)
        .map(|k| {
            let left = right - (k + 1) as f64 * digit;
            (
                left.floor().max(0.0) as usize,
                ((left + digit).ceil() as usize).min(w),
            )
        })
        .collect();
    let cells = (0..CELLS)
        .map(|k| {
            let left = right - (k + 1) as f64 * digit;
            (0..10)
                .map(|d| {
                    let c = char::from(b'0' + d);
                    shifts()
                        .iter()
                        .map(|(dx, dy)| {
                            let baseline = BASELINE * scale.1 + dy;
                            layer(font, &[(c, left + dx)], scale, baseline, cols[k], h)
                        })
                        .collect()
                })
                .collect()
        })
        .collect();
    let plus = shifts()
        .iter()
        .map(|(dx, dy)| {
            let end = right - plus + dx;
            let chars: Vec<(char, f64)> = (0..4)
                .map(|j| ('9', end - (4 - j) as f64 * digit))
                .chain([('+', end)])
                .collect();
            layer(font, &chars, scale, BASELINE * scale.1 + dy, (0, w), h)
        })
        .collect();
    Drawn { cols, cells, plus }
}

pub struct ShapeRead {
    pub text: String,
    pub total: f64,
    pub fit: f64,    // how far the number's cells are from the drawing, luma a pixel
    pub margin: f64, // how much worse the runner-up is, luma a pixel
    shift: usize,
    digits: Vec<usize>, // per cell from the right
}

// the positions within a quarter pixel of this one
fn around(si: usize) -> Vec<usize> {
    let (x, y) = shifts()[si];
    (0..shifts().len())
        .filter(|i| {
            let (dx, dy) = shifts()[*i];
            (dx - x).abs() < 0.3 && (dy - y).abs() < 0.3
        })
        .collect()
}

// the best number over these positions
fn decode(d: &Drawn, obs: &[f32], bg: &[f32], w: usize, h: usize, at: &[usize]) -> ShapeRead {
    let mut best: Option<ShapeRead> = None;
    for &si in at {
        let mut errs = [[0.0f64; 10]; CELLS];
        let mut blank = [0.0f64; CELLS];
        let mut plus = [0.0f64; CELLS];
        for k in 0..CELLS {
            let (c0, c1) = d.cols[k];
            let n = ((c1 - c0) * h).max(1) as f64;
            let p = &d.plus[si];
            let (mut eb, mut ep) = (0.0, 0.0);
            for y in 0..h {
                for x in c0..c1 {
                    let i = y * w + x;
                    eb += (bg[i] - obs[i]).abs() as f64;
                    ep += (p.a[i] + p.b[i] * bg[i] - obs[i]).abs() as f64;
                }
            }
            blank[k] = eb / n;
            plus[k] = ep / n;
            for (digit, err) in errs[k].iter_mut().enumerate() {
                let g = &d.cells[k][digit][si];
                let mut e = 0.0;
                let mut j = 0;
                for y in 0..h {
                    for x in c0..c1 {
                        let i = y * w + x;
                        e += (g.a[j] + g.b[j] * bg[i] - obs[i]).abs() as f64;
                        j += 1;
                    }
                }
                *err = e / n;
            }
        }
        // every candidate here: (total, text, its cells' error, least lead of a digit, digits)
        let mut candidates: Vec<(f64, String, f64, f64, Vec<usize>)> = vec![];
        for len in 0..=CELLS {
            let (mut total, mut text, mut used, mut lead) =
                (0.0, String::new(), 0.0, f64::INFINITY);
            let mut picked = vec![];
            for k in 0..CELLS {
                if k >= len {
                    total += blank[k];
                    continue;
                }
                // no leading zero, and a single item shows no number
                let ok = |d: usize| !(k == len - 1 && (d == 0 || (len == 1 && d == 1)));
                let mut ranked: Vec<(f64, usize)> = (0..10)
                    .filter(|d| ok(*d))
                    .map(|d| (errs[k][d], d))
                    .collect();
                ranked.sort_by(|a, b| a.0.total_cmp(&b.0));
                total += ranked[0].0;
                used += ranked[0].0;
                lead = lead.min(ranked[1].0 - ranked[0].0);
                text.insert(0, char::from(b'0' + ranked[0].1 as u8));
                picked.push(ranked[0].1);
            }
            candidates.push((total, text, used / len.max(1) as f64, lead, picked));
        }
        let plus_total: f64 = plus.iter().sum();
        candidates.push((
            plus_total,
            "9999+".into(),
            plus_total / CELLS as f64,
            f64::INFINITY,
            vec![],
        ));
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        let top = &candidates[0];
        if best.as_ref().is_none_or(|b| top.0 < b.total) {
            best = Some(ShapeRead {
                text: top.1.clone(),
                total: top.0,
                fit: if top.1.is_empty() {
                    top.0 / CELLS as f64
                } else {
                    top.2
                },
                margin: top.3.min(candidates[1].0 - top.0),
                shift: si,
                digits: top.4.clone(),
            });
        }
    }
    best.unwrap()
}

// How much the slot is lit (the hover highlight), from the first answer:
// observed - a = b * (gain * background + offset), least squares.
fn lit(d: &Drawn, read: &ShapeRead, obs: &[f32], bg: &[f32], w: usize, h: usize) -> Vec<f32> {
    let (mut a, mut b) = (vec![0.0f32; w * h], vec![1.0f32; w * h]);
    if read.text == "9999+" {
        let p = &d.plus[read.shift];
        (a, b) = (p.a.clone(), p.b.clone());
    }
    for (k, digit) in read.digits.iter().enumerate() {
        let (c0, c1) = d.cols[k];
        let g = &d.cells[k][*digit][read.shift];
        let mut j = 0;
        for y in 0..h {
            for x in c0..c1 {
                a[y * w + x] = g.a[j];
                b[y * w + x] = g.b[j];
                j += 1;
            }
        }
    }
    let (mut s11, mut s12, mut s22, mut t1, mut t2) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..obs.len() {
        let (u, v, y) = ((b[i] * bg[i]) as f64, b[i] as f64, (obs[i] - a[i]) as f64);
        s11 += u * u;
        s12 += u * v;
        s22 += v * v;
        t1 += u * y;
        t2 += v * y;
    }
    let det = s11 * s22 - s12 * s12;
    if det.abs() < 1e-9 {
        return bg.to_vec();
    }
    let gain = ((t1 * s22 - t2 * s12) / det).clamp(0.7, 2.0);
    let offset = ((s11 * t2 - s12 * t1) / det).clamp(-40.0, 80.0);
    bg.iter()
        .map(|v| (*v as f64 * gain + offset) as f32)
        .collect()
}

fn luma(p: [f64; 3]) -> f32 {
    (p[0] * 0.299 + p[1] * 0.587 + p[2] * 0.114) as f32
}

// The number on a slot by its shapes, however sure; None without a font. `number` is the crop as
// captured, `icon` the slot's icon crop and `template` its matched icon, as the clean-up takes them.
pub fn shape_read(
    number: &RgbaImage,
    icon: &RgbaImage,
    template: &RgbaImage,
    brightness: f64,
) -> Option<ShapeRead> {
    let (w, h) = number.dimensions();
    let mut guard = NUMBER_FONT.write();
    let NumberFont { font, drawn } = guard.as_mut()?;
    let d = drawn
        .entry((w, h))
        .or_insert_with(|| timed("ocr/shapes/draw", || draw(font, w as usize, h as usize)));
    let obs: Vec<f32> = number
        .pixels()
        .map(|p| luma([p[0] as f64, p[1] as f64, p[2] as f64]))
        .collect();
    let bg: Vec<f32> = timed("ocr/shapes/background", || {
        number_background(template, icon, w, h, brightness)
    })
    .into_iter()
    .map(luma)
    .collect();
    let (w, h) = (w as usize, h as usize);
    let all: Vec<usize> = (0..shifts().len()).collect();
    let first = timed("ocr/shapes/match", || decode(d, &obs, &bg, w, h, &all));
    // again over the background lit as the first answer says it is, near where that one was
    let bg = lit(d, &first, &obs, &bg, w, h);
    Some(timed("ocr/shapes/match", || {
        decode(d, &obs, &bg, w, h, &around(first.shift))
    }))
}

// the number if the shapes are sure of it
pub fn read_by_shapes(
    number: &RgbaImage,
    icon: &RgbaImage,
    template: &RgbaImage,
    brightness: f64,
) -> Option<String> {
    shape_read(number, icon, template, brightness)
        .filter(|x| x.margin >= MIN_MARGIN)
        .map(|x| x.text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev_utils::native::ROOT;

    // a number drawn by the model itself over a plain background reads back
    #[test]
    fn reads_its_own_drawing() {
        let bytes = std::fs::read(format!("{ROOT}/public/fonts/English.ttf")).unwrap();
        load_number_font(bytes);
        let guard = NUMBER_FONT.read();
        let font = &guard.as_ref().unwrap().font;
        let (w, h) = (46, 14);
        let d = draw(font, w, h);
        let bg = vec![60.0f32; w * h];
        let si = shifts().iter().position(|s| *s == (0.25, -0.25)).unwrap();
        for text in ["807", "35", "9999+", "1204"] {
            let mut obs = bg.clone();
            if text == "9999+" {
                let p = &d.plus[si];
                obs = (0..w * h).map(|i| p.a[i] + p.b[i] * bg[i]).collect();
            } else {
                for (k, c) in text.chars().rev().enumerate() {
                    let g = &d.cells[k][c.to_digit(10).unwrap() as usize][si];
                    let (c0, c1) = d.cols[k];
                    let mut j = 0;
                    for y in 0..h {
                        for x in c0..c1 {
                            obs[y * w + x] = g.a[j] + g.b[j] * bg[y * w + x];
                            j += 1;
                        }
                    }
                }
            }
            let all: Vec<usize> = (0..shifts().len()).collect();
            assert_eq!(decode(&d, &obs, &bg, w, h, &all).text, text);
        }
    }
}
