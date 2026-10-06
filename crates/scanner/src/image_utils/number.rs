use crate::{
    constants::{
        COMBINED_NUMBER_HEIGHT, ICON_HEIGHT, ICON_WIDTH, NUMBER_HEIGHT, NUMBER_TOP_MARGIN,
    },
    image_utils::brightness::brightness_lut,
};
use image::{GrayImage, Luma, Rgba, RgbaImage};

type Rgb = [f64; 3];

pub struct NumberParams {
    pub max_saturation: f64, // a digit is white: unsaturated ...
    pub min_luma: f64,       // ... and bright
    pub soft: f64,           // both limits are ramps this wide either side
    pub tolerance: f64,      // how far the template may be off, per channel
    pub min_alpha: f64,      // white the template cannot explain by at least this much is a digit
    pub max_shade: f64, // a pixel at most this fraction of the template's brightness is the number's shadow
    pub reach: f64,     // the shadow has to be within the strip's height over this, on both sides
}

impl Default for NumberParams {
    fn default() -> Self {
        NumberParams {
            max_saturation: 40.0,
            min_luma: 100.0,
            soft: 10.0,
            tolerance: 24.0,
            min_alpha: 0.55,
            max_shade: 0.6,
            reach: 4.0,
        }
    }
}

fn rgb(p: &Rgba<u8>) -> Rgb {
    [p[0] as f64, p[1] as f64, p[2] as f64]
}

fn luma(p: Rgb) -> f64 {
    p[0] * 0.299 + p[1] * 0.587 + p[2] * 0.114
}

// 0 for anything that is not white, the luma for white, a ramp between
fn whiteness(p: Rgb, params: &NumberParams) -> f64 {
    let (max, min) = (p[0].max(p[1]).max(p[2]), p[0].min(p[1]).min(p[2]));
    let saturation = if max == 0.0 {
        0.0
    } else {
        (max - min) * 255.0 / max
    };
    let ramp = |over: f64| ((over + params.soft) / (2.0 * params.soft)).clamp(0.0, 1.0);
    luma(p) * ramp(params.max_saturation - saturation) * ramp(luma(p) - params.min_luma)
}

// The template resampled as the game would draw it into a crop of w x h pixels that covers the slot's
// full width and `rows` 1440p rows from `top`, moved by `shift` 1440p pixels.
fn sample_template(
    template: &RgbaImage,
    w: u32,
    h: u32,
    top: f64,
    rows: f64,
    shift: (f64, f64),
) -> Vec<Rgb> {
    let to_template = template.width() as f64 / ICON_WIDTH;
    let at = |x: f64, y: f64| {
        let clamp = |v: f64, size: u32| v.clamp(0.0, size as f64 - 1.0);
        let (x, y) = (clamp(x, template.width()), clamp(y, template.height()));
        let (x0, y0) = (x.floor() as u32, y.floor() as u32);
        let (x1, y1) = (
            (x0 + 1).min(template.width() - 1),
            (y0 + 1).min(template.height() - 1),
        );
        let (fx, fy) = (x - x0 as f64, y - y0 as f64);
        let mix = |a: Rgb, b: Rgb, f: f64| [0, 1, 2].map(|c| a[c] * (1.0 - f) + b[c] * f);
        mix(
            mix(
                rgb(template.get_pixel(x0, y0)),
                rgb(template.get_pixel(x1, y0)),
                fx,
            ),
            mix(
                rgb(template.get_pixel(x0, y1)),
                rgb(template.get_pixel(x1, y1)),
                fx,
            ),
            fy,
        )
    };
    let mut out = Vec::with_capacity((w * h) as usize);
    for j in 0..h {
        for i in 0..w {
            // four samples per pixel, since the template is larger than the crop
            let mut sum = [0.0; 3];
            for (sx, sy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                let x = ((i as f64 + sx) * ICON_WIDTH / w as f64 + shift.0) * to_template - 0.5;
                let y = (top + (j as f64 + sy) * rows / h as f64 + shift.1) * to_template - 0.5;
                let sample = at(x, y);
                sum = [0, 1, 2].map(|c| sum[c] + sample[c] / 4.0);
            }
            out.push(sum);
        }
    }
    out
}

