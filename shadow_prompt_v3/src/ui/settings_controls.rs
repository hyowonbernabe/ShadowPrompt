// settings_controls.rs — Custom Win32 controls for the Settings dialog.
//
// Five control types:
//   HotkeyCapture  — keyboard shortcut capture field
//   ColorButton    — color swatch + ChooseColor dialog
//   CornerDropdown — screen corner picker (ComboBox)
//   NumberSpinner  — validated numeric input (EDIT + ES_NUMBER)
//   ToggleSwitch   — on/off toggle
//
// Parent (settings_dialog) integration:
//   1. register_classes()              — call once at startup.
//   2. WM_DRAWITEM  → draw_control()  — for ColorButton and ToggleSwitch.
//   3. WM_COMMAND / BN_CLICKED → on_click() — for ColorButton and ToggleSwitch.
//   4. WM_CTLCOLORLISTBOX → color_listbox() — for CornerDropdown dark theme.
//   5. read_*()                        — get current values.
//   6. cleanup()                       — call before DestroyWindow on ColorButton / ToggleSwitch.
//      (HotkeyCapture cleans itself up in WM_DESTROY; Corner and Number have no heap data.)

use std::sync::{Once, OnceLock};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, Ellipse, EndPaint, FillRect,
    GetMonitorInfoW, GetStockObject, InvalidateRect, MonitorFromPoint, RoundRect, SelectObject,
    SetBkColor, SetBkMode, SetTextColor, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, DT_CENTER,
    DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, HBRUSH, HDC, HFONT,
    HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST, NULL_BRUSH, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
    PS_SOLID, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, ReleaseCapture, SetCapture, SetFocus};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor, GetClientRect,
    IsChild, IsWindow,
    GetMessageW, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    LoadCursorW, PostQuitMessage, RegisterClassExW, SendMessageW, SetForegroundWindow,
    SetWindowDisplayAffinity, SetWindowLongPtrW, SetWindowTextW, ShowWindow, TranslateMessage,
    GA_ROOT, GWLP_USERDATA, HMENU, IDC_ARROW, MSG, SW_SHOW, WDA_EXCLUDEFROMCAPTURE,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_PAINT, WNDCLASSEXW, WS_CHILD,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_TABSTOP, WS_VISIBLE,
};

// ---------------------------------------------------------------------------
// palette (same dark theme as settings_dialog / account_dialog)
// ---------------------------------------------------------------------------

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}
#[allow(dead_code)]
const BG: COLORREF = rgb(0x1F, 0x1E, 0x1D);
#[allow(dead_code)]
const SURFACE: COLORREF = rgb(0x30, 0x30, 0x2E);
const SURFACE_HOVER: COLORREF = rgb(0x38, 0x37, 0x35);
const FIELD_BG: COLORREF = rgb(0x26, 0x26, 0x24);
const BORDER: COLORREF = rgb(0x3E, 0x3D, 0x39);
const TEXT: COLORREF = rgb(0xF5, 0xF4, 0xEE);
const MUTED: COLORREF = rgb(0xA8, 0xA6, 0x9D);
const ACCENT: COLORREF = rgb(0xD9, 0x77, 0x57);
const OK_TEXT: COLORREF = rgb(0x8F, 0xC2, 0x7E);

// ---------------------------------------------------------------------------
// DPI scaling
// ---------------------------------------------------------------------------

fn s(px: i32) -> i32 {
    let dpi = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() }.max(96) as i32;
    px * dpi / 96
}

// ---------------------------------------------------------------------------
// Win32 message / style constants not in our `windows` features
// ---------------------------------------------------------------------------

const WM_SETFONT: u32 = 0x0030;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const WM_KILLFOCUS: u32 = 0x0008;
const WM_SETFOCUS: u32 = 0x0007;
const WM_LBUTTONDOWN: u32 = 0x0201;
const ES_NUMBER: u32 = 0x2000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const WM_GETFONT: u32 = 0x0031;
const WM_ACTIVATE: u32 = 0x0006;
const WM_COMMAND: u32 = 0x0111;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_QUIT: u32 = 0x0012;
const WM_GETDLGCODE: u32 = 0x0087;
const WM_CAPTURECHANGED: u32 = 0x0215;
const DLGC_WANTALLKEYS: isize = 0x0004;
const WA_INACTIVE: usize = 0;
const EN_CHANGE: u32 = 0x0300;
const VK_ESCAPE: usize = 0x1B;
const VK_RETURN: usize = 0x0D;
const VK_UP: usize = 0x26;
const VK_DOWN: usize = 0x28;
const BS_OWNERDRAW: u32 = 0x000B;
const ODS_SELECTED: u32 = 0x0001;
const CLEARTYPE_QUALITY: u32 = 5;
const DWMWA_USE_IMMERSIVE_DARK_MODE: i32 = 20;
const DWMWA_WINDOW_CORNER_PREFERENCE: i32 = 33;
const DWMWA_BORDER_COLOR: i32 = 34;
const DWMWCP_ROUND: u32 = 2;

