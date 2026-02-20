use crate::config::VisualsConfig;
use std::sync::mpsc::Receiver;
use std::thread;
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetSystemMetrics, PeekMessageW,
    PostQuitMessage, RegisterClassW, SetLayeredWindowAttributes, SetWindowPos, ShowWindow,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, HCURSOR, HICON, HMENU, LWA_ALPHA, LWA_COLORKEY, MSG,
    PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW,
    WM_DESTROY, WM_ERASEBKGND, WM_PAINT, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP, WS_VISIBLE,
};

const HWND_TOPMOST: HWND = HWND(-1_isize as *mut std::ffi::c_void);

#[allow(dead_code)]
pub enum UICommand {
    SetColor(u32),
    DrawDebugRect(i32, i32, i32, i32),
    ClearDebugRect,
    #[allow(dead_code)]
    Quit,
    HideToggle,
    SetOverlayText(String),
    ClearOverlayText,
    UpdateOverlayConfig(i32, u8, u8),
}

static mut CURRENT_COLOR: u32 = 0x0000FF00;
static mut IS_HIDDEN: bool = false;
static mut OVERLAY_TEXT: String = String::new();
static mut OVERLAY_FONT_SIZE: i32 = 16;
static mut OVERLAY_BG_OPACITY: u8 = 200;
static mut OVERLAY_TEXT_OPACITY: u8 = 255;

pub struct UIManager;

impl UIManager {
    pub fn start(rx: Receiver<UICommand>, config: VisualsConfig) {
        thread::spawn(move || {
            #[allow(static_mut_refs)]
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

                // 2b. Text Overlay Window Class
                let overlay_class_name = w!("ShadowPromptTextOverlay");
                let wc_overlay = WNDCLASSW {
                    hCursor: HCURSOR::default(),
                    hIcon: HICON::default(),
                    lpszClassName: overlay_class_name,
                    hInstance: instance,
                    lpfnWndProc: Some(overlay_wnd_proc),
                    ..Default::default()
                };
                RegisterClassW(&wc_overlay);

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
                    x,
                    y,
                    size,
                    size,
                    HWND::default(),
                    HMENU::default(),
                    instance,
                    None,
                )
                .unwrap_or(HWND::default());


                // Create Debug Window (Hidden initially)
                let hwnd_debug = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                    debug_class_name,
                    w!("DebugOverlay"),
                    WS_POPUP, // Not visible initially
                    0,
                    0,
                    0,
                    0,
                    HWND::default(),
                    HMENU::default(),
                    instance,
                    None,
                )
                .unwrap_or(HWND::default());

                // Create Text Overlay Window (Hidden initially)
                // Store config values for later use
                OVERLAY_FONT_SIZE = config.text_overlay_font_size.clamp(1, 48);
                OVERLAY_TEXT_OPACITY = config.text_overlay_text_opacity;
                let overlay_offset = config.text_overlay_offset;
                let overlay_x_axis = config.text_overlay_x_axis;
                let overlay_y_axis = config.text_overlay_y_axis;

                // Calculate overlay position (default bottom-right with axis adjustments)
                let base_x = screen_w - overlay_offset;
                let base_y = screen_h - OVERLAY_FONT_SIZE - overlay_offset;
                let position_str = config.text_overlay_position.to_lowercase();
                let (overlay_x, overlay_y) = match position_str.as_str() {
                    "top-left" => (
                        overlay_offset + overlay_x_axis,
                        overlay_offset + overlay_y_axis,
                    ),
                    "top-right" => (base_x + overlay_x_axis, overlay_offset + overlay_y_axis),
                    "bottom-left" => (overlay_offset + overlay_x_axis, base_y + overlay_y_axis),
                    "bottom-right" => (base_x + overlay_x_axis, base_y + overlay_y_axis),
                    _ => (base_x + overlay_x_axis, base_y + overlay_y_axis), // default to bottom-right
                };

                let hwnd_overlay = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
                    overlay_class_name,
                    w!("TextOverlay"),
                    WS_POPUP,
                    overlay_x,
                    overlay_y,
                    100, // Initial width, will be resized dynamically
                    OVERLAY_FONT_SIZE,
                    HWND::default(),
                    HMENU::default(),
                    instance,
                    None,
                )
                .unwrap_or(HWND::default());

                if hwnd.0.is_null() {
                    return;
                }

                let _ = ShowWindow(hwnd, SW_SHOW);

