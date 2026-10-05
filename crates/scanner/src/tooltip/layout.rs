use crate::buffer::Buffer;
use crate::scanner_state::Tradability;
use crate::tooltip::chest::{ChestKind, ChestLayout, ChestRowLayout};
use crate::tooltip::common::{CURSOR_REACH, distance, median, text_lines};
use crate::tooltip::detect::TitleBar;

const ICON_BOTTOM: f64 = 108.0; // the item icon ends this far below the title bar
const CHARACTER_BIND_WIDTH: f64 = 160.0; // "Bound to Character" ends 173px in, "Bound to Roster" 143px. English only
const PANEL_TEXT_LEFT: f64 = 69.0; // chest rows: the icon ends 63px in, names and counts start at 76
const PANEL_ICON_LEFT: f64 = 20.0;
const PANEL_ICON_SIZE: f64 = 43.0;
const PANEL_ROW_GAP: f64 = 13.3; // lines of one chest row are at most 9px apart, rows at least 17
const PANEL_END_GAP: f64 = 30.0; // rows are 17 to 19px apart, the panel ends 50px before anything else
const SHORT_PHRASE: f64 = 53.0; // "all" is 13px of yellow, the other two phrases 157 and more

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
    // frame rectangle (x0, y0, x1, y1) of the "Amount Stacked" number, no margin
    pub amount: Option<(usize, usize, usize, usize)>,
    pub chest: Option<ChestLayout>,
    // last row the shortcut hints under the body were seen on
    pub bottom: usize,
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

    let extent = |top: usize, bottom: usize, from: usize, flag: u8| {
        let mut hits = (from..columns).filter(|x| count(top, bottom, *x, x + 1, flag) > 0);
        let first = hits.next()?;
        Some((first, hits.last().unwrap_or(first)))
    };

    // chest contents sit on a pure black panel, read on its own further down
    let black: Vec<usize> = (0..rows)
        .filter(|y| count(*y, y + 1, 0, columns, BLACK) * 2 > columns)
        .collect();
    let panel = (black.len() as f64 >= 40.0 * s).then(|| (black[0], black[black.len() - 1]));

    let start = (CURSOR_REACH * s).round() as usize - pad;
    let has_text: Vec<bool> = (0..rows)
        .map(|y| count(y, y + 1, start, columns, TEXT) > 0)
        .collect();
    let (lines, merged): (Vec<_>, Vec<_>) = text_lines(&has_text, 8.0 * s)
        .into_iter()
        .filter(|(top, _)| !panel.is_some_and(|p| p.0 <= *top && *top <= p.1))
        .partition(|(top, bottom)| (bottom - top) as f64 <= 27.0 * s);
    let lines: Vec<Line> = lines
        .into_iter()
        .map(|(top, bottom)| {
            let extent = |flag: u8| extent(top, bottom, start, flag);
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

    // "Amount Stacked: N" / "Total Amount Owned: N": white label, then yellow up to the end of the line.
    // Compression can take the yellow out of a thin last digit, so the line may run on a little, but not in white.
    let amounts: Vec<&Line> = lines
        .iter()
        .filter(|line| {
            line.yellow.is_some_and(|(first, last)| {
                let label = first.saturating_sub((21.0 * s).round() as usize);
                last as f64 + 16.0 * s >= line.right as f64
                    && count(line.top, line.bottom, last + 1, line.right + 1, WHITE) == 0
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
    // a cursor beyond its usual reach joins the lines it is over into one block, "Untradable" among them
    let covered = merged
        .iter()
        .any(|(top, _)| stacked.is_none_or(|amount| *top < amount.top));
    let tradability = match untradable {
        None => (!covered).then_some(Tradability::Tradable),
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

    // Header, then one row per item: icon, name over one or two lines, "xN" under it.
    // Names start right of the icon, so the lines are taken from there.
    let chest = panel.and_then(|(panel_top, panel_bottom)| {
        let text_left = (PANEL_TEXT_LEFT * s).round() as usize - pad;
        let in_panel: Vec<bool> = (0..rows)
            .map(|y| {
                panel_top <= y && y <= panel_bottom && count(y, y + 1, text_left, columns, TEXT) > 0
            })
            .collect();
        let lines = text_lines(&in_panel, 8.0 * s);
        // The yellow phrase tells the three kinds apart: "Obtain [all] of the following items." is one
        // short word, "There is a [chance that you can obtain one]" runs to the end of the line,
        // "You can [select and obtain 1] of the" has white after it. Only "all" fits on one line.
        let (top, bottom) = *lines.first()?;
        let (first, last) = extent(top, bottom, text_left, YELLOW)?;
        let kind = if ((last - first) as f64) < SHORT_PHRASE * s {
            ChestKind::ObtainAll
        } else if last as f64 + 8.0 * s >= extent(top, bottom, text_left, TEXT)?.1 as f64 {
            ChestKind::Random
        } else {
            ChestKind::SelectOne
        };
        let header = if kind == ChestKind::ObtainAll { 1 } else { 2 };

        let to_frame = |(top, bottom): (usize, usize)| {
            let (first, last) = extent(top, bottom, text_left, TEXT).unwrap();
            // The recogniser wants a little room around the text and no more: a wider margin lets
            // the icon's edge in on the left, and it then reads next to nothing.
            let (across, down) = ((5.33 * s).round() as usize, (2.67 * s).round() as usize);
            (
                left + (first - across).max(text_left),
                title_bottom + top - down,
                left + (last + across + 1).min(columns),
                title_bottom + bottom + down,
            )
        };
        let mut chest_rows: Vec<Vec<(usize, usize)>> = vec![];
        for (index, line) in lines.iter().enumerate().skip(header) {
            let gap = (line.0 - lines[index - 1].1) as f64;
            // over a dark background the body seems to run on into the shortcut hints, which are black too
            if gap > PANEL_END_GAP * s {
                break;
            }
            if index > header && gap < PANEL_ROW_GAP * s {
                chest_rows.last_mut().unwrap().push(*line);
            } else {
                chest_rows.push(vec![*line]);
            }
        }
        Some(ChestLayout {
            kind,
            rows: chest_rows
                .iter()
                .filter(|lines| lines.len() >= 2)
                .map(|lines| {
                    let (names, count) = lines.split_at(lines.len() - 1);
                    let top = title_bottom + names[0].0 - (6.0 * s).round() as usize;
                    ChestRowLayout {
                        names: names.iter().map(|line| to_frame(*line)).collect(),
                        count: to_frame(count[0]),
                        whole: (
                            x + (PANEL_ICON_LEFT * s) as usize,
                            top,
                            x + width - pad,
                            (title_bottom + count[0].1 + 2).max(top + (PANEL_ICON_SIZE * s) as usize),
                        ),
                    }
                })
                .collect(),
        })
    });

    // The shortcut hints are a narrower box under the body, 80% black. Its sides are compared with
    // the bare background just outside it, so it is only followed while that background is bright
    // enough to tell: over a dark one the box seems to end early.
    let (mut bottom, mut misses) = (body_bottom, 0);
    let (outside, inside) = ((2.0 * s) as usize, (20.0 * s).round() as usize);
    for y in body_bottom + 1..buffer.height {
        let level = |x: usize| buffer.rgb(x, y).into_iter().max().unwrap();
        let sides = [(x + outside, x + inside), (x + width - outside, x + width - inside)];
        if sides.iter().any(|(a, _)| level(*a) < 50) {
            continue;
        }
        misses = if sides.iter().all(|(a, b)| level(*b) * 20 <= level(*a) * 7 + 200) {
            bottom = y;
            0
        } else {
            misses + 1
        };
        if misses == 4 {
            break;
        }
    }

    Layout {
        chest,
        bottom,
        tradability,
        amount: stacked.map(|line| {
            (
                left + line.yellow.unwrap().0,
                title_bottom + line.top,
                left + line.right + 1,
                title_bottom + line.bottom,
            )
        }),
    }
}