// Segoe MDL2 Assets glyphs.
const I_CHEVRON: char = '\u{E70D}';
const I_CHECK: char = '\u{E73E}';

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn hex_to_colorref(hex: &str) -> COLORREF {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return rgb(0, 0, 0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    rgb(r, g, b)
}

fn colorref_to_hex(c: COLORREF) -> String {
    let r = (c.0 & 0xFF) as u8;
    let g = ((c.0 >> 8) & 0xFF) as u8;
    let b = ((c.0 >> 16) & 0xFF) as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// True if the given virtual-key code is currently held down.
fn key_down(vk: u16) -> bool {
    (unsafe { GetAsyncKeyState(vk as i32) }) & (0x8000u16 as i16) != 0
}

/// Map a virtual-key code to a lowercase name for the hotkey string.
/// Returns None for modifier-only keys and unknown codes.
fn vk_name(vk: u16) -> Option<String> {
    Some(match vk {
        // Modifier VKs are handled separately (ctrl/shift/alt/win prefix).
        0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0..=0xA5 => return None,

        // Digits 0-9.
        0x30..=0x39 => ((vk as u8) as char).to_string(),
        // Letters A-Z, lowercased.
        0x41..=0x5A => ((vk as u8 - b'A' + b'a') as char).to_string(),

        // Function keys F1-F24.
        0x70..=0x87 => format!("f{}", vk - 0x6F),

        // Named keys.
        0x08 => "backspace".into(),
        0x09 => "tab".into(),
        0x0D => "enter".into(),
        0x13 => "pause".into(),
        0x14 => "capslock".into(),
        0x1B => "escape".into(),
        0x20 => "space".into(),
        0x21 => "pageup".into(),
        0x22 => "pagedown".into(),
        0x23 => "end".into(),
        0x24 => "home".into(),
        0x25 => "left".into(),
        0x26 => "up".into(),
        0x27 => "right".into(),
        0x28 => "down".into(),
        0x2C => "printscreen".into(),
        0x2D => "insert".into(),
        0x2E => "delete".into(),

        // Numpad.
        0x60..=0x69 => format!("num{}", vk - 0x60),
        0x6A => "num*".into(),
        0x6B => "num+".into(),
        0x6D => "num-".into(),
        0x6E => "num.".into(),
        0x6F => "num/".into(),
        0x90 => "numlock".into(),
        0x91 => "scrolllock".into(),

        // OEM keys (US layout).
        0xBA => ";".into(),
        0xBB => "=".into(),
        0xBC => ",".into(),
        0xBD => "-".into(),
        0xBE => ".".into(),
        0xBF => "/".into(),
        0xC0 => "`".into(),
        0xDB => "[".into(),
        0xDC => "\\".into(),
        0xDD => "]".into(),
        0xDE => "'".into(),

        _ => return None,
    })
}

/// Build "ctrl+shift+alt+win+keyname" from modifier state + key name.
fn format_hotkey(ctrl: bool, shift: bool, alt: bool, win: bool, key: &str) -> String {
    let mut parts: Vec<&str> = Vec::with_capacity(5);
    if ctrl {
        parts.push("ctrl");
    }
    if shift {
        parts.push("shift");
    }
    if alt {
        parts.push("alt");
    }
    if win {
        parts.push("win");
    }
    parts.push(key);
    parts.join("+")
}

/// Read the text of a standard Win32 EDIT / STATIC control.
fn read_window_text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let n = GetWindowTextW(hwnd, &mut buf).max(0) as usize;
        String::from_utf16_lossy(&buf[..n]).trim().to_string()
    }
}

// ---------------------------------------------------------------------------
// per-control data stored via GWLP_USERDATA
// ---------------------------------------------------------------------------

struct HotkeyData {
    value: String,
    prev: String,
    capturing: bool,
    font: HFONT,
}

enum OwnerData {
    Color(COLORREF),
    Toggle(bool),
    Corner(usize), // index into CORNERS
}

// ===========================================================================
// class registration
// ===========================================================================

/// Register custom window classes used by these controls. Call once at startup.
pub fn register_classes() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(hotkey_proc),
            hInstance: hinstance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            lpszClassName: w!("SysHotkeyCapture"),
            ..Default::default()
        };
        RegisterClassExW(&wc);
    });
}

// ===========================================================================
// 1. HotkeyCapture
// ===========================================================================

unsafe extern "system" fn hotkey_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    // Helper: borrow the per-control data (may be null before WM_CREATE finishes).
    let data_ptr = || GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut HotkeyData;

    match msg {
        // ---- paint ----------------------------------------------------------
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);

            // Field background.
            let fill = CreateSolidBrush(FIELD_BG);
            FillRect(hdc, &rc, fill);
            let _ = DeleteObject(HGDIOBJ(fill.0));

            // 1px rounded border.
            let pen = CreatePen(PS_SOLID, s(1).max(1), BORDER);
            let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));
            let old_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH));
            let r = s(3);
            let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, r, r);
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brush);
            let _ = DeleteObject(HGDIOBJ(pen.0));

            // Text content.
            let ptr = data_ptr();
            if !ptr.is_null() {
                let d = &*ptr;
                let text_rc = RECT {
                    left: rc.left + s(6),
                    top: rc.top,
                    right: rc.right - s(4),
                    bottom: rc.bottom,
                };
                SetBkMode(hdc, TRANSPARENT);
                SelectObject(hdc, HGDIOBJ(d.font.0));

                if d.capturing {
                    SetTextColor(hdc, MUTED);
                    let mut t: Vec<u16> = "Press keys\u{2026}".encode_utf16().collect();
                    let mut trc = text_rc;
                    DrawTextW(hdc, &mut t, &mut trc, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
                } else {
                    let display = if d.value.is_empty() {
                        "\u{2014}" // em-dash placeholder for "none"
                    } else {
                        &d.value
                    };
                    SetTextColor(hdc, TEXT);
                    let mut t: Vec<u16> = display.encode_utf16().collect();
                    let mut trc = text_rc;
                    DrawTextW(hdc, &mut t, &mut trc, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
                }
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        // ---- enter capture on click or focus --------------------------------
        WM_LBUTTONDOWN | WM_SETFOCUS => {
            if msg == WM_LBUTTONDOWN {
                // Without focus the keystrokes go elsewhere and capture never ends.
                let _ = SetFocus(hwnd);
            }
            let ptr = data_ptr();
            if !ptr.is_null() {
                let d = &mut *ptr;
                if !d.capturing {
                    d.prev = d.value.clone();
                    d.capturing = true;
                    let _ = InvalidateRect(hwnd, None, true);
                }
            }
            LRESULT(0)
        }

        // ---- cancel capture on focus loss -----------------------------------
        WM_KILLFOCUS => {
            let ptr = data_ptr();
            if !ptr.is_null() {
                let d = &mut *ptr;
                if d.capturing {
                    d.value = d.prev.clone();
                    d.capturing = false;
                    let _ = InvalidateRect(hwnd, None, true);
                }
            }
            LRESULT(0)
        }

        // While capturing, ask the parent's IsDialogMessageW to hand us every key, so Tab,
        // Enter and arrows can be captured and Escape cancels capture instead of closing Settings.
        WM_GETDLGCODE => {
            let ptr = data_ptr();
            if !ptr.is_null() && (*ptr).capturing {
                LRESULT(DLGC_WANTALLKEYS)
            } else {
                DefWindowProcW(hwnd, msg, wp, lp)
            }
        }

        // ---- key capture ----------------------------------------------------
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let ptr = data_ptr();
            if ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wp, lp);
            }
            let d = &mut *ptr;
            if !d.capturing {
                return DefWindowProcW(hwnd, msg, wp, lp);
            }

            let vk = (wp.0 & 0xFFFF) as u16;
            let has_mod =
                key_down(0x11) || key_down(0x10) || key_down(0x12) || key_down(0x5B) || key_down(0x5C);

            // Escape with no modifiers: cancel capture, restore previous value.
            if vk == 0x1B && !has_mod {
                d.value = d.prev.clone();
                d.capturing = false;
                let _ = InvalidateRect(hwnd, None, true);
                return LRESULT(0);
            }

            // Delete / Backspace with no modifiers: clear the hotkey.
            if (vk == 0x2E || vk == 0x08) && !has_mod {
                d.value.clear();
                d.capturing = false;
                let _ = InvalidateRect(hwnd, None, true);
                return LRESULT(0);
            }

            // Resolve the pressed key to a name.
            if let Some(name) = vk_name(vk) {
                d.value = format_hotkey(
                    key_down(0x11), // Ctrl
                    key_down(0x10), // Shift
                    key_down(0x12), // Alt
                    key_down(0x5B) || key_down(0x5C), // Win
                    &name,
                );
                d.capturing = false;
                let _ = InvalidateRect(hwnd, None, true);
            }
            // Unknown or modifier-only key: stay in capture mode.
            LRESULT(0)
        }

        // Swallow key-up during capture so the system doesn't beep.
        WM_KEYUP | WM_SYSKEYUP => {
            let ptr = data_ptr();
            if !ptr.is_null() && (*ptr).capturing {
                return LRESULT(0);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }

        // ---- cleanup --------------------------------------------------------
        WM_DESTROY => {
            let ptr = data_ptr();
            if !ptr.is_null() {
                let _ = Box::from_raw(ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

/// Create a hotkey capture control. Returns HWND.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn create_hotkey(
    parent: HWND,
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    value: &str,
    font: HFONT,
) -> HWND {
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("SysHotkeyCapture"),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as isize as *mut _),
        GetModuleHandleW(None).unwrap_or_default(),
        None,
    )
    .unwrap_or_default();

    let data = Box::into_raw(Box::new(HotkeyData {
        value: value.to_string(),
        prev: String::new(),
        capturing: false,
        font,
    }));
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as isize);
    hwnd
}

