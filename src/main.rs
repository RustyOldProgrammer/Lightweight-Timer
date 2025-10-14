#![windows_subsystem = "windows"]
use std::thread;
use std::time::Duration;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, RECT, COLORREF};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT, DT_CENTER, DT_SINGLELINE, DrawTextW, SetBkMode, SetTextColor, DT_VCENTER , TRANSPARENT };
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW, PostQuitMessage,
      RegisterClassW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    IDC_ARROW, MSG, SW_SHOW, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE, PostMessageW, WM_CREATE, WM_TIMER
};
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS};
// [state]  ctrl + [
const HOTKEY_ID_START_PAUSE: i32 = 1;
// [state]  ctrl + R
const HOTKEY_ID_RESET: i32 = 2;
// to be used so no conflicts in games
const HOTKEY_MODIFIERS_CTRL: u32 = 0x0002; // CTRL only
const HOTKEY_MODIFIERS_NONE: u32 = 0x0000; // No modifier
//start recording and pause recording
const HOTKEY_VK_START_PAUSE: u32 = 0xDB; //  = '['
//fully reset to 00:00
const HOTKEY_VK_RESET: u32 = 0x52; // 'R'

fn to_wstring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }
unsafe extern "system" fn window_proc(hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_CREATE => {
            let hwnd_copy = hwnd;
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(200));
                let _ = PostMessageW(hwnd_copy, WM_PAINT, WPARAM(0), LPARAM(0));
            });
            LRESULT(0)
        }
         WM_TIMER | WM_PAINT => {
            if msg == WM_PAINT {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x000000u32));
                let mut rect = RECT { left: 0, top: 0, right: 320, bottom: 100 };
                let mut buf = to_wstring("00:00");
                DrawTextW(hdc, &mut buf, &mut rect, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
                EndPaint(hwnd, &ps);
            } else {
                PostMessageW(hwnd, WM_PAINT, WPARAM(0), LPARAM(0));
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
fn main() -> windows::core::Result<()> {
    unsafe {
        let h_instance = GetModuleHandleW(None)?;
        let class_name = to_wstring("Timer");
        let mut msg = MSG::default();

        let h_cursor = LoadCursorW(None, IDC_ARROW)?;
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: h_instance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            hCursor: h_cursor,
            ..Default::default()
        };

        let atom = RegisterClassW(&wnd_class);
        if atom == 0 {
            return Err(windows::core::Error::from_win32());
        }

        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(to_wstring("Timer").as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            500,
            400,
            None,
            None,
            h_instance,
            None,
        );
        if hwnd.0 == 0 {
            return Err(windows::core::Error::from_win32());
        }
        ShowWindow(hwnd, SW_SHOW);

        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