// where the template sits in this slot, found on the icon below the number, which nothing covers
pub fn align(template: &RgbaImage, icon: &RgbaImage) -> (f64, f64) {
    let (w, h) = icon.dimensions();
    let score = |shift: (f64, f64)| {
        sample_template(template, w, h, COMBINED_NUMBER_HEIGHT, ICON_HEIGHT, shift)
            .iter()
            .zip(icon.pixels())
            .map(|(a, b)| (0..3).map(|c| (a[c] - b[c] as f64).abs()).sum::<f64>())
            .sum::<f64>()
    };
    let mut best = (f64::MAX, (0.0, 0.0));
    // The grid is good to about a twentieth of a pixel since anchor templates are cut to whole
    // pixels, so a quarter pixel either way is enough: every slot over the native stills wants no
    // more, and reads match the old two-pass search over four times this span.
    for dy in -1..=1 {
        for dx in -1..=1 {
            let shift = (dx as f64 * 0.25, dy as f64 * 0.25);
            let s = score(shift);
            if s < best.0 {
                best = (s, shift);
            }
        }
    }
    best.1
}

// what the slot looks like behind the number, at the brightness the game is showing it
pub fn number_background(
    template: &RgbaImage,
    icon: &RgbaImage,
    w: u32,
    h: u32,
    brightness: f64,
) -> Vec<Rgb> {
    let lut = brightness_lut(brightness);
    // the template is at the reference brightness; undo the normalisation to get what is on screen
    let inverse: [f64; 256] = std::array::from_fn(|reference| {
        (0..256)
            .min_by_key(|v| (lut[*v] as i32 - reference as i32).abs())
            .unwrap() as f64
    });
    sample_template(
        template,
        w,
        h,
        NUMBER_TOP_MARGIN,
        NUMBER_HEIGHT,
        align(template, icon),
    )
    .into_iter()
    .map(|p| p.map(|v| inverse[v.round().clamp(0.0, 255.0) as usize]))
    .collect()
}

// The game draws the number over the icon as opaque white digits with a soft black shadow around them,
// and its brightness setting changes the icon but not the number. So a pixel is
//     white * alpha + background * shade * (1 - alpha)
// Per pixel this gives the least white (alpha) the template cannot account for, and how much the
// template had to be darkened (shade). Either tells a digit from a white part of the icon: the digit is
// white the template does not have, or white inside the shadow.
pub fn number_mask(number: &RgbaImage, background: &[Rgb], params: &NumberParams) -> GrayImage {
    let (w, h) = number.dimensions();
    let bg = |x: i32, y: i32| {
        background[(y.clamp(0, h as i32 - 1) * w as i32 + x.clamp(0, w as i32 - 1)) as usize]
    };
    let mut alpha = vec![1.0; (w * h) as usize];
    let mut shadow = vec![false; (w * h) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let observed = rgb(number.get_pixel(x as u32, y as u32));
            'alpha: for step in 0..16 {
                let a = step as f64 / 16.0;
                let target = observed.map(|v| (v - a * 255.0) / (1.0 - a));
                let mut least_shade: Option<f64> = None;
                // the template can be a pixel off, so any neighbour may explain this one
                for (dx, dy) in (0..9).map(|n| (n % 3 - 1, n / 3 - 1)) {
                    let candidate = bg(x + dx, y + dy);
                    let norm: f64 = candidate.iter().map(|v| v * v).sum::<f64>().max(1.0);
                    let s = ((0..3).map(|c| target[c] * candidate[c]).sum::<f64>() / norm)
                        .clamp(0.0, 1.0);
                    let error = (0..3)
                        .map(|c| (target[c] - s * candidate[c]).abs())
                        .fold(0.0, f64::max);
                    if error * (1.0 - a) <= params.tolerance {
                        least_shade = Some(least_shade.map_or(s, |old| old.max(s)));
                    }
                }
                if let Some(s) = least_shade {
                    let index = (y * w as i32 + x) as usize;
                    alpha[index] = a;
                    shadow[index] = a == 0.0 && s <= params.max_shade;
                    break 'alpha;
                }
            }
        }
    }

    let reach = (h as f64 / params.reach).round() as i32;
    let is_shadow = |x: i32, y: i32| {
        x >= 0 && y >= 0 && x < w as i32 && y < h as i32 && shadow[(y * w as i32 + x) as usize]
    };
    GrayImage::from_fn(w, h, |x, y| {
        let white = whiteness(rgb(number.get_pixel(x, y)), params);
        if white == 0.0 {
            return Luma([0]);
        }
        let (x, y) = (x as i32, y as i32);
        let within =
            |dx: i32, dy: i32| (1..=reach).any(|step| is_shadow(x + dx * step, y + dy * step));
        let in_shadow = [(1, 0), (0, 1), (1, 1), (1, -1)]
            .iter()
            .any(|(dx, dy)| within(*dx, *dy) && within(-dx, -dy));
        let unexplained = alpha[(y * w as i32 + x) as usize] >= params.min_alpha;
        Luma([if unexplained || in_shadow {
            white as u8
        } else {
            0
        }])
    })
}
