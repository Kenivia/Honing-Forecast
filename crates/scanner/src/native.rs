// What the native bins share: the config and the recogniser from public/, and a state over a
// frame buffer the bin owns.
use crate::{
    buffer::Buffer,
    scanner_state::ScannerState,
    ocr::recognize::load_ocr_engine,
    setup::set_config,
};
use std::{fs, sync::Once};

pub const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

pub fn load() {
    static LOADED: Once = Once::new();
    LOADED.call_once(|| {
        let config = fs::read(format!("{ROOT}/public/ScannerConfig.msgpack")).unwrap();
        set_config(rmp_serde::from_slice(&config).unwrap());
        load_ocr_engine(fs::read(format!("{ROOT}/public/text-recognition.rten")).unwrap());
    });
}

// The game resolution of a capture, where nothing says: 1078-tall recordings are still 1080p,
// and a native ultrawide capture is as wide as the game.
pub fn guess_game(width: usize, height: usize) -> (u32, u32) {
    let ui_height = (height as f64 / 360.0).round() as u32 * 360;
    ((ui_height * 16 / 9).max(width as u32 / 100 * 100), ui_height)
}

// a state and the frame it scans, to be filled by the caller
pub fn new_state(
    width: usize,
    height: usize,
    game: (u32, u32),
    forced_21_9: bool,
) -> (ScannerState, Vec<u8>) {
    load();
    let mut pixels = vec![0u8; width * height * 4];
    let buffer = Buffer {
        pointer: Some(pixels.as_mut_ptr() as usize),
        width,
        height,
        size: pixels.len(),
        lut: None,
        brightness: 0.0,
    };
    (ScannerState::new(buffer, game.0, game.1, forced_21_9), pixels)
}
