// Debug: which chests of scanner_setup/templates/chests.json a tooltip's rows read as.
//   cargo run --release --bin chest_rows -- "<name> | <count>" ...
use hf_scanner::tooltip::chest::chests_listing;

fn main() {
    let rows: Vec<(String, String)> = std::env::args()
        .skip(1)
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
