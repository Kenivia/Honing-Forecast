// Debug: which chests of scanner_setup/templates/chests.json a tooltip's rows read as.
//   cargo run --release --bin chest_rows -- "<name> | <count>" ...
use clap::Parser;
use hf_scanner::tooltip::chest::chests_listing;

#[derive(Parser)]
struct Cli {
    /// "<name> | <count>", one per row
    rows: Vec<String>,
}

fn main() {
    let rows: Vec<(String, String)> = Cli::parse()
        .rows
        .iter()
        .map(|row| {
            let (name, count) = row.split_once(" | ").unwrap();
            (name.to_string(), count.to_string())
        })
        .collect();
    let (listing, whole) = chests_listing(&rows);
    println!("{}", if whole { "all of" } else { "part of" });
    for (variant, score) in listing {
        println!(
            "{:?}  {score:.3}  {} {}",
            variant.kind, variant.id, variant.title
        );
    }
}
