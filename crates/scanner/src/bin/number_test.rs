// Scores slot-number pre-processing variants with the real recogniser, on crops from number_dump.
//   cargo run --release --bin number_test -- <dump dir> [variant...]
use hf_scanner::{
    image_utils::{
        brightness::normalize_brightness,
        common::Rect,
        number::{NumberParams, number_background, number_mask},
        ocr::{number_strip, recognize_line},
    },
    native,
    scanner_state::ScannerState,
    setup::{BASE_ICONS, OneIconConfig},
};
use image::{
    GrayImage, Luma, Rgba, RgbaImage,
    imageops::{FilterType, crop_imm, resize},
};
use imageproc::region_labelling::{Connectivity, connected_components};
use std::{collections::HashMap, env, fs};


fn luma(p: &Rgba<u8>) -> i32 {
    (p[0] as i32 * 299 + p[1] as i32 * 587 + p[2] as i32 * 114) / 1000
}
fn saturation(p: &Rgba<u8>) -> i32 {
    let (max, min) = (
        p[0].max(p[1]).max(p[2]) as i32,
        p[0].min(p[1]).min(p[2]) as i32,
    );
    if max == 0 { 0 } else { (max - min) * 255 / max }
}

// the slot's template where the number is drawn, lined up with the observed strip
fn background(name: &str, observed: &RgbaImage) -> RgbaImage {
    let base = BASE_ICONS.read()[name].data.clone();
    let (w, h) = observed.dimensions();
    let scaled = resize(&base, w + 2, w + 2, FilterType::CatmullRom);
    let top = (4.0 * w as f64 / 61.0).round() as u32;
    let mut best = (u64::MAX, RgbaImage::new(1, 1));
    for dy in 0..3 {
        for dx in 0..3 {
            let candidate = crop_imm(&scaled, dx, top + dy, w, h).to_image();
            // the darkest two thirds of differences, so the number itself does not steer the alignment
            let mut diffs: Vec<u64> = candidate
                .pixels()
                .zip(observed.pixels())
                .map(|(a, b)| (0..3).map(|c| a[c].abs_diff(b[c]) as u64).sum())
                .collect();
            diffs.sort();
            let score = diffs[..diffs.len() * 2 / 3].iter().sum();
            if score < best.0 {
                best = (score, candidate);
            }
        }
    }
    best.1
}

fn finish(mask: &GrayImage, invert: bool) -> RgbaImage {
    let image = RgbaImage::from_fn(mask.width(), mask.height(), |x, y| {
        let v = mask.get_pixel(x, y)[0];
        let v = if invert { 255 - v } else { v };
        Rgba([v, v, v, 255])
    });
    let image = if param("CROP", 0) > 0 {
        let cols: Vec<u32> = (0..mask.width())
            .filter(|x| (0..mask.height()).any(|y| mask.get_pixel(*x, y)[0] > 0))
            .collect();
        match (cols.first(), cols.last()) {
            (Some(a), Some(b)) => {
                let pad = param("CROP", 0) as u32;
                let x0 = a.saturating_sub(pad);
                let x1 = (b + 1 + pad).min(mask.width());
                crop_imm(&image, x0, 0, x1 - x0, image.height()).to_image()
            }
            _ => image,
        }
    } else {
        image
    };
    let width = image.width() * 64 / image.height();
    let filter = match param("FILTER", 0) {
        1 => FilterType::Triangle,
        2 => FilterType::Lanczos3,
        3 => FilterType::Nearest,
        _ => FilterType::CatmullRom,
    };
    resize(&image, width, 64, filter)
}

fn param(name: &str, default: i32) -> i32 {
    env::var(name)
        .map(|x| x.parse().unwrap())
        .unwrap_or(default)
}

