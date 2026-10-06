// Prints the template alignment of every recognised slot, with its grid position, to see whether
// the offset follows a pattern instead of needing a search per slot.
//   cargo run --release --bin align_map -- <image>...
use hf_scanner::{
    image_utils::{
        number::align,
        ocr::{pre_process_icon_number, recognize_line},
    },
    scanner_state::ScannerState,
    setup::BASE_ICONS,
};
use std::{env, fs};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn main() {
    println!("file\tinventory\tpage\trow\tcol\tnum_x\tnum_y\ticon\tshift_x\tshift_y\tread");
    for path in env::args().skip(1) {
        let image = image::open(&path).unwrap().to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let mut pixels = image.into_raw();
        let mut state = ScannerState::default();
        let ui_height = (height as f64 / 360.0).round() as u32 * 360;
        let forced = path.contains("21 by 9");
        state.screen_info.game_width =
            if forced { width as u32 } else { ui_height * 16 / 9 };
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
        for _ in 0..40 {
            state.cropper();
            state.ocr_queue.clear();
        }
        let name = path.split(['/', '\\']).last().unwrap().replace(' ', "_");
        let icons = BASE_ICONS.read();
        let mut slots: Vec<_> = state.slot_infos.iter().collect();
        slots.sort_by_key(|(a, _)| {
            (format!("{:?}", a.inventory_type), a.page_num, a.pos_in_inv)
        });
        for (address, info) in slots {
            let Some((icon, _)) = &info.icon_name_score else {
                continue;
            };
            let shift = align(&icons[icon].data, &info.observed_icon.data);
            // the strip the recogniser would get, to see whether the shift changes the read
            let processed = pre_process_icon_number(
                info.observed_number.clone(),
                &info.observed_icon.data,
                &icons[icon].data,
                state.screen_info.brightness.unwrap(),
            );
            let read: String =
                recognize_line(&processed.data).chars().filter(char::is_ascii_digit).collect();
            println!(
                "{name}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{icon}\t{}\t{}\t{read}",
                address.inventory_type,
                address.page_num,
                address.pos_in_inv.1,
                address.pos_in_inv.0,
                info.observed_number.offset.top_left.0,
                info.observed_number.offset.top_left.1,
                shift.0,
                shift.1
            );
        }
    }
}
