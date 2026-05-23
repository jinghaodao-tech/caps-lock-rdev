#![windows_subsystem = "windows"]
use rdev::{grab, Event, EventType, Key};
use std::process::Command;
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, SetForegroundWindow, IsWindowVisible, WNDENUMPROC};

const F13_SCANCODE: u32 = 124;
static mut F13_PRESSED: bool = false;

// enum_procのシグネチャを LPARAM に修正
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // ポインタを元の String の参照に戻す
    let target = &*(lparam.0 as *const String);
    let mut buffer = [0u16; 256];
    let len = GetWindowTextW(hwnd, &mut buffer);
    let title = String::from_utf16_lossy(&buffer[..len as usize]);
    
    if title.contains(target) && IsWindowVisible(hwnd).as_bool() {
        SetForegroundWindow(hwnd);
        return BOOL(0); // 見つかったら探索終了
    }
    BOOL(1) // 継続
}

fn activate_window(name: &str) {
    let name_str = name.to_string();
    unsafe {
        // 文字列のポインタを LPARAM として渡す
        EnumWindows(Some(enum_proc), LPARAM(&name_str as *const _ as isize));
    }
}

fn main() {
    // 多重起動防止
    let output = Command::new("tasklist").args(["/FI", "IMAGENAME eq hyperkey_launcher.exe", "/NH"]).output().unwrap();
    if String::from_utf8_lossy(&output.stdout).matches("hyperkey_launcher.exe").count() > 1 { std::process::exit(0); }

    let callback = |event: Event| -> Option<Event> {
        match event.event_type {
            EventType::KeyPress(Key::Unknown(F13_SCANCODE)) => { unsafe { F13_PRESSED = true; } None }
            EventType::KeyRelease(Key::Unknown(F13_SCANCODE)) => { unsafe { F13_PRESSED = false; } None }
            EventType::KeyPress(key) => unsafe {
                if F13_PRESSED {
                    match key {
                        Key::KeyT => { F13_PRESSED = false; activate_window("Terminal"); },
                        Key::KeyQ => { F13_PRESSED = false; activate_window("Obsidian"); },
                        Key::KeyV => { F13_PRESSED = false; activate_window("Code"); },
                        Key::Escape => std::process::exit(0),
                        _ => { F13_PRESSED = false; return Some(event); }
                    }
                    None
                } else { Some(event) }
            },
            _ => Some(event),
        }
    };
    let _ = grab(callback);
}