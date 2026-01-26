use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, COLORREF, LPARAM, WPARAM, LRESULT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, PostQuitMessage, RegisterClassW,
    ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HICON, HCURSOR, HMENU,
    SW_SHOW, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, EndPaint, FillRect, PAINTSTRUCT,
};
use std::sync::mpsc::Receiver;
use std::thread;

pub enum UICommand {
    SetColor(u32), // 0x00BBGGRR
    #[allow(dead_code)]
    Quit,
}

static mut CURRENT_COLOR: u32 = 0x0000FF00; // Green default

pub struct UIManager;

impl UIManager {
    pub fn start(rx: Receiver<UICommand>) {
        thread::spawn(move || {
            unsafe {
                let instance = HINSTANCE::default();
                let class_name = w!("ShadowPromptIndicator");

                let wc = WNDCLASSW {
                    hCursor: HCURSOR::default(),
                    hIcon: HICON::default(),
                    lpszClassName: class_name,
                    hInstance: instance,
                    lpfnWndProc: Some(wnd_proc),
                    style: CS_HREDRAW | CS_VREDRAW,
                    ..Default::default()
                };

                RegisterClassW(&wc);

                // 1x1 pixel at Top-Right. 
                // We'll hardcode position for now, or get screen width.
                let x = 1920 - 5; // Placeholder
                let y = 0;

                let hwnd = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                    class_name,
                    w!(""),
                    WS_POPUP | WS_VISIBLE,
                    x, y, 5, 5,
                    HWND::default(),
                    HMENU::default(),
                    instance,
                    None,
                ).unwrap_or(HWND::default()); // Force unwrap if Result, or strict HWND if not (this line attempts to handle both via method call if generic, but here we assume Result behavior based on error)
               
                // Check if valid
                if hwnd.0.is_null() {
                    eprintln!("Failed to create window");
                    return;
                }

                let _ = ShowWindow(hwnd, SW_SHOW);
                
                // Opacity
                use windows::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes;
                use windows::Win32::UI::WindowsAndMessaging::LWA_ALPHA;
                
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);

                // Message Loop with non-blocking channel check
                use windows::Win32::UI::WindowsAndMessaging::PeekMessageW;
                use windows::Win32::UI::WindowsAndMessaging::PM_REMOVE;

                loop {
                    let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
                    
                    // Check GUI messages
                    while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                        if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                            return;
                        }
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }

                    // Check Channel
                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            UICommand::SetColor(c) => {
                                CURRENT_COLOR = c;
                                // Force repaint
                                use windows::Win32::Graphics::Gdi::InvalidateRect;
                                let _ = InvalidateRect(hwnd, None, false);
                            },
                            UICommand::Quit => {
                                PostQuitMessage(0);
                            }
                        }
                    }

                    thread::sleep(std::time::Duration::from_millis(16));
                }
            }
        });
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let color = COLORREF(CURRENT_COLOR); // Blue-Green-Red
            let brush = CreateSolidBrush(color);
            FillRect(hdc, &ps.rcPaint, brush);
            let _ = windows::Win32::Graphics::Gdi::DeleteObject(brush);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
