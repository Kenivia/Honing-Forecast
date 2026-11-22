// Optional debug store of every crop that is read, with what was read (`market_scan --strips`):
// <dir>/<n>_<kind>.png plus a line per crop in <dir>/index.jsonl, checked afterwards by tools/verify_strips.py.
use image::RgbaImage;
use std::{io::Write, sync::Mutex};

struct Store {
    dir: String,
    n: usize,
    context: String,
}

static STORE: Mutex<Option<Store>> = Mutex::new(None);

/// Starts storing into a new folder under strips/ (assets::data), named by the time.
pub fn start() {
    let dir = crate::assets::data_dir("strips")
        .join(chrono::Local::now().format("%Y%m%d-%H%M%S").to_string())
        .to_str()
        .unwrap()
        .to_string();
    std::fs::create_dir_all(&dir).unwrap();
    *STORE.lock().unwrap() = Some(Store {
        dir,
        n: 0,
        context: String::new(),
    });
}

/// What the next crops belong to (an item, a search), kept with each.
pub fn context(c: &str) {
    if let Some(s) = STORE.lock().unwrap().as_mut() {
        s.context = c.to_string();
    }
}

pub fn save(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32, kind: &str, read: &str) {
    let mut guard = STORE.lock().unwrap();
    let Some(s) = guard.as_mut() else { return };
    s.n += 1;
    let file = format!("{:06}_{kind}.png", s.n);
    image::imageops::crop_imm(img, x0, y0, x1 - x0, y1 - y0)
        .to_image()
        .save(format!("{}/{file}", s.dir))
        .unwrap();
    let line = serde_json::json!({
        "file": file, "kind": kind, "read": read, "context": s.context,
        "x0": x0, "x1": x1, "y0": y0, "y1": y1,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}/index.jsonl", s.dir))
        .unwrap();
    writeln!(f, "{line}").unwrap();
}
