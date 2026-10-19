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

// OCR as slow as in the browser, for the native bins: only so many px of strip are read per scan,
// in the order the page's workers take them, and a text is back with the scan after it was read.
// OCR_PX=<px per scan>; without it every line is read at once.
pub struct Sim {
    pub capacity: Option<f64>,
    credit: f64,
    backlog: Vec<(crate::ocr::jobs::OcrJob, u64)>,
    ready: Vec<(u32, String)>,
    kinds: std::collections::HashMap<u32, &'static str>,
    // per kind: lines and strip px
    pub sent: std::collections::BTreeMap<&'static str, (usize, f64)>,
    pub peak: usize,
    busy: usize,
    scans: u64,
    waits: Vec<u64>,
    seen: std::collections::HashMap<u32, (u64, u64)>,
    // per hover that wrote something: scans from first seen to its last text in
    done: Vec<u64>,
    idle: usize,
    // hovers that ended since the caller last took them
    pub ended: Vec<Hover>,
    begun: std::collections::HashSet<u32>,
    // per hover: the scan it first wrote a slot or a chest on
    wrote: std::collections::HashMap<u32, u64>,
}

impl Sim {
    pub fn new() -> Sim {
        let capacity = std::env::var("OCR_PX").ok().map(|x| x.parse().unwrap());
        Sim {
            capacity,
            credit: 0.0,
            backlog: vec![],
            ready: vec![],
            kinds: Default::default(),
            sent: Default::default(),
            peak: 0,
            busy: 0,
            scans: 0,
            waits: vec![],
            seen: Default::default(),
            done: vec![],
            idle: 0,
            ended: vec![],
            begun: Default::default(),
            wrote: Default::default(),
        }
    }

    // texts read since the last call, for the scan that is about to run
    pub fn take_ready(&mut self) -> Vec<(u32, String)> {
        std::mem::take(&mut self.ready)
    }

    fn read(&mut self, job: crate::ocr::jobs::OcrJob, queued: u64, scan: u64) -> f64 {
        if job.group != 0 {
            self.begun.insert(job.group);
        }
        let strip = job.line.strip();
        let px = strip.width().next_multiple_of(50) as f64;
        let numbers = job.line.digits_only();
        let text = crate::ocr::recognize::recognize_line(&strip, numbers);
        let entry = self
            .sent
            .entry(self.kinds.get(&job.id).copied().unwrap_or("number"))
            .or_default();
        entry.0 += 1;
        entry.1 += px;
        self.waits.push(scan - queued);
        self.ready.push((job.id, text));
        px
    }

    // after a scan: takes the lines it asked for and reads what there is time for
    pub fn step(&mut self, state: &mut ScannerState, scan: u64) {
        self.scans += 1;
        for read in state.pending_reads.iter() {
            let kinds = [
                ("title", read.title.as_ref().map(|x| x.0.clone())),
                ("amount", read.amount.map(|x| x.to_vec())),
                (
                    "chest",
                    read.chest.as_ref().map(|x| {
                        x.iter()
                            .flat_map(|x| x.0.iter().copied().chain([x.1]))
                            .collect()
                    }),
                ),
                ("description", read.body.clone()),
            ];
            for (kind, jobs) in kinds {
                for id in jobs.into_iter().flatten() {
                    self.kinds.entry(id).or_insert(kind);
                }
            }
        }
        for &id in state.currency.fields.values().flat_map(|x| &x.waiting) {
            self.kinds.entry(id).or_insert("currency");
        }
        if let Some(hover) = &state.hover {
            self.seen.entry(hover.id).or_insert((scan, scan)).1 = scan;
        }
        for hover in state.hover.iter().chain(&state.past_hovers) {
            if hover.slot.is_some() || hover.chest_index.is_some() {
                self.wrote.entry(hover.id).or_insert(scan);
            }
        }
        for job in state.ocr_queue.drain(..) {
            // a line that comes again is wanted sooner than it was
            match self.backlog.iter_mut().find(|(old, _)| old.id == job.id) {
                Some((old, _)) => (old.priority, old.group) = (job.priority, job.group),
                None => self.backlog.push((job, scan)),
            }
        }
        // as the page's workers take them, see OcrRelay.ts
        let begun = &self.begun;
        self.backlog.sort_by_key(|(job, _)| {
            (
                job.priority,
                !begun.contains(&job.group),
                std::cmp::Reverse(job.group),
                job.id,
            )
        });
        self.peak = self.peak.max(self.backlog.len());
        self.busy += (self.backlog.len() > 30) as usize;
        for hover in std::mem::take(state.hover_log.get_or_insert_default()) {
            self.note(&hover, scan);
            self.ended.push(hover);
        }
        self.idle += self.backlog.is_empty() as usize;
        match self.capacity {
            None => {
                for (job, queued) in std::mem::take(&mut self.backlog) {
                    self.read(job, queued, scan);
                }
                let ready = self.take_ready();
                state.apply_ocr(ready);
                for hover in std::mem::take(state.hover_log.get_or_insert_default()) {
                    self.note(&hover, scan);
                    self.ended.push(hover);
                }
            }
            Some(capacity) => {
                self.credit = if self.backlog.is_empty() {
                    0.0
                } else {
                    self.credit + capacity
                };
                while self.credit > 0.0 && !self.backlog.is_empty() {
                    let (job, queued) = self.backlog.remove(0);
                    self.credit -= self.read(job, queued, scan);
                }
            }
        }
    }

    // a hover that is over with its last texts in
    fn note(&mut self, hover: &Hover, scan: u64) {
        if hover.slot.is_some() || hover.chest_index.is_some() {
            let first = self.seen.get(&hover.id).map_or(scan, |x| x.0);
            let wrote = *self.wrote.entry(hover.id).or_insert(scan);
            self.done.push(wrote.saturating_sub(first));
        }
    }

    // at the end of the input: everything still waiting is read
    pub fn finish(&mut self, state: &mut ScannerState, scan: u64) {
        loop {
            for job in state.ocr_queue.drain(..) {
                self.backlog.push((job, scan));
            }
            if self.backlog.is_empty() && self.ready.is_empty() {
                break;
            }
            for (job, queued) in std::mem::take(&mut self.backlog) {
                self.read(job, queued, scan);
            }
            let ready = self.take_ready();
            state.apply_ocr(ready);
            for hover in std::mem::take(state.hover_log.get_or_insert_default()) {
                self.note(&hover, scan);
                self.ended.push(hover);
            }
        }
    }

    pub fn summary(&mut self) -> String {
        let pct = |list: &mut Vec<u64>| {
            list.sort();
            if list.is_empty() {
                return "-".to_string();
            }
            format!(
                "{}/{}/{}",
                list[list.len() / 2],
                list[list.len() * 9 / 10],
                list[list.len() - 1]
            )
        };
        let (lines, px) = self
            .sent
            .values()
            .fold((0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
        let kinds: Vec<String> = self
            .sent
            .iter()
            .map(|(kind, (lines, px))| format!("{kind} {lines}/{:.0}k", px / 1000.0))
            .collect();
        format!(
            "sim: capacity {:?} scans {} lines {lines} px {:.0}k [{}] peak {} over30 {:.0}% idle {:.0}% line wait med/p90/max {} useful hovers {} wrote after {}",
            self.capacity,
            self.scans,
            px / 1000.0,
            kinds.join(", "),
            self.peak,
            100.0 * self.busy as f64 / self.scans.max(1) as f64,
            100.0 * self.idle as f64 / self.scans.max(1) as f64,
            pct(&mut self.waits),
            self.done.len(),
            pct(&mut self.done),
        )
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
