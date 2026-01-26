use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, COLORREF, LPARAM, WPARAM, LRESULT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, PostQuitMessage, RegisterClassW,
    ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HICON, HCURSOR, HMENU,
    SW_SHOW, SW_HIDE, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
    SetWindowPos, SWP_NOACTIVATE, SWP_SHOWWINDOW, SWP_HIDEWINDOW,
    PeekMessageW, PM_REMOVE, SetLayeredWindowAttributes, LWA_ALPHA, MSG,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, EndPaint, FillRect, PAINTSTRUCT, InvalidateRect, DeleteObject,
};
use std::sync::mpsc::Receiver;
use std::thread;

const HWND_TOPMOST: HWND = HWND(-1 as isize as *mut std::ffi::c_void);

pub enum UICommand {
    SetColor(u32), 
    DrawDebugRect(i32, i32, i32, i32),
    ClearDebugRect,
    #[allow(dead_code)]
    Quit,
}

static mut CURRENT_COLOR: u32 = 0x0000FF00; 

pub struct UIManager;

impl UIManager {
    pub fn start(rx: Receiver<UICommand>) {
        thread::spawn(move || {
            unsafe {
                let instance = HINSTANCE::default();
                let class_name = w!("ShadowPromptIndicator");
                let debug_class_name = w!("ShadowPromptDebug");

                // 1. Indicator Window Class
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

                // 2. Debug Overlay Window Class (Black Box)
                let wc_debug = WNDCLASSW {
                    hCursor: HCURSOR::default(),
                    hIcon: HICON::default(),
                    lpszClassName: debug_class_name,
                    hInstance: instance,
                    lpfnWndProc: Some(debug_wnd_proc),
                    ..Default::default()
                };
                RegisterClassW(&wc_debug);

                // Create Indicator
                let x = 1920 - 5; 
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
                ).unwrap_or(HWND::default());

                // Create Debug Window (Hidden initially)
                let hwnd_debug = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED, 
                    debug_class_name,
                    w!("DebugOverlay"),
                    WS_POPUP, // Not visible initially
                    0, 0, 0, 0,
                    HWND::default(),
                    HMENU::default(),
                    instance,
                    None,
                ).unwrap_or(HWND::default());

                if hwnd.0.is_null() { return; }

                let _ = ShowWindow(hwnd, SW_SHOW);
                
                // Opacity for Indicator
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);

                // Opacity for Debug (50%)
                let _ = SetLayeredWindowAttributes(hwnd_debug, COLORREF(0), 128, LWA_ALPHA);

                // Loop
                loop {
                    let mut msg = MSG::default();
                    while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                        if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT { return; }
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }

                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            UICommand::SetColor(c) => {
                                CURRENT_COLOR = c;
                                let _ = InvalidateRect(hwnd, None, false);
                            },
                            UICommand::DrawDebugRect(x, y, w, h) => {
                                let _ = ShowWindow(hwnd_debug, SW_SHOW);
                                let _ = SetWindowPos(hwnd_debug, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE | SWP_SHOWWINDOW);
                            },
                            UICommand::ClearDebugRect => {
                                let _ = ShowWindow(hwnd_debug, SW_HIDE); 
                            },
                            UICommand::Quit => { PostQuitMessage(0); }
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
            let color = COLORREF(CURRENT_COLOR); 
            let brush = CreateSolidBrush(color);
            FillRect(hdc, &ps.rcPaint, brush);
            let _ = DeleteObject(brush);
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

unsafe extern "system" fn debug_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            // Black Color for Debug
            let color = COLORREF(0x00000000); 
            let brush = CreateSolidBrush(color);
            FillRect(hdc, &ps.rcPaint, brush);
            let _ = DeleteObject(brush);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
