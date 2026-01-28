use crate::config::VisualsConfig;
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, COLORREF, LPARAM, WPARAM, LRESULT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, PostQuitMessage, RegisterClassW,
    ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HICON, HCURSOR, HMENU,
    SW_SHOW, SW_HIDE, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
    SetWindowPos, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    PeekMessageW, PM_REMOVE, SetLayeredWindowAttributes, LWA_ALPHA, MSG,
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, EndPaint, FillRect, PAINTSTRUCT, InvalidateRect, DeleteObject,
};
use std::sync::mpsc::Receiver;
use std::thread;

const HWND_TOPMOST: HWND = HWND(-1_isize as *mut std::ffi::c_void);

#[allow(dead_code)]
pub enum UICommand {
    SetColor(u32), 
    SetSecondaryColor(u32),
    DrawDebugRect(i32, i32, i32, i32),
    ClearDebugRect,
    #[allow(dead_code)]
    Quit,
}

static mut CURRENT_COLOR: u32 = 0x0000FF00; 
static mut SECONDARY_COLOR: u32 = 0x00000000;

pub struct UIManager;

impl UIManager {
    pub fn start(rx: Receiver<UICommand>, config: VisualsConfig) {
        thread::spawn(move || {
            unsafe {
                let instance = HINSTANCE::default();
                let class_name = w!("ShadowPromptIndicator");
                let secondary_class_name = w!("ShadowPromptIndicatorSecondary");
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

                // 1b. Secondary Indicator Window Class
                let wc_sec = WNDCLASSW {
                    hCursor: HCURSOR::default(),
                    hIcon: HICON::default(),
                    lpszClassName: secondary_class_name,
                    hInstance: instance,
                    lpfnWndProc: Some(secondary_wnd_proc),
                    style: CS_HREDRAW | CS_VREDRAW,
                    ..Default::default()
                };
                RegisterClassW(&wc_sec);

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

                // Calculate Position
                let screen_w = GetSystemMetrics(SM_CXSCREEN);
                let screen_h = GetSystemMetrics(SM_CYSCREEN);
                
                let size = config.size;
                let offset = config.offset;
                let user_x = config.x_axis;
                let user_y = config.y_axis;
                
                let (mut x, mut y) = match config.position.as_str() {
                    "top-left" => (offset, offset),
                    "bottom-left" => (offset, screen_h - size - offset),
                    "bottom-right" => (screen_w - size - offset, screen_h - size - offset),
                    _ => (screen_w - size - offset, offset), // Default top-right
                };

                // Apply User Axis Overrides
                // Right/Left Logic: +X is Right
                x += user_x;

                // Up/Down Logic: "Subtracting will bring it down" -> -Y = Down (User Intention)
                // Screen Y: +Down. So to move Down (+Y), we need to Subtract UserY ONLY IF UserY is negative?
                // Wait. "Subtracting will bring it down": 
                // X - (-50) = X + 50 (Right). 
                // Y - (-50) = Y + 50 (Down).
                // So "User Input" maps to "-ScreenDelta".
                // Let's adhere to "Subtracting moves LEFT/DOWN".
                // Standard Screen: -X is Left. +Y is Down.
                // User: "Subtracting X is Left". Okay, so User X = Screen X.
                // User: "Subtracting Y is Down". Okay, so User Y = -Screen Y (because +ScreenY is Down).
                // Wait, if I subtract 10 -> -10. 
                // If I want it to go DOWN (+ScreenY), I need POSITIVE ScreenY.
                // So -10 User -> +10 Screen.
                // So ScreenY_Delta = -UserY.
                
                // Let's re-read: "If Y axis, subtracting will bring it down".
                // Current Y = 0.
                // User Y = -10 (Subtracted 10 from 0 default).
                // Target Y = 10 (Down 10 pixels).
                // So yes, subtract UserY.
                
                y -= user_y;


                // Create Indicator 1 (Main)
                let hwnd = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                    class_name,
                    w!(""),
                    WS_POPUP | WS_VISIBLE,
                    x, y, size, size,
                    HWND::default(),
                    HMENU::default(),
                    instance,
                    None,
                ).unwrap_or(HWND::default());

                // Create Indicator 2 (Secondary)
                // Default heuristic: Place BELOW if Top, ABOVE if Bottom?
                // Or just stack vertically always?
                // Let's place it just *below* the main one by default, separated by size?
                // Or just adding `size` to Y?
                // If Bottom, maybe above?
                // Let's assume Top alignment for now as default.
                // If it's bottom aligned, we might want to stack upwards.
                
                let sec_y = if config.position.contains("bottom") {
                    y - size // Stack above
                } else {
                    y + size // Stack below
                };

                let hwnd_sec = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                    secondary_class_name,
                    w!(""),
                    WS_POPUP | WS_VISIBLE,
                    x, sec_y, size, size,
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
                let _ = ShowWindow(hwnd_sec, SW_SHOW);
                
                // Opacity for Indicator 1
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);

                // Opacity for Indicator 2
                let _ = SetLayeredWindowAttributes(hwnd_sec, COLORREF(0), 255, LWA_ALPHA);

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
                             UICommand::SetSecondaryColor(c) => {
                                SECONDARY_COLOR = c;
                                let _ = InvalidateRect(hwnd_sec, None, false);
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

unsafe extern "system" fn secondary_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let color = COLORREF(SECONDARY_COLOR); 
            let brush = CreateSolidBrush(color);
            FillRect(hdc, &ps.rcPaint, brush);
            let _ = DeleteObject(brush);
            let _ = EndPaint(hwnd, &ps);
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
