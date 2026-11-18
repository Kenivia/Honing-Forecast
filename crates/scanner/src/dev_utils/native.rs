// What the native bins share: the config and the recogniser from public/, and a state over a
// frame buffer the bin owns.

use std::{collections::BTreeMap, fs, sync::Once};

use crate::{
    buffer::Buffer,
    interface::{
        scan_result::{SlotResult, SlotStatus},
        setup::{load_ocr_engine, set_config},
    },
    ocr::shapes::load_number_font,
    scanner_state::{Prior, ScannerState, SlotAddress},
    tooltip::{chest::Chest, hover::Hover},
};

pub const ROOT: &str = env!("HF_ROOT");

pub fn load() {
    static LOADED: Once = Once::new();
    LOADED.call_once(|| {
        let config = fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap();
        set_config(rmp_serde::from_slice(&config).unwrap());
        load_ocr_engine(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
        // TEXT_STRIP="levels=60,200" makes tooltip and currency strips another way (TextParams)
        if let Ok(spec) = std::env::var("TEXT_STRIP") {
            *crate::ocr::text::TEXT_PARAMS.write() = crate::ocr::text::TextParams::parse(&spec);
        }
        // NO_SHAPES=1 reads every number by the OCR
        if std::env::var("NO_SHAPES").is_err() {
            load_number_font(fs::read(format!("{ROOT}/public/fonts/English.ttf")).unwrap());
        }
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
// missing) with the icon, amount and tradability that were set; no
// amount set is a single item, no tradability set is not checked. A slot set with no item is right
// when it is irrelevant. Prints each slot and returns how many were right, of how many.
pub fn check_edits(state: &ScannerState, end: &str) -> (usize, usize) {
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
            || slot.label.map(|x| x.as_str()) == item
            || slot.item.map(|x| x.as_str()) == item;
        let read_tradability = slot.tradability.map(|x| format!("{x:?}"));
        let ok = match item {
            None => matches!(slot.status, SlotStatus::Irrelevant),
            Some(_) => {
                is_item
                    && slot.value == Some(amount)
                    && (tradability.is_none() || read_tradability.as_deref() == tradability)
                    && matches!(slot.status, SlotStatus::Good | SlotStatus::NeedTradability)
            }
        };
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

// The chest an export takes a slot for: the one read in it, else one read in its column with
// its icon, the one with its amount first.
fn chest_read(state: &ScannerState, slot: &SlotResult) -> Option<Vec<u32>> {
    let a = slot.address;
    let info = &state.slot_infos[&a];
    let column = (a.inventory_type, a.page_num, a.pos_in_inv.1);
    let exact: Vec<&Chest> = state.chests.iter().filter(|x| x.slot == Some(a)).collect();
    let found = if exact.is_empty() {
        state
            .chests
            .iter()
            .filter(|x| x.column == Some(column) && info.shows(&x.icons))
            .collect()
    } else {
        exact
    };
    let digits = |x: &String| {
        x.chars()
            .filter(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .ok()
    };
    let read = found
        .iter()
        .find(|x| x.amount.as_ref().and_then(digits) == slot.value)
        .or(found.first());
    read.map(|x| x.variants.clone())
}

// What an export at the end would save, as priors: the slots in order, with the end record's
// hand edits over them.
pub fn priors_of(state: &ScannerState, end: &str) -> Vec<Prior> {
    let end: serde_json::Value = serde_json::from_str(end).unwrap();
    let slots = state.result(true).slots;
    let mut out: BTreeMap<String, Prior> = BTreeMap::new();
    let key = |a: &SlotAddress| format!("{a:?}");
    for slot in &slots {
        let Some((icon, _)) = slot.icon_name_score else {
            continue;
        };
        if !matches!(slot.status, SlotStatus::Good | SlotStatus::Irrelevant) {
            continue;
        }
        let prior = Prior {
            address: slot.address,
            icon: icon.clone(),
            item: slot.item.cloned(),
            label: slot.label.cloned(),
            tradability: slot.tradability,
            variants: chest_read(state, slot).unwrap_or(slot.variants.clone()),
        };
        out.insert(key(&slot.address), prior);
    }
    for edit in end["edits"]
        .as_object()
        .into_iter()
        .flat_map(|x| x.values())
    {
        let address: SlotAddress = serde_json::from_value(edit["address"].clone()).unwrap();
        let Some(slot) = slots.iter().find(|x| x.address == address) else {
            continue;
        };
        let Some((icon, _)) = slot.icon_name_score else {
            continue;
        };
        let prior = Prior {
            address,
            icon: icon.clone(),
            item: edit["item"].as_str().map(String::from),
            label: None,
            tradability: serde_json::from_value(edit["tradability"].clone()).unwrap(),
            variants: chest_read(state, slot).unwrap_or(slot.variants.clone()),
        };
        out.insert(key(&address), prior);
    }
    out.into_values().collect()
}
