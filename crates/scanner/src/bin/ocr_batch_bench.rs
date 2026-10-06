// Does ocrs get faster per line when several lines go in one call? Reads strips dumped by
// ocr_widths and times one call per line against one call for the whole batch.
//   RAYON_NUM_THREADS=1 cargo run --release --bin ocr_batch_bench -- <strip dir>
use hf_scanner::{image_utils::ocr::recognize_line, setup::load_ocr_engine};
use image::RgbaImage;
use ocrs::ImageSource;
use rten_imageproc::{PointF, RotatedRect, Vec2};
use std::{collections::BTreeMap, env, fs, time::Instant};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const BATCHES: [usize; 6] = [1, 2, 4, 8, 12, 20];

// What a batched recognize would do: stack the strips into one image, one rect each, one call.
fn recognize_batch(strips: &[RgbaImage]) -> Vec<String> {
    let height = strips[0].height();
    let width = strips.iter().map(|x| x.width()).max().unwrap();
    let mut canvas = RgbaImage::new(width, height * strips.len() as u32);
    for (index, strip) in strips.iter().enumerate() {
        image::imageops::replace(&mut canvas, strip, 0, (index as u32 * height) as i64);
    }
    let lines: Vec<Vec<RotatedRect>> = strips
        .iter()
        .enumerate()
        .map(|(index, strip)| {
            let top = index as u32 * height;
            vec![RotatedRect::new(
                PointF::from_yx(top as f32 + height as f32 / 2.0, strip.width() as f32 / 2.0),
                Vec2::from_yx(1., 0.),
                strip.width() as f32,
                height as f32,
            )]
        })
        .collect();
    let read = hf_scanner::setup::OCR_ENGINE.read();
    let engine = read.as_ref().unwrap();
    let (w, h) = canvas.dimensions();
    let input = engine
        .prepare_input(ImageSource::from_bytes(&canvas, (w, h)).unwrap())
        .unwrap();
    engine
        .recognize_text(&input, &lines)
        .unwrap()
        .into_iter()
        .map(|line| line.map_or(String::new(), |x| x.to_string()))
        .collect()
}

fn main() {
    let dir = env::args().nth(1).unwrap();
    load_ocr_engine(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());

    // strips by the width bucket ocrs would pad them to
    let mut buckets: BTreeMap<u32, Vec<RgbaImage>> = BTreeMap::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let image = image::open(&path).unwrap().to_rgba8();
        buckets
            .entry(image.width().next_multiple_of(50))
            .or_default()
            .push(image);
    }
    println!("RAYON_NUM_THREADS={:?}", env::var("RAYON_NUM_THREADS").ok());
    println!("buckets: {:?}", buckets.iter().map(|(w, v)| (*w, v.len())).collect::<Vec<_>>());

    for (bucket, strips) in buckets.iter().filter(|(_, v)| v.len() >= 20) {
        println!("\n=== {bucket} px bucket, {} strips", strips.len());
        println!("  batch   one by one      batched     per line   speedup  same text");
        for size in BATCHES {
            let groups: Vec<&[RgbaImage]> = strips.chunks(size).filter(|c| c.len() == size).collect();
            if groups.is_empty() {
                continue;
            }
            // warm up, the first call of a size allocates its work space
            recognize_batch(groups[0]);
            let clock = Instant::now();
            let one: Vec<Vec<String>> = groups
                .iter()
                .map(|g| g.iter().map(|s| recognize_line(s)).collect())
                .collect();
            let single = clock.elapsed().as_secs_f64() * 1000.0;
            let clock = Instant::now();
            let many: Vec<Vec<String>> = groups.iter().map(|g| recognize_batch(g)).collect();
            let batched = clock.elapsed().as_secs_f64() * 1000.0;
            let lines = (groups.len() * size) as f64;
            println!(
                "  {size:5}  {single:9.0} ms  {batched:9.0} ms  {:7.1} ms  {:6.2}x  {}",
                batched / lines,
                single / batched,
                if one == many { "yes" } else { "NO" }
            );
        }
    }
}
