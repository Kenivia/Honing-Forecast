// Runs a debug capture from the page again, frame for frame, and says what went on in it.
//   cargo run --release --bin capture_replay -- <file.hfcap> [options]
// Always: whether every scan ended as recorded (as it is run on its trace, unless --reread), each
// hover once its last texts were in (the scans it was seen on, what it read and every vote), the
// chests and the slots at the end.
//   --reference <file>      writes the page's own account of the end, corrections included (JSON)
//   --slot "Roster 0 0,2"   every change to that slot (as the page's edits name it), and the
//                           hovers that had it beside them
//   --ocr                   every line sent to be read with what it is for, and its text when back
//   --from N --to M         the scans --ocr and --dump cover (all by default)
//   --dump <folder>         the frames scanned and the lines sent in those scans, as PNG
//   --reread                runs the frames alone, as any build can: every line is read again
//                           here, each back one scan after it was sent, and the slots are not
//                           cut by the time budget. For trying a change; the run then differs
//                           from the trace, which is expected.
//   --no-edits              leaves out what the user set by hand, to see what the scanner makes
//                           of those slots
//   --check                 holds the slots against the hand edits of the end record, the only
//                           right answers a capture has (see capture_edit); implies --no-edits
use hf_scanner::{
    dev_utils::{
        capture::read_frames,
        native::{self, check_edits, describe, describe_chest, print_statuses},
    },
    scanner_state::{ScannerState, SlotAddress},
    tooltip::hover::Hover,
};
use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
};

fn option(args: &[String], name: &str) -> Option<String> {
    let at = args.iter().position(|x| x == name)?;
    Some(args[at + 1].clone())
}

fn key(address: &SlotAddress) -> String {
    let (row, column) = address.pos_in_inv;
    format!(
        "{:?} {} {row},{column}",
        address.inventory_type, address.page_num
    )
}

// in the same order on every run
fn sorted<K: Ord, V>(votes: impl IntoIterator<Item = (K, V)>) -> BTreeMap<K, V> {
    votes.into_iter().collect()
}

// everything a hover voted on, which describe leaves out
fn votes(hover: &Hover) -> String {
    let beside: Vec<String> = hover.candidates.iter().map(|x| key(&x.0)).collect();
    let lit: Vec<String> = hover.lit.iter().map(key).collect();
    let level = hover.chest_place.1.as_ref().map(key);
    format!(
        "      bar {:?}  sent (title, amount, chest, description) {:?}  icon tries {} read body {}\n      votes: title {:?}  description {:?}  icon {:?}  amount {:?}  tradability {:?}\n      beside {beside:?}
      lit {lit:?}  level with {level:?}",
        hover.bar,
        hover.sent,
        hover.icon_tries,
        hover.read_body,
        sorted(&hover.title_votes),
        sorted(&hover.body_votes),
        sorted(&hover.icon_votes),
        sorted(&hover.amount_votes),
        sorted(&hover.tradability_votes),
    )
}

