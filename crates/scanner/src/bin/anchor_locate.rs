// Checks whether a window's slots sit where the anchor says they should. For each found anchor it
// locates the anchor template sub-pixel, the same way icon_locate does for slots, and prints how far
// that is from the position the matcher reported. If the anchor is off by what the slots are off by,
// the matcher is at fault; if the anchor is right and the slots are not, the offsets in the anchor
// to inventory map are.
//   cargo run --release --bin anchor_locate -- <image>...
use hf_scanner::{
    constants::ANCHORS,
    image_utils::{
        brightness::normalize_brightness,
        common::{FloatRectangle, Rectangle, get_resizer},
        resize::crop_buffer,
    },
    native,
    scanner_state::ScannerState,
    setup::icon_lookup,
};
use std::env;


fn main() {
    println!("file\tscale\tanchor\tvariant\treported_x\treported_y\tdx\tdy\tscore_reported\tscore_best");
    for path in env::args().skip(1) {
        let image = image::open(&path).unwrap().to_rgba8();
        let (width, height) = (image.width() as usize, image.height() as usize);
        // a still is the whole game window, so the capture size is the game resolution
        let game = (width as u32, height as u32);
        let (mut state, mut pixels) = native::new_state(width, height, game, path.contains("21 by 9"));
        pixels.copy_from_slice(image.as_raw());
        state.cropper();

        let name = path.split(['/', '\\']).last().unwrap().replace(' ', "_");
        let brightness = state.screen_info.brightness.unwrap();
        let scale = state.screen_info.scale_factor;
        let found: Vec<(String, &'static str, FloatRectangle)> = ANCHORS
            .iter()
            .flat_map(|spec| {
                let positions = state.anchors[&spec.anchor_type].positions.clone();
                spec.variants
                    .iter()
                    .zip(positions)
                    .filter_map(|(variant, position)| {
                        let rect = position?.position;
                        Some((
                            format!("{:?}", spec.anchor_type),
                            variant.name,
                            FloatRectangle {
                                top_left: rect.top_left,
                                width: rect.width as f64,
                                height: rect.height as f64,
                            },
                        ))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        for (anchor, variant, reported) in found {
            let score = |state: &mut ScannerState, dx: f64, dy: f64| {
                let at = FloatRectangle {
                    top_left: (reported.top_left.0 + dx, reported.top_left.1 + dy),
                    width: reported.width,
                    height: reported.height,
                };
                let mut observed =
                    crop_buffer(at, get_resizer(&mut state.resizer), state.buffer, None);
                normalize_brightness(&mut observed, brightness);
                let template = icon_lookup(
                    variant,
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
            let at_reported = score(&mut state, 0.0, 0.0);
            if at_reported == f64::MAX {
                continue;
            }
            let mut best = (at_reported, 0.0, 0.0);
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
                "{name}\t{scale}\t{anchor}\t{variant}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}",
                reported.top_left().0,
                reported.top_left().1,
                best.1,
                best.2,
                at_reported,
                best.0
            );
        }
    }
}
