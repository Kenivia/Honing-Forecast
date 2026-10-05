use crate::buffer::Buffer;
use crate::scanner_state::Tradability;
use crate::tooltip::common::{CURSOR_REACH, distance, median, text_lines};
use crate::tooltip::detect::TitleBar;

const ICON_BOTTOM: f64 = 108.0; // the item icon ends this far below the title bar
const CHARACTER_BIND_WIDTH: f64 = 160.0; // "Bound to Character" ends 173px in, "Bound to Roster" 143px. English only

const TEXT: u8 = 1;
const YELLOW: u8 = 2;
const RED: u8 = 4;
const WHITE: u8 = 8;
const BRIGHT: u8 = 16;
const BLACK: u8 = 32;

fn classify([r, g, b]: [i32; 3]) -> u8 {
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let mut class = 0;
    if max > 110 {
        class |= TEXT;
    }
    if max > 150 {
        class |= BRIGHT;
    }
    if max <= 10 {
        class |= BLACK;
    }
    if r > 190 && b < 110 && g * 4 > r * 3 && g < r {
        class |= YELLOW;
    }
    if r > 130 && g * 5 < r * 3 && b * 5 < r * 3 {
        class |= RED;
    }
    if min > 150 && max - min < 30 {
        class |= WHITE;
    }
    class
}

struct Line {
    top: usize,
    bottom: usize,
    left: usize,
    right: usize,
    yellow: Option<(usize, usize)>,
    red: bool,
}

pub struct Layout {
    // None while the cursor hides the bind line
    pub tradability: Option<Tradability>,
    // frame rectangle (x0, y0, x1, y1) around the "Amount Stacked" number
    pub amount: Option<(usize, usize, usize, usize)>,
}

// The largest cursor reaches about 120px into the tooltip and merges with any line it touches,
// so lines are only looked at from the columns beyond it.
pub fn parse_layout(buffer: &Buffer, bar: &TitleBar, s: f64) -> Layout {
    let (x, width) = (bar.x, bar.width);
    let title_bottom = bar.y + bar.height;
    // the body is ~97% opaque, so its margins stay within a few levels of one colour
    let body_colour = median(
        (title_bottom + 3..title_bottom + 9)
            .map(|y| buffer.rgb(x + width - 3, y))
            .collect(),
    );
    let (mut body_bottom, mut misses) = (title_bottom, 0);
    for y in title_bottom..buffer.height {
        let off = |x0: usize| (x0..x0 + 3).any(|x| distance(buffer.rgb(x, y), body_colour) > 14);
        misses = if off(x + 1) && off(x + width - 4) {
            misses + 1
        } else {
            0
        };
        if misses == 3 {
            break;
        }
        if misses == 0 {
            body_bottom = y;
        }
    }

    let pad = (5.0 * s).round() as usize;
    let (left, columns, rows) = (x + pad, width - 2 * pad, body_bottom - title_bottom);
    let class: Vec<u8> = (0..rows * columns)
        .map(|i| classify(buffer.rgb(left + i % columns, title_bottom + i / columns)))
        .collect();
    let count = |y0: usize, y1: usize, x0: usize, x1: usize, flag: u8| {
        (y0..y1)
            .flat_map(|y| class[y * columns + x0..y * columns + x1].iter())
            .filter(|c| **c & flag != 0)
            .count()
    };

    // chest contents sit on a pure black panel, nothing in it is wanted
    let black: Vec<usize> = (0..rows)
        .filter(|y| count(*y, y + 1, 0, columns, BLACK) * 2 > columns)
        .collect();
    let panel = (black.len() as f64 >= 40.0 * s).then(|| (black[0], black[black.len() - 1]));

    let start = (CURSOR_REACH * s).round() as usize - pad;
    let has_text: Vec<bool> = (0..rows)
        .map(|y| count(y, y + 1, start, columns, TEXT) > 0)
        .collect();
    let lines: Vec<Line> = text_lines(&has_text, 8.0 * s)
        .into_iter()
        .filter(|(top, bottom)| {
            (bottom - top) as f64 <= 27.0 * s && !panel.is_some_and(|p| p.0 <= *top && *top <= p.1)
        })
        .map(|(top, bottom)| {
            let extent = |flag: u8| {
                let mut hits = (start..columns).filter(|x| count(top, bottom, *x, x + 1, flag) > 0);
                let first = hits.next()?;
                Some((first, hits.last().unwrap_or(first)))
            };
            let (left, right) = extent(TEXT).unwrap();
            Line {
                top,
                bottom,
                left,
                right,
                yellow: extent(YELLOW).filter(|_| count(top, bottom, start, columns, YELLOW) >= 4),
                red: count(top, bottom, start, columns, RED) * 2
                    > count(top, bottom, start, columns, BRIGHT),
            }
        })
        .collect();

    // "Amount Stacked: N" / "Total Amount Owned: N": white label, then yellow up to the end of the line
    let amounts: Vec<&Line> = lines
        .iter()
        .filter(|line| {
            line.yellow.is_some_and(|(first, last)| {
                let label = first.saturating_sub((21.0 * s).round() as usize);
                last + 2 >= line.right
                    && count(line.top, line.bottom, label, first, WHITE) as f64 >= 8.0 * s
            })
        })
        .collect();
    // stacked and total are one line apart, which rules out a description line that happens to qualify
    let first = amounts
        .windows(2)
        .position(|pair| {
            let gap = (pair[1].top - pair[0].top) as f64;
            22.5 * s <= gap && gap <= 31.0 * s
        })
        .unwrap_or(0);
    let stacked = amounts.get(first);

    let header = || {
        lines
            .iter()
            .filter(|line| stacked.is_none_or(|amount| line.top < amount.top))
    };
    let untradable = header().find(|line| line.red && line.left as f64 > 0.45 * width as f64);
    // "Bound to ..." is the line above "Untradable" that runs in from the left and stops short.
    // Only where it ends is used: "Character" ends further right than "Roster".
    let tradability = match untradable {
        None => Some(Tradability::Tradable),
        Some(untradable) => {
            let bind = header()
                .filter(|line| {
                    ICON_BOTTOM * s <= line.top as f64
                        && line.top < untradable.top
                        && line.left as f64 <= start as f64 + 4.0 * s
                        && (line.right as f64) < 0.6 * width as f64
                })
                .last();
            bind.map(|line| {
                if (line.right + pad) as f64 > CHARACTER_BIND_WIDTH * s {
                    Tradability::CharBound
                } else {
                    Tradability::RosterBound
                }
            })
        }
    };

    Layout {
        tradability,
        amount: stacked.map(|line| {
            let (first, last) = line.yellow.unwrap();
            (
                left + first - 3,
                title_bottom + line.top - 3,
                left + last + 4,
                title_bottom + line.bottom + 3,
            )
        }),
    }
}
