// Dumps every recognised slot's number strip from stills, for the number OCR experiments.
//   cargo run --release --bin number_dump -- <out dir> <image>...
use hf_scanner::scanner_state::ScannerState;
use std::{env, fs};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let out = &args[0];
    fs::create_dir_all(out).unwrap();
    let mut manifest = String::new();
    for (index, path) in args[1..].iter().enumerate() {
        let image = image::open(path).unwrap().to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let mut pixels = image.into_raw();
        let mut state = ScannerState::default();
        let ui_height = (height as f64 / 360.0).round() as u32 * 360;
        let forced = path.contains("21 by 9");
        state.screen_info.game_width = if forced {
            width as u32
        } else {
            ui_height * 16 / 9
        };
        state.screen_info.game_height = ui_height;
        state.screen_info.forced_21_9 = forced;
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
        state.cropper();
        state.cropper();
        let mut slots: Vec<_> = state.slot_infos.iter().collect();
        slots.sort_by_key(|(a, _)| (format!("{:?}", a.inventory_type), a.page_num, a.pos_in_inv));
        for (address, info) in slots {
            let name = &info.icon_name_score.as_ref().unwrap().0;
            let id = format!(
                "{index:02}_{:?}_{}_{}_{}",
                address.inventory_type,
                address.page_num,
                address.pos_in_inv.0,
                address.pos_in_inv.1
            );
            info.observed_number
                .data
                .save(format!("{out}/{id}.png"))
                .unwrap();
            info.observed_icon
                .data
                .save(format!("{out}/{id}_icon.png"))
                .unwrap();
            manifest += &format!(
                "{id}\t{name}\t{}\t{}\t{}\t{path}\n",
                state.screen_info.brightness.unwrap(),
                state.screen_info.effective_height,
                info.amount.clone().unwrap_or_default().trim()
            );
        }
    }
    fs::write(format!("{out}/manifest.tsv"), manifest).unwrap();
}
