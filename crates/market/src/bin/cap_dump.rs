// Dumps every Nth frame of a .hfcap as a downscaled PNG: cap_dump <file> <out dir> <every> <scale> [from to]
use hf_scanner::dev_utils::capture::{Record, read_records};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (every, scale): (usize, u32) = (a[3].parse().unwrap(), a[4].parse().unwrap());
    let from: usize = a.get(5).map_or(0, |x| x.parse().unwrap());
    let to: usize = a.get(6).map_or(usize::MAX, |x| x.parse().unwrap());
    let file = std::fs::read(&a[1]).unwrap();
    std::fs::create_dir_all(&a[2]).unwrap();
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
                    img.save(format!("{}/f{n:05}.png", a[2])).unwrap();
                }
                n += 1;
            }
            Record::End(_) => {}
        }
    }
    println!("{n} frames, {w}x{h}");
}
