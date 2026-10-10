// Native harness for the tooltip reader: runs the whole scanner over stills or a recording and prints each hover.
//   cargo run --release --bin tooltip_test -- <image>...
//   python scripts/tooltips/dump_frames.py <mp4> | cargo run --release --bin tooltip_test -- --stdin <width> <height>
use hf_scanner::{
    constants::ANCHORS,
    native,
    ocr::jobs::OcrJob,
    scanner_state::ScannerState,
    tooltip::{chest::Chest, hover::Hover},
};
use std::{env, io::Read, time::Instant};

fn new_state(width: usize, height: usize) -> (ScannerState, Vec<u8>) {
    let (mut state, pixels) = native::new_state(width, height, native::guess_game(width, height), false);
    // SLOT_BUDGET=1 keeps the time budget, as in the browser; runs then differ with the machine's load
    state.no_slot_budget = env::var("SLOT_BUDGET").is_err();
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
    let chest = hover
        .chest_index
        .map(|index| describe_chest(&state.chests[index]));
    let rows: Vec<String> = hover
        .chest_rows
        .iter()
        .map(|row| format!("{} | {}", row.name_read, row.count_read))
        .collect();
    format!(
        "at {:?}  title {:?}  label {:?}  amount {:?}  {:?}  slot {:?}  (last read '{}')\n      chest {:?}\n      rows {:?}",
        hover.position,
        hover.title,
        hover.label,
        hover.amount,
        hover.tradability,
        slot,
        hover.last_read_title,
        chest,
        rows
    )
}

fn describe_chest(chest: &Chest) -> String {
    let contents: Vec<String> = chest
        .contents
        .iter()
        .map(|x| {
format!("{} x{}", x.item, x.amount)
        })
        .collect();
    let place = match (chest.slot, chest.column) {
        (Some(slot), _) => format!(
            "{:?} p{} {:?}",
            slot.inventory_type, slot.page_num, slot.pos_in_inv
        ),
        (None, Some((inventory, page, column))) => format!("{inventory:?} p{page} column {column}"),
        _ => "nowhere".to_string(),
    };
    format!(
        "{:?} [{}]  x{}  {:?}  {place}  '{}' {:?} {:?}",
        chest.kind,
        if chest.irrelevant { "irrelevant".to_string() } else { contents.join(", ") },
        chest.amount.clone().unwrap_or_default(),
        chest.tradability,
        chest.title,
        chest.variants,
        chest.icons
    )
}

