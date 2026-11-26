// Holds the last export's priors against debug captures. Each capture is replayed as it was, then
// twice more with what an export at its end would save as priors:
//   no tooltips   how many slots the priors alone put in order, and whether as the first run had them
//   flipped       every prior's tradability turned, tooltips on: a hovered slot keeps its tooltip's
//   cargo run --release --bin prior_test -- <file.hfcap>...
use clap::Parser;
use hf_scanner::{
    dev_utils::{
        capture::read_frames,
        native::{self, Options, check_edits, priors_of},
    },
    interface::scan_result::SlotResult,
    scanner_state::{Prior, ScannerState, Tradability},
};
use std::{collections::BTreeMap, fs};

#[derive(Parser)]
struct Cli {
    captures: Vec<String>,
    #[command(flatten)]
    options: Options,
}

fn run(file: &[u8], priors: Vec<Prior>, no_tooltips: bool) -> (ScannerState, Vec<u8>) {
    let (mut start, frames, _) = read_frames(file);
    let (mut state, pixels) = native::replay_state(&mut start);
    state.set_priors(priors);
    state.no_tooltips = no_tooltips;
    for input in frames {
        state.replay(input, vec![]);
        let texts: Vec<(u32, String)> = std::mem::take(&mut state.ocr_queue)
            .iter()
            .map(|job| (job.id, job.line.read()))
            .collect();
        state.apply_ocr(texts);
        state.clear_changed();
    }
    native::read_rest(&mut state);
    (state, pixels)
}

fn counts(slots: &[SlotResult]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for slot in slots {
        *out.entry(format!("{:?}", slot.status)).or_default() += 1;
    }
    out
}

// what the manifest takes from a slot
fn reading(slot: &SlotResult) -> String {
    format!(
        "{:?} {:?} x{:?} {:?} {:?} {:?}",
        slot.status, slot.item, slot.value, slot.tradability, slot.label, slot.variants
    )
}

fn flip(x: Tradability) -> Tradability {
    match x {
        Tradability::Tradable => Tradability::CharBound,
        Tradability::RosterBound => Tradability::Tradable,
        Tradability::CharBound => Tradability::RosterBound,
    }
}

fn main() {
    let cli = Cli::parse();
    native::configure(&cli.options);
    for path in cli.captures {
        println!("== {path}");
        let file = fs::read(&path).unwrap();
        let end = read_frames(&file).2.unwrap_or("{}".into());

        let (first, _a) = run(&file, vec![], false);
        let base = first.result(true).slots;
        println!("as recorded: {:?}", counts(&base));
        let priors = priors_of(&first, &end);
        println!("{} priors", priors.len());

        let (bare, _b) = run(&file, priors.clone(), true);
        let slots = bare.result(true).slots;
        let assumed = slots.iter().filter(|x| x.assumed).count();
        println!("no tooltips: {:?}, {assumed} assumed", counts(&slots));
        let mut differ = 0;
        for slot in &slots {
            let Some(was) = base.iter().find(|x| x.address == slot.address) else {
                continue;
            };
            let ordered = |x: &SlotResult| format!("{:?}", x.status) == "Good";
            if ordered(slot) && reading(slot) != reading(was) {
                differ += 1;
                println!(
                    "  differs {:?}\n    was {}\n    now {}",
                    slot.address,
                    reading(was),
                    reading(slot)
                );
            }
        }
        println!("no tooltips: {differ} slots in order read otherwise than as recorded");
        for slot in &slots {
            let has_prior = bare.priors.contains_key(&slot.address);
            if has_prior && format!("{:?}", slot.status) == "NeedHover" {
                println!(
                    "  still {:?} {:?}: {}",
                    slot.address, slot.icon_name_score, slot.reason
                );
            }
        }
        let (right, all, lines) = check_edits(&bare, &end);
        println!(
            "{}",
            lines.join(
                "
"
            )
        );
        println!("no tooltips check: {right} of {all} slots set by hand read right");

        let flipped: Vec<Prior> = priors
            .iter()
            .map(|x| Prior {
                tradability: x.tradability.map(flip),
                ..x.clone()
            })
            .collect();
        let (turned, _c) = run(&file, flipped, false);
        let slots = turned.result(true).slots;
        let mut kept = 0;
        let mut overruled = 0;
        for slot in &slots {
            if !turned.slot_infos[&slot.address].hovered {
                continue;
            }
            let Some(was) = base.iter().find(|x| x.address == slot.address) else {
                continue;
            };
            if slot.tradability == was.tradability {
                kept += 1;
            } else {
                overruled += 1;
                println!(
                    "  prior beat tooltip {:?}: was {:?} now {:?}",
                    slot.address, was.tradability, slot.tradability
                );
            }
        }
        println!(
            "flipped: {kept} hovered slots kept their tooltip's tradability, {overruled} did not"
        );
    }
}
