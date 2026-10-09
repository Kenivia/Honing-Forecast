// Every stage of one slot's number clean-up as images, each time the scanner reads that slot.
//   cargo run --release --bin number_probe -- <out dir> <Roster|CharStorage|CharInventory> <page> <row> <col> <image>
//   python scripts/tooltips/dump_frames.py <mp4> | cargo run --release --bin number_probe -- <out dir> <inv> <page> <row> <col> --stdin <width> <height>
//   python scripts/tooltips/number_stages.py <out dir>     (one picture per read)
// Page, row and column count from 0. Files are f<frame>_<stage>.png.
use hf_scanner::{
    image_utils::{
        number::{NumberParams, number_background, number_layers, number_mask},
        ocr::{number_strip, recognize_line},
    },
    native,
    scanner_state::{InventoryType, ScannerState, SlotAddress},
    setup::BASE_ICONS,
};
use image::{GrayImage, Luma, Rgba, RgbaImage};
use std::{env, fs, io::Read};


fn grey(w: u32, h: u32, value: impl Fn(usize) -> f64) -> GrayImage {
    GrayImage::from_fn(w, h, |x, y| Luma([(value((y * w + x) as usize) * 255.0).clamp(0.0, 255.0) as u8]))
}

fn dump(state: &ScannerState, address: &SlotAddress, out: &str, frame: usize) {
    let info = &state.slot_infos[address];
    let name = &info.icon_name_score.as_ref().unwrap().0;
    let brightness = state.screen_info.brightness.unwrap();
    let (number, icon) = (&info.observed_number.data, &info.observed_icon.data);
    let (w, h) = number.dimensions();
    let template = BASE_ICONS.read()[name].data.clone();
    let params = NumberParams::default();
    let background = number_background(&template, icon, w, h, brightness);
    let (alpha, by_colour, shadow) = number_layers(number, &background, &params);
    let strip = number_strip(number, icon, &template, brightness, &params);
    let path = |stage: &str| format!("{out}/f{frame:04}_{stage}.png");
    icon.save(path("0_icon")).unwrap();
    number.save(path("1_captured")).unwrap();
    RgbaImage::from_fn(w, h, |x, y| {
        let c = background[(y * w + x) as usize];
        Rgba([c[0] as u8, c[1] as u8, c[2] as u8, 255])
    })
    .save(path("2_background"))
    .unwrap();
    grey(w, h, |i| alpha[i]).save(path("3_alpha_fit")).unwrap();
    grey(w, h, |i| by_colour[i]).save(path("4_alpha_colour")).unwrap();
    grey(w, h, |i| shadow[i] as u8 as f64).save(path("5_shadow")).unwrap();
    number_mask(number, &background, &params).save(path("6_mask")).unwrap();
    strip.save(path("7_sent")).unwrap();
    println!("f{frame:04}\t{name}\tbrightness {brightness:.2}\treads {:?}", recognize_line(&strip, true));
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let out = &args[0];
    fs::create_dir_all(out).unwrap();
    let address = SlotAddress {
        inventory_type: match args[1].as_str() {
            "Roster" => InventoryType::Roster,
            "CharStorage" => InventoryType::CharStorage,
            _ => InventoryType::CharInventory,
        },
        page_num: args[2].parse().unwrap(),
        pos_in_inv: (args[3].parse().unwrap(), args[4].parse().unwrap()),
    };
    let still = (args[5] != "--stdin").then(|| image::open(&args[5]).unwrap().to_rgba8());
    let (width, height): (usize, usize) = match &still {
        Some(image) => (image.width() as usize, image.height() as usize),
        None => (args[6].parse().unwrap(), args[7].parse().unwrap()),
    };
    let (mut state, mut pixels) = native::new_state(width, height, native::guess_game(width, height), false);

    let mut stdin = std::io::stdin().lock();
    for frame in 0.. {
        match &still {
            Some(image) if frame < 30 => pixels.copy_from_slice(image.as_raw()),
            Some(_) => break,
            None => {
                if stdin.read_exact(&mut pixels).is_err() {
                    break;
                }
            }
        }
        state.cropper();
        // a job still waiting was asked for on this frame
        if state.slot_infos.get(&address).is_some_and(|x| x.amount_job.is_some()) {
            dump(&state, &address, out, frame);
        }
        state.run_ocr_inline();
    }
    println!("ends as {:?}", state.slot_infos.get(&address).map(|x| &x.amount));
}
