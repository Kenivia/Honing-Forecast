// Mouse and keyboard through SendInput, in game client pixels. Every action checks focus and F10 first.
use crate::game::{Game, check_abort, ms};
use windows_sys::Win32::UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*};

fn send(inputs: &[INPUT]) {
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
}

fn mouse(dx: i32, dy: i32, data: i32, flags: u32) -> INPUT {
    let mut i = INPUT {
        r#type: INPUT_MOUSE,
        ..Default::default()
    };
    i.Anonymous.mi = MOUSEINPUT {
        dx,
        dy,
        mouseData: data as u32,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    i
}

fn key(vk: u16, scan: u16, flags: u32) -> INPUT {
    let mut i = INPUT {
        r#type: INPUT_KEYBOARD,
        ..Default::default()
    };
    i.Anonymous.ki = KEYBDINPUT {
        wVk: vk,
        wScan: scan,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    i
}

fn jitter(base: u64) -> u64 {
    base + fastrand::u64(0..=base / 2)
}

fn abs_move(sx: i32, sy: i32) -> INPUT {
    let m = |i| unsafe { GetSystemMetrics(i) };
    let (vx, vy, vw, vh) = (
        m(SM_XVIRTUALSCREEN),
        m(SM_YVIRTUALSCREEN),
        m(SM_CXVIRTUALSCREEN),
        m(SM_CYVIRTUALSCREEN),
    );
    let nx = ((sx - vx) as i64 * 65535 + (vw as i64 - 2)) / (vw as i64 - 1);
    let ny = ((sy - vy) as i64 * 65535 + (vh as i64 - 2)) / (vh as i64 - 1);
    mouse(
        nx as i32,
        ny as i32,
        0,
        MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
    )
}

impl Game {
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.focus();
        send(&[abs_move(self.x + x, self.y + y)]);
        ms(jitter(60));
    }

    fn button(&mut self, x: i32, y: i32, down: u32, up: u32) {
        self.move_to(x, y);
        send(&[mouse(0, 0, 0, down)]);
        ms(jitter(50));
        send(&[mouse(0, 0, 0, up)]);
        ms(jitter(120));
    }

    pub fn click(&mut self, x: i32, y: i32) {
        self.button(x, y, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP);
    }

    pub fn right_click(&mut self, x: i32, y: i32) {
        self.button(x, y, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP);
    }

    pub fn double_click(&mut self, x: i32, y: i32) {
        self.move_to(x, y);
        for _ in 0..2 {
            send(&[mouse(0, 0, 0, MOUSEEVENTF_LEFTDOWN)]);
            ms(40);
            send(&[mouse(0, 0, 0, MOUSEEVENTF_LEFTUP)]);
            ms(60);
        }
        ms(jitter(100));
    }

    pub fn drag(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        self.move_to(x0, y0);
        send(&[mouse(0, 0, 0, MOUSEEVENTF_LEFTDOWN)]);
        ms(80);
        for i in 1..=15 {
            check_abort();
            send(&[abs_move(
                self.x + x0 + (x1 - x0) * i / 15,
                self.y + y0 + (y1 - y0) * i / 15,
            )]);
            ms(20);
        }
        ms(80);
        send(&[mouse(0, 0, 0, MOUSEEVENTF_LEFTUP)]);
        ms(jitter(120));
    }

    /// Positive notches scroll down.
    pub fn scroll(&mut self, x: i32, y: i32, notches: i32) {
        self.move_to(x, y);
        for _ in 0..notches.abs() {
            check_abort();
            send(&[mouse(0, 0, -120 * notches.signum(), MOUSEEVENTF_WHEEL)]);
            ms(jitter(60));
        }
        ms(jitter(150));
    }

    /// A named key or chord such as `esc`, `enter`, `ctrl+a`.
    pub fn key(&mut self, chord: &str) {
        self.focus();
        let vks: Vec<(u16, bool)> = chord.split('+').map(named_vk).collect();
        for &(vk, ext) in &vks {
            send(&[vk_input(vk, ext, false)]);
            ms(30);
        }
        ms(jitter(40));
        for &(vk, ext) in vks.iter().rev() {
            send(&[vk_input(vk, ext, true)]);
            ms(30);
        }
        ms(jitter(120));
    }

    /// Types text exactly, case and spaces kept, as unicode characters.
    pub fn type_text(&mut self, text: &str) {
        self.focus();
        for c in text.encode_utf16() {
            check_abort();
            send(&[key(0, c, KEYEVENTF_UNICODE)]);
            ms(jitter(25));
            send(&[key(0, c, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP)]);
            ms(jitter(25));
        }
        ms(jitter(100));
    }
}

fn vk_input(vk: u16, ext: bool, up: bool) -> INPUT {
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
    let flags = if ext { KEYEVENTF_EXTENDEDKEY } else { 0 } | if up { KEYEVENTF_KEYUP } else { 0 };
    key(vk, scan, flags)
}

fn named_vk(name: &str) -> (u16, bool) {
    let n = name.to_lowercase();
    let plain = |vk| (vk, false);
    let ext = |vk| (vk, true);
    match n.as_str() {
        "esc" | "escape" => plain(VK_ESCAPE),
        "enter" | "return" => plain(VK_RETURN),
        "tab" => plain(VK_TAB),
        "space" => plain(VK_SPACE),
        "backspace" | "bs" => plain(VK_BACK),
        "ctrl" => plain(VK_CONTROL),
        "shift" => plain(VK_SHIFT),
        "alt" => plain(VK_MENU),
        "delete" | "del" => ext(VK_DELETE),
        "home" => ext(VK_HOME),
        "end" => ext(VK_END),
        "up" => ext(VK_UP),
        "down" => ext(VK_DOWN),
        "left" => ext(VK_LEFT),
        "right" => ext(VK_RIGHT),
        "pgup" => ext(VK_PRIOR),
        "pgdn" => ext(VK_NEXT),
        _ if n.len() == 1 && n.chars().next().unwrap().is_ascii_alphanumeric() => {
            plain(n.to_ascii_uppercase().as_bytes()[0] as u16)
        }
        _ if n.starts_with('f') && n[1..].parse::<u16>().is_ok() => {
            plain(VK_F1 + n[1..].parse::<u16>().unwrap() - 1)
        }
        _ => panic!("unknown key {name}"),
    }
}
