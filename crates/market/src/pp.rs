// Text by PP-OCRv5 mobile English (Hugging Face monkt/paddleocr-onnx, languages/english: rec.onnx and dict.txt
// in ignored/market/models/en_PP-OCRv5_mobile_rec), run by rten. It reads the purchase window's names and the
// Auction House's time left far better than ocrs (verify_strips.py, 2026-11-20).
use image::{GrayImage, RgbaImage, imageops};
use rten::Model;
use rten_tensor::{NdTensor, prelude::*};
use std::sync::OnceLock;

const HEIGHT: u32 = 48;

fn model() -> &'static (Model, Vec<String>) {
    static M: OnceLock<(Model, Vec<String>)> = OnceLock::new();
    M.get_or_init(|| {
        let model = Model::load_static_slice(crate::assets::PP_REC).unwrap();
        let dict = crate::assets::PP_DICT;
        (model, dict.split('\n').map(str::to_string).collect())
    })
}

fn percentile(v: &[f32], p: f32) -> f32 {
    let mut s = v.to_vec();
    s.sort_by(f32::total_cmp);
    s[((s.len() - 1) as f32 * p / 100.0).round() as usize]
}

// Light text on dark, as the model wants it: the ink's columns (the largest run, so a table's divider
// does not count), padded, stretched, inverted, 48 px high.
fn strip(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> Option<GrayImage> {
    let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let g: Vec<f32> = (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| {
            let p = img.get_pixel(x, y);
            (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0
        })
        .collect();
    let bg = percentile(&g, 50.0);
    let dev: Vec<f32> = (0..w)
        .map(|x| {
            (0..h)
                .map(|y| (g[y * w + x] - bg).abs())
                .fold(0.0, f32::max)
        })
        .collect();
    let limit = 40f32.max(0.35 * dev.iter().cloned().fold(0.0, f32::max));
    let on: Vec<usize> = (0..w).filter(|&x| dev[x] > limit).collect();
    // runs of ink columns, split where the gap is over twice the height
    let mut runs: Vec<(usize, usize, usize)> = vec![];
    for &x in &on {
        match runs.last_mut() {
            Some(r) if x - r.1 <= 2 * h => {
                r.1 = x;
                r.2 += 1;
            }
            _ => runs.push((x, x, 1)),
        }
    }
    let &(a, b, _) = runs.iter().max_by_key(|r| r.2)?;
    let (a, b) = (a.saturating_sub(2), (b + 3).min(w));
    let pad = 4;
    let (cw, ch) = (b - a + 2 * pad, h + 2 * pad);
    let mut c = vec![bg; cw * ch];
    for y in 0..h {
        for x in a..b {
            c[(y + pad) * cw + x - a + pad] = g[y * w + x];
        }
    }
    let (lo, hi) = (percentile(&c, 1.0), percentile(&c, 99.5));
    let px: Vec<u8> = c
        .iter()
        .map(|v| 255 - (((v - lo) / (hi - lo).max(1.0)).clamp(0.0, 1.0) * 255.0) as u8)
        .collect();
    let gray = GrayImage::from_raw(cw as u32, ch as u32, px).unwrap();
    let nw = ((cw as f32 * HEIGHT as f32 / ch as f32).round() as u32).max(8);
    Some(imageops::resize(
        &gray,
        nw,
        HEIGHT,
        imageops::FilterType::CatmullRom,
    ))
}

/// The text in a rectangle, "" when there is no ink.
pub fn read(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> String {
    let Some(s) = strip(img, x0, x1, y0, y1) else {
        return String::new();
    };
    let (model, dict) = model();
    let (w, h) = (s.width() as usize, s.height() as usize);
    let mut input = NdTensor::<f32, 4>::zeros([1, 3, h, w]);
    for (x, y, p) in s.enumerate_pixels() {
        let v = (p[0] as f32 / 255.0 - 0.5) / 0.5;
        for c in 0..3 {
            input[[0, c, y as usize, x as usize]] = v;
        }
    }
    let out: NdTensor<f32, 3> = model
        .run_one(input.view().into(), None)
        .unwrap()
        .try_into()
        .unwrap();
    // CTC: the best class per step, repeats and blanks (0) dropped
    let mut text = String::new();
    let mut prev = 0;
    for t in 0..out.size(1) {
        let row = out.slice((0, t));
        let best = (0..row.size(0))
            .max_by(|&a, &b| row[[a]].total_cmp(&row[[b]]))
            .unwrap();
        if best != prev && best != 0 {
            text += dict.get(best - 1).map_or(" ", |s| s.as_str());
        }
        prev = best;
    }
    text
}
