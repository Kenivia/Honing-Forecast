// What the native bins share: the config and the recogniser from public/, and a state over a
// frame buffer the bin owns.
use crate::{
    buffer::Buffer,
    scanner_state::ScannerState,
    ocr::recognize::load_ocr_engine,
    setup::set_config,
    tooltip::{chest::Chest, hover::Hover},
};
use std::{fs, sync::Once};

pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

pub fn load() {
    static LOADED: Once = Once::new();
    LOADED.call_once(|| {
        let config = fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap();
        set_config(rmp_serde::from_slice(&config).unwrap());
        load_ocr_engine(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
    });
}

// The game resolution of a capture, where nothing says: 1078-tall recordings are still 1080p,
// and a native ultrawide capture is as wide as the game.
pub fn guess_game(width: usize, height: usize) -> (u32, u32) {
    let ui_height = (height as f64 / 360.0).round() as u32 * 360;
    ((ui_height * 16 / 9).max(width as u32 / 100 * 100), ui_height)
}

// a state and the frame it scans, to be filled by the caller
pub fn new_state(
    width: usize,
    height: usize,
    game: (u32, u32),
    forced_21_9: bool,
) -> (ScannerState, Vec<u8>) {
    load();
    let mut pixels = vec![0u8; width * height * 4];
    let buffer = Buffer {
        pointer: Some(pixels.as_mut_ptr() as usize),
        width,
        height,
        size: pixels.len(),
        lut: None,
        brightness: 0.0,
    };
    (ScannerState::new(buffer, game.0, game.1, forced_21_9), pixels)
}

// what the bins print of a hover, a chest and the slots
pub fn describe(state: &ScannerState, hover: &Hover) -> String {
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

pub fn describe_chest(chest: &Chest) -> String {
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
pub fn print_statuses(state: &ScannerState) {
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