// white-ish pixels, the same test the current pipeline's first two steps make
fn white_mask(observed: &RgbaImage) -> GrayImage {
    GrayImage::from_fn(observed.width(), observed.height(), |x, y| {
        let p = observed.get_pixel(x, y);
        if param("SOFT", 0) > 0 {
            // ramps instead of cut-offs, so the letter edges stay smooth
            let soft = param("SOFT", 0);
            let by_sat = (param("SAT", 40) + soft - saturation(p)).clamp(0, 2 * soft);
            let by_luma = (luma(p) - param("LUMA", 100) + soft).clamp(0, 2 * soft);
            return Luma([(luma(p) * by_sat * by_luma / (4 * soft * soft)) as u8]);
        }
        let on = saturation(p) <= param("SAT", 40) && luma(p) >= param("LUMA", 100);
        Luma([if on { luma(p) as u8 } else { 0 }])
    })
}

fn white_mask_with(image: &RgbaImage, sat: i32, min_luma: i32) -> GrayImage {
    GrayImage::from_fn(image.width(), image.height(), |x, y| {
        let p = image.get_pixel(x, y);
        Luma([if saturation(p) <= sat && luma(p) >= min_luma {
            255
        } else {
            0
        }])
    })
}

// how much brighter / darker the observed pixel is than anything the template has around that spot
fn brighter_darker(observed: &RgbaImage, bg: &RgbaImage, x: u32, y: u32) -> (i32, i32) {
    let (w, h) = observed.dimensions();
    let (mut max, mut min) = (0, 255);
    for ny in y.saturating_sub(1)..(y + 2).min(h) {
        for nx in x.saturating_sub(1)..(x + 2).min(w) {
            let l = luma(bg.get_pixel(nx, ny));
            max = max.max(l);
            min = min.min(l);
        }
    }
    let l = luma(observed.get_pixel(x, y));
    ((l - max).max(0), (min - l).max(0))
}

fn components(
    mask: &GrayImage,
) -> (
    image::ImageBuffer<Luma<u32>, Vec<u32>>,
    HashMap<u32, Vec<(u32, u32)>>,
) {
    let binary = GrayImage::from_fn(mask.width(), mask.height(), |x, y| {
        Luma([if mask.get_pixel(x, y)[0] > 0 { 255 } else { 0 }])
    });
    let labels = connected_components(&binary, Connectivity::Four, Luma([0u8]));
    let mut members: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
    for (x, y, label) in labels.enumerate_pixels() {
        if label[0] != 0 {
            members.entry(label[0]).or_default().push((x, y));
        }
    }
    (labels, members)
}

fn fparam(name: &str, default: f64) -> f64 {
    env::var(name)
        .map(|x| x.parse().unwrap())
        .unwrap_or(default)
}

// the compositing model, then the same specks / left crop / resize as the current pipeline
fn model(raw: &RgbaImage, icon: &RgbaImage, icon_name: &str, brightness: f64) -> RgbaImage {
    let defaults = NumberParams::default();
    let params = NumberParams {
        tolerance: fparam("TOL", defaults.tolerance),
        min_alpha: fparam("ALPHA", defaults.min_alpha),
        max_shade: fparam("SHADE", defaults.max_shade),
        reach: fparam("REACH", defaults.reach),
        min_luma: fparam("LUMA", defaults.min_luma),
        max_saturation: fparam("SAT", defaults.max_saturation),
        soft: fparam("SOFT", defaults.soft),
        min_chroma: fparam("CHROMA", defaults.min_chroma),
        low_alpha: fparam("LOW", defaults.low_alpha),
    };
    let template = BASE_ICONS.read()[icon_name].data.clone();
    let (w, h) = raw.dimensions();
    let background = number_background(&template, icon, w, h, brightness);
    if env::var("SHOW_BG").is_ok() {
        return RgbaImage::from_fn(w, h, |x, y| {
            let p = background[(y * w + x) as usize];
            Rgba([p[0] as u8, p[1] as u8, p[2] as u8, 255])
        });
    }
    let mut mask = number_mask(raw, &background, &params);
    let binary = GrayImage::from_fn(w, h, |x, y| Luma([(mask.get_pixel(x, y)[0] > 0) as u8]));
    let labels = connected_components(&binary, Connectivity::Eight, Luma([0u8]));
    let mut sizes: HashMap<u32, u32> = HashMap::new();
    for label in labels.pixels() {
        *sizes.entry(label[0]).or_default() += 1;
    }
    let speck = fparam("SPECK", (h * h / 32) as f64) as u32;
    for (x, y, label) in labels.enumerate_pixels() {
        if label[0] != 0 && sizes[&label[0]] <= speck {
            mask.put_pixel(x, y, Luma([0]));
        }
    }
    let image = RgbaImage::from_fn(w, h, |x, y| {
        let v = mask.get_pixel(x, y)[0];
        Rgba([v, v, v, 255])
    });
    let first = (0..w).find(|x| (0..h).any(|y| mask.get_pixel(*x, y)[0] > 0));
    let margin = fparam("LEFT", (h / 5) as f64) as u32;
    let left = if margin >= 100 {
        0
    } else {
        first.map_or(0, |x| x.saturating_sub(margin))
    };
    let image = crop_imm(&image, left, 0, w - left, h).to_image();
    resize(&image, image.width() * 64 / h, 64, FilterType::CatmullRom)
}

