// pixel constants are 1440p like the rest of the scanner; most were measured at 1080p
pub const TITLE_WIDTH: f64 = 409.33; // 307 at 1080p
pub const CURSOR_REACH: f64 = 133.0; // how far the largest cursor gets into the tooltip from its left edge

// row ranges that have anything set, at least min_height tall
pub fn text_lines(rows: &[bool], min_height: f64) -> Vec<(usize, usize)> {
    let mut lines = vec![];
    let mut start = None;
    for y in 0..=rows.len() {
        let on = y < rows.len() && rows[y];
        if on && start.is_none() {
            start = Some(y);
        }
        if !on && let Some(top) = start.take() {
            if (y - top) as f64 >= min_height {
                lines.push((top, y));
            }
        }
    }
    lines
}

pub fn median(mut pixels: Vec<[i32; 3]>) -> [i32; 3] {
    std::array::from_fn(|c| {
        pixels.sort_by_key(|p| p[c]);
        pixels[pixels.len() / 2][c]
    })
}

pub fn distance(a: [i32; 3], b: [i32; 3]) -> i32 {
    (0..3).map(|c| (a[c] - b[c]).abs()).max().unwrap()
}