/// Read the current hotkey string from the control.
pub fn read_hotkey(hwnd: HWND) -> String {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const HotkeyData;
        if ptr.is_null() {
            return String::new();
        }
        (*ptr).value.clone()
    }
}

// ===========================================================================
// 2. ColorButton
// ===========================================================================

/// Create a color picker button. Returns HWND.
///
/// Displays a color swatch + hex code. On click (routed through `on_click`),
/// opens the system ChooseColor dialog.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn create_color(
    parent: HWND,
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    hex: &str,
    font: HFONT,
) -> HWND {
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as isize as *mut _),
        GetModuleHandleW(None).unwrap_or_default(),
        None,
    )
    .unwrap_or_default();

    SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));

    let data = Box::into_raw(Box::new(OwnerData::Color(hex_to_colorref(hex))));
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as isize);
    hwnd
}

/// Read the current color as "#RRGGBB".
pub fn read_color(hwnd: HWND) -> String {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OwnerData;
        if ptr.is_null() {
            return "#000000".into();
        }
        match &*ptr {
            OwnerData::Color(c) => colorref_to_hex(*c),
            _ => "#000000".into(),
        }
    }
}

// ===========================================================================
// 3. CornerDropdown
// ===========================================================================

const CORNERS: [&str; 4] = ["top-left", "top-right", "bottom-left", "bottom-right"];

/// Create a corner picker: a dark field box that opens a dark list popup (via `on_click`).
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn create_corner(
    parent: HWND,
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    value: &str,
    font: HFONT,
) -> HWND {
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as isize as *mut _),
        GetModuleHandleW(None).unwrap_or_default(),
        None,
    )
    .unwrap_or_default();

    SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    let sel = CORNERS.iter().position(|&c| c == value).unwrap_or(0);
    let data = Box::into_raw(Box::new(OwnerData::Corner(sel)));
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as isize);
    hwnd
}

/// Read the selected corner string.
pub fn read_corner(hwnd: HWND) -> String {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OwnerData;
        let idx = match ptr.as_ref() {
            Some(OwnerData::Corner(i)) => *i,
            _ => 0,
        };
        CORNERS[idx].to_string()
    }
}

// ===========================================================================
// 4. NumberSpinner (validated EDIT — no UpDown since Win32_UI_Controls is off)
// ===========================================================================

/// Create a number input field. Returns HWND.
///
/// `digits_only` (whole numbers) uses ES_NUMBER; decimals, negatives and "x, y" pairs need a
/// plain EDIT. Decided by the field's type, not its current value ("1.0" shows as "1").
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn create_number(
    parent: HWND,
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    value: &str,
    digits_only: bool,
    font: HFONT,
) -> HWND {
    let style = WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WINDOW_STYLE(ES_AUTOHSCROLL | if digits_only { ES_NUMBER } else { 0 });

    let text: Vec<u16> = value.encode_utf16().chain([0]).collect();
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("EDIT"),
        PCWSTR(text.as_ptr()),
        style,
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as isize as *mut _),
        GetModuleHandleW(None).unwrap_or_default(),
        None,
    )
    .unwrap_or_default();

    SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    hwnd
}

/// Read the number as a string.
pub fn read_number(hwnd: HWND) -> String {
    read_window_text(hwnd)
}

// ===========================================================================
// 5. ToggleSwitch
// ===========================================================================

/// Create a toggle switch (owner-drawn). Returns HWND.
///
/// Drawing and click handling are routed through `draw_control` and `on_click`.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn create_toggle(
    parent: HWND,
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    on: bool,
    font: HFONT,
) -> HWND {
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("BUTTON"),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as isize as *mut _),
        GetModuleHandleW(None).unwrap_or_default(),
        None,
    )
    .unwrap_or_default();

    SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));

    let data = Box::into_raw(Box::new(OwnerData::Toggle(on)));
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as isize);
    hwnd
}

/// Read the toggle state as "true" or "false".
pub fn read_toggle(hwnd: HWND) -> String {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OwnerData;
        if ptr.is_null() {
            return "false".into();
        }
        match &*ptr {
            OwnerData::Toggle(on) => on.to_string(),
            _ => "false".into(),
        }
    }
}

// ===========================================================================
// owner-drawn painting (WM_DRAWITEM dispatch)
// ===========================================================================