fn variant(
    name: &str,
    raw: &OneIconConfig,
    icon: &RgbaImage,
    icon_name: &str,
    brightness: f64,
    state: &mut ScannerState,
) -> RgbaImage {
    if name == "model" {
        return model(&raw.data, icon, icon_name, brightness);
    }
    if name == "current" {
        let _ = state;
        return number_strip(
            &raw.data,
            icon,
            &BASE_ICONS.read()[icon_name].data.clone(),
            brightness,
            &NumberParams::default(),
        );
    }
    let mut normalised = raw.clone();
    normalize_brightness(&mut normalised, brightness);
    let observed = normalised.data;
    let bg = background(icon_name, &observed);
    let (w, h) = observed.dimensions();
    let invert = !name.ends_with("_noinv");
    match name.trim_end_matches("_noinv") {
        "bg" => resize(&bg, w * 64 / h, 64, FilterType::CatmullRom),
        "obs" => resize(&observed, w * 64 / h, 64, FilterType::CatmullRom),
        // just the white test
        "white" => finish(&white_mask(&observed), invert),
        // white, and brighter than the template says the spot should be
        "brighter" => {
            let white = white_mask(&observed);
            let mask = GrayImage::from_fn(w, h, |x, y| {
                let keep = brighter_darker(&observed, &bg, x, y).0 >= param("BRIGHTER", 30);
                Luma([if keep { white.get_pixel(x, y)[0] } else { 0 }])
            });
            finish(&mask, invert)
        }
        // white blobs are kept if they are brighter than the template, or ringed by the number's dark outline
        "blobs" => {
            let white = white_mask(&observed);
            let (labels, members) = components(&white);
            let mut mask = GrayImage::new(w, h);
            for pixels in members.values() {
                let brighter = pixels
                    .iter()
                    .filter(|(x, y)| {
                        brighter_darker(&observed, &bg, *x, *y).0 >= param("BRIGHTER", 30)
                    })
                    .count();
                let (mut ring, mut dark) = (0, 0);
                for (x, y) in pixels {
                    for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                        let (nx, ny) = (*x as i32 + dx, *y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let (nx, ny) = (nx as u32, ny as u32);
                        if labels.get_pixel(nx, ny)[0] != 0 {
                            continue;
                        }
                        ring += 1;
                        dark += (brighter_darker(&observed, &bg, nx, ny).1 >= param("DARKER", 25))
                            as usize;
                    }
                }
                let keep = brighter * 100 >= pixels.len() * param("BRIGHT_SHARE", 50) as usize
                    || (ring > 0 && dark * 100 >= ring * param("RING_SHARE", 60) as usize);
                if keep {
                    for (x, y) in pixels {
                        mask.put_pixel(*x, *y, *white.get_pixel(*x, *y));
                    }
                }
            }
            finish(&mask, invert)
        }
        // white blobs are dropped when the template is white there too and no dark outline rings them
        "clean" => {
            let white = white_mask(&observed);
            let bg_white = white_mask_with(
                &bg,
                param("SAT", 40) + param("BG_SLACK", 20),
                param("LUMA", 100) - param("BG_SLACK", 20),
            );
            let (labels, members) = components(&white);
            let mut mask = white.clone();
            for pixels in members.values() {
                let explained = pixels
                    .iter()
                    .filter(|(x, y)| {
                        (y.saturating_sub(1)..(y + 2).min(h)).any(|ny| {
                            (x.saturating_sub(1)..(x + 2).min(w))
                                .any(|nx| bg_white.get_pixel(nx, ny)[0] > 0)
                        })
                    })
                    .count();
                let (mut ring, mut dark) = (0, 0);
                for (x, y) in pixels {
                    for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                        let (nx, ny) = (*x as i32 + dx, *y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let (nx, ny) = (nx as u32, ny as u32);
                        if labels.get_pixel(nx, ny)[0] != 0 {
                            continue;
                        }
                        ring += 1;
                        dark += (brighter_darker(&observed, &bg, nx, ny).1 >= param("DARKER", 25))
                            as usize;
                    }
                }
                let outlined = ring > 0 && dark * 100 >= ring * param("RING_SHARE", 50) as usize;
                // a number is whiter than whatever the template has under it
                let whiter = pixels
                    .iter()
                    .map(|(x, y)| luma(observed.get_pixel(*x, *y)) - luma(bg.get_pixel(*x, *y)))
                    .sum::<i32>()
                    >= param("WHITER", 20) * pixels.len() as i32;
                if explained * 100 >= pixels.len() * param("EXPLAINED", 60) as usize
                    && !outlined
                    && !whiter
                {
                    for (x, y) in pixels {
                        mask.put_pixel(*x, *y, Luma([0]));
                    }
                }
            }
            finish(&mask, invert)
        }
        // no masking at all: the difference from the template, as a grey image
        "difference" => {
            let mask = GrayImage::from_fn(w, h, |x, y| {
                let (brighter, _) = brighter_darker(&observed, &bg, x, y);
                Luma([(brighter * 255 / 160).min(255) as u8])
            });
            finish(&mask, invert)
        }
        other => panic!("unknown variant {other}"),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let dir = &args[0];
    native::load();
    let mut state = ScannerState::default();

    let labels = fs::read_to_string(format!("{dir}/labels.tsv")).unwrap();
    let samples: Vec<Vec<&str>> = labels
        .lines()
        .map(|l| l.split('\t').collect())
        .filter(|s: &Vec<&str>| s[3] != "Lv")
        .collect();
    let save = env::var("SAVE").ok();
    for name in &args[1..] {
        let (mut right, mut right_real, mut real, mut right_strict) = (0, 0, 0, 0);
        let mut wrong = vec![];
        for sample in &samples {
            let (id, icon_name, brightness, label) =
                (sample[0], sample[1], sample[2].parse().unwrap(), sample[3]);
            let data = image::open(format!("{dir}/{id}.png")).unwrap().to_rgba8();
            let raw = OneIconConfig {
                offset: Rect::ui((0.0, 0.0), data.width() as f64, data.height() as f64),
                data,
                name: String::new(),
                tag: String::new(),
                required_confidence: None,
            };
            let icon = image::open(format!("{dir}/{id}_icon.png"))
                .unwrap()
                .to_rgba8();
            let processed = variant(name, &raw, &icon, icon_name, brightness, &mut state);
            let read = recognize_line(&processed, true);
            let digits: String = read.chars().filter(char::is_ascii_digit).collect();
            let ok = digits == label;
            right += ok as usize;
            right_strict += (read.trim().trim_end_matches('+') == label) as usize;
            // "Fusion" is mostly other items the icon matcher let through
            if icon_name != "Fusion" {
                real += 1;
                right_real += ok as usize;
            }
            if !ok {
                wrong.push(format!("{id} {icon_name}: '{read}' want '{label}'"));
            }
            if let Some(save) = &save {
                fs::create_dir_all(format!("{save}/{name}")).unwrap();
                processed.save(format!("{save}/{name}/{id}.png")).unwrap();
            }
        }
        println!(
            "{name}: digits right {right}/{} ({right_real}/{real} without Fusion), whole string right {right_strict}",
            samples.len()
        );
        if env::var("WRONG").is_ok() {
            for line in wrong {
                println!("    {line}");
            }
        }
    }
}
