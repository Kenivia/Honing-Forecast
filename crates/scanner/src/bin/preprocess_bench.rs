// Times the scanner's stages over a still, including the fine-grained number pre-processing
// timers, and prints what each cost.
//   cargo run --release --bin preprocess_bench -- <image>...
use hf_scanner::{scanner_state::ScannerState, timing};
use std::{env, fs};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn main() {
    for path in env::args().skip(1) {
        let image = image::open(&path).unwrap().to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let mut pixels = image.into_raw();
        let mut state = ScannerState::default();
        let ui_height = (height as f64 / 360.0).round() as u32 * 360;
        state.screen_info.game_width = ui_height * 16 / 9;
        state.screen_info.game_height = ui_height;
        state.buffer.width = width;
        state.buffer.height = height;
        state.buffer.size = pixels.len();
        state.buffer.pointer = Some(pixels.as_mut_ptr() as usize);
        state.config = rmp_serde::from_slice(
            &fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap(),
        )
        .unwrap();
        state.model = Some(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
        state.set_config();
        state.set_ocr_engine();
        state.initialize_anchors();
        state.initialize_page_num_infos();
        // enough scans for every slot to get its turn
        for _ in 0..30 {
            state.cropper();
            state.ocr_queue.clear();
        }
        let mut timings = timing::take();
        timings.sort_by(|a, b| b.1.total_cmp(&a.1));
        println!("\n=== {path}");
        println!("  {} slots read", state.slot_infos.len());
        println!("  stage                  total ms   calls   per call");
        for (name, total, calls) in timings {
            println!("  {name:22} {total:9.1} {calls:7}   {:8.3}", total / calls as f64);
        }
    }
}