/// Paint an owner-drawn control (ColorButton or ToggleSwitch).
/// Call from the parent's WM_DRAWITEM handler.
pub(crate) unsafe fn draw_control(
    hdc: HDC,
    hwnd_item: HWND,
    rc: RECT,
    item_state: u32,
    bg_brush: HBRUSH,
) {
    let ptr = GetWindowLongPtrW(hwnd_item, GWLP_USERDATA) as *const OwnerData;
    if ptr.is_null() {
        return;
    }

    let pressed = item_state & ODS_SELECTED != 0;
    FillRect(hdc, &rc, bg_brush);

    let font = HFONT(SendMessageW(hwnd_item, WM_GETFONT, WPARAM(0), LPARAM(0)).0 as *mut _);
    SelectObject(hdc, HGDIOBJ(font.0));
    match &*ptr {
        OwnerData::Color(color) => {
            let inner = draw_field_box(hdc, rc, pressed);
            draw_color_swatch(hdc, inner, *color);
        }
        OwnerData::Toggle(on) => draw_toggle_track(hdc, rc, *on),
        OwnerData::Corner(i) => {
            let inner = draw_field_box(hdc, rc, pressed);
            draw_label(hdc, font, TEXT, CORNERS[*i], inner);
        }
    }
}

/// The shared look for every "click to choose" field (corner, color, model list):
/// dark rounded box with a chevron on the right. Returns the rect left for content.
pub(crate) unsafe fn draw_field_box(hdc: HDC, rc: RECT, pressed: bool) -> RECT {
    round_rect(hdc, rc, Some(if pressed { SURFACE_HOVER } else { FIELD_BG }), BORDER, s(3));
    let chevron_w = s(18);
    let crc = RECT { left: rc.right - chevron_w, top: rc.top, right: rc.right - s(4), bottom: rc.bottom };
    draw_label(hdc, icon_font(), MUTED, &I_CHEVRON.to_string(), crc);
    RECT { left: rc.left + s(6), top: rc.top, right: rc.right - chevron_w, bottom: rc.bottom }
}

unsafe fn round_rect(hdc: HDC, rc: RECT, fill: Option<COLORREF>, border: COLORREF, radius: i32) {
    let brush = fill.map(|c| CreateSolidBrush(c));
    let pen = CreatePen(PS_SOLID, s(1).max(1), border);
    let old_b = SelectObject(hdc, brush.map(|b| HGDIOBJ(b.0)).unwrap_or_else(|| GetStockObject(NULL_BRUSH)));
    let old_p = SelectObject(hdc, HGDIOBJ(pen.0));
    let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
    if let Some(b) = brush {
        let _ = DeleteObject(HGDIOBJ(b.0));
    }
    let _ = DeleteObject(HGDIOBJ(pen.0));
}

/// Single-line, vertically centred, left-aligned (centred if `rc` is icon-sized), ellipsized text.
unsafe fn draw_label(hdc: HDC, font: HFONT, color: COLORREF, s_: &str, mut rc: RECT) {
    SelectObject(hdc, HGDIOBJ(font.0));
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, color);
    let mut wide: Vec<u16> = s_.encode_utf16().collect();
    let align = if rc.right - rc.left <= s(24) { DT_CENTER } else { DT_LEFT };
    DrawTextW(hdc, &mut wide, &mut rc, align | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);
}

/// Segoe MDL2 icon font, created once per process.
pub fn icon_font() -> HFONT {
    static FONT: OnceLock<usize> = OnceLock::new();
    HFONT(*FONT.get_or_init(|| unsafe {
        CreateFontW(
            -s(12), 0, 0, 0, 400, 0, 0, 0,
            DEFAULT_CHARSET.0 as u32, OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32, CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32, w!("Segoe MDL2 Assets"),
        )
        .0 as usize
    }) as *mut _)
}

/// Internal: color-field content (swatch + hex label) inside the field box.
unsafe fn draw_color_swatch(hdc: HDC, rc: RECT, color: COLORREF) {
    let h = rc.bottom - rc.top;

    // Color swatch (small square with the selected color).
    let swatch_size = (h - s(10)).max(s(4));
    let swatch_x = rc.left;
    let swatch_y = rc.top + (h - swatch_size) / 2;

    let sw_brush = CreateSolidBrush(color);
    let sw_pen = CreatePen(PS_SOLID, s(1).max(1), BORDER);
    let ob = SelectObject(hdc, HGDIOBJ(sw_brush.0));
    let op = SelectObject(hdc, HGDIOBJ(sw_pen.0));
    let sr = s(2);
    let _ = RoundRect(
        hdc,
        swatch_x,
        swatch_y,
        swatch_x + swatch_size,
        swatch_y + swatch_size,
        sr,
        sr,
    );
    SelectObject(hdc, ob);
    SelectObject(hdc, op);
    let _ = DeleteObject(HGDIOBJ(sw_brush.0));
    let _ = DeleteObject(HGDIOBJ(sw_pen.0));

    // Hex code text.
    let hex = colorref_to_hex(color);
    let text_left = swatch_x + swatch_size + s(6);
    let mut text_rc = RECT {
        left: text_left,
        top: rc.top,
        right: rc.right - s(4),
        bottom: rc.bottom,
    };
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, TEXT);
    let mut wide: Vec<u16> = hex.encode_utf16().collect();
    DrawTextW(
        hdc,
        &mut wide,
        &mut text_rc,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
    );
}

/// Internal: paint the toggle (track + knob + On/Off label).
unsafe fn draw_toggle_track(hdc: HDC, rc: RECT, on: bool) {
    let h = rc.bottom - rc.top;
    let track_w = s(36);
    let track_h = h.min(s(18));
    let track_y = rc.top + (h - track_h) / 2;
    let track_r = track_h / 2;

    // Track background: accent when on, muted surface when off.
    let track_fill = if on { ACCENT } else { SURFACE_HOVER };
    let track_border = if on { ACCENT } else { BORDER };

    let t_brush = CreateSolidBrush(track_fill);
    let t_pen = CreatePen(PS_SOLID, s(1).max(1), track_border);
    let ob = SelectObject(hdc, HGDIOBJ(t_brush.0));
    let op = SelectObject(hdc, HGDIOBJ(t_pen.0));
    let _ = RoundRect(
        hdc,
        rc.left,
        track_y,
        rc.left + track_w,
        track_y + track_h,
        track_r,
        track_r,
    );
    SelectObject(hdc, ob);
    SelectObject(hdc, op);
    let _ = DeleteObject(HGDIOBJ(t_brush.0));
    let _ = DeleteObject(HGDIOBJ(t_pen.0));

    // Sliding knob (circle).
    let knob_pad = s(3);
    let knob_d = track_h - knob_pad * 2;
    let knob_y = track_y + knob_pad;
    let knob_x = if on {
        rc.left + track_w - knob_pad - knob_d
    } else {
        rc.left + knob_pad
    };

    let k_brush = CreateSolidBrush(TEXT);
    let k_pen = CreatePen(PS_SOLID, 0, TEXT); // outline matches fill
    let ob2 = SelectObject(hdc, HGDIOBJ(k_brush.0));
    let op2 = SelectObject(hdc, HGDIOBJ(k_pen.0));
    let _ = Ellipse(hdc, knob_x, knob_y, knob_x + knob_d, knob_y + knob_d);
    SelectObject(hdc, ob2);
    SelectObject(hdc, op2);
    let _ = DeleteObject(HGDIOBJ(k_brush.0));
    let _ = DeleteObject(HGDIOBJ(k_pen.0));

    // "On" / "Off" label.
    let label = if on { "On" } else { "Off" };
    let label_color = if on { OK_TEXT } else { MUTED };
    let mut text_rc = RECT {
        left: rc.left + track_w + s(6),
        top: rc.top,
        right: rc.right,
        bottom: rc.bottom,
    };
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, label_color);
    let mut wide: Vec<u16> = label.encode_utf16().collect();
    DrawTextW(
        hdc,
        &mut wide,
        &mut text_rc,
        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
    );
}

