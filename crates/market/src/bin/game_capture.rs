// Stills and debug captures (.hfcap) of the live game, for working on the inventory scanner with the
// game driven by the lostark-market MCP. The cursor is not in the frames.
//   game_capture still <file.png>
//   game_capture record <file.hfcap> [--seconds N]   until N seconds, F10, or <file>.stop exists
use clap::Parser;
use hf_market::game::{Game, abort_held, ms};
use hf_scanner::dev_utils::capture::{InputRecorder, MAGIC, START, Start, record};
use hf_scanner::scanner_state::Detected;
use std::{
    fs,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

// as the page's CAPTURE_FPS
const FRAME: Duration = Duration::from_millis(33);

#[derive(Parser)]
enum Cli {
    /// The client area at full resolution
    Still { file: String },
    /// A debug capture with no end record (capture_edit adds one)
    Record {
        file: String,
        #[arg(long)]
        seconds: Option<f64>,
    },
}

fn main() {
    let (file, seconds) = match Cli::parse() {
        Cli::Still { file } => (file, None),
        Cli::Record { file, seconds } => (file, Some(seconds)),
    };
    let path = Path::new(&file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut game = Game::find();
    match seconds {
        None => game.shot(None).save(path).unwrap(),
        Some(seconds) => record_to(&mut game, path, seconds),
    }
}

fn record_to(game: &mut Game, path: &Path, seconds: Option<f64>) {
    let stop = path.with_extension("stop");
    let _ = fs::remove_file(&stop);
    let mut out = std::io::BufWriter::new(fs::File::create(path).unwrap());
    let first = game.shot(None);
    let start = Start {
        width: first.width() as usize,
        height: first.height() as usize,
        forced_21_9: Detected {
            value: None,
            manual: false,
        },
        brightness: Detected {
            value: None,
            manual: false,
        },
        commit: "game_capture".into(),
        priors: vec![],
    };
    out.write_all(MAGIC).unwrap();
    out.write_all(&record(START, &start)).unwrap();
    let mut recorder = InputRecorder::default();
    let began = Instant::now();
    let (mut frames, mut frame) = (0, first);
    loop {
        let at = Instant::now();
        let raw = frame.as_raw();
        recorder.buffer(raw.len()).copy_from_slice(raw);
        out.write_all(&recorder.record()).unwrap();
        frames += 1;
        let elapsed = began.elapsed().as_secs_f64();
        if seconds.is_some_and(|x| elapsed >= x) || abort_held() || stop.exists() {
            break;
        }
        ms(FRAME.saturating_sub(at.elapsed()).as_millis() as u64);
        frame = game.shot(None);
    }
    out.flush().unwrap();
    let _ = fs::remove_file(&stop);
    println!(
        "{frames} frames in {:.1} s to {}",
        began.elapsed().as_secs_f64(),
        path.display()
    );
}
