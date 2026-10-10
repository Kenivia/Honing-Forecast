use crate::buffer::Buffer;
use crate::image_utils::ocr::OCR_LINE_HEIGHT;
use crate::tooltip::common::text_lines;
use crate::tooltip::detect::TitleBar;
use image::{
    GrayImage, Luma,
    imageops::{FilterType, resize},
};

// A line of text cut out of a frame, as how much each pixel is text. It is cut on every frame of
// a hover and only made into a strip for the few that are read.
#[derive(Debug, Clone)]
pub struct Ink(GrayImage);

impl Ink {
    // light text on a dark bar to dark text on white, which is what the recogniser reads best
    pub fn strip(&self) -> GrayImage {
        let most = (*self.0.as_raw().iter().max().unwrap()).max(1) as u32;
        let image = GrayImage::from_fn(self.0.width(), self.0.height(), |x, y| {
            Luma([255 - (self.0.get_pixel(x, y)[0] as u32 * 255 / most) as u8])
        });
        // the recogniser works on lines 64px tall
        let width = image.width() * OCR_LINE_HEIGHT / image.height();
        resize(&image, width, OCR_LINE_HEIGHT, FilterType::CatmullRom)
    }
}

// whatever is bright
pub fn text_image(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> Ink {
    ink_image(buffer, x0, y0, x1, y1, |[r, g, b]| r.max(g).max(b))
}

// only what is in the amount yellow (255, 213, 0), so white text and the background drop out
pub fn yellow_image(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> Ink {
    ink_image(buffer, x0, y0, x1, y1, |[r, g, b]| (r.min(g) - b).max(0))
}

fn ink_image(
    buffer: &Buffer,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    ink: impl Fn([i32; 3]) -> i32,
) -> Ink {
    Ink(GrayImage::from_fn((x1 - x0) as u32, (y1 - y0) as u32, |x, y| {
        Luma([ink(buffer.rgb(x0 + x as usize, y0 + y as usize)) as u8])
    }))
}

// The title's lines, and whether every line is centred in the bar.
// A line whose side gaps differ has the cursor over it: "Great Destiny Leapstone" with its
// start covered reads as a different, valid item.
pub fn title_lines(buffer: &Buffer, bar: &TitleBar, s: f64) -> (Vec<Ink>, bool) {
    let is_text =
        |x: usize, y: usize| buffer.rgb(bar.x + x, bar.y + y).into_iter().max().unwrap() > 120;
    let is_coloured = |x: usize, y: usize| {
        let pixel = buffer.rgb(bar.x + x, bar.y + y);
        let (max, min) = (
            pixel.into_iter().max().unwrap(),
            pixel.into_iter().min().unwrap(),
        );
        max > 120 && (max - min) * 4 > max
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
        let margin = (5.0 * s).round() as usize;
        words.push(text_image(
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
