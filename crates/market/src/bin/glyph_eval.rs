// Scores the number reader on ignored/market/dataset/labels.json: glyph_eval
// With strips folders (market_scan --strips), reads their number crops again and prints each read that changed.
use hf_market::glyph;

fn main() {
    let folders: Vec<String> = std::env::args().skip(1).collect();
    if !folders.is_empty() {
        return reread(&folders);
    }
    let dir = concat!(env!("HF_ROOT"), "/ignored/market/dataset");
    let labels: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(format!("{dir}/labels.json")).unwrap())
            .unwrap();
    let mut imgs = std::collections::HashMap::new();
    let (mut right, mut total) = (
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
    );
    let t = std::time::Instant::now();
    for l in &labels {
        let f = l["file"].as_str().unwrap();
        let img = imgs
            .entry(f.to_string())
            .or_insert_with(|| image::open(format!("{dir}/{f}")).unwrap().to_rgba8());
        let r = |k: &str| l[k].as_u64().unwrap() as u32;
        let read = glyph::read(img, r("x0"), r("x1"), r("y0"), r("y1"));
        let want: String = l["label"]
            .as_str()
            .unwrap()
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| if c == ',' { '.' } else { c })
            .collect();
        let kind = l["kind"].as_str().unwrap().to_string();
        *total.entry(kind.clone()).or_insert(0) += 1;
        if read == want
            || kind == "ah_time"
                && read
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .eq(want.chars().filter(|c| c.is_ascii_digit()))
        {
            *right.entry(kind).or_insert(0) += 1;
        } else {
            println!("{f} {:?} want {want} read {read}", l["kind"]);
        }
    }
    for (k, n) in &total {
        println!("{k}: {}/{n}", right.get(k).unwrap_or(&0));
    }
    println!("{:?}", t.elapsed());
}

fn reread(folders: &[String]) {
    let (mut n, mut changed) = (0, 0);
    for dir in folders {
        for line in std::fs::read_to_string(format!("{dir}/index.jsonl"))
            .unwrap()
            .lines()
        {
            let e: serde_json::Value = serde_json::from_str(line).unwrap();
            if e["kind"] != "glyph" {
                continue;
            }
            let img = image::open(format!("{dir}/{}", e["file"].as_str().unwrap()))
                .unwrap()
                .to_rgba8();
            let t = glyph::read(&img, 0, img.width(), 0, img.height());
            n += 1;
            if t != e["read"].as_str().unwrap() {
                changed += 1;
                println!(
                    "{dir} {} {} {:?} -> {t:?}",
                    e["file"], e["context"], e["read"]
                );
            }
        }
    }
    println!("{n} numbers, {changed} changed");
}
