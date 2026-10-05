use crate::buffer::Buffer;
use crate::image_utils::ocr::{OCR_LINE_HEIGHT, recognize_line};
use crate::tooltip::common::text_lines;
use crate::tooltip::detect::TitleBar;
use image::{
    RgbaImage,
    imageops::{FilterType, resize},
};

// light text on a dark bar to dark text on white, which is what the recogniser reads best
pub fn text_image(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> RgbaImage {
    ink_image(buffer, x0, y0, x1, y1, |[r, g, b]| r.max(g).max(b))
}

// only what is in the amount yellow (255, 213, 0), so white text and the background drop out
pub fn yellow_image(buffer: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize) -> RgbaImage {
    ink_image(buffer, x0, y0, x1, y1, |[r, g, b]| (r.min(g) - b).max(0))
}

fn ink_image(
    buffer: &Buffer,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    ink: impl Fn([i32; 3]) -> i32,
) -> RgbaImage {
    let mut image = buffer.crop(x0, y0, x1, y1);
    let ink = |pixel: &image::Rgba<u8>| ink([pixel.0[0] as i32, pixel.0[1] as i32, pixel.0[2] as i32]);
    let most = image.pixels().map(ink).max().unwrap().max(1);
    for pixel in image.pixels_mut() {
        let value = 255 - (ink(pixel) * 255 / most) as u8;
        pixel.0 = [value, value, value, 255];
    }
    // the recogniser works on lines 64px tall
    let width = image.width() * OCR_LINE_HEIGHT / image.height();
    resize(&image, width, OCR_LINE_HEIGHT, FilterType::CatmullRom)
}

// The title, one OCR call per line, and whether every line is centred in the bar.
// A line whose side gaps differ has the cursor over it: "Great Destiny Leapstone" with its
// start covered reads as a different, valid item.
pub fn read_title(buffer: &Buffer, bar: &TitleBar, s: f64) -> (String, bool) {
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
            let end = coloured
                .windows(2)
                .find(|pair| pair[1] - pair[0] > word_gap);
            let name_end = end.map_or(coloured[coloured.len() - 1], |pair| pair[0]);
            name_is_over = name_end + word_gap < last;
            last = name_end;
        } else if name_is_coloured {
            break;
        }
        // the lines are too close together for any margin above or below
        let margin = (5.0 * s).round() as usize;
        words.push(recognize_line(&text_image(
            buffer,
            bar.x + first.saturating_sub(margin),
            bar.y + top.saturating_sub(1),
            bar.x + (last + margin + 1).min(bar.width),
            bar.y + (bottom + 1).min(bar.height),
        )));
        if name_is_over {
            break;
        }
    }
    let name = words.join(" ");
    // drop the "[X n]" stack count, whose bracket can come back as an unknown character, and the "(Bound)" marker
    let end = name
        .find("(Bound")
        .or(name.find('['))
        .or(name.find("?X"))
        .or(name.find("?x"));
    let name = &name[..end.unwrap_or(name.len())];
    (
        name.split_whitespace().collect::<Vec<_>>().join(" "),
        centred,
    )
}
