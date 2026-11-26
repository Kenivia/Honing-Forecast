// Every stage of one slot's number clean-up as images, each time the scanner reads that slot.
//   cargo run --release --bin number_probe -- <out dir> <Roster|CharStorage|CharInventory> <page> <row> <col> <image or capture.hfcap>
//   python scanner_setup/scanner/number_stages.py <out dir>     (one picture per read)
// Page, row and column count from 0. Files are f<frame>_<stage>.png.
use clap::Parser;
use hf_scanner::{
    dev_utils::native::{self, Options, replay_capture_with, scan_still_with},
    interface::setup::BASE_ICONS,
    ocr::{
        number::{
            Layers, NumberParams, number_background, number_layers, number_mask, number_strip,
        },
        recognize::recognize_line,
    },
    scanner_state::{InventoryType, ScannerState, SlotAddress},
};
use image::{GrayImage, Luma, Rgba, RgbaImage};
use std::fs;

#[derive(Parser)]
struct Cli {
    out: String,
    inventory: String,
    page: usize,
    row: usize,
    column: usize,
    source: String,
    #[command(flatten)]
    options: Options,
}

fn grey(w: u32, h: u32, value: impl Fn(usize) -> f64) -> GrayImage {
    GrayImage::from_fn(w, h, |x, y| {
        Luma([(value((y * w + x) as usize) * 255.0).clamp(0.0, 255.0) as u8])
    })
}

fn dump(state: &ScannerState, address: &SlotAddress, out: &str, frame: usize) {
    let info = &state.slot_infos[address];
    let name = &info.icon_name_score.as_ref().unwrap().0;
    let brightness = state.screen_info.brightness.value.unwrap();
    let (number, icon) = (&info.observed_number.data, &info.observed_icon.data);
    let (w, h) = number.dimensions();
    let template = BASE_ICONS.read()[name].data.clone();
    let params = NumberParams::default();
    let background = number_background(&template, icon, w, h, brightness);
    let Layers {
        alpha,
        by_colour,
        shadow,
        ..
    } = number_layers(number, &background, &params);
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
    grey(w, h, |i| by_colour[i])
        .save(path("4_alpha_colour"))
        .unwrap();
    grey(w, h, |i| shadow[i] as u8 as f64)
        .save(path("5_shadow"))
        .unwrap();
    number_mask(number, &background, &params)
        .save(path("6_mask"))
        .unwrap();
    strip.save(path("7_sent")).unwrap();
    println!(
        "f{frame:04}\t{name}\tbrightness {brightness:.2}\treads {:?}",
        recognize_line(&strip, true)
    );
}

fn main() {
    let cli = Cli::parse();
    native::configure(&cli.options);
    let out = &cli.out;
    fs::create_dir_all(out).unwrap();
    let address = SlotAddress {
        inventory_type: match cli.inventory.as_str() {
            "Roster" => InventoryType::Roster,
            "CharStorage" => InventoryType::CharStorage,
            _ => InventoryType::CharInventory,
        },
        page_num: cli.page,
        pos_in_inv: (cli.row, cli.column),
    };
    // a job still waiting was asked for on this frame
    let each = |state: &mut ScannerState, frame: usize| {
        if state
            .slot_infos
            .get(&address)
            .is_some_and(|x| x.amount_job.is_some())
        {
            dump(state, &address, out, frame);
        }
    };
    let state = if cli.source.ends_with(".hfcap") {
        replay_capture_with(&cli.source, each).0
    } else {
        scan_still_with(&cli.source, each).0
    };
    println!(
        "ends as {:?}",
        state.slot_infos.get(&address).map(|x| &x.amount)
    );
}
