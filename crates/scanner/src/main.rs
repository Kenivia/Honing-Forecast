use hf_scanner::scanner_state::ScannerState;
use image;
use std::{fs, io::Result, mem::forget};

const CONFIG_PATH: &str =
    "C:/Users/Kenivia/Documents/VSC/Honing-Forecast/public/ScannerConfig.msgpack";
const MODEL_PATH: &str =
    "C:/Users/Kenivia/Documents/VSC/Honing-Forecast/public/text-recognition.rten";
const INPUT_PATH: &str = "C:/Users/Kenivia/Documents/VSC/Honing-Forecast/scripts/brightness/inputs";
const OUTPUT_PATH: &str =
    "C:/Users/Kenivia/Documents/VSC/Honing-Forecast/scripts/brightness/all icons";
fn main() -> Result<()> {
    for bright_entry in fs::read_dir(INPUT_PATH)? {
        let bright_entry = bright_entry?;
        let bright_path = bright_entry.path();

        let brightness = bright_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<u32>()
            .unwrap();
        dbg!(&bright_path);

        let data = image::open(bright_path).unwrap().to_rgba8();

        let mut scanner_state = ScannerState::default();
        scanner_state.screen_info.total_width = 1920;
        scanner_state.screen_info.total_height = 1080;
        scanner_state.debugging = true;
        scanner_state.buffer.width = 1920;
        scanner_state.buffer.height = 1080;
        scanner_state.buffer.size = 1080 * 4 * 1920;

        let mut buf = data.into_raw();
        let ptr = Some(buf.as_mut_ptr() as usize);
        forget(buf);
        scanner_state.buffer.pointer = ptr;

        let bytes = fs::read(CONFIG_PATH)?;
        scanner_state.config = rmp_serde::from_slice(&bytes).unwrap();

        let bytes = fs::read(MODEL_PATH)?;
        scanner_state.model = Some(bytes);
        scanner_state.set_config();
        scanner_state.set_ocr_engine();
        scanner_state.initialize_anchors();
        scanner_state.initialize_page_num_infos();

        scanner_state.cropper();

        let _ = fs::create_dir_all(format!("{}/{}", OUTPUT_PATH, brightness.to_string()));
        for (name, (_, _, _, icons)) in scanner_state.debug_info {
            for (number, icon) in icons.iter().enumerate() {
                let _ = icon.data.save(if number > 1 {
                    format!(
                        "{}/{}/{}-{}.png",
                        OUTPUT_PATH,
                        brightness.to_string(),
                        name,
                        number.to_string(),
                    )
                } else {
                    format!("{}/{}/{}.png", OUTPUT_PATH, brightness.to_string(), name,)
                });
            }
        }
    }

    Ok(())
}
