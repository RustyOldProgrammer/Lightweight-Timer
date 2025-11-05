#![windows_subsystem = "windows"]
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::thread;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, EndPaint, HFONT, InvalidateRect, PAINTSTRUCT,
    SetBkMode, SetTextColor, FillRect, UpdateWindow, TRANSPARENT, CreateSolidBrush
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, 
    LoadCursorW, PostMessageW, RegisterClassW, SetLayeredWindowAttributes,
    SetWindowPos, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, HWND_TOPMOST, IDC_ARROW, MSG, SW_SHOW, WM_CREATE, WM_DESTROY,
    WM_HOTKEY, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_OVERLAPPEDWINDOW,
    WS_POPUP, SET_WINDOW_POS_FLAGS, LAYERED_WINDOW_ATTRIBUTES_FLAGS,
    // SendMessageW, WM_NCLBUTTONDOWN, HTCAPTION
};
    // use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;


// === Configurable constants ===
struct Config {
    pos_x: i32,
    pos_y: i32,
    width: i32,
    height: i32,
    font_size: i32,
    topbar_color: COLORREF,
    click_through: bool,
}
const CONFIG: Config = Config {
    pos_x: 20,
    pos_y: 20,
    width: 320,
    height: 100,
    font_size: 48,
    topbar_color: COLORREF(0x202020), // dark gray
    click_through: true,
};

// === Hotkeys ===
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
const HOTKEY_ID_TOGGLE_TRANSPARENCY: i32 = 3;
const HOTKEY_VK_TOGGLE_TRANSPARENCY: u32 = 0xDE; // VK_OEM_7 = '#'

const TIMER_UPDATE_INTERVAL_MS: u64 = 200;


fn to_wstring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let mins = secs / 60;
    let sec = secs % 60;
    format!("{:02}:{:02}", mins, sec)
}


thread_local! {
    static TIMER_STATE: RefCell<TimerState> = RefCell::new(TimerState::new());
    static TRANSPARENCY_STATE: RefCell<bool> = RefCell::new(false); // false = visible, true = transparent
}

struct TimerState {
    hwnd: HWND,
    running: Rc<RefCell<bool>>,
    start: Rc<RefCell<Option<Instant>>>,
    elapsed: Rc<RefCell<Duration>>,
    font: Option<HFONT>,
}

impl TimerState {
    fn new() -> Self {
        Self {
            hwnd: HWND(0),
            running: Rc::new(RefCell::new(false)),
            start: Rc::new(RefCell::new(None)),
            elapsed: Rc::new(RefCell::new(Duration::ZERO)),
            font: None,
        }
    }
}


unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Create font
            let font = CreateFontW(
                CONFIG.font_size, 0, 0, 0, 700,
                0, 0, 0, 0, 0, 0, 0, 0,
                PCWSTR(to_wstring("Segoe UI").as_ptr()),
            );

            TIMER_STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.font = Some(font);
                st.hwnd = hwnd;
            });

            TRANSPARENCY_STATE.with(|t| *t.borrow_mut() = false);


            let hwnd_copy = hwnd;
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(TIMER_UPDATE_INTERVAL_MS));
                let _ = PostMessageW(hwnd_copy, WM_TIMER, WPARAM(0), LPARAM(0));
            });

            LRESULT(0)
        }

        WM_TIMER => {
            InvalidateRect(hwnd, None, true);
            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            // Draw top bar only
            let topbar_rect = RECT { left: 0, top: 0, right: CONFIG.width, bottom: 25 };
            let hbrush = CreateSolidBrush(CONFIG.topbar_color);
            let _ = FillRect(hdc, &topbar_rect, hbrush);

            // Clear timer text area before drawing new number
            let timer_bg_rect = RECT { left: 0, top: 28, right: CONFIG.width, bottom: 60 };
            let timer_bg_brush = CreateSolidBrush(COLORREF(0x202020)); // match topbar color or use another
            let _ = FillRect(hdc, &timer_bg_rect, timer_bg_brush);

            // Set text color to black and background to transparent
            use windows::Win32::Graphics::Gdi::{DrawTextW, DT_CENTER, DT_SINGLELINE, DT_VCENTER};
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLORREF(0x000000)); // black

            let text = TIMER_STATE.with(|s| {
                let st = s.borrow();
                let mut total = *st.elapsed.borrow();
                if *st.running.borrow() {
                    if let Some(start) = *st.start.borrow() {
                        total += start.elapsed();
                    }
                }
                format_duration(total)
            });

            let mut rect = RECT { left: 0, top: 28, right: CONFIG.width, bottom: 60 };
            let mut buf = to_wstring(&text);
            DrawTextW(hdc, &mut buf, &mut rect, DT_CENTER | DT_SINGLELINE | DT_VCENTER);

            EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_HOTKEY => {
            match wparam.0 as i32 {
                HOTKEY_ID_START_PAUSE => {
                    TIMER_STATE.with(|s| {
                        let st = s.borrow_mut();
                        let mut running = st.running.borrow_mut();
                        if !*running {
                            *st.start.borrow_mut() = Some(Instant::now());
                            *running = true;
                        } else {
                            if let Some(start) = *st.start.borrow() {
                                *st.elapsed.borrow_mut() += start.elapsed();
                            }
                            *st.start.borrow_mut() = None;
                            *running = false;
                        }
                    });
                }
                HOTKEY_ID_RESET => {
                    TIMER_STATE.with(|s| {
                        let st = s.borrow_mut();
                        *st.elapsed.borrow_mut() = Duration::ZERO;
                        if *st.running.borrow() {
                            *st.start.borrow_mut() = Some(Instant::now());
                        }
                    });
                }
                HOTKEY_ID_TOGGLE_TRANSPARENCY => {
                    TRANSPARENCY_STATE.with(|t| {
                        let mut transparent = t.borrow_mut();
                        *transparent = !*transparent;
                        let alpha = if *transparent { 8 } else { 255 };
                        unsafe {
                            SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LAYERED_WINDOW_ATTRIBUTES_FLAGS(0x02));
                        }
                    });
                }
                _ => {}
            }
            InvalidateRect(hwnd, None, true);
            LRESULT(0)
        }

        WM_DESTROY => LRESULT(0),

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// === Entry point ===
fn main() -> windows::core::Result<()> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = to_wstring("RustTimerOverlayClass");

        let wc = WNDCLASSW {
            hInstance: hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            lpfnWndProc: Some(window_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };

        RegisterClassW(&wc);

        let mut ex_style = WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW;
        if CONFIG.click_through {
            ex_style |= WS_EX_TRANSPARENT;
        }

        let hwnd = CreateWindowExW(
            ex_style,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(to_wstring("Timer").as_ptr()),
            WS_POPUP | WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT, CW_USEDEFAULT, CONFIG.width, CONFIG.height,
            None, None, hinstance, None,
        );

    // Default to visible on startup
    SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LAYERED_WINDOW_ATTRIBUTES_FLAGS(0x02));

        SetWindowPos(hwnd, HWND_TOPMOST, CONFIG.pos_x, CONFIG.pos_y, CONFIG.width, CONFIG.height, SET_WINDOW_POS_FLAGS(0));

    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);

        RegisterHotKey(
            hwnd,
            HOTKEY_ID_START_PAUSE,
            HOT_KEY_MODIFIERS(HOTKEY_MODIFIERS_NONE),
            HOTKEY_VK_START_PAUSE,
        );
        RegisterHotKey(
            hwnd,
            HOTKEY_ID_RESET,
            HOT_KEY_MODIFIERS(HOTKEY_MODIFIERS_CTRL),
            HOTKEY_VK_RESET,
        );
        RegisterHotKey(
            hwnd,
            HOTKEY_ID_TOGGLE_TRANSPARENCY,
            HOT_KEY_MODIFIERS(HOTKEY_MODIFIERS_NONE),
            HOTKEY_VK_TOGGLE_TRANSPARENCY,
        );

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterHotKey(hwnd, HOTKEY_ID_START_PAUSE);
        UnregisterHotKey(hwnd, HOTKEY_ID_RESET);
        UnregisterHotKey(hwnd, HOTKEY_ID_TOGGLE_TRANSPARENCY);
    }
    Ok(())
}
