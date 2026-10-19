// For trying other ways of telling a slot's icon: every distinct look of every slot of a debug
// capture, as the scanner compares it, with the templates at that resolution.
//   cargo run --release --bin slot_dump -- <file.hfcap> <folder>
// Writes <folder>/reference.json (the page's end record), index.jsonl (one line per look: slot,
// first scan, scans it was seen on, on how many of them a tooltip was over or beside it), per look the normalised icon crop
// (_i), the same with MARGIN px around it (_m) and the whole slot as captured (_r), and the
// templates in <folder>/../templates-<ui height>.
use hf_scanner::{
    anchor_to_slot::anchor_defs::{COMBINED_NUMBER_HEIGHT, SLOT_ORDER},
    dev_utils::capture::read_frames,
    dev_utils::native,
    image_utils::{
        brightness::normalize_brightness,
        common::{Rect, get_resizer},
    },
    interface::setup::{BASE_ICONS, icon_lookup},
};
use std::{collections::HashMap, env, fs};

const MARGIN: f64 = 3.0;
const MAX_LOOKS: usize = 40;
const STAYS: usize = 3;

struct Look {
    first: u64,
    scans: usize,
    hovered: usize,
    file: String,
    brightness: f64,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (path, out) = (&args[0], &args[1]);
    fs::create_dir_all(out).unwrap();
    let file = fs::read(path).unwrap();
    let (start, frames, end) = read_frames(&file);
    if let Some(end) = end {
        fs::write(format!("{out}/reference.json"), end).unwrap();
    }
    let (mut state, _pixels) = native::new_state(start.width, start.height);
    state.screen_info.forced_21_9 = start.forced_21_9;
    state.screen_info.brightness = start.brightness;

    let mut looks: HashMap<String, HashMap<u64, Look>> = HashMap::new();
    for (input, trace) in frames {
        let index = state.replayed;
        state.replay(input, Some(trace), vec![]);
        let scanned = state.scanned;
        state.clear_changed();
        let Some(brightness) = state.screen_info.brightness.value.filter(|_| scanned) else {
            continue;
        };
        let s = state.screen_info.scale_factor;
        // under the tooltip, which may run to the bottom of the screen, or in the column it is of
        let hover = state
            .hover
            .as_ref()
            .map(|x| (x.bar, x.candidates.iter().map(|x| x.0).collect::<Vec<_>>()));
        let pages = state.active_page_num();
        for address in SLOT_ORDER.iter() {
            if pages[&address.inventory_type] != Some(address.page_num) {
                continue;
            }
            let Some(position) = state.anchored_slot_address_position(address) else {
                continue;
            };
            let (left, top) = position.top_left;
            let number = COMBINED_NUMBER_HEIGHT * s;
            let (x0, y0) = (
                left.floor() as usize - 4,
                (top - number).floor() as usize - 4,
            );
            let (x1, y1) = (
                (left + position.width).ceil() as usize + 4,
                (top + position.height).ceil() as usize + 4,
            );
            if x1 > start.width || y1 > start.height {
                continue;
            }
            // of the icon alone, as the scanner tells a slot has not changed: the frame around it
            // lets the world behind through
            let hash = state.buffer.hash(
                left as usize,
                top as usize,
                (left + position.width).ceil() as usize,
                (top + position.height).ceil() as usize,
            );
            let hovered = hover.as_ref().is_some_and(|(bar, beside)| {
                beside.contains(address)
                    || (x1 + 8 > bar.x && x0 < bar.x + bar.width + 8 && y1 > bar.y)
            });
            let (row, column) = address.pos_in_inv;
            let key = format!(
                "{:?} {} {row},{column}",
                address.inventory_type, address.page_num
            );
            let slot = looks.entry(key.clone()).or_default();
            let saved = slot.values().filter(|x| !x.file.is_empty()).count();
            let look = slot.entry(hash).or_insert(Look {
                first: index,
                scans: 0,
                hovered: 0,
                file: String::new(),
                brightness,
            });
            look.scans += 1;
            look.hovered += hovered as usize;
            // With a tooltip about, only a look that stays. A clean one may be seen once: a still
            // screen is not scanned.
            let due = if hovered {
                look.scans == STAYS
            } else {
                look.file.is_empty()
            };
            if !due || !look.file.is_empty() || saved >= MAX_LOOKS {
                continue;
            }
            let name = format!("{}_{}", key.replace([' ', ','], "_"), saved);
            let mut icon = state.crop_buffer(position);
            normalize_brightness(&mut icon, brightness);
            icon.data.save(format!("{out}/{name}_i.png")).unwrap();
            let mut wide = state.crop_buffer(Rect {
                top_left: (left - MARGIN, top - MARGIN),
                width: position.width + 2.0 * MARGIN,
                height: position.height + 2.0 * MARGIN,
                ..position
            });
            normalize_brightness(&mut wide, brightness);
            wide.data.save(format!("{out}/{name}_m.png")).unwrap();
            state
                .buffer
                .raw_crop(x0, y0, x1, y1)
                .save(format!("{out}/{name}_r.png"))
                .unwrap();
            let look = looks.get_mut(&key).unwrap().get_mut(&hash).unwrap();
            look.file = name;
            look.brightness = brightness;
        }
    }

    let height = state.screen_info.effective_height;
    let templates = format!("{out}/../templates-{height}");
    fs::create_dir_all(&templates).unwrap();
    let names: Vec<String> = BASE_ICONS
        .read()
        .iter()
        .filter(|x| x.1.is_slot_icon())
        .map(|x| x.0.clone())
        .collect();
    for name in names {
        let icon = icon_lookup(&name, height, get_resizer(&mut state.resizer));
        icon.data.save(format!("{templates}/{name}.png")).unwrap();
    }

    let mut lines = vec![];
    for (slot, by_hash) in looks {
        for look in by_hash.values().filter(|x| !x.file.is_empty()) {
            lines.push(format!(
                r#"{{"slot":"{slot}","file":"{}","first":{},"scans":{},"hovered":{},"brightness":{},"ui_height":{height}}}"#,
                look.file, look.first, look.scans, look.hovered, look.brightness
            ));
        }
    }
    lines.sort();
    fs::write(format!("{out}/index.jsonl"), lines.join("\n")).unwrap();
    println!("{} looks of {out}", lines.len());
}
