use crate::buffer::Buffer;
use crate::tooltip::common::{TITLE_WIDTH, distance, median};
use ahash::AHashMap;

const TITLE_COLOUR: [i32; 3] = [26, 30, 34]; // normalised; (30, 34, 39) on screen at brightness 58
const TOLERANCE: i32 = 8;
const TITLE_MIN_ROWS: f64 = 5.0; // the clean strip above the title text is 15 rows
const TITLE_HEIGHTS: (f64, f64) = (33.0, 107.0); // 45 with a one-line title, 69 with two
const BODY_TO_TITLE: [f64; 3] = [0.44, 0.53, 0.49]; // body colour (11, 16, 17) over the title colour
const BODY_ROWS: f64 = 133.0;
const SLACK: usize = 3;

#[derive(Debug, Clone, Copy, Default)]
pub struct TitleBar {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

// The title bar is opaque and one colour, so the strip above its text is a flat run that ends at
// the tooltip's right edge. Only half of it has to be visible, the cursor can cover the left.
pub fn find_title(buffer: &Buffer, s: f64) -> Option<TitleBar> {
    let width = (TITLE_WIDTH * s).round() as usize;
    let window = width / 2;
    let (data, lut) = (buffer.data(), buffer.lut.as_ref().unwrap());
    // raw channel values that land on the title colour once normalised
    let near: [[bool; 256]; 3] = std::array::from_fn(|c| {
        std::array::from_fn(|v| (lut[v] as i32 - TITLE_COLOUR[c]).abs() <= TOLERANCE)
    });

    let mut ends: AHashMap<usize, Vec<usize>> = AHashMap::new();
    for y in 0..buffer.height {
        let row = &data[y * buffer.width * 4..(y + 1) * buffer.width * 4];
        let mut run = 0;
        for x in 0..=buffer.width {
            if x < buffer.width
                && near[0][row[x * 4] as usize]
                && near[1][row[x * 4 + 1] as usize]
                && near[2][row[x * 4 + 2] as usize]
            {
                run += 1;
                continue;
            }
            if run > window && run <= width + SLACK + 1 {
                ends.entry(x - 1).or_default().push(y);
            }
            run = 0;
        }
    }

    let mut candidates: Vec<(usize, usize)> =
        ends.iter().map(|(r, rows)| (rows.len(), *r)).collect();
    candidates.sort_by(|a, b| b.cmp(a));
    let need = (TITLE_MIN_ROWS * s).round() as usize;
    for (_, right) in candidates {
        // compression smears the run's end over a few columns
        let mut rows: Vec<usize> = (right.saturating_sub(SLACK)..=right + SLACK)
            .filter_map(|r| ends.get(&r))
            .flatten()
            .copied()
            .collect();
        rows.sort();
        rows.dedup();
        let Some(top) = rows
            .windows(need)
            .find(|w| w[need - 1] - w[0] == need - 1)
            .map(|w| w[0])
        else {
            continue;
        };
        if right + 1 < width {
            continue;
        }
        let x = right + 1 - width;

        let (mut strip, mut raw_strip) = (vec![], vec![]);
        for y in top..top + need {
            for x in right - window..right - 4 {
                strip.push(buffer.rgb(x, y));
                raw_strip.push(buffer.raw(x, y));
            }
        }
        let (title_colour, raw_title) = (median(strip), median(raw_strip));
        // Blue-grey; rules out the neutral greys of the other windows, which come out at -1 to 1.
        // A tooltip is about 8, but one that was only up for a few frames of a compressed recording
        // can be as low as 3.
        if title_colour[2] - title_colour[0] < 2 {
            continue;
        }

        // Below the title the right margin changes to the body colour and stays there. This is
        // looked for in the captured colours: at a low brightness setting the two are a couple of
        // levels apart, which normalising blows up along with the noise.
        let column = |y: usize| buffer.rgb(right - 2, y);
        let step = (raw_title.into_iter().max().unwrap() / 4).max(2);
        let changed = |y: usize| distance(buffer.raw(right - 2, y), raw_title) >= step;
        let Some(height) =
            (top..buffer.height - 3).position(|y| (y..y + 3).all(changed))
        else {
            continue;
        };
        if (height as f64) < TITLE_HEIGHTS.0 * s || height as f64 > TITLE_HEIGHTS.1 * s {
            continue;
        }
        let body_top = top + height + 3;
        let body_rows = (BODY_ROWS * s).round() as usize;
        if body_top + body_rows > buffer.height {
            continue;
        }
        let body: Vec<[i32; 3]> = (body_top..body_top + body_rows).map(column).collect();
        let body_colour = median(body.clone());
        let raw_body = median((body_top..body_top + body_rows).map(|y| buffer.raw(right - 2, y)).collect());
        let steady = body
            .iter()
            .filter(|p| distance(**p, body_colour) <= 14)
            .count() as f64
            > 0.7 * body_rows as f64;
        // ~97% opaque over anything from black to white, so what is captured is up to ~9 levels
        // above its own colour, whatever the brightness setting
        let lifted = (0..3).all(|c| {
            (0..=9).any(|lift| {
                let own = lut[(raw_body[c] - lift).max(0) as usize] as f64;
                (own - title_colour[c] as f64 * BODY_TO_TITLE[c]).abs() <= 2.0 + TOLERANCE as f64
            })
        });
        if steady && lifted {
            return Some(TitleBar {
                x,
                y: top,
                width,
                height,
            });
        }
    }
    None
}
