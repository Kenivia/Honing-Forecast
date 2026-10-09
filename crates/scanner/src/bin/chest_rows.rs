// Debug: which chests of templates/chests.json a tooltip's rows read as.
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
    for (variant, score) in chests_listing(&rows) {
        println!("{:?}  {score:.3}  {} {}", variant.kind, variant.id, variant.title);
    }
}
