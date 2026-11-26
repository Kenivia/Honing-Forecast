// Text helpers around the recognisers: parsing numbers, comparing names, reading a box (pp.rs).
use image::RgbaImage;

pub fn parse_number(t: &str, decimal: bool) -> Option<Option<f64>> {
    let t = t.trim();
    if t == "-" {
        return Some(None);
    }
    let sep = |c: char| c == ',' || c == '.';
    if t.is_empty()
        || !t.chars().all(|c| c.is_ascii_digit() || sep(c))
        || !t.starts_with(|c: char| c.is_ascii_digit())
    {
        return None;
    }
    let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    let v: f64 = digits.parse().ok()?;
    let b = t.as_bytes();
    if decimal {
        // the point is always followed by exactly one digit
        if b.len() < 3 || !sep(b[b.len() - 2] as char) {
            return None;
        }
        return Some(Some(v / 10.0));
    }
    // separators group thousands
    let groups: Vec<&str> = t.split(sep).collect();
    if groups[1..].iter().any(|g| g.len() != 3) || groups[0].len() > 3 && groups.len() > 1 {
        return None;
    }
    Some(Some(v))
}

fn ink(img: &RgbaImage, x: u32, y: u32) -> bool {
    let p = img.get_pixel(x, y);
    p[0].max(p[1]).max(p[2]) > 140
}

/// Rows of ink inside a rectangle, as (top, bottom) inclusive; gaps of a row or two split lines.
pub fn lines(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> Vec<(u32, u32)> {
    let mut out = vec![];
    let mut start = None;
    for y in y0..=y1 {
        let on = y < y1 && (x0..x1).any(|x| ink(img, x, y));
        match (on, start) {
            (true, None) => start = Some(y),
            (false, Some(s)) => {
                if y - s >= 4 {
                    out.push((s, y - 1));
                }
                start = None;
            }
            _ => {}
        }
    }
    out
}

/// The text inside a rectangle, by PP-OCRv5 (pp.rs): it reads the window names and the time left that ocrs
/// got wrong (2026-11-20). It writes no spaces.
pub fn read_box(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> String {
    let t = crate::pp::read(img, x0, x1, y0, y1);
    crate::strips::save(img, x0, x1, y0, y1, "ocr", &t);
    t
}

/// The digits of a text, which are compared apart from the rest: one wrong numeral barely changes how alike two names are.
pub fn numerals(t: &str) -> String {
    t.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Whether a read is this name: the same numerals and the rest at least 0.9 alike.
pub fn is_name(read: &str, name: &str) -> bool {
    let letters = |s: &str| {
        s.chars()
            .filter(|c| !c.is_ascii_digit())
            .collect::<String>()
    };
    numerals(read) == numerals(name) && similar(&letters(read), &letters(name)) >= 0.9
}

/// A number inside a rectangle, see `parse_number`.
pub fn number_box(
    img: &RgbaImage,
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
    decimal: bool,
) -> Result<Option<f64>, String> {
    let t = read_box(img, x0, x1, y0, y1);
    parse_number(&t, decimal).ok_or(t)
}

/// How alike two texts are, 0 to 1, ignoring case and spaces.
pub fn similar(a: &str, b: &str) -> f64 {
    let n = |s: &str| {
        s.to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    strsim::normalized_levenshtein(&n(a), &n(b))
}

/// The closest of `names` to `text`, if close enough.
pub fn best_match<'a>(text: &str, names: &[&'a str], min: f64) -> Option<&'a str> {
    let (best, score) = names
        .iter()
        .map(|n| (*n, similar(text, n)))
        .max_by(|a, b| a.1.total_cmp(&b.1))?;
    (score >= min).then_some(best)
}