// ===========================================================================
// click handling (WM_COMMAND / BN_CLICKED dispatch)
// ===========================================================================

/// Handle a click on a ColorButton, ToggleSwitch or CornerDropdown.
///
/// Call from the parent's WM_COMMAND handler when a BN_CLICKED notification
/// arrives for a control created by `create_color`, `create_toggle` or `create_corner`.
pub(crate) unsafe fn on_click(hwnd: HWND) {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OwnerData;
    if ptr.is_null() {
        return;
    }
    // Popups run their own message loop (which repaints this control), so copy the value out
    // first and write the result back after, instead of holding `&mut *ptr` across the loop.
    let next = match *ptr {
        OwnerData::Color(c) => pick_color(hwnd, c).map(OwnerData::Color),
        OwnerData::Toggle(on) => Some(OwnerData::Toggle(!on)),
        OwnerData::Corner(i) => choose_from_list(hwnd, &CORNERS, i).map(OwnerData::Corner),
    };
    // Belt and braces: only write back if the control (and its data) survived the popup.
    let still_ours = IsWindow(hwnd).as_bool() && GetWindowLongPtrW(hwnd, GWLP_USERDATA) == ptr as isize;
    if let (Some(v), true) = (next, still_ours) {
        *ptr = v;
    }
    let _ = InvalidateRect(hwnd, None, true);
}

// ===========================================================================
// cleanup
// ===========================================================================

/// Free heap data for a ColorButton, ToggleSwitch or CornerDropdown.
///
/// Call this before `DestroyWindow` on controls created by `create_color`,
/// `create_toggle` or `create_corner`. Do NOT call on other control types
/// (HotkeyCapture handles its own cleanup; NumberSpinner has no heap data).
pub(crate) unsafe fn cleanup(hwnd: HWND) {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if ptr != 0 {
        let _ = Box::from_raw(ptr as *mut OwnerData);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
    }
}

// ===========================================================================
// shared dark popup (corner list, color picker, model picker)
// ===========================================================================

/// How a popup's modal loop ended.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PopupEnd {
    /// The popup set its own "done" flag (a click on an item, or focus moved away).
    Done,
    Enter,
    Escape,
}

/// Creates a borderless, topmost, dark-chromed, capture-excluded popup at (x, y).
pub(crate) unsafe fn make_popup(class: PCWSTR, owner: HWND, x: i32, y: i32, w: i32, h: i32) -> HWND {
    let hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        class,
        w!(""),
        WS_POPUP,
        x, y, w, h,
        owner,
        None,
        GetModuleHandleW(None).unwrap_or_default(),
        None,
    )
    .unwrap_or_default();
    for (attr, val) in [
        (DWMWA_USE_IMMERSIVE_DARK_MODE, 1u32),
        (DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND),
        (DWMWA_BORDER_COLOR, BORDER.0),
    ] {
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(attr), &val as *const u32 as *const _, 4);
    }
    let capturable = cfg!(feature = "debug") && std::env::var_os("SHADOWPROMPT_CAPTURABLE").is_some();
    if !capturable {
        let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
    }
    hwnd
}

/// Top-left for a `w`×`h` popup under `anchor` (screen coords), flipped above / clamped to stay
/// on the anchor's monitor.
pub fn place_below(anchor: RECT, w: i32, h: i32) -> (i32, i32) {
    unsafe {
        let mid = POINT { x: (anchor.left + anchor.right) / 2, y: (anchor.top + anchor.bottom) / 2 };
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        let _ = GetMonitorInfoW(MonitorFromPoint(mid, MONITOR_DEFAULTTONEAREST), &mut mi);
        let wa = mi.rcWork;
        let mut y = anchor.bottom + s(2);
        if y + h > wa.bottom {
            y = anchor.top - s(2) - h;
        }
        let x = anchor.left.min(wa.right - w).max(wa.left);
        (x, y.max(wa.top))
    }
}

/// Shows `hwnd`, pumps messages until `done()`, Enter or Escape, then destroys it.
pub(crate) unsafe fn run_popup(hwnd: HWND, done: impl Fn() -> bool) -> PopupEnd {
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = SetForegroundWindow(hwnd);
    let mut msg = MSG::default();
    let mut end = PopupEnd::Done;
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        if msg.message == WM_KEYDOWN && (msg.wParam.0 == VK_ESCAPE || msg.wParam.0 == VK_RETURN) {
            end = if msg.wParam.0 == VK_ESCAPE { PopupEnd::Escape } else { PopupEnd::Enter };
            break;
        }
        // A click outside closes the popup and is swallowed, like a native dropdown. Passing it
        // on would run e.g. a tab switch inside this loop, destroying the control that opened us.
        let outside = msg.hwnd != hwnd && !IsChild(hwnd, msg.hwnd).as_bool();
        if outside && matches!(msg.message, 0x0201 | 0x0203 | 0x0204 | 0x0207 | 0x00A1 | 0x00A4) {
            break;
        }
        if done() {
            break;
        }
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
        if done() {
            break;
        }
    }
    // Our loop swallowed a WM_QUIT meant for the owner's loop; hand it back.
    if msg.message == WM_QUIT {
        PostQuitMessage(msg.wParam.0 as i32);
    }
    let _ = DestroyWindow(hwnd);
    end
}

