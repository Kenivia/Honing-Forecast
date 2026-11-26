// Dumps every Nth frame of a .hfcap as a downscaled PNG: cap_dump <file> <out dir> <every> <scale> [from to]
use clap::Parser;
use hf_scanner::dev_utils::capture::{Record, read_records};

#[derive(Parser)]
struct Cli {
    capture: String,
    out: String,
    every: usize,
    scale: u32,
    #[arg(default_value_t = 0)]
    from: usize,
    #[arg(default_value_t = usize::MAX)]
    to: usize,
}

fn main() {
    let Cli {
        capture,
        out,
        every,
        scale,
        from,
        to,
    } = Cli::parse();
    let file = std::fs::read(&capture).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    let (mut w, mut h, mut data, mut n) = (0, 0, vec![], 0);
    for r in read_records(&file) {
        match r {
            Record::Start(s) => {
                (w, h) = (s.width as u32, s.height as u32);
                data = vec![0u8; (w * h * 4) as usize];
            }
            Record::Input(i) => {
                i.apply(&mut data);
                if n % every == 0 && n >= from && n <= to {
                    let img = image::RgbaImage::from_raw(w, h, data.clone()).unwrap();
                    let img = image::imageops::resize(
                        &img,
                        w / scale,
                        h / scale,
                        image::imageops::FilterType::Triangle,
                    );
                    img.save(format!("{out}/f{n:05}.png")).unwrap();
                }
                n += 1;
            }
            Record::End(_) => {}
        }
    }
    println!("{n} frames, {w}x{h}");
}
