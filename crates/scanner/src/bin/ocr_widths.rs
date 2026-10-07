// Prints the width of every OCR strip the scanner asks for, with what kind of text it is,
// for the OCR batching experiments. Jobs queued by the slot step are numbers; the rest are
// matched against the pending read that wanted them.
//   python scripts/tooltips/dump_frames.py <mp4> | cargo run --release --bin ocr_widths -- <width> <height>
use hf_scanner::scanner_state::ScannerState;
use std::{collections::HashMap, env, fs, io::Read};

// STRIP_DIR=<dir> also saves every strip there, for the batching bench;
// MAX_FRAMES=<n> stops early

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (width, height): (usize, usize) = (args[0].parse().unwrap(), args[1].parse().unwrap());
    let mut state = ScannerState::default();
    let mut pixels = vec![0u8; width * height * 4];
    let ui_height = (height as f64 / 360.0).round() as u32 * 360;
    state.screen_info.game_width = ui_height * 16 / 9;
    state.screen_info.game_height = ui_height;
    state.buffer.width = width;
    state.buffer.height = height;
    state.buffer.size = pixels.len();
    state.buffer.pointer = Some(pixels.as_mut_ptr() as usize);
    state.config =
        rmp_serde::from_slice(&fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap())
            .unwrap();
    state.model = Some(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
    state.set_config();
    state.set_ocr_engine();
    state.initialize_anchors();
    state.initialize_page_num_infos();

    let strip_dir = env::var("STRIP_DIR").ok();
    if let Some(dir) = &strip_dir {
        fs::create_dir_all(dir).unwrap();
    }
    let max_frames: usize = env::var("MAX_FRAMES").map_or(usize::MAX, |x| x.parse().unwrap());
    let mut stdin = std::io::stdin().lock();
    let mut frame = 0;
    while stdin.read_exact(&mut pixels).is_ok() {
        frame += 1;
        let scan = state.worth_scanning();
        state.update_scale();
        if scan {
            state.update_anchors();
            state.update_page_status();
            state.update_slots();
            // whatever the slot step queued is a slot's number strip
            let mut numbers = Vec::new();
            for job in &state.ocr_queue {
                println!("number {} {frame}", job.image.width());
                numbers.push(job.id);
            }
            let save = |kind: &str, id: u32, state: &ScannerState| {
                if let Some(dir) = &strip_dir
                    && let Some(job) = state.ocr_queue.iter().find(|j| j.id == id)
                {
                    let w = job.image.width();
                    job.image
                        .save(format!("{dir}/{kind}_{w:04}_{id}.png"))
                        .unwrap();
                }
            };
            for id in numbers.clone() {
                save("number", id, &state);
            }
            state.update_tooltip();
            let mut widths: HashMap<u32, u32> = HashMap::new();
            let mut queued = Vec::new();
            for job in &state.ocr_queue {
                if !numbers.contains(&job.id) {
                    widths.insert(job.id, job.image.width());
                    if strip_dir.is_some() {
                        queued.push((job.id, job.image.clone()));
                    }
                }
            }
            // the pending reads say which of those ids is which kind of text; a frame can send
            // two reads, one of them a hover being retired
            for read in &state.pending_reads {
                let mut say = |kind: &str, id: u32| {
                    if let Some(w) = widths.remove(&id) {
                        println!("{kind} {w} {frame}");
                        if let Some(dir) = &strip_dir
                            && let Some(job) = queued.iter().find(|j| j.0 == id)
                        {
                            job.1.save(format!("{dir}/{kind}_{w:04}_{id}.png")).unwrap();
                        }
                    }
                };
                if let Some((lines, _)) = &read.title {
                    for id in lines {
                        say("title", *id);
                    }
                }
                for id in read.amount.iter().flatten() {
                    say("amount", *id);
                }
                for row in read.chest.iter().flatten() {
                    for id in &row.0 {
                        say("chest_name", *id);
                    }
                    say("chest_count", row.1);
                }
            }
            for width in widths.values() {
                println!("unattributed {width} {frame}");
            }
            state.run_ocr_inline();
        }
        if frame % 200 == 0 {
            eprintln!("frame {frame}");
        }
        if frame >= max_frames {
            break;
        }
    }
    eprintln!("{frame} frames");
}
