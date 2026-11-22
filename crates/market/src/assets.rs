// Templates and models built into the exe, each template folder overridable file by file from the profile folder
// (dev: the crate's own folders, so new crops work without a rebuild), and where logs and the database go.
use include_dir::{Dir, include_dir};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

static REFS: Dir = include_dir!("$CARGO_MANIFEST_DIR/refs");
static NAMES: Dir = include_dir!("$CARGO_MANIFEST_DIR/names");
static GEMS: Dir = include_dir!("$CARGO_MANIFEST_DIR/gems");
static ITEMS: Dir = include_dir!("$CARGO_MANIFEST_DIR/items");

pub static PP_REC: &[u8] = include_bytes!(concat!(
    env!("HF_ROOT"),
    "/ignored/market/models/en_PP-OCRv5_mobile_rec/rec.onnx"
));
pub static PP_DICT: &str = include_str!(concat!(
    env!("HF_ROOT"),
    "/ignored/market/models/en_PP-OCRv5_mobile_rec/dict.txt"
));
pub static OCRS: &[u8] = include_bytes!(concat!(env!("HF_ROOT"), "/public/text-recognition.rten"));
pub static FONT: &[u8] = include_bytes!(concat!(env!("HF_ROOT"), "/public/fonts/English.ttf"));

static PROFILE: OnceLock<PathBuf> = OnceLock::new();
static DATA: OnceLock<PathBuf> = OnceLock::new();

/// Portable layout: profile/ and data/ next to the exe. Without it, the crate's folders and ignored/market.
pub fn portable() {
    let dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    PROFILE.set(dir.join("profile")).unwrap();
    DATA.set(dir.join("data")).unwrap();
}

pub fn profile() -> &'static Path {
    PROFILE.get_or_init(|| env!("CARGO_MANIFEST_DIR").into())
}

/// Logs, the database, debug screenshots, configs; created on first use.
pub fn data() -> &'static Path {
    let d = DATA.get_or_init(|| concat!(env!("HF_ROOT"), "/ignored/market").into());
    std::fs::create_dir_all(d).unwrap();
    d
}

/// A folder under data(), created.
pub fn data_dir(sub: &str) -> PathBuf {
    let d = data().join(sub);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn builtin(sub: &str) -> &'static Dir<'static> {
    match sub {
        "refs" => &REFS,
        "names" => &NAMES,
        "gems" => &GEMS,
        "items" => &ITEMS,
        _ => panic!("no template folder {sub}"),
    }
}

// a file's key: its name, without "@x_y" and the extension
fn key(file: &str) -> &str {
    let stem = file.rsplit_once('.').map_or(file, |s| s.0);
    stem.split_once('@').map_or(stem, |s| s.0)
}

/// Every file of a template folder as (file name, bytes): the profile's, then the built-in ones it does not replace.
pub fn files(sub: &str) -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = std::fs::read_dir(profile().join(sub))
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .map(|p| {
            let name = p.file_name().unwrap().to_str().unwrap().to_string();
            (name, std::fs::read(&p).unwrap())
        })
        .collect();
    for f in builtin(sub).files() {
        let name = f.path().file_name().unwrap().to_str().unwrap();
        if !out.iter().any(|(n, _)| key(n) == key(name)) {
            out.push((name.to_string(), f.contents().to_vec()));
        }
    }
    out
}

/// The file of a template folder whose key is `k`, profile first.
pub fn find(sub: &str, k: &str) -> Option<(String, Vec<u8>)> {
    for e in std::fs::read_dir(profile().join(sub)).into_iter().flatten() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_str().unwrap().to_string();
        if key(&name) == k {
            return Some((name, std::fs::read(&p).unwrap()));
        }
    }
    builtin(sub)
        .files()
        .find(|f| key(f.path().file_name().unwrap().to_str().unwrap()) == k)
        .map(|f| {
            (
                f.path().file_name().unwrap().to_str().unwrap().to_string(),
                f.contents().to_vec(),
            )
        })
}
