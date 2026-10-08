// TEMPORARY debug harness: pages and slot recognitions per frame. Delete when done.
use hf_scanner::{
    constants::{ALL_PAGE_NUM, ALL_SLOT_ADDRESSS},
    image_utils::{
        close_enough::close_enough,
        common::{Rectangle, get_resizer},
        resize::crop_buffer,
    },
    scanner_state::{InventoryType, ScannerState, SlotAddress},
    setup::{BASE_ICONS, icon_lookup},
};
use std::{collections::HashMap, env, fs, io::Read};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (width, height): (usize, usize) = (args[0].parse().unwrap(), args[1].parse().unwrap());
    let dump: Vec<usize> = args[2..].iter().map(|x| x.parse().unwrap()).collect();
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
    state.set_config();
    state.initialize_anchors();
    state.initialize_page_num_infos();

    let mut stdin = std::io::stdin().lock();
    let mut index = 0;
    let mut pages = String::new();
    let mut known: HashMap<SlotAddress, String> = HashMap::new();
    while stdin.read_exact(&mut pixels).is_ok() {
        let scan = state.worth_scanning();
        state.update_scale();
        if scan {
            state.update_anchors();
            state.update_page_status();
            let mut now = vec![];
            for inv in [InventoryType::Roster, InventoryType::CharStorage, InventoryType::CharInventory] {
                now.push(format!("{inv:?} {:?} -> {:?}", state.page_num_infos[&inv], state.active_page_num()[&inv]));
            }
            let now = now.join(" | ");
            if now != pages {
                println!("{index}: {now}");
                pages = now;
            }
            state.update_slots();
            state.ocr_queue.clear();
            for (address, info) in &state.slot_infos {
                let name = info.icon_name_score.as_ref().map(|x| x.0.clone()).unwrap_or_default();
                if known.get(address) != Some(&name) {
                    if !name.is_empty() {
                        println!(
                            "{index}:   slot {:?} p{} {:?} = {name} {:.3}",
                            address.inventory_type, address.page_num, address.pos_in_inv,
                            info.icon_name_score.as_ref().unwrap().1
                        );
                    }
                    known.insert(*address, name);
                }
            }
        }
        if dump.contains(&index) {
            let brightness = state.screen_info.brightness.unwrap();
            // unthresholded tab scores
            for (inv, names) in ALL_PAGE_NUM.iter() {
                let Some(root) = state.inventory_root(*inv) else { continue };
                for (active, inactive) in names {
                    for name in [active, inactive] {
                        let mut icon = icon_lookup(name, state.screen_info.effective_height, get_resizer(&mut state.resizer)).clone();
                        icon.required_confidence = Some(0.0);
                        let mut seen = crop_buffer(icon.offset.use_root(&root), get_resizer(&mut state.resizer), state.buffer, None);
                        println!("{index}: tab {name}: {:?}", close_enough(&icon, &mut seen, brightness));
                    }
                }
            }
            // best unthresholded icon for every slot of the active pages
            let active = state.active_page_num();
            let names: Vec<String> = BASE_ICONS.read().iter().filter(|x| x.1.tag == "Icon").map(|x| x.0.clone()).collect();
            let mut lines = vec![];
            for address in ALL_SLOT_ADDRESSS.keys() {
                if active[&address.inventory_type] != Some(address.page_num) { continue }
                let Some(position) = state.anchored_slot_address_position(address) else { continue };
                let mut best = (0.0, String::new());
                for name in &names {
                    let mut icon = icon_lookup(name, state.screen_info.effective_height, get_resizer(&mut state.resizer)).clone();
                    icon.required_confidence = Some(0.0);
                    let mut seen = crop_buffer(position, get_resizer(&mut state.resizer), state.buffer, None);
                    if let Some(score) = close_enough(&icon, &mut seen, brightness) {
                        if score > best.0 { best = (score, name.clone()) }
                    }
                }
                if best.0 > 0.85 {
                    lines.push(format!("{index}: best {:?} p{} {:?} {} {:.3}", address.inventory_type, address.page_num, address.pos_in_inv, best.1, best.0));
                }
            }
            lines.sort();
            println!("{}", lines.join("\n"));
        }
        index += 1;
    }
}
