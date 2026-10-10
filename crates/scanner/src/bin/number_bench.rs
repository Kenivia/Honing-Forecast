// Scores slot-number clean-up variants: every time the scanner reads a slot's number, the same crop
// is cleaned up with each of VARIANTS and read with both alphabets.
//   cargo run --release --bin number_bench -- <image>... > stills.tsv
//   python scripts/tooltips/dump_frames.py <mp4> | cargo run --release --bin number_bench -- --stdin <width> <height> > rec.tsv
//   python scripts/tooltips/number_bench.py stills.tsv rec.tsv ...
// Lines: READ source frame slot icon, then per variant the full-alphabet read and the digits one;
// TRUTH source slot icon amount, for slots a tooltip was tied to.
use hf_scanner::{
    native,
    ocr::{
        number::{NumberParams, number_strip},
        recognize::recognize_line,
    },
    scanner_state::{ScannerState, SlotAddress},
    setup::BASE_ICONS,
};
use std::{env, io::Read};


// the first is what the scanner does; add what is being tried
fn variants() -> Vec<(&'static str, NumberParams)> {
    let now = NumberParams::default;
    vec![
        ("now", now()),
        ("no neighbour pass", NumberParams { low_alpha: now().min_alpha, ..now() }),
        ("no colour test", NumberParams { min_chroma: f64::INFINITY, ..now() }),
    ]
}

fn place(address: &SlotAddress) -> String {
    format!("{:?} p{} {:?}", address.inventory_type, address.page_num, address.pos_in_inv)
}

fn report(state: &ScannerState, address: &SlotAddress, source: &str, frame: usize) {
    let info = &state.slot_infos[address];
    let name = &info.icon_name_score.as_ref().unwrap().0;
    let template = BASE_ICONS.read()[name].data.clone();
    let mut line = format!("READ\t{source}\t{frame}\t{}\t{name}", place(address));
    for (variant, params) in variants() {
        let strip = number_strip(
            &info.observed_number.data,
            &info.observed_icon.data,
            &template,
            state.screen_info.brightness.unwrap(),
            &params,
        );
        // SAVE=<folder> keeps every strip
        if let Ok(folder) = env::var("SAVE") {
            let id = format!("{frame}_{}_{variant}", place(address)).replace([' ', ',', '(', ')'], "_");
            strip.save(format!("{folder}/{id}.png")).unwrap();
        }
        for numbers in [false, true] {
            line += &format!("\t{}", recognize_line(&strip, numbers).trim());
        }
    }
    println!("{line}");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    println!("VARIANTS\t{}", variants().iter().map(|x| x.0).collect::<Vec<_>>().join("\t"));
    let from_stdin = args[0] == "--stdin";
    let sources: Vec<String> = if from_stdin { vec!["stdin".to_string()] } else { args.clone() };
    for source in &sources {
        let still = (!from_stdin).then(|| image::open(source).unwrap().to_rgba8());
        let (width, height): (usize, usize) = match &still {
            Some(image) => (image.width() as usize, image.height() as usize),
            None => (args[1].parse().unwrap(), args[2].parse().unwrap()),
        };
        let (mut state, mut pixels) = native::new_state(width, height, native::guess_game(width, height), false);

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
                report(&state, &address, source, frame);
            }
            state.run_ocr_inline();
        }
        for (address, info) in &state.slot_infos {
            if let (Some(amount), Some(icon)) = (&info.tooltip_amount, &info.icon_name_score) {
                println!("TRUTH\t{source}\t{}\t{}\t{amount}", place(address), icon.0);
            }
        }
    }
}