                // Opacity for Indicator 1
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);

                // Opacity for Debug (50%)
                let _ = SetLayeredWindowAttributes(hwnd_debug, COLORREF(0), 128, LWA_ALPHA);

                // Opacity for Text Overlay - use color key for transparent background
                let _ = SetLayeredWindowAttributes(hwnd_overlay, COLORREF(0), 255, LWA_COLORKEY);

                // Loop
                loop {
                    let mut msg = MSG::default();
                    while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                        if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                            return;
                        }
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }

                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            UICommand::SetColor(c) => {
                                CURRENT_COLOR = c;
                                let _ = InvalidateRect(hwnd, None, false);
                            }
                            UICommand::DrawDebugRect(x, y, w, h) => {
                                let _ = ShowWindow(hwnd_debug, SW_SHOW);
                                let _ = SetWindowPos(
                                    hwnd_debug,
                                    HWND_TOPMOST,
                                    x,
                                    y,
                                    w,
                                    h,
                                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                                );
                            }
                            UICommand::ClearDebugRect => {
                                let _ = ShowWindow(hwnd_debug, SW_HIDE);
                            }
                            UICommand::Quit => {
                                PostQuitMessage(0);
                            }
                            UICommand::HideToggle => {
                                IS_HIDDEN = !IS_HIDDEN;
                                if IS_HIDDEN {
                                    let _ = ShowWindow(hwnd, SW_HIDE);
                                    let _ = ShowWindow(hwnd_overlay, SW_HIDE);
                                } else {
                                    let _ = ShowWindow(hwnd, SW_SHOW);
                                }
                            }
                            UICommand::SetOverlayText(text) => {
                                OVERLAY_TEXT = text;
                                let _ = ShowWindow(hwnd_overlay, SW_SHOW);
                                let _ = InvalidateRect(hwnd_overlay, None, false);
                            }
                            UICommand::ClearOverlayText => {
                                OVERLAY_TEXT.clear();
                                let _ = ShowWindow(hwnd_overlay, SW_HIDE);
                            }
                            UICommand::UpdateOverlayConfig(font_size, bg_opacity, text_opacity) => {
                                OVERLAY_FONT_SIZE = font_size;
                                OVERLAY_BG_OPACITY = bg_opacity;
                                OVERLAY_TEXT_OPACITY = text_opacity;
                                let _ = InvalidateRect(hwnd_overlay, None, false);
                            }
                        }
                    }
                    thread::sleep(std::time::Duration::from_millis(16));
                }
            }
        });
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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

unsafe extern "system" fn debug_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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

#[allow(static_mut_refs)]
unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            // Fill with black (transparent due to color key)
            let brush = CreateSolidBrush(COLORREF(0));
            FillRect(hdc, &ps.rcPaint, brush);
            let _ = DeleteObject(brush);

            // Draw text
            if !OVERLAY_TEXT.is_empty() {
                use windows::core::PCWSTR;
                use windows::Win32::Foundation::SIZE;
                use windows::Win32::Graphics::Gdi::{
                    CreateFontW, DeleteObject, DrawTextW, GetTextExtentPoint32W, SelectObject,
                    SetBkMode, SetTextColor, DT_LEFT, DT_NOCLIP, TRANSPARENT,
                };
                use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SET_WINDOW_POS_FLAGS};

                let font_name: Vec<u16> = "Arial\0".encode_utf16().collect();

                let font = CreateFontW(
                    OVERLAY_FONT_SIZE,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    PCWSTR::from_raw(font_name.as_ptr()),
                );

                let _ = SelectObject(hdc, font);
                let _ = SetBkMode(hdc, TRANSPARENT);
                let _ = SetTextColor(hdc, COLORREF(0x00FFFFFF));

                let text: Vec<u16> = OVERLAY_TEXT.encode_utf16().collect();

                // Calculate required size
                let mut size = SIZE::default();
                let text_slice: &[u16] = &text;
                let _ = GetTextExtentPoint32W(hdc, text_slice, &mut size);

                // Resize window to fit text exactly (no padding)
                let width = size.cx;
                let height = OVERLAY_FONT_SIZE;

                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    width.max(10),
                    height.max(1),
                    SET_WINDOW_POS_FLAGS(0),
                );

                // Draw text with no padding
                let mut rect = ps.rcPaint;
                rect.left = 0;
                rect.top = 0;

                let mut text_with_null: Vec<u16> = OVERLAY_TEXT
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                let _ = DrawTextW(hdc, &mut text_with_null, &mut rect, DT_LEFT | DT_NOCLIP);

                let _ = DeleteObject(font);
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_ERASEBKGND => {
            // Prevent default background erasing
            LRESULT(1)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
