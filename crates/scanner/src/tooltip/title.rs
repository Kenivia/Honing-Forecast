use crate::buffer::Buffer;
use crate::ocr::{jobs::Line, text::text_line};
use crate::tooltip::common::text_lines;
use crate::tooltip::detect::TitleBar;

// The title's lines, and whether every line is centred in the bar.
// A line whose side gaps differ has the cursor over it: "Great Destiny Leapstone" with its
// start covered reads as a different, valid item.
pub fn title_lines(buffer: &Buffer, bar: &TitleBar, s: f64, margin: f64) -> (Vec<Line>, bool) {
    let is_text =
        |x: usize, y: usize| buffer.rgb(bar.x + x, bar.y + y).into_iter().max().unwrap() > 120;
    let is_coloured = |x: usize, y: usize| {
        let pixel = buffer.rgb(bar.x + x, bar.y + y);
        let (max, min) = (
            pixel.into_iter().max().unwrap(),
            pixel.into_iter().min().unwrap(),
        );
        max > 120 && (max - min) * 6 > max
    };
    let rows: Vec<bool> = (0..bar.height)
        .map(|y| (0..bar.width).any(|x| is_text(x, y)))
        .collect();
    let mut centred = true;
    let mut words = vec![];
    let (mut name_is_coloured, mut name_is_over) = (false, false);
    for (top, bottom) in text_lines(&rows, 8.0 * s) {
        let mut columns = (0..bar.width).filter(|x| (top..bottom).any(|y| is_text(*x, y)));
        let first = columns.next().unwrap();
        let last = columns.last().unwrap_or(first);
        centred &= (first as f64 - (bar.width - 1 - last) as f64).abs() <= 8.0 * s;
        // The name is in the rarity colour and the "[X n]" count after it is white. The recogniser trips
        // over the bracket ("Pouch III [X" comes back without the III), so a coloured name is read
        // alone and whatever follows it is skipped. A white name keeps its count, cut off further down.
        let coloured: Vec<usize> = (first..=last)
            .filter(|x| (top..bottom).filter(|y| is_coloured(*x, *y)).count() >= 2)
            .collect();
        let mut last = last;
        // a title starts with the name
        if coloured
            .first()
            .is_some_and(|x| (x - first) as f64 <= 4.0 * s)
        {
            name_is_coloured = true;
            let word_gap = (13.5 * s) as usize;
            // a hyphen is too thin to count as a coloured column, but it is not a gap either
            let mut name_end = coloured[0];
            for x in coloured[0]..=last {
                if x - name_end > word_gap {
                    break;
                }
                if (top..bottom).any(|y| is_coloured(x, y)) {
                    name_end = x;
                }
            }
            name_is_over = name_end + word_gap < last;
            last = name_end;
        } else if name_is_coloured {
            break;
        }
        // the lines are too close together for any margin above or below
        let margin = (margin * s).round() as usize;
        words.push(text_line(
            buffer,
            bar.x + first.saturating_sub(margin),
            bar.y + top.saturating_sub(1),
            bar.x + (last + margin + 1).min(bar.width),
            bar.y + (bottom + 1).min(bar.height),
        ));
        if name_is_over {
            break;
        }
    }
    (words, centred)
}

// the title from what its lines read as
pub fn join_title(words: &[String]) -> String {
    let name = words.join(" ");
    // drop the "[X n]" stack count, whose bracket can come back as an unknown character, and the "(Bound)" marker
    // a name can have brackets of its own ("[Event] Metallurgy: Hellfire [11-14]")
    let end = name
        .find("(Bound")
        .or(["[X", "[x", "?X", "?x"].into_iter().find_map(|count| name.rfind(count)));
    let name = &name[..end.unwrap_or(name.len())];
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}
