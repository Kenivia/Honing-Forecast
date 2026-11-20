// A small command language for driving the game by hand, shared by market_ctl and market_mcp.
// Commands are separated by ';', coordinates are 1080p client pixels:
//   focus | click x y | rclick x y | dclick x y | move x y | drag x0 y0 x1 y1 | scroll x y notches (down +)
//   key chord (esc, enter, alt+y, ctrl+a) | type text | wait ms
//   shot path [x y w h] [scale] | ocr x0 x1 y0 y1 | num x0 x1 y0 y1 | ref name x y w h | looks name [dy]
use crate::{
    game::{Game, ms},
    glyph, ocr, ui,
};
use image::RgbaImage;

/// Runs a script; returns what the reading commands printed, one line each.
pub fn run(g: &mut Game, script: &str) -> String {
    let mut out = String::new();
    for cmd in script.split(';').map(str::trim).filter(|c| !c.is_empty()) {
        let (name, rest) = cmd.split_once(' ').unwrap_or((cmd, ""));
        let n: Vec<i32> = rest
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        let u = |i: usize| n[i] as u32;
        match name {
            "focus" => g.focus(),
            "click" => g.click(n[0], n[1]),
            "rclick" => g.right_click(n[0], n[1]),
            "dclick" => g.double_click(n[0], n[1]),
            "move" => g.move_to(n[0], n[1]),
            "drag" => g.drag(n[0], n[1], n[2], n[3]),
            "scroll" => g.scroll(n[0], n[1], n[2]),
            "key" => g.key(rest.trim()),
            "type" => g.type_text(rest),
            "wait" => ms(n[0] as u64),
            "ref" => ui::save_ref(
                &g.shot(None),
                rest.split_whitespace().next().unwrap(),
                u(0),
                u(1),
                u(2),
                u(3),
            ),
            "looks" => {
                let name = rest.split_whitespace().next().unwrap();
                out += &format!(
                    "{name} {:.1}\n",
                    ui::ref_distance(&g.shot(None), name, n.first().copied().unwrap_or(0))
                );
            }
            "num" => out += &format!("{:?}\n", glyph::read(&g.shot(None), u(0), u(1), u(2), u(3))),
            "ocr" => {
                out += &format!(
                    "{:?}\n",
                    ocr::read_box(&g.shot(None), u(0), u(1), u(2), u(3))
                )
            }
            "shot" => {
                let mut parts = rest.split_whitespace();
                let path = parts.next().unwrap();
                let v: Vec<u32> = parts.map(|s| s.parse().unwrap()).collect();
                shot(g, &v).save(path).unwrap();
            }
            _ => out += &format!("unknown command {name}\n"),
        }
    }
    out
}

/// A screenshot, optionally cropped (x y w h) and shrunk by a whole factor (last number).
pub fn shot(g: &mut Game, v: &[u32]) -> RgbaImage {
    let crop = (v.len() >= 4).then(|| (v[0], v[1], v[2], v[3]));
    let img = g.shot(crop);
    if v.len() == 5 || v.len() == 1 {
        let s = *v.last().unwrap();
        return image::imageops::resize(
            &img,
            img.width() / s,
            img.height() / s,
            image::imageops::FilterType::Triangle,
        );
    }
    img
}
