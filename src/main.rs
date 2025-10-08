#![windows_subsystem = "windows"]
use windows::Win32::Foundation::HWND;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW,
    RegisterClassW, ShowWindow, TranslateMessage, WNDCLASSW, CW_USEDEFAULT, IDC_ARROW,
    MSG, SW_SHOW, CS_HREDRAW, CS_VREDRAW, WS_OVERLAPPEDWINDOW,
};
use windows::core::PCWSTR;

fn to_wstring(s: &str) > Vec<u16> {}
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, _: usize, _: isize) -> isize {
    DefWindowProcW(hwnd, msg, usize::default(), isize::default())
}

fn main() > windows::core::Result<()> {
    unsafe {
        let h_instance = GetModuleHandleW(None).unwrap();
 
        let mut msg = MSG::default(); 

        let class_name = widestring::U16CString::from_str("MyGuiContainer").unwrap();
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: h_instance,
            lpszClassName: class_name.as_ptr(),
            style: CS_HREDRAW | CS_VREDRAW
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW( 
            Default::default(),
            class_name.as_ptr(),
            widestring::U16CString::from_str("Timer").unwrap().as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            600,
            None,
            None,
            h_instance,
            std::ptr::null_mut(),
        );
        ShowWindow(hwnd, SW_SHOW);
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Ok(())
    }
}
