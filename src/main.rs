use windows::win32:UI;
use windows::win32::system;
use windows::win32::foundation::HWND;
fn main() {
    unsafe{
        let mut msg = MSG::default(); 

        let class_name = widestring::U16CString::from_str("MyGuiContainer").unwrap();
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: h_instance,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(

        )
    }
}
