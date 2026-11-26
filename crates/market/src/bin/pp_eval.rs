// Reads every text crop of a strips folder with pp.rs, beside what the pass read: pp_eval <strips folder>
use clap::Parser;
use hf_market::pp;

#[derive(Parser)]
struct Cli {
    strips: String,
}

fn main() {
    let dir = Cli::parse().strips;
    for line in std::fs::read_to_string(format!("{dir}/index.jsonl"))
        .unwrap()
        .lines()
    {
        let e: serde_json::Value = serde_json::from_str(line).unwrap();
        if e["kind"] != "ocr" {
            continue;
        }
        let img = image::open(format!("{dir}/{}", e["file"].as_str().unwrap()))
            .unwrap()
            .to_rgba8();
        let t = pp::read(&img, 0, img.width(), 0, img.height());
        println!(
            "{}\t{}\t{:?}\t{:?}",
            e["file"].as_str().unwrap(),
            e["context"],
            e["read"].as_str().unwrap(),
            t
        );
    }
}
