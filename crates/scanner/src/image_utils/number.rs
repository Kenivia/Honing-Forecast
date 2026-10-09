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
    pub min_chroma: f64, // a background has to be at least this coloured for its colour to tell
    pub low_alpha: f64,  // enough for a pixel that touches a digit pixel
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
            min_chroma: 40.0,
            low_alpha: 0.3,
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

// How much white a pixel needs over this background going by colour alone. Shading keeps a colour's
// hue and white washes it out, so a grey pixel over a coloured part of the icon is nearly all digit,
// however bright that part is. Says nothing over a background that is grey itself.
fn alpha_by_colour(observed: Rgb, background: Rgb, params: &NumberParams) -> f64 {
    let spread = |p: Rgb| (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2]));
    let ((low, high), (bg_low, bg_high)) = (spread(observed), spread(background));
    if bg_high - bg_low < params.min_chroma {
        return 0.0;
    }
    let kept = ((high - low) / (bg_high - bg_low)).min(1.0);
    (low - kept * bg_low) / 255.0
}

// per pixel: the least white the template leaves unexplained, the same going by colour, and
// whether it is the number's shadow
pub fn number_layers(
    number: &RgbaImage,
    background: &[Rgb],
    params: &NumberParams,
) -> (Vec<f64>, Vec<f64>, Vec<bool>) {
    let (w, h) = number.dimensions();
    let bg = |x: i32, y: i32| {
        background[(y.clamp(0, h as i32 - 1) * w as i32 + x.clamp(0, w as i32 - 1)) as usize]
    };
    let mut alpha = vec![1.0; (w * h) as usize];
    let mut by_colour = vec![0.0; (w * h) as usize];
    let mut shadow = vec![false; (w * h) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let observed = rgb(number.get_pixel(x as u32, y as u32));
            let index = (y * w as i32 + x) as usize;
            by_colour[index] = (0..9)
                .map(|n| alpha_by_colour(observed, bg(x + n % 3 - 1, y + n / 3 - 1), params))
                .fold(1.0, f64::min);
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
                    alpha[index] = a;
                    shadow[index] = a == 0.0 && s <= params.max_shade;
                    break 'alpha;
                }
            }
        }
    }
    (alpha, by_colour, shadow)
}

// The game draws the number over the icon as opaque white digits with a soft black shadow around them,
// and its brightness setting changes the icon but not the number. So a pixel is
//     white * alpha + background * shade * (1 - alpha)
// Per pixel this gives the least white (alpha) the template cannot account for, and how much the
// template had to be darkened (shade). Either tells a digit from a white part of the icon: the digit is
// white the template does not have, or white inside the shadow.
pub fn number_mask(number: &RgbaImage, background: &[Rgb], params: &NumberParams) -> GrayImage {
    let (w, h) = number.dimensions();
    let (alpha, by_colour, shadow) = number_layers(number, background, params);
    let reach = (h as f64 / params.reach).round() as i32;
    let is_shadow = |x: i32, y: i32| {
        x >= 0 && y >= 0 && x < w as i32 && y < h as i32 && shadow[(y * w as i32 + x) as usize]
    };
    let white: Vec<f64> = number.pixels().map(|p| whiteness(rgb(p), params)).collect();
    let at = |x: i32, y: i32| (y * w as i32 + x) as usize;
    let mut digit = vec![false; (w * h) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let within =
                |dx: i32, dy: i32| (1..=reach).any(|step| is_shadow(x + dx * step, y + dy * step));
            let in_shadow = [(1, 0), (0, 1), (1, 1), (1, -1)]
                .iter()
                .any(|(dx, dy)| within(*dx, *dy) && within(-dx, -dy));
            let index = at(x, y);
            let unexplained = alpha[index].max(by_colour[index]) >= params.min_alpha;
            digit[index] = white[index] > 0.0 && (unexplained || in_shadow);
        }
    }
    // a pixel short of the limit still counts where it touches a digit pixel
    let mut grew = true;
    while grew {
        grew = false;
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let index = at(x, y);
                if digit[index]
                    || white[index] == 0.0
                    || alpha[index].max(by_colour[index]) < params.low_alpha
                {
                    continue;
                }
                let touches = (0..9).any(|n| {
                    let (nx, ny) = (x + n % 3 - 1, y + n / 3 - 1);
                    nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 && digit[at(nx, ny)]
                });
                if touches {
                    digit[index] = true;
                    grew = true;
                }
            }
        }
    }
    GrayImage::from_fn(w, h, |x, y| {
        let index = (y * w + x) as usize;
        Luma([if digit[index] { white[index] as u8 } else { 0 }])
    })
}
