// Which synthetic input methods the game accepts. Run once normally, once from an elevated terminal.
// Hotkeys are read with GetAsyncKeyState, so they work while the game has focus.
use std::{fs::OpenOptions, io::Write, ptr::null_mut, thread::sleep, time::Duration};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, HANDLE, POINT},
    Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        Threading::{
            GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
    UI::{
        HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::{
            GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowThreadProcessId,
            SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
            SetCursorPos,
        },
    },
};

const MARK: u16 = VK_F8;
const YES: u16 = VK_F9;
const NO: u16 = VK_F10;

struct Log(std::fs::File);
impl Log {
    fn line(&mut self, s: &str) {
        println!("{s}");
        writeln!(self.0, "{s}").unwrap();
    }
}

fn ms(n: u64) {
    sleep(Duration::from_millis(n));
}

fn pressed(vk: u16) -> bool {
    (unsafe { GetAsyncKeyState(vk as i32) } as u16) & 0x8000 != 0
}

// Waits for a fresh press of one of `keys` and its release.
fn wait_key(keys: &[u16]) -> u16 {
    while keys.iter().any(|&k| pressed(k)) {
        ms(10);
    }
    loop {
        for &k in keys {
            if pressed(k) {
                while pressed(k) {
                    ms(10);
                }
                return k;
            }
        }
        ms(10);
    }
}

fn cursor() -> (i32, i32) {
    let mut p = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut p) };
    (p.x, p.y)
}

fn send(inputs: &[INPUT]) -> u32 {
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    }
}

