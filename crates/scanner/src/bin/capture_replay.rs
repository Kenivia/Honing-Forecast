// Runs a debug capture from the page again, scan for scan, and says what went on in it.
//   cargo run --release --bin capture_replay -- <file.hfcap> [options]
// Always: whether every scan ended as recorded, each hover once its last texts were in (the scans
// it was seen on, what it read and every vote), the chests and the slots at the end.
//   --reference <file>      writes the page's own account of the end, corrections included (JSON)
//   --slot "Roster 0 0,2"   every change to that slot (as the page's edits name it), and the
//                           hovers that had it beside them
//   --ocr                   every line sent to be read with what it is for, and its text when back
//   --from N --to M         the scans --ocr and --dump cover (all by default)
//   --dump <folder>         the frames scanned and the lines sent in those scans, as PNG
//   --reread                reads every line again here, in place of the recorded texts, each
//                           back one scan after it was sent. For trying a change: once the
//                           scanner asks for other lines than it did, the recorded texts no
//                           longer fit, and the run differs from the recording either way.
//   --no-edits              leaves out what the user set by hand, to see what the scanner makes
//                           of those slots
use hf_scanner::{
    capture::{Record, read_records},
    native::{self, describe, describe_chest, print_statuses},
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
    format!("{:?} {} {row},{column}", address.inventory_type, address.page_num)
}

// in the same order on every run
fn sorted<K: Ord, V>(votes: impl IntoIterator<Item = (K, V)>) -> BTreeMap<K, V> {
    votes.into_iter().collect()
}

// everything a hover voted on, which describe leaves out
fn votes(hover: &Hover) -> String {
    let beside: Vec<String> = hover.candidates.iter().map(|x| key(&x.0)).collect();
    format!(
        "      bar {:?}  sent (title, amount, chest, description) {:?}  icon tries {} shared {}\n      votes: title {:?}  description {:?}  icon {:?}  amount {:?}  tradability {:?}\n      beside {beside:?}",
        hover.bar,
        hover.sent,
        hover.icon_tries,
        hover.shared_icon,
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
    let no_edits = args.iter().any(|x| x == "--no-edits");
    // texts read here, for the next scan
    let mut read: Vec<(u32, String)> = vec![];
    let from: u64 = option(&args, "--from").map_or(0, |x| x.parse().unwrap());
    let to: u64 = option(&args, "--to").map_or(u64::MAX, |x| x.parse().unwrap());
    if let Some(folder) = &dump {
        fs::create_dir_all(folder).unwrap();
    }

    let file = fs::read(path).unwrap();
    let mut records = read_records(&file);
    let Some(Record::Start(start)) = records.next() else { panic!("no start record") };
    println!("{start:?}");
    let game = (start.game_width, start.game_height);
    let (mut state, _pixels) = native::new_state(start.width, start.height, game, start.forced_21_9);
    state.hover_log = Some(vec![]);

    // per hover: the first and last scan it was on screen
    let mut seen: HashMap<u32, (u64, u64)> = HashMap::new();
    // per line sent off: what it is for
    let mut lines: HashMap<u32, String> = HashMap::new();
    let mut reads = 0;
    let mut skipped = 0;
    let mut slot_was = String::new();
    for record in records {
        let mut scan = match record {
            Record::Scan(scan) => scan,
            Record::End(reference) => {
                if let Some(path) = &reference_path {
                    fs::write(path, reference).unwrap();
                }
                continue;
            }
            Record::Start(_) => panic!("a second start record"),
        };
        let index = state.replayed;
        let covered = (from..=to).contains(&index);
        let scanned = scan.frame.is_some();
        if reread {
            scan.ocr_results = std::mem::take(&mut read);
        }
        if no_edits {
            scan.edits.clear();
        }
        skipped += !scanned as usize;
        for edit in &scan.edits {
            println!("scan {index}: edit {edit:?}");
        }
        if ocr && covered {
            for (id, text) in &scan.ocr_results {
                println!("scan {index}: line {id} back as {text:?}  ({})", lines.get(id).map_or("", |x| x));
            }
        }
        state.replay_scan(scan);

        if let Some(hover) = &state.hover {
            seen.entry(hover.id).or_insert((index, index)).1 = index;
            let beside = hover.candidates.iter().any(|x| Some(key(&x.0)) == slot);
            if beside && seen[&hover.id].0 == index {
                println!("scan {index}: hover {} comes up beside {}", hover.id, slot.as_ref().unwrap());
            }
        }
        // the reads sent off in this scan are the last ones of the list
        for read in state.pending_reads.iter().skip(reads) {
            let kinds = [
                ("title", read.title.as_ref().map(|x| x.0.clone())),
                ("amount", read.amount.map(|x| x.to_vec())),
                ("chest", read.chest.as_ref().map(|x| x.iter().flat_map(|x| x.0.iter().copied().chain([x.1])).collect())),
                ("description", read.body.clone()),
            ];
            for (kind, jobs) in kinds {
                for (line, id) in jobs.into_iter().flatten().enumerate() {
                    lines.entry(id).or_insert(format!("hover {} {kind} line {line}", read.hover));
                }
            }
        }
        reads = state.pending_reads.len();
        for job in &state.ocr_queue {
            if reread {
                read.push((job.id, job.line.read()));
            }
            let what = lines.entry(job.id).or_insert("a slot's number".to_string());
            if ocr && covered {
                println!("scan {index}: line {} sent, {what}, {}x{}", job.id, job.line.crop.width(), job.line.crop.height());
            }
            if let Some(folder) = dump.as_ref().filter(|_| covered) {
                job.line.crop.save(format!("{folder}/line-{:05}.png", job.id)).unwrap();
            }
        }
        if let Some(folder) = dump.as_ref().filter(|_| covered && scanned) {
            let frame = state.buffer.raw_crop(0, 0, start.width, start.height);
            frame.save(format!("{folder}/scan-{index:05}.png")).unwrap();
        }
        for hover in std::mem::take(state.hover_log.as_mut().unwrap()) {
            let (first, last) = seen[&hover.id];
            println!("hover {} on scans {first}-{last}, done at {index}\n      {}\n{}", hover.id, describe(&state, &hover), votes(&hover));
        }
        if let Some(address) = state.slot_infos.keys().find(|x| Some(key(x)) == slot).copied() {
            let now = slot_line(&state, &address);
            if now != slot_was {
                println!("scan {index}: slot {now}");
                slot_was = now;
            }
        }
        state.clear_changed();
    }

    println!("{} scans, {skipped} of them skipped as still", state.replayed);
    match state.replay_mismatch {
        Some(scan) => println!("DIFFERS from the recording, first at scan {scan}"),
        None => println!("every scan ended as recorded"),
    }
    // still on screen, or still waiting for texts, when the recording stopped
    for hover in state.hover.iter().chain(&state.past_hovers) {
        let (first, last) = seen[&hover.id];
        println!("hover {} on scans {first}-{last}, not done\n      {}\n{}", hover.id, describe(&state, hover), votes(hover));
    }
    println!("{} chests", state.chests.len());
    for chest in &state.chests {
        println!("  {}", describe_chest(chest));
    }
    print_statuses(&state);
}