// what the page would colour each known slot
fn print_statuses(state: &ScannerState) {
    let mut slots = state.result(true).slots;
    slots.sort_by_key(|slot| (slot.address.inventory_type as u8, slot.address.page_num, slot.address.pos_in_inv));
    for slot in slots {
        if let Some((icon, score)) = slot.icon_name_score {
            let a = slot.address;
            println!(
                "  {:?} p{} {:?}  {icon} {score:.3}  {:?}  value {:?}  {}{}",
                a.inventory_type,
                a.page_num,
                a.pos_in_inv,
                slot.status,
                slot.value,
                slot.reason,
                if slot.variants.is_empty() {
                    String::new()
                } else {
                    format!(
                        "  levelled {} also {:?} variants {:?}",
                        state.slot_infos[&a].levelled, slot.alternatives, slot.variants
                    )
                }
            );
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args[0] != "--stdin" {
        for path in &args {
            let image = image::open(path).unwrap().to_rgba8();
            let (mut state, mut pixels) =
                new_state(image.width() as usize, image.height() as usize);
            pixels.copy_from_slice(image.as_raw());
            // enough scans for every slot to be read, they are looked at a few at a time
            for _ in 0..30 {
                state.cropper();
                state.run_ocr_inline();
            }
            println!("{path}");
            let scores: Vec<f64> = state
                .slot_infos
                .values()
                .filter_map(|slot| Some(slot.icon_name_score.as_ref()?.1))
                .collect();
            println!(
                "  brightness {:?}, {} slots recognised, mean confidence {:.4}",
                state.screen_info.brightness,
                scores.len(),
                scores.iter().sum::<f64>() / scores.len() as f64
            );
            match &state.hover {
                Some(hover) => println!("  {}", describe(&state, hover)),
                None => println!("  no tooltip"),
            }
            print_statuses(&state);
        }
        return;
    }

    let (width, height) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let (mut state, mut pixels) = new_state(width, height);
    let mut stdin = std::io::stdin().lock();
    let (mut index, mut seen, mut hovers, mut start) = (0, 0, 0, 0);
    let mut previous: Option<Hover> = None;
    let mut spent = 0.0;
    let mut brightness = None;
    let mut skipped = 0;
    // OCR_PER_FRAME=n reads at most n lines a frame, as if OCR were as slow as in the browser
    let per_frame: Option<usize> = env::var("OCR_PER_FRAME").ok().map(|x| x.parse().unwrap());
    let (mut backlog, mut longest, mut read): (Vec<OcrJob>, usize, usize) = (vec![], 0, 0);
    // frames each line waited, first reads and later ones
    let mut queued: std::collections::HashMap<u32, usize> = Default::default();
    let mut waits: [Vec<usize>; 2] = Default::default();
    loop {
        let more = stdin.read_exact(&mut pixels).is_ok();
        if more {
            // cropper(), with the tooltip step and all OCR timed on their own
            let scan = state.worth_scanning();
            skipped += !scan as usize;
            state.update_scale();
            if scan {
                state.update_anchors();
            }
            if state.screen_info.brightness != brightness {
                brightness = state.screen_info.brightness;
                // each found anchor's own estimate
                let anchors: Vec<String> = ANCHORS
                    .iter()
                    .flat_map(|spec| {
                        let found = &state.anchors[&spec.anchor_type].positions;
                        spec.variants
                            .iter()
                            .zip(found)
                            .filter_map(|(variant, found)| {
                                found.map(|x| {
                                    format!("{:?} {} {:.1?}", spec.anchor_type, variant.name, x.brightness)
                                })
                            })
                    })
                    .collect();
                eprintln!(
                    "frame {index}: brightness {:.2} over {} anchors, now found: {anchors:?}",
                    brightness.unwrap(),
                    state.screen_info.brightness_anchors
                );
            }
            if scan {
                state.update_page_status();
                state.update_slots();
            }
            let clock = Instant::now();
            if scan {
                state.update_tooltip();
            }
            match per_frame {
                None => state.run_ocr_inline(),
                // like the browser: only so many lines are read per frame, the rest wait
                Some(lines) => {
                    for job in &state.ocr_queue {
                        queued.insert(job.id, index);
                    }
                    backlog.append(&mut state.ocr_queue);
                    backlog.sort_by_key(|job| (job.priority, job.id));
                    longest = longest.max(backlog.len());
                    let batch: Vec<_> = backlog.drain(..lines.min(backlog.len())).collect();
                    read += batch.len();
                    for job in &batch {
                        let class = (job.priority > 0) as usize;
                        waits[class].push(index - queued[&job.id]);
                    }
                    state.apply_ocr(
                        batch
                            .iter()
                            .map(|job| (job.id, job.line.read()))
                            .collect(),
                    );
                }
            }
            spent += clock.elapsed().as_secs_f64() * 1000.0;
        } else {
            // what is still waiting is read before the last hover is printed
            state.ocr_queue.append(&mut backlog);
            read += state.ocr_queue.len();
            state.run_ocr_inline();
            state.hover = None;
        }
        let moved = match (&previous, &state.hover) {
            (Some(old), Some(new)) => {
                old.position.0.abs_diff(new.position.0) > 4
                    || old.position.1.abs_diff(new.position.1) > 4
            }
            (Some(_), None) => true,
            _ => false,
        };
        if moved {
            hovers += 1;
            println!(
                "{start:4}-{:4}  {}",
                index - 1,
                describe(&state, previous.as_ref().unwrap())
            );
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
    println!("{skipped} frames not scanned");
    eprintln!(
        "stage timings (name, ms, calls): {:?}",
        hf_scanner::timing::take()
    );
    println!(
        "{index} frames, tooltip on {seen}, {hovers} hovers, {:.1} ms per frame in the tooltip step and OCR, brightness {:?}",
        spent / index as f64,
        state.screen_info.brightness
    );
    println!("{read} lines read, longest OCR queue {longest}");
    for (name, waits) in ["first", "later"].iter().zip(&mut waits) {
        waits.sort();
        if let Some(most) = waits.last() {
            println!(
                "{name} reads: {} lines, waited {} frames at the median, {} at the 90th percentile, {most} at most",
                waits.len(),
                waits[waits.len() / 2],
                waits[waits.len() * 9 / 10]
            );
        }
    }
    println!("{} chests", state.chests.len());
    for chest in &state.chests {
        println!("  {}", describe_chest(chest));
    }
    let mut infos: Vec<_> = state.slot_infos.iter().collect();
    infos.sort_by_key(|(address, _)| (address.inventory_type as u8, address.page_num, address.pos_in_inv));
    for (address, info) in infos {
        if info.tradability.is_some() || info.tooltip_amount.is_some() {
            println!(
                "{:?} p{} {:?}  {}  slot amount {:?}  tooltip amount {:?}  {:?}  {:?}",
                address.inventory_type,
                address.page_num,
                address.pos_in_inv,
                info.icon_name_score.as_ref().unwrap().0,
                info.amount,
                info.tooltip_amount,
                info.tradability,
                info.label
            );
        }
    }
    print_statuses(&state);
}