fn slot_line(state: &ScannerState, address: &SlotAddress) -> String {
    match state.slot_infos.get(address) {
        None => "not known".to_string(),
        Some(x) => format!(
            "{:?} also {:?}  amount {:?} of reads {:?}  tooltip amount {:?}  {:?}  label {:?}  hovered {} failed {}",
            x.icon_name_score,
            x.alternatives,
            x.amount,
            x.amount_reads,
            x.tooltip_amount,
            x.tradability,
            x.label,
            x.hovered,
            x.tooltip_failed
        ),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let path = &args[0];
    let slot = option(&args, "--slot");
    let ocr = args.iter().any(|x| x == "--ocr");
    let dump = option(&args, "--dump");
    let reference_path = option(&args, "--reference");
    let reread = args.iter().any(|x| x == "--reread");
    let check = args.iter().any(|x| x == "--check");
    let no_edits = check || args.iter().any(|x| x == "--no-edits");
    let mut sim = native::Sim::new();
    let from: u64 = option(&args, "--from").map_or(0, |x| x.parse().unwrap());
    let to: u64 = option(&args, "--to").map_or(u64::MAX, |x| x.parse().unwrap());
    if let Some(folder) = &dump {
        fs::create_dir_all(folder).unwrap();
    }

    let file = fs::read(path).unwrap();
    let (start, frames, end) = read_frames(&file);
    if let (Some(path), Some(end)) = (&reference_path, &end) {
        fs::write(path, end).unwrap();
    }
    println!("{start:?}");
    let (mut state, _pixels) = native::new_state(start.width, start.height);
    state.screen_info.forced_21_9 = start.forced_21_9;
    state.screen_info.brightness = start.brightness;
    state.hover_log = Some(vec![]);

    // per hover: the first and last scan it was on screen
    let mut seen: HashMap<u32, (u64, u64)> = HashMap::new();
    // per line sent off: what it is for
    let mut lines: HashMap<u32, String> = HashMap::new();
    let mut reads = 0;
    let mut skipped = 0;
    let mut slot_was = String::new();
    for (mut input, trace) in frames {
        let index = state.replayed;
        let covered = (from..=to).contains(&index);
        let (trace, texts) = match reread {
            true => (None, sim.take_ready()),
            false => (Some(trace), vec![]),
        };
        if no_edits {
            input.edits.clear();
        }
        for edit in &input.edits {
            println!("scan {index}: edit {edit:?}");
        }
        if ocr && covered {
            for (id, text) in trace.as_ref().map_or(&texts, |x| &x.ocr_results) {
                println!(
                    "scan {index}: line {id} back as {text:?}  ({})",
                    lines.get(id).map_or("", |x| x)
                );
            }
        }
        state.replay(input, trace, texts);
        let scanned = state.scanned;
        skipped += !scanned as usize;

        if let Some(hover) = &state.hover {
            seen.entry(hover.id).or_insert((index, index)).1 = index;
            let beside = hover.candidates.iter().any(|x| Some(key(&x.0)) == slot);
            if beside && seen[&hover.id].0 == index {
                println!(
                    "scan {index}: hover {} comes up beside {}",
                    hover.id,
                    slot.as_ref().unwrap()
                );
            }
        }
        // the reads sent off in this scan are the last ones of the list
        for read in state.pending_reads.iter().skip(reads) {
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
                for (line, id) in jobs.into_iter().flatten().enumerate() {
                    lines
                        .entry(id)
                        .or_insert(format!("hover {} {kind} line {line}", read.hover));
                }
            }
        }
        reads = state.pending_reads.len();
        for job in &state.ocr_queue {
            let what = lines.entry(job.id).or_insert("a slot's number".to_string());
            if ocr && covered {
                println!(
                    "scan {index}: line {} sent, {what}, {}x{}",
                    job.id,
                    job.line.crop.width(),
                    job.line.crop.height()
                );
            }
            if let Some(folder) = dump.as_ref().filter(|_| covered) {
                job.line
                    .crop
                    .save(format!("{folder}/line-{:05}.png", job.id))
                    .unwrap();
            }
        }
        if let Some(folder) = dump.as_ref().filter(|_| covered && scanned) {
            let frame = state.buffer.raw_crop(0, 0, start.width, start.height);
            frame.save(format!("{folder}/scan-{index:05}.png")).unwrap();
        }
        if reread {
            sim.step(&mut state, index);
        }
        let ended: Vec<Hover> = sim
            .ended
            .drain(..)
            .chain(std::mem::take(state.hover_log.as_mut().unwrap()))
            .collect();
        for hover in ended {
            let (first, last) = seen[&hover.id];
            println!(
                "hover {} on scans {first}-{last}, done at {index}\n      {}\n{}",
                hover.id,
                describe(&state, &hover),
                votes(&hover)
            );
        }
        if let Some(address) = state
            .slot_infos
            .keys()
            .find(|x| Some(key(x)) == slot)
            .copied()
        {
            let now = slot_line(&state, &address);
            if now != slot_was {
                println!("scan {index}: slot {now}");
                slot_was = now;
            }
        }
        state.clear_changed();
    }

    if reread {
        let end = state.replayed;
        sim.finish(&mut state, end);
        eprintln!("{}", sim.summary());
    }
    println!(
        "{} scans, {skipped} of them skipped as still",
        state.replayed
    );
    match (reread, state.replay_mismatch) {
        (true, _) => {}
        (false, Some(scan)) => println!("DIFFERS from the recording, first at scan {scan}"),
        (false, None) => println!("every scan ended as recorded"),
    }
    // still on screen, or still waiting for texts, when the recording stopped
    for hover in state.hover.iter().chain(&state.past_hovers) {
        let (first, last) = seen[&hover.id];
        println!(
            "hover {} on scans {first}-{last}, not done\n      {}\n{}",
            hover.id,
            describe(&state, hover),
            votes(hover)
        );
    }
    println!("{} chests", state.chests.len());
    for chest in &state.chests {
        println!("  {}", describe_chest(chest));
    }
    print_statuses(&state);
    if check {
        let (right, all) = check_edits(&state, end.as_deref().unwrap_or("{}"));
        println!("check: {right} of {all} slots set by hand read right");
    }
}
