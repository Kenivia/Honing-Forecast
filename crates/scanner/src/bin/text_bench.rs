// Scores text strip variants (TextParams): replays debug captures as the scanner would, keeps every
// distinct tooltip and currency line it sends to be read, and reads each under every variant.
//   cargo run --release --bin text_bench -- <capture.hfcap>... > text.tsv
//   python scanner_setup/scanner/text_bench.py text.tsv
// Lines: VARIANTS then the names; LINE source id role unit, then the read under each variant. The
// role is title, name or count (a chest row), amount, description or other (currency and the
// like); lines of one unit are one text. --strips <folder> also writes each line's strips.
use clap::Parser;
use hf_scanner::{
    dev_utils::{
        capture::read_frames,
        native::{self, Options},
    },
    ocr::{
        jobs::{Kind, OCRLine},
        recognize::recognize_line,
        text::{TextParams, text_strip_with},
    },
};
use std::{collections::BTreeMap, fs};

#[derive(Parser)]
struct Cli {
    captures: Vec<String>,
    #[arg(long)]
    strips: Option<String>,
    #[command(flatten)]
    options: Options,
}

// the first is what the scanner does; add what is being tried
fn variants() -> Vec<(&'static str, TextParams)> {
    [
        "now",
        "levels=60,200",
        "levels=40,220",
        "levels=80,180",
        "unsharp=2,10",
        "unsharp=1,5",
        "filter=triangle",
        "filter=triangle levels=60,200",
    ]
    .into_iter()
    .map(|spec| (spec, TextParams::parse(spec)))
    .collect()
}

fn main() {
    let cli = Cli::parse();
    native::configure(&cli.options);
    let (sources, strips) = (cli.captures, cli.strips);
    let variants = variants();
    println!(
        "VARIANTS\t{}",
        variants.iter().map(|x| x.0).collect::<Vec<_>>().join("\t")
    );
    for source in &sources {
        let (lines, roles) = capture(source);
        let name = source.rsplit(['/', '\\']).next().unwrap();
        for (id, line) in &lines {
            let (role, unit) = roles
                .get(id)
                .cloned()
                .unwrap_or(("other".into(), format!("line {id}")));
            let reads: Vec<String> = variants
                .iter()
                .map(|(spec, params)| {
                    let strip = text_strip_with(
                        &line.crop,
                        matches!(line.kind, Kind::Yellow),
                        line.brightness,
                        params,
                    );
                    if let Some(folder) = &strips {
                        let spec = spec.replace([' ', '=', ','], "_");
                        strip
                            .save(format!("{folder}/{name}-{id:05}-{spec}.png"))
                            .unwrap();
                    }
                    recognize_line(&strip, line.digits_only())
                        .trim()
                        .replace('\t', " ")
                })
                .collect();
            println!("LINE\t{name}\t{id}\t{role}\t{unit}\t{}", reads.join("\t"));
        }
        eprintln!("{name}: {} lines", lines.len());
    }
}

// every text line of a capture by id, and what each is a line of
#[allow(clippy::type_complexity)]
fn capture(source: &str) -> (BTreeMap<u32, OCRLine>, BTreeMap<u32, (String, String)>) {
    let file = fs::read(source).unwrap();
    let (mut start, frames, _) = read_frames(&file);
    let (mut state, _pixels) = native::replay_state(&mut start);
    let mut lines = BTreeMap::new();
    let mut roles = BTreeMap::new();
    // reads still waiting after the last scan; the ones past them were sent in this one
    let mut reads = 0;
    let mut sent = 0;
    for input in frames {
        state.replay(input, vec![]);
        for read in state.pending_reads.iter().skip(reads) {
            sent += 1;
            let unit = format!("hover {} read {sent}", read.hover);
            let mut role = |id: u32, role: &str, unit: String| {
                roles.entry(id).or_insert((role.to_string(), unit));
            };
            for id in read.title.iter().flat_map(|x| x.0.clone()) {
                role(id, "title", unit.clone());
            }
            for id in read.amount.iter().flat_map(|x| x.to_vec()) {
                role(id, "amount", format!("{unit} amount {id}"));
            }
            for (row, (names, count)) in read.chest.iter().flatten().enumerate() {
                for id in names {
                    role(*id, "name", format!("{unit} row {row}"));
                }
                role(*count, "count", format!("{unit} row {row}"));
            }
            for id in read.body.iter().flatten() {
                role(*id, "description", format!("{unit} description {id}"));
            }
        }
        for job in &state.ocr_queue {
            if !matches!(job.line.kind, Kind::Number { .. }) {
                lines.entry(job.id).or_insert_with(|| job.line.clone());
            }
        }
        state.run_ocr_inline();
        reads = state.pending_reads.len();
        state.clear_changed();
    }
    (lines, roles)
}
