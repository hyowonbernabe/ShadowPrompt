// Simple test overlay - configurable text display
// Usage: cargo run --bin test_overlay

#![allow(unused_imports, dead_code)]

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextColor, DT_LEFT, DT_NOCLIP, PAINTSTRUCT,
    TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetSystemMetrics, PeekMessageW,
    PostQuitMessage, RegisterClassW, SetLayeredWindowAttributes, SetWindowPos, ShowWindow,
    CS_HREDRAW, CS_VREDRAW, HCURSOR, HICON, HMENU, HWND_TOPMOST, LWA_ALPHA, LWA_COLORKEY, MSG,
    PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WM_DESTROY, WM_ERASEBKGND, WM_PAINT, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_POPUP,
};

// ===== CONFIGURATION - EDIT THESE VALUES =====
const TEST_TEXT: &str = "TEST";
const WINDOW_HEIGHT: i32 = 8; // Tight height matching font
const FONT_SIZE: i32 = 8; // Font height in pixels
const OFFSET: i32 = 10; // Distance from screen edge (bottom-right)

// Axis adjustments (positive = right/down, negative = left/up)
const X_AXIS: i32 = 0; // Adjust horizontal position
const Y_AXIS: i32 = 0; // Adjust vertical position

// Appearance
const TEXT_OPACITY: u8 = 255; // 0-255 (0 = invisible, 255 = fully opaque)
const TEXT_COLOR: u32 = 0x00FFFFFF; // White text (0x00RRGGBB format)
                                    // =============================================

fn main() {
    unsafe {
        println!("Creating overlay window...");

        let instance = HINSTANCE::default();
        let class_name: Vec<u16> = "TestOverlay\0".encode_utf16().collect();

        let wc = WNDCLASSW {
            hCursor: HCURSOR::default(),
            hIcon: HICON::default(),
            lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
            hInstance: instance,
            lpfnWndProc: Some(wnd_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            eprintln!("Failed to register window class");
            return;
        }
        println!("Window class registered");

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let y = screen_h - WINDOW_HEIGHT - OFFSET;

        println!(
            "Screen: {}x{}, Height: {}",
            screen_w, screen_h, WINDOW_HEIGHT
        );

        let window_title: Vec<u16> = "Test\0".encode_utf16().collect();
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_LAYERED,
            PCWSTR::from_raw(class_name.as_ptr()),
            PCWSTR::from_raw(window_title.as_ptr()),
            WS_POPUP,
            0,
            y,
            50,
            WINDOW_HEIGHT,
            HWND::default(),
            HMENU::default(),
            instance,
            None,
        );

        if hwnd.is_err() || hwnd.as_ref().unwrap().0.is_null() {
            eprintln!("Failed to create window: {:?}", hwnd);
            return;
        }

        let hwnd = hwnd.unwrap();
        println!("Window created: {:?}", hwnd);

        let _ = ShowWindow(hwnd, SW_SHOW);
        // Make black pixels transparent
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_COLORKEY);

        println!("Window should be visible at bottom-right");
        println!("Press Ctrl+C to close");

        let mut msg = MSG::default();
        loop {
            while PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                    return;
                }
                let _ = DispatchMessageW(&msg);
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
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

            // Create font first to measure text
            let font_name: Vec<u16> = "Arial\0".encode_utf16().collect();
            let font = CreateFontW(
                FONT_SIZE,
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

            // Measure text width
            let text_vec: Vec<u16> = TEST_TEXT.encode_utf16().collect();
            let mut size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &text_vec, &mut size);

            // Calculate position with axis adjustments
            let text_width = size.cx;
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let x = screen_w - text_width - OFFSET + X_AXIS;
            let y = screen_h - WINDOW_HEIGHT - OFFSET + Y_AXIS;

            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                x,
                y,
                text_width,
                WINDOW_HEIGHT,
                windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS(0),
            );

            // Fill background with black (transparent due to color key)
            let brush = CreateSolidBrush(COLORREF(0));
            FillRect(hdc, &ps.rcPaint, brush);
            let _ = DeleteObject(brush);

            // Text with opacity
            let _ = SetBkMode(hdc, TRANSPARENT);
            let _ = SetTextColor(hdc, COLORREF(TEXT_COLOR));

            let mut text: Vec<u16> = TEST_TEXT.encode_utf16().collect();
            let mut rect = ps.rcPaint;
            rect.left = 0;
            rect.top = 0;
            let _ = DrawTextW(
                hdc,
                &mut text,
                &mut rect,
                DT_LEFT | windows::Win32::Graphics::Gdi::DT_NOCLIP,
            );

            let _ = DeleteObject(font);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_ERASEBKGND => {
            // Prevent default background erasing (returns 1 to indicate we handled it)
            LRESULT(1)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
