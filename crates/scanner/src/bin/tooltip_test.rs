// Native harness for the tooltip reader: runs the whole scanner over stills or a recording and prints each hover.
//   cargo run --release --bin tooltip_test -- <image>...
//   python scripts/tooltips/dump_frames.py <mp4> | cargo run --release --bin tooltip_test -- --stdin <width> <height>
use hf_scanner::{
    scanner_state::ScannerState,
    tooltip::{chest::Chest, hover::Hover},
};
use std::{env, fs, io::Read, time::Instant};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn new_state(width: usize, height: usize) -> (ScannerState, Vec<u8>) {
    let mut state = ScannerState::default();
    let mut pixels = vec![0u8; width * height * 4];
    // 1078-tall recordings are still 1080p
    let ui_height = (height as f64 / 360.0).round() as u32 * 360;
    state.screen_info.game_width = ui_height * 16 / 9;
    state.screen_info.game_height = ui_height;
    state.buffer.width = width;
    state.buffer.height = height;
    state.buffer.size = pixels.len();
    state.buffer.pointer = Some(pixels.as_mut_ptr() as usize);
    let config = fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap();
    state.config = rmp_serde::from_slice(&config).unwrap();
    state.model = Some(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
    state.set_config();
    state.set_ocr_engine();
    state.initialize_anchors();
    state.initialize_page_num_infos();
    (state, pixels)
}

fn describe(state: &ScannerState, hover: &Hover) -> String {
    let slot = hover.slot.map(|address| {
        let info = &state.slot_infos[&address];
        format!(
            "{:?} p{} {:?} holding {} x{}",
            address.inventory_type,
            address.page_num,
            address.pos_in_inv,
            info.icon_name_score.as_ref().unwrap().0,
            info.amount.clone().unwrap_or_default()
        )
    });
    let chest = hover.chest_index.map(|index| describe_chest(&state.chests[index]));
    let rows: Vec<String> = hover
        .chest_rows
        .iter()
        .map(|row| format!("{} | {}", row.name_read, row.count_read))
        .collect();
    format!(
        "at {:?}  title {:?}  amount {:?}  {:?}  slot {:?}  (last read '{}')\n      chest {:?}\n      rows {:?}",
        hover.position, hover.title, hover.amount, hover.tradability, slot, hover.last_read_title, chest, rows
    )
}

fn describe_chest(chest: &Chest) -> String {
    let contents: Vec<String> = chest
        .contents
        .iter()
        .map(|x| format!("{} x{}{}", x.item, x.amount, if x.bound { " (Bound)" } else { "" }))
        .collect();
    let place = match (chest.slot, chest.column) {
        (Some(slot), _) => format!("{:?} p{} {:?}", slot.inventory_type, slot.page_num, slot.pos_in_inv),
        (None, Some((inventory, page, column))) => format!("{inventory:?} p{page} column {column}"),
        _ => "nowhere".to_string(),
    };
    format!(
        "{:?} [{}]  x{}  {:?}  {place}  '{}'",
        chest.kind,
        contents.join(", "),
        chest.amount.clone().unwrap_or_default(),
        chest.tradability,
        chest.last_read_title
    )
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args[0] != "--stdin" {
        for path in &args {
            let image = image::open(path).unwrap().to_rgba8();
            let (mut state, mut pixels) = new_state(image.width() as usize, image.height() as usize);
            pixels.copy_from_slice(image.as_raw());
            for _ in 0..4 {
                state.cropper();
            }
            println!("{path}");
            println!("  brightness {:?}", state.screen_info.brightness);
            match &state.hover {
                Some(hover) => println!("  {}", describe(&state, hover)),
                None => println!("  no tooltip"),
            }
        }
        return;
    }

    let (width, height) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let (mut state, mut pixels) = new_state(width, height);
    let mut stdin = std::io::stdin().lock();
    let (mut index, mut seen, mut hovers, mut start) = (0, 0, 0, 0);
    let mut previous: Option<Hover> = None;
    let mut spent = 0.0;
    loop {
        let more = stdin.read_exact(&mut pixels).is_ok();
        if more {
            // cropper(), with the tooltip step timed on its own
            state.update_scale();
            state.update_anchors();
            state.update_page_status();
            state.update_slots();
            let clock = Instant::now();
            state.update_tooltip();
            spent += clock.elapsed().as_secs_f64() * 1000.0;
        } else {
            state.hover = None;
        }
        let moved = match (&previous, &state.hover) {
            (Some(old), Some(new)) => {
                old.position.0.abs_diff(new.position.0) > 4 || old.position.1.abs_diff(new.position.1) > 4
            }
            (Some(_), None) => true,
            _ => false,
        };
        if moved {
            hovers += 1;
            println!("{start:4}-{:4}  {}", index - 1, describe(&state, previous.as_ref().unwrap()));
        }
        if moved || previous.is_none() {
            start = index;
        }
        if !more {
            break;
        }
        seen += state.hover.as_ref().is_some_and(|hover| hover.missed == 0) as usize;
        previous = state.hover.clone();
        index += 1;
    }
    println!(
        "{index} frames, tooltip on {seen}, {hovers} hovers, {:.1} ms per frame in the tooltip step, brightness {:?}",
        spent / index as f64,
        state.screen_info.brightness
    );
    println!("{} chests", state.chests.len());
    for chest in &state.chests {
        println!("  {}", describe_chest(chest));
    }
    for (address, info) in &state.slot_infos {
        if info.tradability.is_some() || info.tooltip_amount.is_some() {
            println!(
                "{:?} p{} {:?}  {}  slot amount {:?}  tooltip amount {:?}  {:?}",
                address.inventory_type,
                address.page_num,
                address.pos_in_inv,
                info.icon_name_score.as_ref().unwrap().0,
                info.amount,
                info.tooltip_amount,
                info.tradability
            );
        }
    }
}
