// Debug: for every slot of a still that looks like something, the icons it is closest to.
//   cargo run --release --bin slot_scores -- <image>...
use hf_scanner::{constants::ALL_SLOT_ADDRESSS, scanner_state::ScannerState};
use std::{env, fs};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn main() {
    for path in env::args().skip(1) {
        let image = image::open(&path).unwrap().to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let mut state = ScannerState::default();
        let mut pixels = image.into_raw();
        let ui_height = (height as f64 / 360.0).round() as u32 * 360;
        state.screen_info.game_width = (ui_height * 16 / 9).max(width as u32 / 100 * 100);
        state.screen_info.game_height = ui_height;
        state.buffer.width = width;
        state.buffer.height = height;
        state.buffer.size = pixels.len();
        state.buffer.pointer = Some(pixels.as_mut_ptr() as usize);
        state.config = rmp_serde::from_slice(&fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap()).unwrap();
        state.model = Some(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
        state.set_config();
        state.set_ocr_engine();
        state.initialize_anchors();
        state.initialize_page_num_infos();
        for _ in 0..30 {
            state.cropper();
            state.run_ocr_inline();
        }
        println!("{path}  brightness {:?}", state.screen_info.brightness);
        for inventory in [hf_scanner::scanner_state::InventoryType::Roster, hf_scanner::scanner_state::InventoryType::CharStorage, hf_scanner::scanner_state::InventoryType::CharInventory] {
            println!("  root of {inventory:?}: {:?}", state.inventory_root(inventory).map(|x| x.top_left));
        }
        let pages = state.active_page_num();
        let mut addresses: Vec<_> = ALL_SLOT_ADDRESSS
            .keys()
            .filter(|a| pages[&a.inventory_type] == Some(a.page_num) && state.inventory_root(a.inventory_type).is_some())
            .copied()
            .collect();
        addresses.sort_by_key(|a| (format!("{:?}", a.inventory_type), a.pos_in_inv));
        for address in addresses {
            let scores = state.slot_scores(&address);
            if scores[0].1.max(scores[0].2) < 0.9 {
                continue;
            }
            let known = state.slot_infos.get(&address).and_then(|x| x.icon_name_score.clone());
            let top: Vec<String> = scores.iter().map(|x| format!("{} {:.3}/{:.3} #{}", x.0, x.1, x.2, x.3)).collect();
            println!("  {:?} p{} {:?}  read as {:?}  closest: {}", address.inventory_type, address.page_num, address.pos_in_inv, known.map(|x| x.0), top.join(" | "));
        }
    }
}