fn register_popup_class(name: PCWSTR, proc_: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT) {
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(proc_),
            hInstance: GetModuleHandleW(None).unwrap_or_default().into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            lpszClassName: name,
            ..Default::default()
        };
        RegisterClassExW(&wc); // fails harmlessly if already registered
    }
}

/// Paints into an off-screen bitmap, then copies it to the window in one go (no flicker).
unsafe fn paint_buffered(hwnd: HWND, draw: impl FnOnce(HDC, RECT)) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut rc = RECT::default();
    let _ = GetClientRect(hwnd, &mut rc);
    let mem = CreateCompatibleDC(hdc);
    let bmp = CreateCompatibleBitmap(hdc, rc.right, rc.bottom);
    let old = SelectObject(mem, HGDIOBJ(bmp.0));
    draw(mem, rc);
    let _ = BitBlt(hdc, 0, 0, rc.right, rc.bottom, mem, 0, 0, SRCCOPY);
    SelectObject(mem, old);
    let _ = DeleteObject(HGDIOBJ(bmp.0));
    let _ = DeleteDC(mem);
    let _ = EndPaint(hwnd, &ps);
}

unsafe fn fill_rect(hdc: HDC, rc: RECT, color: COLORREF) {
    let b = CreateSolidBrush(color);
    FillRect(hdc, &rc, b);
    let _ = DeleteObject(HGDIOBJ(b.0));
}

fn field_font(field: HWND) -> HFONT {
    unsafe { HFONT(SendMessageW(field, WM_GETFONT, WPARAM(0), LPARAM(0)).0 as *mut _) }
}

fn mouse_xy(lp: LPARAM) -> (i32, i32) {
    ((lp.0 & 0xFFFF) as i16 as i32, ((lp.0 >> 16) & 0xFFFF) as i16 as i32)
}

// ---- list popup (corner picker) -------------------------------------------

const LIST_ROW_H: i32 = 26;

struct ListPop {
    hwnd: HWND,
    items: &'static [&'static str],
    current: usize,
    hover: Option<usize>,
    chosen: Option<usize>,
    done: bool,
    font: HFONT,
}

thread_local! {
    static LIST_POP: std::cell::Cell<*mut ListPop> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// Opens a dark list under `field`; returns the clicked (or Enter-ed) index, None if dismissed.
pub(crate) unsafe fn choose_from_list(field: HWND, items: &'static [&'static str], current: usize) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    register_popup_class(w!("SysListPop"), list_proc);
    let mut anchor = RECT::default();
    let _ = GetWindowRect(field, &mut anchor);
    let w = anchor.right - anchor.left;
    let h = items.len() as i32 * s(LIST_ROW_H) + s(8);
    let (x, y) = place_below(anchor, w, h);
    let state = Box::into_raw(Box::new(ListPop {
        hwnd: HWND::default(),
        items,
        current,
        hover: Some(current),
        chosen: None,
        done: false,
        font: field_font(field),
    }));
    debug_assert!(LIST_POP.with(|c| c.get()).is_null(), "list popup already open");
    LIST_POP.with(|c| c.set(state));
    let hwnd = make_popup(w!("SysListPop"), GetAncestor(field, GA_ROOT), x, y, w, h);
    (*state).hwnd = hwnd;
    let end = run_popup(hwnd, || (*state).done);
    let p = Box::from_raw(LIST_POP.with(|c| c.replace(std::ptr::null_mut())));
    match end {
        PopupEnd::Done => p.chosen,
        PopupEnd::Enter => p.hover,
        PopupEnd::Escape => None,
    }
}

fn list_row_at(p: &ListPop, y: i32) -> Option<usize> {
    let i = (y - s(4)).div_euclid(s(LIST_ROW_H));
    (i >= 0 && (i as usize) < p.items.len()).then_some(i as usize)
}

