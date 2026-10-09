// Finds where each slot's icon really is by matching its template over sub-pixel crops, instead of
// trusting the grid position. Prints the offset from the grid so it can be checked for a pattern,
// and what the match score gains by moving there.
//   cargo run --release --bin icon_locate -- <image>...
use hf_scanner::{
    image_utils::{
        brightness::normalize_brightness,
        common::get_resizer,
    },
    native,
    scanner_state::ScannerState,
    setup::icon_lookup,
};
use std::env;


fn main() {
    println!(
        "file\tscale\teff\tinventory\troot_x\troot_y\tpage\trow\tcol\ticon\tgrid_x\tgrid_y\tdx\tdy\tscore_grid\tscore_int\tscore_best"
    );
    for path in env::args().skip(1) {
        let image = image::open(&path).unwrap().to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        // a still is the whole game window, so the capture size is the game resolution
        let game = (width as u32, height as u32);
        let (mut state, mut pixels) = native::new_state(width, height, game, path.contains("21 by 9"));
        pixels.copy_from_slice(image.as_raw());
        for _ in 0..40 {
            state.cropper();
            state.ocr_queue.clear();
        }

        let name = path.split(['/', '\\']).last().unwrap().replace(' ', "_");
        let brightness = state.screen_info.brightness.unwrap();
        let scale = state.screen_info.scale_factor;
        let mut addresses: Vec<_> = state
            .slot_infos
            .iter()
            .filter_map(|(a, info)| info.icon_name_score.as_ref().map(|x| (*a, x.0.clone())))
            .collect();
        addresses
            .sort_by_key(|(a, _)| (format!("{:?}", a.inventory_type), a.page_num, a.pos_in_inv));

        let eff = state.screen_info.effective_height;
        for (address, icon) in addresses {
            let Some(grid) = state.anchored_slot_address_position(&address) else {
                continue;
            };
            let root = state
                .inventory_root(address.inventory_type)
                .unwrap();
            // mean absolute RGB difference of the crop at this offset against the template
            let score = |state: &mut ScannerState, dx: f64, dy: f64| {
                let mut observed = state.crop_buffer(grid.shifted((dx, dy)));
                normalize_brightness(&mut observed, brightness);
                let template = icon_lookup(
                    &icon,
                    state.screen_info.effective_height,
                    get_resizer(&mut state.resizer),
                );
                if template.data.dimensions() != observed.data.dimensions() {
                    return f64::MAX;
                }
                let (a, b) = (template.data.as_raw(), observed.data.as_raw());
                let total: u64 = a
                    .chunks_exact(4)
                    .zip(b.chunks_exact(4))
                    .map(|(p, q)| {
                        p[0].abs_diff(q[0]) as u64
                            + p[1].abs_diff(q[1]) as u64
                            + p[2].abs_diff(q[2]) as u64
                    })
                    .sum();
                total as f64 / (a.len() / 4 * 3) as f64
            };

            let at_grid = score(&mut state, 0.0, 0.0);
            // what close_enough can already recover: the best of the nine whole-pixel shifts
            let mut at_int = f64::MAX;
            for iy in -1..=1 {
                for ix in -1..=1 {
                    at_int = at_int.min(score(&mut state, ix as f64, iy as f64));
                }
            }
            let mut best = (at_grid, 0.0, 0.0);
            // coarse over a couple of pixels, then fine around the winner
            for (step, span) in [(0.25, 2.0), (0.0625, 0.25)] {
                let (cx, cy) = (best.1, best.2);
                let steps = (span / step) as i32;
                for iy in -steps..=steps {
                    for ix in -steps..=steps {
                        let (dx, dy) = (cx + ix as f64 * step, cy + iy as f64 * step);
                        let s = score(&mut state, dx, dy);
                        if s < best.0 {
                            best = (s, dx, dy);
                        }
                    }
                }
            }
            println!(
                "{name}\t{scale}\t{eff}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{icon}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}",
                address.inventory_type,
                root.0,
                root.1,
                address.page_num,
                address.pos_in_inv.1,
                address.pos_in_inv.0,
                grid.top_left.0,
                grid.top_left.1,
                best.1,
                best.2,
                at_grid,
                at_int,
                best.0
            );
        }
    }
}
