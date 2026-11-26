// The whole scanner over stills: brightness, slots recognised, the hover and every status.
// Debug captures go through capture_replay.
//   cargo run --release --bin tooltip_test -- <image>...
use clap::Parser;
use hf_scanner::dev_utils::native::{self, Options, describe, print_statuses, scan_still};

#[derive(Parser)]
struct Cli {
    images: Vec<String>,
    #[command(flatten)]
    options: Options,
}

fn main() {
    let cli = Cli::parse();
    native::configure(&cli.options);
    for path in &cli.images {
        let (state, _pixels) = scan_still(path);
        println!("{path}");
        let scores: Vec<f64> = state
            .slot_infos
            .values()
            .filter_map(|slot| Some(slot.icon_name_score.as_ref()?.1))
            .collect();
        println!(
            "  brightness {:?}, {} slots recognised, mean confidence {:.4}",
            state.screen_info.brightness.value,
            scores.len(),
            scores.iter().sum::<f64>() / scores.len() as f64
        );
        match &state.hover {
            Some(hover) => println!("  {}", describe(&state, hover)),
            None => println!("  no tooltip"),
        }
        print_statuses(&state);
    }
}
