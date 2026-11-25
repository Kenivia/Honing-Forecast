// The game window: finding it, bringing it to the front, and stopping (F10 or STOP) by unwinding with `Stopped`.
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread::sleep,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::ClientToScreen,
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        Threading::{AttachThreadInput, GetCurrentThreadId},
    },
    UI::{
        HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F10},
        WindowsAndMessaging::*,
    },
};

pub const ABORT_KEY: u16 = VK_F10;

#[derive(Clone, Copy, Debug)]
pub struct Game {
    pub hwnd: HWND,
    pub pid: u32,
    /// Client area origin and size in screen pixels.
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub fn ms(n: u64) {
    sleep(Duration::from_millis(n));
}

pub fn abort_held() -> bool {
    (unsafe { GetAsyncKeyState(ABORT_KEY as i32) } as u16) & 0x8000 != 0
}

/// Set from another thread to stop a pass at its next action, as F10 does.
pub static STOP: AtomicBool = AtomicBool::new(false);

/// The panic payload of a stopped pass.
pub struct Stopped;

pub fn check_abort() {
    if abort_held() || STOP.load(Ordering::Relaxed) {
        std::panic::panic_any(Stopped);
    }
}

/// The panic payload of a round that found the region closed (maintenance), with the screen.
pub struct Closed(pub image::RgbaImage);

/// A panic hook that is quiet about `Stopped` and `Closed`.
pub fn quiet_stops() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if !info.payload().is::<Stopped>() && !info.payload().is::<Closed>() {
            prev(info);
        }
    }));
}

/// What a caught panic was: "stopped" or its message.
pub fn panic_text(e: &(dyn std::any::Any + Send)) -> String {
    if e.is::<Stopped>() {
        "stopped".into()
    } else if e.is::<Closed>() {
        "the region is closed".into()
    } else if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = e.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "panic".into()
    }
}

pub fn running() -> bool {
    game_pid().is_some()
}

fn game_pid() -> Option<u32> {
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    let mut e: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    e.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut ok = unsafe { Process32FirstW(snap, &mut e) };
    let mut pid = 0;
    while ok != 0 {
        let len = e.szExeFile.iter().position(|&c| c == 0).unwrap();
        if String::from_utf16_lossy(&e.szExeFile[..len]).eq_ignore_ascii_case("LOSTARK.exe") {
            pid = e.th32ProcessID;
        }
        ok = unsafe { Process32NextW(snap, &mut e) };
    }
    unsafe { windows_sys::Win32::Foundation::CloseHandle(snap) };
    (pid != 0).then_some(pid)
}

struct Search {
    pid: u32,
    best: HWND,
    area: i64,
}

unsafe extern "system" fn enum_cb(hwnd: HWND, l: LPARAM) -> windows_sys::core::BOOL {
    let s = unsafe { &mut *(l as *mut Search) };
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == s.pid && unsafe { IsWindowVisible(hwnd) } != 0 {
        let mut r = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        unsafe { GetWindowRect(hwnd, &mut r) };
        let area = (r.right - r.left) as i64 * (r.bottom - r.top) as i64;
        if area > s.area {
            s.area = area;
            s.best = hwnd;
        }
    }
    1
}

impl Game {
    pub fn find() -> Game {
        Self::try_find().expect("LOSTARK.exe is not running or has no visible window")
    }

    /// None while the game is not running or shows no window (loading).
    pub fn try_find() -> Option<Game> {
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        let mut s = Search {
            pid: game_pid()?,
            best: std::ptr::null_mut(),
            area: 0,
        };
        unsafe { EnumWindows(Some(enum_cb), &mut s as *mut _ as LPARAM) };
        if s.best.is_null() {
            return None;
        }
        let mut g = Game {
            hwnd: s.best,
            pid: s.pid,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        g.refresh_rect();
        Some(g)
    }

    pub fn refresh_rect(&mut self) {
        let mut r = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        unsafe { GetClientRect(self.hwnd, &mut r) };
        let mut p = POINT { x: 0, y: 0 };
        unsafe { ClientToScreen(self.hwnd, &mut p) };
        (self.x, self.y, self.w, self.h) = (p.x, p.y, r.right, r.bottom);
    }

    pub fn focused(&self) -> bool {
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut pid) };
        pid == self.pid
    }

    /// Brings the game to the front; panics if that fails.
    pub fn focus(&mut self) {
        check_abort();
        if self.focused() {
            return;
        }
        for _ in 0..5 {
            unsafe {
                if IsIconic(self.hwnd) != 0 {
                    ShowWindow(self.hwnd, SW_RESTORE);
                }
                let fg_thread =
                    GetWindowThreadProcessId(GetForegroundWindow(), std::ptr::null_mut());
                let me = GetCurrentThreadId();
                AttachThreadInput(me, fg_thread, 1);
                BringWindowToTop(self.hwnd);
                SetForegroundWindow(self.hwnd);
                AttachThreadInput(me, fg_thread, 0);
            }
            ms(400);
            if self.focused() {
                ms(300);
                self.refresh_rect();
                return;
            }
        }
        panic!("could not bring the game to the front");
    }
}
