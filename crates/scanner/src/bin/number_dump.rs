// Dumps every recognised slot's number strip from stills, for the number OCR experiments.
//   cargo run --release --bin number_dump -- <out dir> <image>...
use hf_scanner::native;
use std::{env, fs};


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let out = &args[0];
    fs::create_dir_all(out).unwrap();
    let mut manifest = String::new();
    for (index, path) in args[1..].iter().enumerate() {
        let image = image::open(path).unwrap().to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let ui_height = (height as f64 / 360.0).round() as u32 * 360;
        let forced = path.contains("21 by 9");
        let game_width = if forced { width as u32 } else { ui_height * 16 / 9 };
        let (mut state, mut pixels) = native::new_state(width, height, (game_width, ui_height), forced);
        pixels.copy_from_slice(image.as_raw());
        state.cropper();
        state.cropper();
        let mut slots: Vec<_> = state.slot_infos.iter().collect();
        slots.sort_by_key(|(a, _)| (format!("{:?}", a.inventory_type), a.page_num, a.pos_in_inv));
        for (address, info) in slots {
            let name = &info.icon_name_score.as_ref().unwrap().0;
            let id = format!(
                "{index:02}_{:?}_{}_{}_{}",
                address.inventory_type,
                address.page_num,
                address.pos_in_inv.0,
                address.pos_in_inv.1
            );
            info.observed_number
                .data
                .save(format!("{out}/{id}.png"))
                .unwrap();
            info.observed_icon
                .data
                .save(format!("{out}/{id}_icon.png"))
                .unwrap();
            manifest += &format!(
                "{id}\t{name}\t{}\t{}\t{}\t{path}\n",
                state.screen_info.brightness.unwrap(),
                state.screen_info.effective_height,
                info.amount.clone().unwrap_or_default().trim()
            );
        }
    }
    fs::write(format!("{out}/manifest.tsv"), manifest).unwrap();
}