fn mouse(dx: i32, dy: i32, flags: u32) -> INPUT {
    let mut i = INPUT {
        r#type: INPUT_MOUSE,
        ..Default::default()
    };
    i.Anonymous.mi = MOUSEINPUT {
        dx,
        dy,
        mouseData: 0,
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

fn abs_move(x: i32, y: i32) -> INPUT {
    let m = |i| unsafe { GetSystemMetrics(i) };
    let (vx, vy, vw, vh) = (
        m(SM_XVIRTUALSCREEN),
        m(SM_YVIRTUALSCREEN),
        m(SM_CXVIRTUALSCREEN),
        m(SM_CYVIRTUALSCREEN),
    );
    let nx = ((x - vx) as i64 * 65535 + (vw as i64 - 2)) / (vw as i64 - 1);
    let ny = ((y - vy) as i64 * 65535 + (vh as i64 - 2)) / (vh as i64 - 1);
    mouse(
        nx as i32,
        ny as i32,
        MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
    )
}

fn click_held(hold: u64) -> u32 {
    let a = send(&[mouse(0, 0, MOUSEEVENTF_LEFTDOWN)]);
    ms(hold);
    a + send(&[mouse(0, 0, MOUSEEVENTF_LEFTUP)])
}

// Relative steps, re-reading the cursor each step so pointer acceleration cannot overshoot.
fn glide(x: i32, y: i32) -> u32 {
    let mut sent = 0;
    for left in (1..=30).rev() {
        let (cx, cy) = cursor();
        let (dx, dy) = (x - cx, y - cy);
        if dx == 0 && dy == 0 {
            break;
        }
        let step = |d: i32| {
            if d.abs() <= left {
                d.signum()
            } else {
                d / left
            }
        };
        sent += send(&[mouse(step(dx), step(dy), MOUSEEVENTF_MOVE)]);
        ms(8);
    }
    sent
}

fn mouse_click(method: usize, (x, y): (i32, i32)) -> u32 {
    match method {
        0 => send(&[
            abs_move(x, y),
            mouse(0, 0, MOUSEEVENTF_LEFTDOWN),
            mouse(0, 0, MOUSEEVENTF_LEFTUP),
        ]),
        1 => {
            let a = send(&[abs_move(x, y)]);
            ms(80);
            a + click_held(60)
        }
        2 => {
            unsafe { SetCursorPos(x, y) };
            ms(80);
            click_held(60)
        }
        _ => {
            let a = glide(x, y);
            ms(80);
            a + click_held(60)
        }
    }
}

const MOUSE_METHODS: [&str; 4] = [
    "M1 absolute move + instant click, one SendInput call",
    "M2 absolute move, 80 ms, click held 60 ms",
    "M3 SetCursorPos, 80 ms, click held 60 ms",
    "M4 relative glide ~250 ms, 80 ms, click held 60 ms",
];

fn type_text(method: usize, text: &str) -> u32 {
    let mut sent = 0;
    for c in text.chars() {
        let vk = c.to_ascii_uppercase() as u16;
        let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
        let (down, up) = match method {
            0 => (key(vk, 0, 0), key(vk, 0, KEYEVENTF_KEYUP)),
            1 => (
                key(0, scan, KEYEVENTF_SCANCODE),
                key(0, scan, KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP),
            ),
            _ => (
                key(0, c as u16, KEYEVENTF_UNICODE),
                key(0, c as u16, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
            ),
        };
        sent += send(&[down]);
        ms(40);
        sent += send(&[up]);
        ms(40);
    }
    sent
}

const KEY_METHODS: [(&str, &str); 3] = [
    ("K1 virtual-key codes", "aaa"),
    ("K2 scan codes", "bbb"),
    ("K3 unicode characters", "ccc"),
];

fn elevation(h: HANDLE) -> String {
    let mut t: HANDLE = null_mut();
    if unsafe { OpenProcessToken(h, TOKEN_QUERY, &mut t) } == 0 {
        return format!("unknown (token denied, error {})", unsafe {
            GetLastError()
        });
    }
    let mut e = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut len = 0;
    let ok = unsafe {
        GetTokenInformation(
            t,
            TokenElevation,
            &mut e as *mut _ as _,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        )
    };
    unsafe { CloseHandle(t) };
    if ok == 0 {
        "unknown".into()
    } else if e.TokenIsElevated != 0 {
        "ELEVATED".into()
    } else {
        "not elevated".into()
    }
}

fn game_processes() -> Vec<(u32, String)> {
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    let mut e: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    e.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut out = vec![];
    let mut ok = unsafe { Process32FirstW(snap, &mut e) };
    while ok != 0 {
        let len = e.szExeFile.iter().position(|&c| c == 0).unwrap();
        let name = String::from_utf16_lossy(&e.szExeFile[..len]);
        if name.to_lowercase().contains("lostark") {
            out.push((e.th32ProcessID, name));
        }
        ok = unsafe { Process32NextW(snap, &mut e) };
    }
    unsafe { CloseHandle(snap) };
    out
}

fn foreground_pid() -> u32 {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut pid) };
    pid
}

fn answer(log: &mut Log, what: &str, sent: u32, games: &[u32]) {
    let fg = if games.contains(&foreground_pid()) {
        "game focused"
    } else {
        "GAME NOT FOCUSED"
    };
    println!("   did it work? F9 yes / F10 no");
    let a = if wait_key(&[YES, NO]) == YES {
        "WORKS"
    } else {
        "fails"
    };
    log.line(&format!(
        "{what}: {a} (SendInput accepted {sent} events, {fg})"
    ));
}

fn main() {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    let dir = concat!(env!("HF_ROOT"), "/ignored/market");
    std::fs::create_dir_all(dir).unwrap();
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/input_test.log"))
        .unwrap();
    let mut log = Log(file);

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    log.line(&format!("=== input_test, unix time {stamp}"));
    log.line(&format!(
        "this program: {}",
        elevation(unsafe { GetCurrentProcess() })
    ));
    let games = game_processes();
    if games.is_empty() {
        log.line("no lostark process found; start the game first");
        return;
    }
    for (pid, name) in &games {
        let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, *pid) };
        let el = if h.is_null() {
            format!("cannot open (error {})", unsafe { GetLastError() })
        } else {
            let el = elevation(h);
            unsafe { CloseHandle(h) };
            el
        };
        log.line(&format!("game process {name} pid {pid}: {el}"));
    }
    let pids: Vec<u32> = games.iter().map(|g| g.0).collect();

    println!(
        "\nMOUSE. In the game, open a window with two tabs (e.g. the Market's tabs), tab A selected."
    );
    println!("Hover tab A and press F8, then hover tab B and press F8.");
    wait_key(&[MARK]);
    let a = cursor();
    println!("   A = {a:?}");
    wait_key(&[MARK]);
    let b = cursor();
    println!("   B = {b:?}");
    for (m, name) in MOUSE_METHODS.iter().enumerate() {
        println!("\n{name}: clicks B, then A after a second. Starting in 2 s.");
        ms(2000);
        let mut sent = mouse_click(m, b);
        let landed_b = cursor();
        ms(1000);
        sent += mouse_click(m, a);
        let landed_a = cursor();
        let what = format!("{name} [landed B {landed_b:?} want {b:?}, A {landed_a:?} want {a:?}]");
        answer(&mut log, &what, sent, &pids);
    }

    println!(
        "\nKEYBOARD. Click into a text box (e.g. the Market search box) yourself, then press F8."
    );
    wait_key(&[MARK]);
    for (m, (name, text)) in KEY_METHODS.iter().enumerate() {
        println!("\n{name}: types \"{text}\" in 1 s.");
        ms(1000);
        let sent = type_text(m, text);
        answer(&mut log, name, sent, &pids);
    }
    log.line("=== done\n");
}
