// Holds the scanner to what is known of the local test inputs in ignored/scanner_tests. Each
// expected.json there (stills/<folder>/, recordings/) names files of its folder with the currencies
// they must read Good, as [char bound, roster bound, tradable]:
//   {"hud 90 bar gold hover.png": {"Gold": [280000, 0, 3960769]}}
// A capture's hand edits (capture_edit) are checked too. Exits 1 if anything is wrong.
//   cargo run --release -p hf-scanner --bin scanner_check [-- <part of a file name>...]
use hf_scanner::{
    dev_utils::native::{ROOT, check_edits, replay_capture, scan_still},
    interface::scan_result::SlotStatus,
    scanner_state::ScannerState,
};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, atomic::AtomicUsize, atomic::Ordering},
};

// captures are held whole in memory, the largest near 1 GB
const WORKERS: usize = 3;

fn currencies(state: &ScannerState, expected: &Value, report: &mut Vec<String>) -> bool {
    let mut ok = true;
    for (name, bands) in expected.as_object().unwrap() {
        let want: Vec<u64> = serde_json::from_value(bands.clone()).unwrap();
        let read = state
            .currency_results()
            .into_iter()
            .find(|x| format!("{:?}", x.currency) == *name)
            .expect(name);
        let right = matches!(read.status, SlotStatus::Good)
            && read.bands.is_some_and(|x| x.to_vec() == want);
        ok &= right;
        report.push(format!(
            "  {name}: {} want {want:?}, read {:?} {:?} {}",
            if right { "right" } else { "WRONG" },
            read.bands,
            read.status,
            read.reason
        ));
    }
    ok
}

fn check(path: &PathBuf, expected: &Value) -> (bool, String) {
    let name = path.to_string_lossy().into_owned();
    let shown = path
        .strip_prefix(ROOT)
        .unwrap()
        .display()
        .to_string()
        .replace('\\', "/");
    let mut report = vec![shown.clone()];
    let ok = if name.ends_with(".hfcap") {
        let (state, _pixels, end) = replay_capture(&name);
        let mut ok = currencies(&state, expected, &mut report);
        let (right, all, lines) = check_edits(&state, end.as_deref().unwrap_or("{}"));
        if all > 0 {
            report.extend(
                lines
                    .into_iter()
                    .filter(|x| x.contains("WRONG") || x.contains("not read")),
            );
            report.push(format!("  hand edits: {right} of {all} read right"));
            ok &= right == all;
        }
        ok
    } else {
        let (state, _pixels) = scan_still(&name);
        currencies(&state, expected, &mut report)
    };
    report[0] = format!("{} {shown}", if ok { "PASS" } else { "FAIL" });
    (ok, report.join("\n"))
}

fn main() {
    let filters: Vec<String> = std::env::args().skip(1).collect();
    let root = PathBuf::from(ROOT).join("ignored/scanner_tests");
    let mut cases = vec![];
    for expected in glob(&root) {
        let all: Value = serde_json::from_slice(&fs::read(&expected).unwrap()).unwrap();
        for (file, want) in all.as_object().unwrap() {
            let path = expected.parent().unwrap().join(file);
            assert!(path.exists(), "{} is listed but missing", path.display());
            if filters.is_empty() || filters.iter().any(|x| file.contains(x.as_str())) {
                cases.push((path, want.clone()));
            }
        }
    }
    let next = AtomicUsize::new(0);
    let failed = Mutex::new(vec![]);
    std::thread::scope(|scope| {
        for _ in 0..WORKERS {
            scope.spawn(|| {
                while let Some((path, want)) = cases.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let (ok, report) = check(path, want);
                    println!("{report}");
                    if !ok {
                        let shown = path.strip_prefix(ROOT).unwrap().display().to_string();
                        failed.lock().unwrap().push(shown.replace('\\', "/"));
                    }
                }
            });
        }
    });
    let failed = failed.into_inner().unwrap();
    println!("{} of {} passed", cases.len() - failed.len(), cases.len());
    if !failed.is_empty() {
        println!("failed:\n  {}", failed.join("\n  "));
        std::process::exit(1);
    }
}

// every expected.json under the folder
fn glob(folder: &PathBuf) -> Vec<PathBuf> {
    let mut found = vec![];
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(glob(&path));
        } else if path.file_name().unwrap() == "expected.json" {
            found.push(path);
        }
    }
    found.sort();
    found
}