unsafe extern "system" fn list_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(p) = LIST_POP.with(|c| c.get()).as_mut().filter(|p| p.hwnd == hwnd) else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_PAINT => {
            paint_buffered(hwnd, |hdc, rc| {
                fill_rect(hdc, rc, BG);
                for (i, item) in p.items.iter().enumerate() {
                    let top = s(4) + i as i32 * s(LIST_ROW_H);
                    let row = RECT { left: s(4), top, right: rc.right - s(4), bottom: top + s(LIST_ROW_H) };
                    if p.hover == Some(i) {
                        round_rect(hdc, row, Some(SURFACE_HOVER), SURFACE_HOVER, s(4));
                    }
                    let trc = RECT { left: row.left + s(8), right: row.right - s(24), ..row };
                    draw_label(hdc, p.font, if i == p.current { TEXT } else { MUTED }, item, trc);
                    if i == p.current {
                        let crc = RECT { left: row.right - s(22), right: row.right - s(4), ..row };
                        draw_label(hdc, icon_font(), ACCENT, &I_CHECK.to_string(), crc);
                    }
                }
            });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_MOUSEMOVE => {
            let hover = list_row_at(p, mouse_xy(lp).1);
            if hover.is_some() && hover != p.hover {
                p.hover = hover;
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            if let Some(i) = list_row_at(p, mouse_xy(lp).1) {
                p.chosen = Some(i);
                p.done = true;
            }
            LRESULT(0)
        }
        WM_KEYDOWN if wp.0 == VK_UP || wp.0 == VK_DOWN => {
            let n = p.items.len();
            let i = p.hover.unwrap_or(p.current);
            p.hover = Some(if wp.0 == VK_UP { (i + n - 1) % n } else { (i + 1) % n });
            let _ = InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        WM_ACTIVATE if wp.0 & 0xFFFF == WA_INACTIVE => {
            p.done = true;
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

// ---- color popup ----------------------------------------------------------

const CP_PAD: i32 = 10;
const CP_W: i32 = 236; // inner width (SV square, hue bar, swatch row)
const CP_SV_H: i32 = 140;
const CP_HUE_H: i32 = 14;
const CP_SWATCH: i32 = 20;
const CP_ROW_H: i32 = 26;
const CP_GAP: i32 = 10;
const ID_HEX: i32 = 1;

const PRESETS: [&str; 10] = [
    "#FF0000", "#FF8800", "#FFFF00", "#00FF00", "#00C8FF",
    "#0066FF", "#8B5CF6", "#FF3DA5", "#FFFFFF", "#808080",
];

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    None,
    Sv,
    Hue,
}

struct ColorPop {
    hwnd: HWND,
    hex_hwnd: HWND,
    hue: f32, // 0..360
    sat: f32, // 0..1
    val: f32, // 0..1
    drag: Drag,
    syncing: bool, // true while we write the hex box ourselves
    done: bool,
    font: HFONT,
    field_brush: HBRUSH,
}

thread_local! {
    static COLOR_POP: std::cell::Cell<*mut ColorPop> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

// Layout rects in popup client coords.
fn cp_sv() -> RECT {
    RECT { left: s(CP_PAD), top: s(CP_PAD), right: s(CP_PAD) + s(CP_W), bottom: s(CP_PAD) + s(CP_SV_H) }
}
fn cp_hue() -> RECT {
    let top = cp_sv().bottom + s(CP_GAP);
    RECT { left: s(CP_PAD), top, right: s(CP_PAD) + s(CP_W), bottom: top + s(CP_HUE_H) }
}
fn cp_swatch(i: usize) -> RECT {
    let top = cp_hue().bottom + s(CP_GAP);
    let n = PRESETS.len() as i32;
    let gap = (s(CP_W) - n * s(CP_SWATCH)) / (n - 1);
    let left = s(CP_PAD) + i as i32 * (s(CP_SWATCH) + gap);
    RECT { left, top, right: left + s(CP_SWATCH), bottom: top + s(CP_SWATCH) }
}
fn cp_bottom_top() -> i32 {
    cp_swatch(0).bottom + s(CP_GAP)
}
fn cp_preview() -> RECT {
    let top = cp_bottom_top();
    RECT { left: s(CP_PAD), top, right: s(CP_PAD) + s(CP_ROW_H) * 2, bottom: top + s(CP_ROW_H) }
}
fn cp_hex() -> RECT {
    let p = cp_preview();
    RECT { left: p.right + s(8), top: p.top, right: p.right + s(8) + s(96), bottom: p.bottom }
}
fn cp_done() -> RECT {
    let p = cp_preview();
    RECT { left: s(CP_PAD) + s(CP_W) - s(64), top: p.top, right: s(CP_PAD) + s(CP_W), bottom: p.bottom }
}
fn cp_size() -> (i32, i32) {
    (s(CP_W) + 2 * s(CP_PAD), cp_preview().bottom + s(CP_PAD))
}

fn hit(rc: RECT, x: i32, y: i32) -> bool {
    x >= rc.left && x < rc.right && y >= rc.top && y < rc.bottom
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let hp = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    let to = |f: f32| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to(r), to(g), to(b))
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let d = max - r.min(g).min(b);
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let sat = if max == 0.0 { 0.0 } else { d / max };
    (h, sat, max)
}

fn parse_hex(s: &str) -> Option<COLORREF> {
    let h = s.trim().trim_start_matches('#');
    (h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit())).then(|| hex_to_colorref(h))
}

impl ColorPop {
    fn color(&self) -> COLORREF {
        let (r, g, b) = hsv_to_rgb(self.hue, self.sat, self.val);
        rgb(r, g, b)
    }
    fn set_color(&mut self, c: COLORREF) {
        let (h, s_, v) = rgb_to_hsv((c.0 & 0xFF) as u8, ((c.0 >> 8) & 0xFF) as u8, ((c.0 >> 16) & 0xFF) as u8);
        // Greys have no hue; keep the current one so the hue bar doesn't jump to red.
        if s_ > 0.0 {
            self.hue = h;
        }
        self.sat = s_;
        self.val = v;
    }
    /// Writes the hex box from the current colour (unless the user is the one typing in it).
    unsafe fn sync_hex(&mut self) {
        self.syncing = true;
        let wide: Vec<u16> = colorref_to_hex(self.color()).encode_utf16().chain([0]).collect();
        let _ = SetWindowTextW(self.hex_hwnd, PCWSTR(wide.as_ptr()));
        self.syncing = false;
    }
    unsafe fn drag_to(&mut self, x: i32, y: i32) {
        match self.drag {
            Drag::Sv => {
                let rc = cp_sv();
                self.sat = ((x - rc.left) as f32 / (rc.right - rc.left - 1) as f32).clamp(0.0, 1.0);
                self.val = 1.0 - ((y - rc.top) as f32 / (rc.bottom - rc.top - 1) as f32).clamp(0.0, 1.0);
            }
            Drag::Hue => {
                let rc = cp_hue();
                self.hue = ((x - rc.left) as f32 / (rc.right - rc.left - 1) as f32).clamp(0.0, 1.0) * 359.9;
            }
            Drag::None => return,
        }
        self.sync_hex();
        let _ = InvalidateRect(self.hwnd, None, false);
    }
}

/// Dark color picker under `field`. Enter / Done / clicking away keep the colour, Escape cancels.
pub(crate) unsafe fn pick_color(field: HWND, initial: COLORREF) -> Option<COLORREF> {
    register_popup_class(w!("SysColorPop"), color_proc);
    let mut anchor = RECT::default();
    let _ = GetWindowRect(field, &mut anchor);
    let (w, h) = cp_size();
    let (x, y) = place_below(anchor, w, h);
    let state = Box::into_raw(Box::new(ColorPop {
        hwnd: HWND::default(),
        hex_hwnd: HWND::default(),
        hue: 0.0,
        sat: 0.0,
        val: 0.0,
        drag: Drag::None,
        syncing: false,
        done: false,
        font: field_font(field),
        field_brush: CreateSolidBrush(FIELD_BG),
    }));
    debug_assert!(COLOR_POP.with(|c| c.get()).is_null(), "color popup already open");
    COLOR_POP.with(|c| c.set(state));
    let p = &mut *state;
    p.set_color(initial);
    p.hwnd = make_popup(w!("SysColorPop"), GetAncestor(field, GA_ROOT), x, y, w, h);
    let hr = cp_hex();
    let inset = s(5);
    p.hex_hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("EDIT"),
        PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(0x0080 /* ES_AUTOHSCROLL */),
        hr.left + s(6),
        hr.top + inset,
        hr.right - hr.left - s(12),
        hr.bottom - hr.top - 2 * inset,
        p.hwnd,
        HMENU(ID_HEX as isize as *mut _),
        GetModuleHandleW(None).unwrap_or_default(),
        None,
    )
    .unwrap_or_default();
    SendMessageW(p.hex_hwnd, WM_SETFONT, WPARAM(p.font.0 as usize), LPARAM(1));
    SendMessageW(p.hex_hwnd, 0x00C5 /* EM_LIMITTEXT */, WPARAM(7), LPARAM(0));
    p.sync_hex();

    let end = run_popup(p.hwnd, || (*state).done);
    let p = Box::from_raw(COLOR_POP.with(|c| c.replace(std::ptr::null_mut())));
    let _ = DeleteObject(HGDIOBJ(p.field_brush.0));
    (end != PopupEnd::Escape).then(|| p.color())
}

unsafe fn paint_color_pop(p: &ColorPop, hdc: HDC, rc: RECT) {
    fill_rect(hdc, rc, BG);

    // Saturation/brightness square and hue bar, rendered as 32-bit top-down DIBs.
    let blit = |r: RECT, px: &dyn Fn(i32, i32, i32, i32) -> (u8, u8, u8)| {
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        let mut buf = vec![0u32; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let (rr, gg, bb) = px(x, y, w, h);
                buf[(y * w + x) as usize] = (rr as u32) << 16 | (gg as u32) << 8 | bb as u32;
            }
        }
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        StretchDIBits(hdc, r.left, r.top, w, h, 0, 0, w, h, Some(buf.as_ptr() as *const _), &bmi, DIB_RGB_COLORS, SRCCOPY);
    };
    let hue = p.hue;
    let sv = cp_sv();
    blit(sv, &|x, y, w, h| hsv_to_rgb(hue, x as f32 / (w - 1) as f32, 1.0 - y as f32 / (h - 1) as f32));
    let hr = cp_hue();
    blit(hr, &|x, _, w, _| hsv_to_rgb(x as f32 / (w - 1) as f32 * 359.9, 1.0, 1.0));

    // Markers: ring on the square, notch on the hue bar.
    let mx = sv.left + (p.sat * (sv.right - sv.left - 1) as f32) as i32;
    let my = sv.top + ((1.0 - p.val) * (sv.bottom - sv.top - 1) as f32) as i32;
    let r = s(6);
    let ring = if p.val > 0.6 && p.sat < 0.4 { rgb(0, 0, 0) } else { rgb(255, 255, 255) };
    round_rect(hdc, RECT { left: mx - r, top: my - r, right: mx + r, bottom: my + r }, None, ring, 2 * r);
    let hx = hr.left + (p.hue / 359.9 * (hr.right - hr.left - 1) as f32) as i32;
    round_rect(hdc, RECT { left: hx - s(3), top: hr.top - s(2), right: hx + s(3), bottom: hr.bottom + s(2) }, None, TEXT, s(3));

    // Quick colours; the one matching the current colour gets a ring.
    let current = p.color();
    for (i, hex) in PRESETS.iter().enumerate() {
        let c = hex_to_colorref(hex);
        let sr = cp_swatch(i);
        round_rect(hdc, sr, Some(c), BORDER, s(4));
        if c == current {
            let o = RECT { left: sr.left - s(2), top: sr.top - s(2), right: sr.right + s(2), bottom: sr.bottom + s(2) };
            round_rect(hdc, o, None, TEXT, s(6));
        }
    }

    // Preview, hex box frame, Done button.
    round_rect(hdc, cp_preview(), Some(current), BORDER, s(4));
    round_rect(hdc, cp_hex(), Some(FIELD_BG), BORDER, s(4));
    let d = cp_done();
    round_rect(hdc, d, Some(ACCENT), ACCENT, s(6));
    let mut wide: Vec<u16> = "Done".encode_utf16().collect();
    let mut drc = d;
    SelectObject(hdc, HGDIOBJ(p.font.0));
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, rgb(255, 255, 255));
    DrawTextW(hdc, &mut wide, &mut drc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

unsafe extern "system" fn color_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(p) = COLOR_POP.with(|c| c.get()).as_mut().filter(|p| p.hwnd == hwnd) else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_PAINT => {
            paint_buffered(hwnd, |hdc, rc| paint_color_pop(p, hdc, rc));
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_LBUTTONDOWN => {
            let (x, y) = mouse_xy(lp);
            if hit(cp_sv(), x, y) {
                p.drag = Drag::Sv;
            } else if hit(cp_hue(), x, y) {
                p.drag = Drag::Hue;
            } else if hit(cp_done(), x, y) {
                p.done = true;
                return LRESULT(0);
            } else if let Some(i) = (0..PRESETS.len()).find(|&i| hit(cp_swatch(i), x, y)) {
                p.set_color(hex_to_colorref(PRESETS[i]));
                p.sync_hex();
                let _ = InvalidateRect(hwnd, None, false);
                return LRESULT(0);
            } else {
                return LRESULT(0);
            }
            SetCapture(hwnd);
            p.drag_to(x, y);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let (x, y) = mouse_xy(lp);
            p.drag_to(x, y);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            p.drag = Drag::None;
            let _ = ReleaseCapture();
            LRESULT(0)
        }
        WM_CAPTURECHANGED => {
            p.drag = Drag::None;
            LRESULT(0)
        }
        WM_COMMAND => {
            let code = ((wp.0 >> 16) & 0xFFFF) as u32;
            if code == EN_CHANGE && !p.syncing {
                if let Some(c) = parse_hex(&read_window_text(p.hex_hwnd)) {
                    p.set_color(c);
                    let _ = InvalidateRect(hwnd, None, false);
                }
            }
            LRESULT(0)
        }
        WM_CTLCOLOREDIT => {
            let hdc = HDC(wp.0 as *mut _);
            SetTextColor(hdc, TEXT);
            SetBkColor(hdc, FIELD_BG);
            LRESULT(p.field_brush.0 as isize)
        }
        WM_ACTIVATE if wp.0 & 0xFFFF == WA_INACTIVE => {
            // A drag that leaves the window keeps capture, so only a real focus change lands here.
            p.done = true;
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips_through_rgb() {
        for hex in ["#FF0000", "#00FF00", "#808080", "#D97757", "#123456", "#FFFFFF", "#000000"] {
            let c = hex_to_colorref(hex);
            let (h, s_, v) = rgb_to_hsv((c.0 & 0xFF) as u8, ((c.0 >> 8) & 0xFF) as u8, ((c.0 >> 16) & 0xFF) as u8);
            let (r, g, b) = hsv_to_rgb(h, s_, v);
            assert_eq!(colorref_to_hex(rgb(r, g, b)), hex);
        }
    }

    #[test]
    fn hex_box_only_accepts_six_hex_digits() {
        assert_eq!(parse_hex("#00ff00"), Some(rgb(0, 255, 0)));
        assert_eq!(parse_hex("00FF0"), None);
        assert_eq!(parse_hex("#GG0000"), None);
    }
}
