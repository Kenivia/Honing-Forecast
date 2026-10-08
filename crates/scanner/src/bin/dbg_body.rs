// TEMPORARY experiment: what the tooltip body's lines read as, per frame. Delete when done.
use hf_scanner::{
    image_utils::{brightness::brightness_lut, ocr::recognize_line},
    scanner_state::ScannerState,
    tooltip::{
        common::text_lines,
        detect::find_title,
        items::match_title,
        title::{join_title, text_image, title_lines},
    },
};
use std::{env, fs, io::Read};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (width, height): (usize, usize) = (args[0].parse().unwrap(), args[1].parse().unwrap());
    let from: usize = args.get(2).map_or(0, |x| x.parse().unwrap());
    let mut state = ScannerState::default();
    let mut pixels = vec![0u8; width * height * 4];
    state.screen_info.game_width = 1920;
    state.screen_info.game_height = 1080;
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

    let mut stdin = std::io::stdin().lock();
    let mut index = 0;
    while stdin.read_exact(&mut pixels).is_ok() {
        state.worth_scanning();
        state.update_scale();
        state.update_anchors();
        index += 1;
        let Some(brightness) = state.screen_info.brightness else { continue };
        if index - 1 < from {
            continue;
        }
        let s = state.screen_info.scale_factor;
        state.buffer.lut = Some(brightness_lut(brightness));
        let buffer = state.buffer;
        let Some(bar) = find_title(&buffer, s) else { continue };
        let (lines, centred) = title_lines(&buffer, &bar, s);
        let read = join_title(&lines.iter().map(recognize_line).collect::<Vec<_>>());
        let title = match_title(&read).and_then(|x| x.title.clone());

        // body lines below the icon, over the whole width
        let pad = (5.0 * s).round() as usize;
        let (left, right) = (bar.x + pad, bar.x + bar.width - pad);
        let top = bar.y + bar.height + (108.0 * s) as usize;
        let bottom = (top + (330.0 * s) as usize).min(buffer.height - 4);
        let is_text = |x: usize, y: usize| buffer.rgb(x, y).into_iter().max().unwrap() > 110;
        let rows: Vec<bool> = (top..bottom).map(|y| (left..right).any(|x| is_text(x, y))).collect();
        let mut body = vec![];
        for (a, b) in text_lines(&rows, 8.0 * s) {
            if (b - a) as f64 > 27.0 * s {
                body.push("<merged>".to_string());
                continue;
            }
            let mut columns = (left..right).filter(|x| (top + a..top + b).any(|y| is_text(*x, y)));
            let first = columns.next().unwrap();
            let last = columns.last().unwrap_or(first);
            let (across, down) = ((4.0 * s).round() as usize, (2.67 * s).round() as usize);
            let image = text_image(
                &buffer,
                first.saturating_sub(across).max(bar.x),
                top + a - down,
                (last + across + 1).min(bar.x + bar.width),
                top + b + down,
            );
            body.push(recognize_line(&image));
        }
        println!(
            "{}\t{}\t{}\t{}\t{}\t{:?}\t{}",
            index - 1,
            bar.x,
            bar.y,
            centred,
            read,
            title,
            body.join(" | ")
        );
    }
}
