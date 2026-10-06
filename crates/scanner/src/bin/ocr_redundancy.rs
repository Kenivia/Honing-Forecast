// Prints what every OCR read of a hover came back with, so the repeat reads can be checked for
// redundancy: whether a second or third read of the same hover ever changes the answer, and how
// often the amount's tie-breaking third crop is needed.
//   python scripts/tooltips/dump_frames.py <mp4> | cargo run --release --bin ocr_redundancy -- <width> <height>
use hf_scanner::scanner_state::ScannerState;
use std::{collections::HashMap, env, fs, io::Read};

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

    let mut stdin = std::io::stdin().lock();
    let mut frame = 0;
    // how many reads of each kind a hover has had
    let mut counts: HashMap<(u32, &str), usize> = HashMap::new();
    while stdin.read_exact(&mut pixels).is_ok() {
        frame += 1;
        let scan = state.worth_scanning();
        state.update_scale();
        if scan {
            state.update_anchors();
            state.update_page_status();
            state.update_slots();
            let before: Vec<u32> = state.ocr_queue.iter().map(|job| job.id).collect();
            state.update_tooltip();
            let new: Vec<u32> =
                state.ocr_queue.iter().map(|j| j.id).filter(|id| !before.contains(id)).collect();
            // the reads sent this frame, kept before resolve_reads drains them
            let fresh: Vec<(u32, Option<Vec<u32>>, Option<[u32; 3]>, Vec<(Vec<u32>, u32)>)> = state
                .pending_reads
                .iter()
                .filter(|read| {
                    let title = read.title.iter().flat_map(|x| x.0.iter().copied());
                    let amount = read.amount.iter().flatten().copied();
                    let chest = read
                        .chest
                        .iter()
                        .flatten()
                        .flat_map(|(names, count, _)| names.iter().copied().chain([*count]));
                    title.chain(amount).chain(chest).any(|id| new.contains(&id))
                })
                .map(|read| {
                    (
                        read.hover,
                        read.title.as_ref().map(|x| x.0.clone()),
                        read.amount,
                        read.chest
                            .iter()
                            .flatten()
                            .map(|(names, count, _)| (names.clone(), *count))
                            .collect(),
                    )
                })
                .collect();
            state.run_ocr_inline();
            let text = |id: &u32| state.ocr_texts.get(id).cloned().unwrap_or_default();
            for (hover, title, amount, chest) in fresh {
                if let Some(lines) = title {
                    let n = counts.entry((hover, "title")).or_default();
                    *n += 1;
                    println!(
                        "title {hover} {n} {:?}",
                        lines.iter().map(text).collect::<Vec<_>>().join(" ")
                    );
                }
                if let Some(jobs) = amount {
                    let n = counts.entry((hover, "amount")).or_default();
                    *n += 1;
                    let [a, b, c] = jobs.map(|id| {
                        text(&id).chars().filter(char::is_ascii_digit).collect::<String>()
                    });
                    println!("amount {hover} {n} {a:?} {b:?} {c:?}");
                }
                if !chest.is_empty() {
                    let n = counts.entry((hover, "chest")).or_default();
                    *n += 1;
                    let rows: Vec<String> = chest
                        .iter()
                        .map(|(names, count)| {
                            format!(
                                "{}={}",
                                names.iter().map(text).collect::<Vec<_>>().join(" "),
                                text(count)
                            )
                        })
                        .collect();
                    println!("chest {hover} {n} {:?}", rows.join(" | "));
                }
            }
        }
        if frame % 200 == 0 {
            eprintln!("frame {frame}");
        }
    }
    eprintln!("{frame} frames");
}
