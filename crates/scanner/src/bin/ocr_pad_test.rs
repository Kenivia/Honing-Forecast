// Does padding a strip out to a longer line change what the recogniser reads? Batching lines of
// different widths means padding them all to the longest, so this checks what that costs in
// accuracy. Each strip is read as it is, then again padded to several widths, with the padding
// either black (what ocrs fills a batch with) or the strip's own background.
//   RAYON_NUM_THREADS=1 cargo run --release --bin ocr_pad_test -- <strip dir>
use hf_scanner::{image_utils::ocr::recognize_line, setup::load_ocr_engine};
use image::{Rgba, RgbaImage};
use std::{collections::BTreeMap, env, fs};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
// how much longer than the strip itself the padded line is
const RATIOS: [f64; 5] = [1.1, 1.25, 1.5, 2.0, 4.0];
const CAP: u32 = 1400;

// the value most of the strip is, which is its background
fn background(image: &RgbaImage) -> Rgba<u8> {
    let mut counts = [0u32; 256];
    for pixel in image.pixels() {
        counts[pixel.0[0] as usize] += 1;
    }
    let value = counts.iter().enumerate().max_by_key(|x| x.1).unwrap().0 as u8;
    Rgba([value, value, value, 255])
}

fn pad(image: &RgbaImage, width: u32, fill: Rgba<u8>) -> RgbaImage {
    let mut out = RgbaImage::from_pixel(width, image.height(), fill);
    image::imageops::replace(&mut out, image, 0, 0);
    out
}

fn main() {
    let dir = env::args().nth(1).unwrap();
    load_ocr_engine(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());

    let mut strips: Vec<(String, RgbaImage)> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let kind = path.file_name().unwrap().to_str().unwrap().split('_').next().unwrap().to_string();
            (kind, image::open(&path).unwrap().to_rgba8())
        })
        .collect();
    strips.sort_by_key(|(_, image)| image.width());

    // how the pipeline uses a read: digits for a number, words for a title
    let as_used = |kind: &str, text: &str| -> String {
        if kind == "title" || kind == "chest" {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        } else {
            text.chars().filter(char::is_ascii_digit).collect()
        }
    };
    // kind -> (fill, padding ratio in tenths) -> (differs as used, differs at all, total)
    let mut tally: BTreeMap<(String, &str, u32), (u32, u32, u32)> = BTreeMap::new();
    let mut examples: Vec<String> = vec![];
    for (kind, image) in &strips {
        let base = recognize_line(image);
        let base_used = as_used(kind, &base);
        for ratio in RATIOS {
            let width = ((image.width() as f64 * ratio) as u32).next_multiple_of(50);
            if width <= image.width() || width > CAP {
                continue;
            }
            let width_key = (ratio * 10.0) as u32;
            for (name, fill) in [("black", Rgba([0, 0, 0, 255])), ("background", background(image))] {
                let read = recognize_line(&pad(image, width, fill));
                let entry = tally.entry((kind.clone(), name, width_key)).or_default();
                entry.2 += 1;
                if read != base {
                    entry.1 += 1;
                }
                if as_used(kind, &read) != base_used {
                    entry.0 += 1;
                    if examples.len() < 30 {
                        examples.push(format!(
                            "  {kind} {}px -> {width}px {name}: {base:?} became {read:?}",
                            image.width()
                        ));
                    }
                }
            }
        }
    }

    println!("\nkind          fill        padded to   strips   differs as used   differs at all");
    for ((kind, fill, key), (used, any, total)) in &tally {
        println!(
            "  {kind:12} {fill:11} {:4.2}x  {total:6}   {used:4} ({:3.0}%)      {any:4} ({:3.0}%)",
            *key as f64 / 10.0,
            100.0 * *used as f64 / *total as f64,
            100.0 * *any as f64 / *total as f64
        );
    }
    println!("\nexamples of a read changing:");
    for line in &examples {
        println!("{line}");
    }
}
