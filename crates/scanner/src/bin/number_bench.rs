// Scores slot-number clean-up variants: every time the scanner reads a slot's number, the same crop
// is cleaned up with each of VARIANTS and read with both alphabets. Then two more: the shape reader
// alone ("shapes"), and as the scanner reads (the shapes when sure, else the first variant's read).
// The shapes read public/fonts/English.ttf.
//   cargo run --release --bin number_bench -- <image or capture.hfcap>... > reads.tsv
//   python scanner_setup/scanner/number_bench.py reads.tsv ...
// A capture's truth is its hand edits, else its tooltip amounts. The clean-up's time per variant goes
// to stderr. --glyphs <folder> keeps every number crop and the background expected behind it, named
// by the line's last field; --save <folder> keeps every strip.
// Lines: READ source frame slot icon, then per variant the full-alphabet read and the digits one;
// TRUTH source slot icon amount, for slots a tooltip was tied to.
use clap::Parser;
use hf_scanner::{
    dev_utils::native::{self, Options, replay_capture_with, scan_still_with},
    interface::setup::BASE_ICONS,
    ocr::{
        number::{NumberParams, number_background, number_strip},
        recognize::recognize_line,
        shapes::{MIN_MARGIN, shape_read},
    },
    scanner_state::{ScannerState, SlotAddress},
};
use image::{Rgba, RgbaImage};
use std::time::{Duration, Instant};

#[derive(Parser)]
struct Cli {
    sources: Vec<String>,
    #[arg(long)]
    glyphs: Option<String>,
    #[arg(long)]
    save: Option<String>,
    #[command(flatten)]
    options: Options,
}

// the first is what the scanner does; add what is being tried
fn variants() -> Vec<(&'static str, NumberParams)> {
    let now = NumberParams::default;
    vec![("now", now())]
}

fn place(address: &SlotAddress) -> String {
    format!(
        "{:?} p{} {:?}",
        address.inventory_type, address.page_num, address.pos_in_inv
    )
}

fn report(
    cli: &Cli,
    state: &ScannerState,
    address: &SlotAddress,
    source: &str,
    frame: usize,
    times: &mut [Duration],
) {
    let info = &state.slot_infos[address];
    let name = &info.icon_name_score.as_ref().unwrap().0;
    let template = BASE_ICONS.read()[name].data.clone();
    let mut line = format!("READ\t{source}\t{frame}\t{}\t{name}", place(address));
    let glyphs = cli.glyphs.as_ref().map(|folder| {
        let id = format!("{source}_{frame}_{}", place(address))
            .replace([' ', ',', '(', ')', '/', '\\', ':', '.'], "_");
        let number = &info.observed_number.data;
        let (w, h) = number.dimensions();
        let brightness = state.screen_info.brightness.value.unwrap();
        let background = number_background(&template, &info.observed_icon.data, w, h, brightness);
        number.save(format!("{folder}/{id}_n.png")).unwrap();
        RgbaImage::from_fn(w, h, |x, y| {
            let c = background[(y * w + x) as usize];
            Rgba([c[0] as u8, c[1] as u8, c[2] as u8, 255])
        })
        .save(format!("{folder}/{id}_b.png"))
        .unwrap();
        id
    });
    let mut first = String::new();
    for (index, (variant, params)) in variants().into_iter().enumerate() {
        let start = Instant::now();
        let strip = number_strip(
            &info.observed_number.data,
            &info.observed_icon.data,
            &template,
            state.screen_info.brightness.value.unwrap(),
            &params,
        );
        times[index] += start.elapsed();
        if let Some(folder) = &cli.save {
            let id =
                format!("{frame}_{}_{variant}", place(address)).replace([' ', ',', '(', ')'], "_");
            strip.save(format!("{folder}/{id}.png")).unwrap();
        }
        for numbers in [false, true] {
            let text = recognize_line(&strip, numbers);
            if index == 0 && numbers {
                first = text.clone();
            }
            line += &format!("\t{}", text.trim());
        }
    }
    let start = Instant::now();
    let read = shape_read(
        &info.observed_number.data,
        &info.observed_icon.data,
        &template,
        state.screen_info.brightness.value.unwrap(),
    )
    .unwrap();
    times[variants().len()] += start.elapsed();
    let scanner = if read.margin >= MIN_MARGIN {
        read.text.clone()
    } else {
        first
    };
    line += &format!("\t{0}\t{0}\t{1}\t{1}", read.text, scanner.trim());
    if let Some(id) = glyphs {
        line += &format!("\t{id}");
    }
    println!("{line}");
}

fn main() {
    let cli = Cli::parse();
    native::configure(&cli.options);
    println!(
        "VARIANTS	{}",
        variants()
            .iter()
            .map(|x| x.0)
            .chain(["shapes", "shapes, else OCR"])
            .collect::<Vec<_>>()
            .join("	")
    );
    let mut times = vec![Duration::ZERO; variants().len() + 1];
    let mut strips = 0;
    for source in &cli.sources {
        // a job still waiting was asked for on this frame
        let mut each = |state: &mut ScannerState, frame: usize| {
            let asked: Vec<SlotAddress> = state
                .slot_infos
                .iter()
                .filter(|(_, info)| info.amount_job.is_some())
                .map(|(address, _)| *address)
                .collect();
            for address in asked {
                report(&cli, state, &address, source, frame, &mut times);
                strips += 1;
            }
        };
        let (state, end) = if source.ends_with(".hfcap") {
            let (state, _pixels, end) = replay_capture_with(source, &mut each);
            (state, end)
        } else {
            (scan_still_with(source, &mut each).0, None)
        };
        truth(&state, source, end);
    }
    let names = variants().into_iter().map(|x| x.0).chain(["shapes"]);
    for (name, time) in names.zip(&times) {
        eprintln!(
            "{name}: {:.0} us a strip over {strips}",
            time.as_secs_f64() * 1e6 / strips.max(1) as f64
        );
    }
}

// the tooltip amounts, then a capture's hand edits, which win
fn truth(state: &ScannerState, source: &str, end: Option<String>) {
    let icon = |address: &SlotAddress| {
        let info = state.slot_infos.get(address)?;
        Some(info.icon_name_score.as_ref()?.0.clone())
    };
    for (address, info) in &state.slot_infos {
        if let (Some(amount), Some(name)) = (&info.tooltip_amount, icon(address)) {
            println!("TRUTH	{source}	{}	{name}	{amount}", place(address));
        }
    }
    let end: serde_json::Value = serde_json::from_str(&end.unwrap_or("{}".into())).unwrap();
    for edit in end["edits"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .values()
    {
        let address: SlotAddress = serde_json::from_value(edit["address"].clone()).unwrap();
        if let (Some(_), Some(name)) = (edit["item"].as_str(), icon(&address)) {
            let amount = edit["amount"].as_u64().unwrap_or(1);
            println!("TRUTH	{source}	{}	{name}	{amount}", place(&address));
        }
    }
}
