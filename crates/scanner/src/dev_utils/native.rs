// What the native bins share: the config and the recogniser from public/, and a state over a
// frame buffer the bin owns.

use std::{fs, sync::Once};

use crate::{
    buffer::Buffer,
    interface::setup::{load_ocr_engine, set_config},
    scanner_state::ScannerState,
    tooltip::{chest::Chest, hover::Hover},
};

pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

pub fn load() {
    static LOADED: Once = Once::new();
    LOADED.call_once(|| {
        let config = fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap();
        set_config(rmp_serde::from_slice(&config).unwrap());
        load_ocr_engine(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
    });
}

// a state and the frame it scans, to be filled by the caller; 21:9 and brightness are detected
pub fn new_state(width: usize, height: usize) -> (ScannerState, Vec<u8>) {
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
    (
        ScannerState::new(buffer, Default::default(), Default::default()),
        pixels,
    )
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
        .map(|x| format!("{} x{}", x.item, x.amount))
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
        if chest.irrelevant {
            "irrelevant".to_string()
        } else {
            contents.join(", ")
        },
        chest.amount.clone().unwrap_or_default(),
        chest.tradability,
        chest.title,
        chest.variants,
        chest.icons
    )
}

// what the page would colour each known slot, and the currencies
pub fn print_statuses(state: &ScannerState) {
    for currency in state.currency_results() {
        println!(
            "  {:?}  {:?}  value {:?}  bands {:?}  {}",
            currency.currency, currency.status, currency.value, currency.bands, currency.reason
        );
    }
    let mut fields: Vec<String> = state
        .currency
        .fields
        .iter()
        .map(|(field, reads)| format!("    {field:?}: {:?} of {} sent", reads.votes, reads.sent))
        .collect();
    fields.sort();
    println!("{}", fields.join("\n"));
    let mut slots = state.result(true).slots;
    slots.sort_by_key(|slot| {
        (
            slot.address.inventory_type as u8,
            slot.address.page_num,
            slot.address.pos_in_inv,
        )
    });
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
                        state.slot_infos[&a].levelled(),
                        slot.alternatives,
                        slot.variants
                    )
                }
            );
        }
    }
}

// At the end of the input: every line still waiting is read, and what that sends off too.
pub fn read_rest(state: &mut ScannerState) {
    while !state.ocr_queue.is_empty() {
        state.run_ocr_inline();
    }
}

// The slots set by hand in a capture's end record, the only right answers it has, against what
// the scanner read there. A slot is right when it is in order (Good, or only its tradability
// missing, which the page may assume) with the icon, amount and tradability that were set; no
// amount set is a single item, no tradability set is not checked. Prints each slot and returns
// how many were right, of how many.
pub fn check_edits(state: &ScannerState, end: &str) -> (usize, usize) {
    use crate::interface::scan_result::SlotStatus;
    let end: serde_json::Value = serde_json::from_str(end).unwrap();
    let slots = state.result(true).slots;
    let edits = end["edits"].as_object().cloned().unwrap_or_default();
    let mut right = 0;
    for (key, edit) in &edits {
        let item = edit["item"].as_str();
        let amount = edit["amount"].as_u64().unwrap_or(1) as u32;
        let tradability = edit["tradability"].as_str();
        let set = format!(
            "{} x{amount} {}",
            item.unwrap_or("?"),
            tradability.unwrap_or("-")
        );
        let slot = slots
            .iter()
            .find(|x| serde_json::to_value(x.address).unwrap() == edit["address"]);
        let Some(slot) = slot else {
            println!("  {key}: set {set}, not read");
            continue;
        };
        let icon = slot.icon_name_score.as_ref().map(|x| x.0.as_str());
        let is_item = icon == item
            || slot.alternatives.iter().any(|x| Some(x.as_str()) == item)
            || slot.label.as_deref() == item;
        let read_tradability = slot.tradability.map(|x| format!("{x:?}"));
        let ok = is_item
            && slot.value == Some(amount)
            && (tradability.is_none() || read_tradability.as_deref() == tradability)
            && matches!(slot.status, SlotStatus::Good | SlotStatus::NeedTradability);
        right += ok as usize;
        println!(
            "  {key}: {} set {set}, read {} x{:?} {:?} {:?} {}",
            if ok { "right" } else { "WRONG" },
            icon.unwrap_or("nothing"),
            slot.value,
            slot.tradability,
            slot.status,
            slot.reason
        );
    }
    (right, edits.len())
}
