// Scores slot-number clean-up variants: every time the scanner reads a slot's number, the same crop
// is cleaned up with each of VARIANTS and read with both alphabets. Then two more: the shape reader
// alone ("shapes"), and as the scanner reads (the shapes when sure, else the first variant's read).
// The shapes read public/fonts/English.ttf.
//   cargo run --release --bin number_bench -- <image>... > stills.tsv
//   python scanner_setup/tooltips/dump_frames.py <mp4> | cargo run --release --bin number_bench -- --stdin <width> <height> > rec.tsv
//   cargo run --release --bin number_bench -- <capture.hfcap>... > caps.tsv
//   python scanner_setup/tooltips/number_bench.py stills.tsv rec.tsv ...
// A capture's truth is its hand edits, else its tooltip amounts. The clean-up's time per variant goes
// to stderr.
// Lines: READ source frame slot icon, then per variant the full-alphabet read and the digits one;
// TRUTH source slot icon amount, for slots a tooltip was tied to.
use hf_scanner::{
    dev_utils::{capture::read_frames, native},
    interface::setup::BASE_ICONS,
    ocr::{
        number::{NumberParams, number_background, number_strip},
        recognize::recognize_line,
        shapes::{MIN_MARGIN, shape_read},
    },
    scanner_state::{ScannerState, SlotAddress},
};
use image::{Rgba, RgbaImage};
use std::{
    env, fs,
    io::Read,
    time::{Duration, Instant},
};

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
    // GLYPHS=<folder> keeps every number crop and the background expected behind it, named by the
    // last field of the line
    let glyphs = env::var("GLYPHS").ok().map(|folder| {
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
        // SAVE=<folder> keeps every strip
        if let Ok(folder) = env::var("SAVE") {
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
    let args: Vec<String> = env::args().skip(1).collect();
    println!(
        "VARIANTS\t{}",
        variants()
            .iter()
            .map(|x| x.0)
            .chain(["shapes", "shapes, else OCR"])
            .collect::<Vec<_>>()
            .join("\t")
    );
    let from_stdin = args[0] == "--stdin";
    let sources: Vec<String> = if from_stdin {
        vec!["stdin".to_string()]
    } else {
        args.clone()
    };
    let mut times = vec![Duration::ZERO; variants().len() + 1];
    let mut strips = 0;
    for source in &sources {
        if source.ends_with(".hfcap") {
            strips += capture(source, &mut times);
            continue;
        }
        let still = (!from_stdin).then(|| image::open(source).unwrap().to_rgba8());
        let (width, height): (usize, usize) = match &still {
            Some(image) => (image.width() as usize, image.height() as usize),
            None => (args[1].parse().unwrap(), args[2].parse().unwrap()),
        };
        let (mut state, mut pixels) = native::new_state(width, height);

        let mut stdin = std::io::stdin().lock();
        for frame in 0.. {
            match &still {
                // enough scans for every slot to get its look
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
            let asked: Vec<SlotAddress> = state
                .slot_infos
                .iter()
                .filter(|(_, info)| info.amount_job.is_some())
                .map(|(address, _)| *address)
                .collect();
            for address in asked {
                report(&state, &address, source, frame, &mut times);
                strips += 1;
            }
            state.run_ocr_inline();
        }
        for (address, info) in &state.slot_infos {
            if let (Some(amount), Some(icon)) = (&info.tooltip_amount, &info.icon_name_score) {
                println!("TRUTH\t{source}\t{}\t{}\t{amount}", place(address), icon.0);
            }
        }
    }
    let names = variants().into_iter().map(|x| x.0).chain(["shapes"]);
    for (name, time) in names.zip(&times) {
        eprintln!(
            "{name}: {:.0} us a strip over {strips}",
            time.as_secs_f64() * 1e6 / strips.max(1) as f64
        );
    }
}

// a debug capture replayed as the scanner would; returns how many strips were made
fn capture(source: &str, times: &mut [Duration]) -> usize {
    let file = fs::read(source).unwrap();
    let (start, frames, end) = read_frames(&file);
    let (mut state, _pixels) = native::new_state(start.width, start.height);
    state.screen_info.forced_21_9 = start.forced_21_9;
    state.screen_info.brightness = start.brightness;
    let mut strips = 0;
    for (frame, input) in frames.into_iter().enumerate() {
        state.replay(input, vec![]);
        let asked: Vec<SlotAddress> = state
            .slot_infos
            .iter()
            .filter(|(_, info)| info.amount_job.is_some())
            .map(|(address, _)| *address)
            .collect();
        for address in asked {
            report(&state, &address, source, frame, times);
            strips += 1;
        }
        state.run_ocr_inline();
    }
    let icon = |address: &SlotAddress| {
        let info = state.slot_infos.get(address)?;
        Some(info.icon_name_score.as_ref()?.0.clone())
    };
    for (address, info) in &state.slot_infos {
        if let (Some(amount), Some(name)) = (&info.tooltip_amount, icon(address)) {
            println!("TRUTH\t{source}\t{}\t{name}\t{amount}", place(address));
        }
    }
    // hand edits last, so they win
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
            println!("TRUTH\t{source}\t{}\t{name}\t{amount}", place(&address));
        }
    }
    strips
}
