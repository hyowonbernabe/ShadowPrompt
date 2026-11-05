// Launch dialog + account screen for the access-code login.
// Design: docs/superpowers/specs/2026-11-06-access-code-login-design.md.
//
// Same stack as the rest of the UI: plain Win32 + GDI, painted by us (no web view, no UI
// framework). Native EDIT controls for typing (so IME, paste, selection all just work) and
// owner-drawn buttons. Dark title/border/rounded corners come from DWM (Windows 11; on Windows 10
// the window is simply square). No taskbar entry, excluded from screen capture like the overlay.
//
// Stealth note: this is a visible window, justified because it only appears (a) on a launch the
// user starts themself, before any test, or (b) on the user's own Account hotkey. Restarts the
// app performs itself skip it (lifecycle::self_restart::RESUME_ENV).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, Ellipse, EndPaint,
    FillRect, GetDC, GetMonitorInfoW, GetStockObject, InvalidateRect, MonitorFromPoint, ReleaseDC,
    RoundRect, SelectObject, SetBkColor, SetBkMode, SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DEFAULT_PITCH, DRAW_TEXT_FORMAT, DT_CALCRECT, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE,
    DT_VCENTER, DT_WORDBREAK, FF_DONTCARE, HBRUSH, HDC, HFONT, HGDIOBJ, MONITORINFO,
    MONITOR_DEFAULTTONEAREST, NULL_BRUSH, OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, GetFocus, SetFocus};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
    GetCursorPos, GetMessageW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsDialogMessageW,
    KillTimer, LoadCursorW, MoveWindow, PostMessageW, PostQuitMessage, RegisterClassExW, SendMessageW,
    SetForegroundWindow, SetTimer, SetWindowDisplayAffinity, ShowWindow, TranslateMessage,
    WindowFromPoint, HMENU, IDC_ARROW, MSG, SW_SHOW, WDA_EXCLUDEFROMCAPTURE, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WM_PAINT, WNDCLASSEXW, WS_CHILD, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    WS_TABSTOP, WS_VISIBLE,
};

use crate::account;
use crate::config::schema::AccountConfig;

// ---- what the launch dialog decided ------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchChoice {
    /// Run with this key. `from_account` = it came from a code login (vs. the config's own key).
    Use { key: String, from_account: bool },
    /// The user closed the dialog: exit without starting.
    Quit,
}

/// Shows the launch dialog on the calling thread and blocks until the user picks something.
/// Dev preview (`cargo run --example dialog_preview --features debug -- <launch|account>
/// [logged-in] [has-key]`): shows the dialog alone, with a made-up account, without starting
/// the app — no single-instance lock, no hotkeys, no clipboard. Buttons that talk to the server
/// still do.
#[doc(hidden)]
pub fn preview(account_screen: bool, logged_in: bool, has_key: bool) {
    let account = if logged_in {
        AccountConfig { code: "preview-2244644".into(), api_key: "sk-or-v1-preview".into() }
    } else {
        AccountConfig::default()
    };
    let key = if has_key { "sk-or-v1-preview-config" } else { "" };
    let mode = if account_screen { Mode::Account } else { Mode::Launch };
    let r = run(mode, account, key.into());
    println!("result: {r:?}");
}

pub fn run_launch(account: &AccountConfig, config_key: &str) -> LaunchChoice {
    run(Mode::Launch, account.clone(), config_key.to_string()).unwrap_or(LaunchChoice::Quit)
}

static ACCOUNT_OPEN: AtomicBool = AtomicBool::new(false);

/// Opens the account screen on its own thread (it has its own message loop). A second press of
/// the hotkey while it's open does nothing.
pub fn open_account_screen() {
    if ACCOUNT_OPEN.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        // Re-read from disk: a code changed earlier in this session is only in the file.
        let account = crate::config::load::load().map(|c| c.account).unwrap_or_default();
        let _ = run(Mode::Account, account, String::new());
        ACCOUNT_OPEN.store(false, Ordering::SeqCst);
    });
}

// ---- look ---------------------------------------------------------------------------------------

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}
const BG: COLORREF = rgb(0x1F, 0x1E, 0x1D);
const SURFACE: COLORREF = rgb(0x30, 0x30, 0x2E);
const SURFACE_HOVER: COLORREF = rgb(0x38, 0x37, 0x35);
const SURFACE_DOWN: COLORREF = rgb(0x42, 0x41, 0x3D);
const FIELD_BG: COLORREF = rgb(0x26, 0x26, 0x24);
const BORDER: COLORREF = rgb(0x3E, 0x3D, 0x39);
const TEXT: COLORREF = rgb(0xF5, 0xF4, 0xEE);
const MUTED: COLORREF = rgb(0xA8, 0xA6, 0x9D);
const DIM: COLORREF = rgb(0x6E, 0x6C, 0x66);
const ACCENT: COLORREF = rgb(0xD9, 0x77, 0x57);
const ACCENT_HOVER: COLORREF = rgb(0xE0, 0x84, 0x65);
const ACCENT_DOWN: COLORREF = rgb(0xC2, 0x61, 0x3F);
const ACCENT_RING: COLORREF = rgb(0xF6, 0xC9, 0xB8);
const DOT_OFF: COLORREF = rgb(0xEC, 0xA8, 0x91);
const ON_ACCENT: COLORREF = rgb(0xFF, 0xFF, 0xFF);
const DANGER: COLORREF = rgb(0xD9, 0x5A, 0x50);
const DANGER_DOWN: COLORREF = rgb(0xC0, 0x48, 0x3F);
const ERROR_TEXT: COLORREF = rgb(0xEC, 0x86, 0x7A);
const OK_TEXT: COLORREF = rgb(0x8F, 0xC2, 0x7E);

// Layout, in 96-dpi pixels (scaled by `s`).
const WIDTH: i32 = 264;
const PAD: i32 = 14;
const TOP: i32 = 12;
const TITLE_H: i32 = 20;
const RADIUS: i32 = 6;
const ROW_H: i32 = 28;
const MAIN_H: i32 = 30;
const FIELD_H: i32 = 28;
const LABEL_H: i32 = 15;
const FOOT_W: i32 = 72;
const FOOT_MAIN_W: i32 = 100;
const GAP: i32 = 6;
const X_SIZE: i32 = 22;

// Segoe MDL2 Assets glyphs (built into Windows 10 and 11).
const I_PERSON: char = '\u{E77B}';
const I_KEY: char = '\u{E8D7}';
const I_LOCK: char = '\u{E72E}';
const I_ADD_PERSON: char = '\u{E8FA}';
const I_CHECK: char = '\u{E73E}';
const I_RENAME: char = '\u{E8AC}';
const I_SIGN_OUT: char = '\u{E7E8}';
const I_DELETE: char = '\u{E74D}';
const I_CLOSE: char = '\u{E711}';
/// Skull and crossbones (Segoe UI Symbol), shown in place of the app name.
const SKULL: char = '\u{2620}';
const I_ERROR: char = '\u{E783}';
const NO_ICON: char = '\0';

fn s(px: i32) -> i32 {
    // SAFETY: plain Win32 query, no arguments.
    let dpi = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() }.max(96) as i32;
    px * dpi / 96
}

// Win32 values used below, spelled out so no extra `windows` features are needed.
const WM_CLOSE: u32 = 0x0010;
const WM_SETCURSOR: u32 = 0x0020;
const WM_SETFONT: u32 = 0x0030;
const WM_DRAWITEM: u32 = 0x002B;
const WM_NCHITTEST: u32 = 0x0084;
const WM_COMMAND: u32 = 0x0111;
const WM_TIMER: u32 = 0x0113;
const WM_CHANGEUISTATE: u32 = 0x0127;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const EM_SETSEL: u32 = 0x00B1;
const EM_SETREADONLY: u32 = 0x00CF;
const WM_APP_DONE: u32 = 0x8000 + 1;
const HTCLIENT: isize = 1;
const HTCAPTION: isize = 2;
const IDOK: i32 = 1;
const IDCANCEL: i32 = 2;
const BN_CLICKED: u32 = 0;
const EN_SETFOCUS: u32 = 0x0100;
const EN_KILLFOCUS: u32 = 0x0200;
const EN_CHANGE: u32 = 0x0300;
const BS_OWNERDRAW: u32 = 0x000B;
const ES_PASSWORD: u32 = 0x0020;
const ES_AUTOHSCROLL: u32 = 0x0080;
const ODS_SELECTED: u32 = 0x0001;
const ODS_DISABLED: u32 = 0x0004;
const ODS_FOCUS: u32 = 0x0010;
const ODS_NOFOCUSRECT: u32 = 0x0200;
/// MAKEWPARAM(UIS_SET, UISF_HIDEFOCUS): hide keyboard focus cues until the keyboard is used.
const HIDE_FOCUS_CUES: usize = 0x0001_0001;
const DWMWA_USE_IMMERSIVE_DARK_MODE: i32 = 20;
const DWMWA_WINDOW_CORNER_PREFERENCE: i32 = 33;
const DWMWA_BORDER_COLOR: i32 = 34;
const DWMWCP_ROUND: u32 = 2;
const CLEARTYPE_QUALITY: u32 = 5;

// Timers
const T_SPIN: usize = 1; // animates the working dots on the busy button
const T_HOVER: usize = 2; // notices the mouse leaving a hovered button
const T_AFTER: usize = 3; // short pause on a success mark before closing / restarting
const T_DISARM: usize = 4; // an armed Delete quietly disarms itself

/// DRAWITEMSTRUCT, laid out by hand (saves pulling in the Win32_UI_Controls feature for one struct).
#[repr(C)]
struct DrawItem {
    ctl_type: u32,
    ctl_id: u32,
    item_id: u32,
    item_action: u32,
    item_state: u32,
    hwnd_item: HWND,
    hdc: HDC,
    rc: RECT,
    item_data: usize,
}

// ---- state ----------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    Launch,
    Account,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Page {
    Home,
    Login,
    Signup,
    ChangeKey,
    ChangeCode,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Style {
    /// The page's one main action: filled accent.
    Primary,
    /// A menu row: flat until hovered.
    Row,
    /// A destructive menu row: red text; red fill once armed.
    Danger,
    /// A footer button next to the primary one (Back).
    Plain,
}

#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Info,
    Error,
    Ok,
}

// Button ids
const B_CONT_ACCOUNT: i32 = 101;
const B_CONT_CONFIG: i32 = 102;
const B_GO_LOGIN: i32 = 103;
const B_GO_SIGNUP: i32 = 104;
const B_DO_LOGIN: i32 = 105;
const B_DO_SIGNUP: i32 = 106;
const B_BACK: i32 = 107;
const B_GO_CHKEY: i32 = 108;
const B_GO_CHCODE: i32 = 109;
const B_LOGOUT: i32 = 110;
const B_DELETE: i32 = 111;
const B_CLOSE: i32 = 112;
const B_SAVE_KEY: i32 = 113;
const B_SAVE_CODE: i32 = 114;
const B_X: i32 = 115;
// Field ids
const F_CODE: i32 = 201;
const F_KEY: i32 = 202;

struct Field {
    id: i32,
    label: &'static str,
    /// Small grey help under the box; an error takes its place.
    hint: &'static str,
    error: String,
    hwnd: HWND,
    label_y: i32,
    box_rc: RECT,
    note_rc: RECT,
}

struct Btn {
    id: i32,
    text: String,
    style: Style,
    icon: char,
    /// Sits in the bottom-right footer row instead of the full-width list.
    footer: bool,
    hwnd: HWND,
}

/// What happens once a success mark has been shown for a moment.
enum After {
    Finish(LaunchChoice),
    Restart(bool),
}

struct Dlg {
    hwnd: HWND,
    mode: Mode,
    page: Page,
    account: AccountConfig,
    config_key: String,
    title: String,
    subtitle: String,
    status: String,
    tone: Tone,
    fields: Vec<Field>,
    buttons: Vec<Btn>,
    /// The button whose job is running (shows working dots); `None` = idle.
    busy: Option<i32>,
    busy_label: &'static str,
    /// The busy button's label once its job succeeded (shown with a check).
    done_label: Option<&'static str>,
    tick: u32,
    hover: HWND,
    after: Option<After>,
    /// Delete needs two clicks; this is set by the first.
    delete_armed: bool,
    font_title: HFONT,
    /// Segoe UI Symbol, for the skull that stands in for the app name.
    font_skull: HFONT,
    font_body: HFONT,
    font_small: HFONT,
    font_icon: HFONT,
    font_icon_small: HFONT,
    subtitle_rc: RECT,
    status_rc: RECT,
    divider_y: Option<i32>,
    bg_brush: HBRUSH,
    field_brush: HBRUSH,
    result: Option<LaunchChoice>,
}

thread_local! {
    /// This thread's dialog (each dialog runs its own message loop on its own thread).
    static DLG: std::cell::Cell<*mut Dlg> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// The current thread's dialog state; only ever touched from that thread's window procedure.
fn dlg() -> Option<&'static mut Dlg> {
    // SAFETY: the pointer comes from `Box::into_raw` in `run` and is freed only after that
    // thread's message loop has ended, so it is valid whenever a window message can arrive.
    unsafe { DLG.with(|c| c.get()).as_mut() }
}

/// Result of a server call, handed from the worker thread back to the window.
enum Done {
    Saved { code: String, key: String },
    KeyChanged { key: String },
    CodeChanged { code: String },
    Deleted,
    /// Message, and the field it's about (shown under that field), if any.
    Fail(String, Option<i32>),
}

// ---- window ---------------------------------------------------------------------------------------

fn run(mode: Mode, account: AccountConfig, config_key: String) -> Option<LaunchChoice> {
    static REGISTER: Once = Once::new();
    unsafe {
        let hinstance = GetModuleHandleW(None).ok()?;
        REGISTER.call_once(|| {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                lpszClassName: w!("SysDialogHost"),
                ..Default::default()
            };
            RegisterClassExW(&wc);
        });

        let font = |size: i32, weight: i32, face: PCWSTR| {
            CreateFontW(
                -s(size), 0, 0, 0, weight, 0, 0, 0,
                DEFAULT_CHARSET.0 as u32, OUT_DEFAULT_PRECIS.0 as u32, CLIP_DEFAULT_PRECIS.0 as u32,
                CLEARTYPE_QUALITY, (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32, face,
            )
        };
        let has_config_key = account::is_real_key(&config_key);
        let state = Box::into_raw(Box::new(Dlg {
            hwnd: HWND::default(),
            mode,
            page: Page::Home,
            account,
            config_key: if has_config_key { config_key } else { String::new() },
            title: String::new(),
            subtitle: String::new(),
            status: String::new(),
            tone: Tone::Info,
            fields: Vec::new(),
            buttons: Vec::new(),
            busy: None,
            busy_label: "",
            done_label: None,
            tick: 0,
            hover: HWND::default(),
            after: None,
            delete_armed: false,
            font_title: font(14, 600, w!("Segoe UI")),
            font_skull: font(17, 400, w!("Segoe UI Symbol")),
            font_body: font(12, 400, w!("Segoe UI")),
            font_small: font(11, 400, w!("Segoe UI")),
            font_icon: font(12, 400, w!("Segoe MDL2 Assets")),
            font_icon_small: font(10, 400, w!("Segoe MDL2 Assets")),
            subtitle_rc: RECT::default(),
            status_rc: RECT::default(),
            divider_y: None,
            bg_brush: CreateSolidBrush(BG),
            field_brush: CreateSolidBrush(FIELD_BG),
            result: None,
        }));
        DLG.with(|c| c.set(state));

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            // Neutral names, like the overlay's: nothing on screen or in window lists says ShadowPrompt.
            w!("SysDialogHost"),
            w!(""),
            WS_POPUP,
            0, 0, s(WIDTH), s(200),
            None, None, hinstance, None,
        )
        .ok()?;
        let d = dlg()?;
        d.hwnd = hwnd;

        // Dark, rounded, thin border (Windows 11). Failures just mean a plainer window.
        let set = |attr: i32, val: u32| {
            let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(attr), &val as *const u32 as *const _, 4);
        };
        set(DWMWA_USE_IMMERSIVE_DARK_MODE, 1);
        set(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND);
        set(DWMWA_BORDER_COLOR, BORDER.0);
        // Same capture exclusion as the overlay: never shows up in screenshots or screen shares.
        // Dev builds can opt out (SHADOWPROMPT_CAPTURABLE) so the dialog can be screenshotted
        // while working on its look; release builds always exclude it.
        let capturable = cfg!(feature = "debug") && std::env::var_os("SHADOWPROMPT_CAPTURABLE").is_some();
        if !capturable {
            let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
        }
        // Like a native dialog: focus rings on buttons only appear once the keyboard is used.
        SendMessageW(hwnd, WM_CHANGEUISTATE, WPARAM(HIDE_FOCUS_CUES), LPARAM(0));

        build_page(d, Place::Center);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);
        focus_first(d);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            // Tab / Shift+Tab between fields and buttons, Enter = primary action, Esc = back.
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        let d = Box::from_raw(DLG.with(|c| c.replace(std::ptr::null_mut())));
        for obj in [
            HGDIOBJ(d.font_title.0), HGDIOBJ(d.font_skull.0), HGDIOBJ(d.font_body.0), HGDIOBJ(d.font_small.0), HGDIOBJ(d.font_icon.0),
            HGDIOBJ(d.font_icon_small.0), HGDIOBJ(d.bg_brush.0), HGDIOBJ(d.field_brush.0),
        ] {
            let _ = DeleteObject(obj);
        }
        d.result
    }
}

/// Page content: (title, subtitle, fields as (id, label, password, initial text, hint),
/// buttons as (id, text, style, icon, in footer)).
type PageSpec = (
    String,
    String,
    Vec<(i32, &'static str, bool, String, &'static str)>,
    Vec<(i32, String, Style, char, bool)>,
);

fn page_spec(d: &Dlg) -> PageSpec {
    let logged_in = d.account.logged_in();
    let who = account::masked(&d.account.code);
    let back = (B_BACK, "Back".to_string(), Style::Plain, NO_ICON, true);
    match (d.mode, d.page) {
        (Mode::Launch, Page::Home) => {
            let mut b = Vec::new();
            if logged_in {
                b.push((B_CONT_ACCOUNT, format!("Continue as {who}"), Style::Primary, I_PERSON, false));
            }
            if !d.config_key.is_empty() {
                let style = if logged_in { Style::Row } else { Style::Primary };
                b.push((B_CONT_CONFIG, "Continue with my key".to_string(), style, I_KEY, false));
            }
            let style = if b.is_empty() { Style::Primary } else { Style::Row };
            b.push((B_GO_LOGIN, "Log in with a code".to_string(), style, I_LOCK, false));
            b.push((B_GO_SIGNUP, "Sign up".to_string(), Style::Row, I_ADD_PERSON, false));
            (SKULL.to_string(), String::new(), vec![], b)
        }
        (Mode::Account, Page::Home) if logged_in => (
            "Account".into(),
            format!("Logged in as {who}"),
            vec![],
            vec![
                (B_GO_CHKEY, "Change API key".into(), Style::Row, I_KEY, false),
                (B_GO_CHCODE, "Change code".into(), Style::Row, I_RENAME, false),
                (B_LOGOUT, "Log out".into(), Style::Row, I_SIGN_OUT, false),
                (B_DELETE, "Delete account".into(), Style::Danger, I_DELETE, false),
                (B_CLOSE, "Done".into(), Style::Primary, NO_ICON, true),
            ],
        ),
        (Mode::Account, Page::Home) => (
            "Account".into(),
            "Not logged in. Using the key in config.toml.".into(),
            vec![],
            vec![
                (B_GO_LOGIN, "Log in with a code".into(), Style::Primary, I_LOCK, false),
                (B_GO_SIGNUP, "Sign up".into(), Style::Row, I_ADD_PERSON, false),
            ],
        ),
        (_, Page::Login) => (
            "Log in".into(),
            "Enter your code to load your key.".into(),
            vec![(F_CODE, "Code", true, String::new(), "")],
            vec![back, (B_DO_LOGIN, "Log in".into(), Style::Primary, NO_ICON, true)],
        ),
        (_, Page::Signup) => (
            "Sign up".into(),
            "Your code loads your key on any PC.".into(),
            vec![
                (F_CODE, "Code", false, String::new(), "Anyone with it can use your key."),
                (F_KEY, "OpenRouter API key", true, d.config_key.clone(), ""),
            ],
            vec![back, (B_DO_SIGNUP, "Create".into(), Style::Primary, NO_ICON, true)],
        ),
        (_, Page::ChangeKey) => (
            "Change API key".into(),
            format!("Replaces the key under {who} everywhere."),
            vec![(F_KEY, "New OpenRouter API key", true, String::new(), "")],
            vec![back, (B_SAVE_KEY, "Save".into(), Style::Primary, NO_ICON, true)],
        ),
        (_, Page::ChangeCode) => (
            "Change code".into(),
            format!("{who} stops working; the key moves over."),
            vec![(F_CODE, "New code", false, String::new(), "")],
            vec![back, (B_SAVE_CODE, "Save".into(), Style::Primary, NO_ICON, true)],
        ),
    }
}

/// Height `text` needs when wrapped to `width` in `font`.
fn measure(d: &Dlg, font: HFONT, text: &str, width: i32) -> i32 {
    if text.is_empty() {
        return 0;
    }
    unsafe {
        let hdc = GetDC(d.hwnd);
        let old = SelectObject(hdc, HGDIOBJ(font.0));
        let mut rc = RECT { left: 0, top: 0, right: width, bottom: 0 };
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(hdc, &mut wide, &mut rc, DT_LEFT | DT_WORDBREAK | DT_CALCRECT);
        SelectObject(hdc, old);
        ReleaseDC(d.hwnd, hdc);
        rc.bottom - rc.top
    }
}

fn inner_w() -> i32 {
    s(WIDTH) - 2 * s(PAD)
}

/// Room left of a status / field-error line for its small icon.
fn note_indent() -> i32 {
    s(15)
}

fn title_rc() -> RECT {
    RECT { left: s(PAD), top: s(TOP), right: s(WIDTH) - s(PAD) - s(X_SIZE), bottom: s(TOP) + s(TITLE_H) }
}

#[derive(Clone, Copy)]
enum Place {
    /// Centre on the work area of the monitor the mouse is on (first show).
    Center,
    /// Keep the window's centre where it is (page changes: grows/shrinks around the middle).
    KeepCenter,
    /// Keep the top edge (errors and status lines push the bottom down).
    KeepTop,
}

/// Positions every control for the current content (fields' errors, status line, armed delete)
/// and sizes the window to fit. Pure arithmetic + MoveWindow; controls are created in `build_page`.
fn layout(d: &mut Dlg, place: Place) {
    let x0 = s(PAD);
    let w = inner_w();
    let mut y = s(TOP) + s(TITLE_H);
    if d.subtitle.is_empty() {
        d.subtitle_rc = RECT::default();
    } else {
        let top = y + s(1);
        d.subtitle_rc = RECT { left: x0, top, right: x0 + w, bottom: top + measure(d, d.font_body, &d.subtitle, w) };
        y = d.subtitle_rc.bottom;
    }
    y += s(12);

    // Fields: label, box, then the hint or the error under it.
    let mut first = true;
    let field_count = d.fields.len();
    for i in 0..field_count {
        if !first {
            y += s(10);
        }
        first = false;
        let note_text = if d.fields[i].error.is_empty() { d.fields[i].hint.to_string() } else { d.fields[i].error.clone() };
        let indent = if d.fields[i].error.is_empty() { 0 } else { note_indent() };
        let note_h = measure(d, d.font_small, &note_text, w - indent);
        let f = &mut d.fields[i];
        f.label_y = y;
        y += s(LABEL_H) + s(3);
        f.box_rc = RECT { left: x0, top: y, right: x0 + w, bottom: y + s(FIELD_H) };
        y = f.box_rc.bottom;
        let edit_h = s(16);
        unsafe {
            let _ = MoveWindow(f.hwnd, x0 + s(8), f.box_rc.top + (s(FIELD_H) - edit_h) / 2, w - s(16), edit_h, true);
        }
        if note_h > 0 {
            y += s(4);
            f.note_rc = RECT { left: x0 + indent, top: y, right: x0 + w, bottom: y + note_h };
            y = f.note_rc.bottom;
        } else {
            f.note_rc = RECT::default();
        }
    }

    // List: the main choice as a filled button, the rest as flat rows; Delete set apart by a rule.
    d.divider_y = None;
    let mut prev: Option<Style> = None;
    for b in d.buttons.iter().filter(|b| !b.footer && b.id != B_X) {
        match (prev, b.style) {
            (None, _) if !first => y += s(10),
            (Some(_), Style::Danger) => {
                d.divider_y = Some(y + s(4));
                y += s(9);
            }
            (Some(Style::Primary), _) => y += s(GAP),
            (Some(_), _) => y += s(2),
            _ => {}
        }
        first = false;
        let h = if b.style == Style::Primary { s(MAIN_H) } else { s(ROW_H) };
        unsafe {
            let _ = MoveWindow(b.hwnd, x0, y, w, h, true);
        }
        y += h;
        prev = Some(b.style);
    }

    // Status line: page-level progress, results and errors not tied to one field.
    if d.status.is_empty() {
        d.status_rc = RECT::default();
    } else {
        y += s(10);
        let indent = if d.tone == Tone::Info { 0 } else { note_indent() };
        let h = measure(d, d.font_small, &d.status, w - indent);
        d.status_rc = RECT { left: x0 + indent, top: y, right: x0 + w, bottom: y + h };
        y += h;
    }

    // Footer: [Back] [Action], right-aligned like a native dialog.
    // The main one is wider so "Logging in" + working dots fit without clipping.
    let footer: Vec<(HWND, i32)> = d
        .buttons
        .iter()
        .filter(|b| b.footer)
        .map(|b| (b.hwnd, if b.style == Style::Primary { s(FOOT_MAIN_W) } else { s(FOOT_W) }))
        .collect();
    if !footer.is_empty() {
        y += s(14);
        let total: i32 = footer.iter().map(|f| f.1).sum::<i32>() + (footer.len() as i32 - 1) * s(GAP);
        let mut x = x0 + w - total;
        for (h, bw) in footer {
            unsafe {
                let _ = MoveWindow(h, x, y, bw, s(ROW_H), true);
            }
            x += bw + s(GAP);
        }
        y += s(ROW_H);
    }

    let width = s(WIDTH);
    let height = y + s(PAD);
    unsafe {
        let mut rc = RECT::default();
        let _ = GetWindowRect(d.hwnd, &mut rc);
        let (x, y) = match place {
            Place::Center => {
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);
                let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
                let _ = GetMonitorInfoW(MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST), &mut mi);
                let wa = mi.rcWork;
                // Slightly above true centre reads as centred (optical centre).
                (wa.left + (wa.right - wa.left - width) / 2, wa.top + (wa.bottom - wa.top - height) * 9 / 20)
            }
            Place::KeepCenter => (rc.left, (rc.top + rc.bottom) / 2 - height / 2),
            Place::KeepTop => (rc.left, rc.top),
        };
        let _ = MoveWindow(d.hwnd, x, y, width, height, true);
        let _ = InvalidateRect(d.hwnd, None, true);
    }
}

/// (Re)creates the controls for the current page, then lays them out.
fn build_page(d: &mut Dlg, place: Place) {
    unsafe {
        for f in d.fields.drain(..) {
            let _ = DestroyWindow(f.hwnd);
        }
        for b in d.buttons.drain(..) {
            let _ = DestroyWindow(b.hwnd);
        }
        d.hover = HWND::default();
        let (title, subtitle, fields, buttons) = page_spec(d);
        d.title = title;
        d.subtitle = subtitle;
        let hinstance = GetModuleHandleW(None).unwrap_or_default();

        for (id, label, password, initial, hint) in fields {
            let style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL | if password { ES_PASSWORD } else { 0 });
            let text: Vec<u16> = initial.encode_utf16().chain([0]).collect();
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0), w!("EDIT"), PCWSTR(text.as_ptr()), style,
                0, 0, 0, 0,
                d.hwnd, HMENU(id as isize as *mut _), hinstance, None,
            )
            .unwrap_or_default();
            SendMessageW(hwnd, WM_SETFONT, WPARAM(d.font_body.0 as usize), LPARAM(1));
            d.fields.push(Field { id, label, hint, error: String::new(), hwnd, label_y: 0, box_rc: RECT::default(), note_rc: RECT::default() });
        }

        for (id, text, style, icon, footer) in buttons {
            let wide: Vec<u16> = text.encode_utf16().chain([0]).collect();
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0), w!("BUTTON"), PCWSTR(wide.as_ptr()),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW),
                0, 0, 0, 0,
                d.hwnd, HMENU(id as isize as *mut _), hinstance, None,
            )
            .unwrap_or_default();
            d.buttons.push(Btn { id, text, style, icon, footer, hwnd });
        }

        // The × in the header row (mouse only; Esc does the same from the keyboard).
        let x_btn = CreateWindowExW(
            WINDOW_EX_STYLE(0), w!("BUTTON"), PCWSTR::null(), WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_OWNERDRAW),
            s(WIDTH) - s(PAD) - s(X_SIZE) + s(5), s(TOP) + (s(TITLE_H) - s(X_SIZE)) / 2, s(X_SIZE), s(X_SIZE),
            d.hwnd, HMENU(B_X as isize as *mut _), hinstance, None,
        )
        .unwrap_or_default();
        d.buttons.push(Btn { id: B_X, text: String::new(), style: Style::Plain, icon: I_CLOSE, footer: false, hwnd: x_btn });
    }
    layout(d, place);
}

/// First field if the page has one, otherwise the main action — so Enter does the obvious thing.
fn focus_first(d: &Dlg) {
    let target = d
        .fields
        .first()
        .map(|f| f.hwnd)
        .or_else(|| d.buttons.iter().find(|b| b.style == Style::Primary).map(|b| b.hwnd));
    if let Some(h) = target {
        unsafe {
            let _ = SetFocus(h);
        }
    }
}

fn text_of(d: &Dlg, id: i32) -> String {
    let Some(f) = d.fields.iter().find(|f| f.id == id) else { return String::new() };
    unsafe {
        let len = GetWindowTextLengthW(f.hwnd).max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let n = GetWindowTextW(f.hwnd, &mut buf).max(0) as usize;
        String::from_utf16_lossy(&buf[..n]).trim().to_string()
    }
}

fn set_status(d: &mut Dlg, text: impl Into<String>, tone: Tone) {
    d.status = text.into();
    d.tone = tone;
    layout(d, Place::KeepTop);
}

/// Shows `msg` under field `id` (red border), and puts the cursor there with the text selected
/// so the user can retype straight away.
fn set_field_error(d: &mut Dlg, id: i32, msg: impl Into<String>) {
    let Some(f) = d.fields.iter_mut().find(|f| f.id == id) else { return };
    f.error = msg.into();
    let h = f.hwnd;
    layout(d, Place::KeepTop);
    unsafe {
        let _ = SetFocus(h);
        SendMessageW(h, EM_SETSEL, WPARAM(0), LPARAM(-1));
    }
}

fn clear_errors(d: &mut Dlg) {
    let had = d.fields.iter().any(|f| !f.error.is_empty()) || (d.tone == Tone::Error && !d.status.is_empty());
    for f in &mut d.fields {
        f.error.clear();
    }
    if d.tone == Tone::Error {
        d.status.clear();
    }
    if had {
        layout(d, Place::KeepTop);
    }
}

/// Locks the page while a job runs: the pressed button shows working dots, the others grey out,
/// fields go read-only (still readable, unlike disabled ones). The × stays live.
fn set_busy(d: &mut Dlg, busy: Option<(i32, &'static str)>) {
    d.busy = busy.map(|b| b.0);
    d.busy_label = busy.map(|b| b.1).unwrap_or("");
    d.done_label = None;
    d.tick = 0;
    let on = busy.is_some();
    unsafe {
        for f in &d.fields {
            SendMessageW(f.hwnd, EM_SETREADONLY, WPARAM(on as usize), LPARAM(0));
        }
        for b in d.buttons.iter().filter(|b| b.id != B_X) {
            // The busy button stays enabled so it keeps its colour; clicks are ignored while busy.
            let _ = EnableWindow(b.hwnd, !on || Some(b.id) == d.busy);
        }
        if on {
            SetTimer(d.hwnd, T_SPIN, 120, None);
        } else {
            let _ = KillTimer(d.hwnd, T_SPIN);
        }
        let _ = InvalidateRect(d.hwnd, None, true);
    }
}

/// Swaps the busy button's dots for a check + `label`, then runs `after` a moment later so the
/// user sees that it worked before the window goes away.
fn succeed(d: &mut Dlg, label: &'static str, after: After) {
    unsafe {
        let _ = KillTimer(d.hwnd, T_SPIN);
        d.done_label = Some(label);
        d.after = Some(after);
        if let Some(b) = d.busy.and_then(|id| d.buttons.iter().find(|b| b.id == id)) {
            let _ = InvalidateRect(b.hwnd, None, false);
        }
        SetTimer(d.hwnd, T_AFTER, 650, None);
    }
}

fn set_armed(d: &mut Dlg, armed: bool) {
    if d.delete_armed == armed {
        return;
    }
    d.delete_armed = armed;
    unsafe {
        if armed {
            SetTimer(d.hwnd, T_DISARM, 6000, None);
        } else {
            let _ = KillTimer(d.hwnd, T_DISARM);
        }
        if let Some(b) = d.buttons.iter().find(|b| b.id == B_DELETE) {
            let _ = InvalidateRect(b.hwnd, None, false);
        }
    }
    if armed {
        set_status(d, "This permanently removes your code and its key from the server.", Tone::Error);
    } else if d.tone == Tone::Error {
        set_status(d, "", Tone::Info);
    }
}

fn go(d: &mut Dlg, page: Page) {
    set_armed(d, false);
    d.page = page;
    d.status.clear();
    build_page(d, Place::KeepCenter);
    focus_first(d);
}

fn finish(d: &mut Dlg, result: Option<LaunchChoice>) {
    d.result = result;
    unsafe {
        let _ = DestroyWindow(d.hwnd);
    }
}

/// What Enter does: the focused button if a button has focus, otherwise the page's main action.
fn enter_target(d: &Dlg) -> Option<i32> {
    let focused = unsafe { GetFocus() };
    d.buttons
        .iter()
        .find(|b| b.hwnd == focused && b.id != B_X)
        .or_else(|| d.buttons.iter().find(|b| b.style == Style::Primary))
        .map(|b| b.id)
}

// ---- actions --------------------------------------------------------------------------------------

enum Job {
    Login(String),
    Signup(String, String),
    ChangeKey(String, String),
    ChangeCode(String, String),
    Delete(String),
}

fn on_action(d: &mut Dlg, id: i32) {
    if d.busy.is_some() {
        return;
    }
    if id != B_DELETE {
        set_armed(d, false);
    }
    match id {
        B_CONT_ACCOUNT => {
            let key = d.account.api_key.clone();
            finish(d, Some(LaunchChoice::Use { key, from_account: true }));
        }
        B_CONT_CONFIG => {
            let key = d.config_key.clone();
            finish(d, Some(LaunchChoice::Use { key, from_account: false }));
        }
        B_GO_LOGIN => go(d, Page::Login),
        B_GO_SIGNUP => go(d, Page::Signup),
        B_GO_CHKEY => go(d, Page::ChangeKey),
        B_GO_CHCODE => go(d, Page::ChangeCode),
        B_BACK => go(d, Page::Home),
        B_CLOSE | B_X => close(d),
        B_DO_LOGIN => {
            clear_errors(d);
            let code = text_of(d, F_CODE);
            if code.is_empty() {
                return set_field_error(d, F_CODE, "Type your code first.");
            }
            start(d, id, Job::Login(code), "Logging in");
        }
        B_DO_SIGNUP => {
            clear_errors(d);
            let (code, key) = (text_of(d, F_CODE), text_of(d, F_KEY));
            if key.is_empty() {
                set_field_error(d, F_KEY, "Paste your OpenRouter API key.");
            }
            if code.is_empty() {
                set_field_error(d, F_CODE, "Pick a code.");
            }
            if code.is_empty() || key.is_empty() {
                return;
            }
            start(d, id, Job::Signup(code, key), "Creating");
        }
        B_SAVE_KEY => {
            clear_errors(d);
            let key = text_of(d, F_KEY);
            if key.is_empty() {
                return set_field_error(d, F_KEY, "Paste the new API key first.");
            }
            start(d, id, Job::ChangeKey(d.account.code.clone(), key), "Saving");
        }
        B_SAVE_CODE => {
            clear_errors(d);
            let code = text_of(d, F_CODE);
            if code.is_empty() {
                return set_field_error(d, F_CODE, "Type the new code first.");
            }
            start(d, id, Job::ChangeCode(d.account.code.clone(), code), "Saving");
        }
        B_LOGOUT => {
            // Not destructive: the account stays on the server; logging in again restores it.
            if let Err(e) = crate::config::load::clear_account() {
                return set_status(d, format!("Couldn't update config.toml: {e}"), Tone::Error);
            }
            restart(false);
        }
        B_DELETE => {
            // GUARD: destructive and irreversible (removes the code and key from the server), so
            // it takes two clicks — the first only arms it (the row turns red and says "Click
            // again to delete"); Esc, any other button or 6 seconds of nothing disarm it.
            if !d.delete_armed {
                return set_armed(d, true);
            }
            set_armed(d, false);
            start(d, id, Job::Delete(d.account.code.clone()), "Deleting");
        }
        _ => {}
    }
}

fn close(d: &mut Dlg) {
    // Account screen: closing mid-request (Esc, Alt+F4, ×) could leave the server and
    // config.toml disagreeing (e.g. deleted on the server, still saved locally). Wait it out.
    // The launch dialog may still quit: the app exits and nothing half-done is kept.
    if d.mode == Mode::Account && d.busy.is_some() {
        return;
    }
    match (d.mode, d.page) {
        (Mode::Launch, _) => finish(d, Some(LaunchChoice::Quit)),
        (Mode::Account, _) => finish(d, None),
    }
}

fn start(d: &mut Dlg, button: i32, job: Job, working: &'static str) {
    d.status.clear();
    layout(d, Place::KeepTop);
    set_busy(d, Some((button, working)));
    let hwnd = d.hwnd.0 as isize;
    std::thread::spawn(move || {
        // The result travels inside the message itself, so it can only ever reach the window that
        // started the job (a shared slot could hand a closed window's result to a new one).
        let done = Box::into_raw(Box::new(run_job(job)));
        // SAFETY: posting to a window owned by another thread is what PostMessage is for. If the
        // window is already gone the message isn't queued, so the result is freed here instead.
        unsafe {
            if PostMessageW(HWND(hwnd as *mut _), WM_APP_DONE, WPARAM(0), LPARAM(done as isize)).is_err() {
                drop(Box::from_raw(done));
            }
        }
    });
}

fn run_job(job: Job) -> Done {
    let kind = match &job {
        Job::Login(..) => "login",
        Job::Signup(..) => "signup",
        Job::ChangeKey(..) => "change_key",
        Job::ChangeCode(..) => "change_code",
        Job::Delete(..) => "delete",
    };
    let start = std::time::Instant::now();
    let done = run_job_inner(job);
    let result = match &done {
        Done::Fail(msg, _) => msg.chars().take(200).collect::<String>(),
        _ => "ok".to_string(),
    };
    match &done {
        Done::Saved { code, .. } | Done::CodeChanged { code } => crate::analytics::set_account(code),
        Done::Deleted => crate::analytics::set_account(""),
        Done::KeyChanged { .. } | Done::Fail(..) => {}
    }
    crate::analytics::track("account", serde_json::json!({ "action": kind, "result": result, "ms": start.elapsed().as_millis() as u64 }));
    done
}

fn run_job_inner(job: Job) -> Done {
    let bad_key = || Done::Fail("OpenRouter doesn't accept this key.".into(), Some(F_KEY));
    let taken = || Done::Fail("That code is taken. Pick another one.".into(), Some(F_CODE));
    let gone = || Done::Fail("This code no longer exists on the server. Log out and sign up again.".into(), None);
    let r: anyhow::Result<Done> = account::block_on(async move {
        Ok(match job {
            Job::Login(code) => match account::login(&code).await? {
                Some(key) => Done::Saved { code, key },
                None => Done::Fail("No account with that code.".into(), Some(F_CODE)),
            },
            Job::Signup(code, key) => {
                if !account::openrouter_key_works(&key).await? {
                    return Ok(bad_key());
                }
                match account::signup(&code, &key).await?.as_str() {
                    "ok" => Done::Saved { code, key },
                    "taken" => taken(),
                    other => Done::Fail(format!("Couldn't sign up ({other})."), None),
                }
            }
            Job::ChangeKey(code, key) => {
                if !account::openrouter_key_works(&key).await? {
                    return Ok(bad_key());
                }
                match account::update(&code, None, Some(&key)).await?.as_str() {
                    "ok" => Done::KeyChanged { key },
                    "not_found" => gone(),
                    other => Done::Fail(format!("Couldn't change the key ({other})."), None),
                }
            }
            Job::ChangeCode(code, new_code) => match account::update(&code, Some(&new_code), None).await?.as_str() {
                "ok" => Done::CodeChanged { code: new_code },
                "taken" => taken(),
                "not_found" => gone(),
                other => Done::Fail(format!("Couldn't change the code ({other})."), None),
            },
            Job::Delete(code) => {
                account::delete(&code).await?;
                Done::Deleted
            }
        })
    });
    r.unwrap_or_else(|e| Done::Fail(format!("{e}"), None))
}

fn on_done(d: &mut Dlg, done: Done) {
    match done {
        Done::Fail(msg, field) => {
            set_busy(d, None);
            match field.filter(|id| d.fields.iter().any(|f| f.id == *id)) {
                Some(id) => set_field_error(d, id, msg),
                None => set_status(d, msg, Tone::Error),
            }
        }
        Done::Saved { code, key } => {
            let saved = crate::config::load::save_account(&code, &key);
            let label = if d.page == Page::Signup { "Created" } else { "Logged in" };
            match d.mode {
                // Launch: the key is used straight from memory, so this run works either way;
                // a failed save only means the next launch won't offer "Continue as".
                Mode::Launch => {
                    if let Err(e) = saved {
                        log::warn!("account: couldn't save login to config.toml: {e}");
                    }
                    succeed(d, label, After::Finish(LaunchChoice::Use { key, from_account: true }))
                }
                // Account screen: the restart reads the key back from config.toml, so if saving
                // failed, restarting would silently fall back to the old key. Say so and stay.
                Mode::Account if saved.is_err() => {
                    set_busy(d, None);
                    let e = saved.unwrap_err();
                    set_status(d, format!("Logged in, but config.toml couldn't be updated ({e}). Close it if it's open elsewhere and try again."), Tone::Error);
                }
                Mode::Account => {
                    set_status(d, "Restarting with your account's key…", Tone::Ok);
                    succeed(d, label, After::Restart(true));
                }
            }
        }
        Done::KeyChanged { key } => {
            let code = d.account.code.clone();
            if let Err(e) = crate::config::load::save_account(&code, &key) {
                set_busy(d, None);
                return set_status(d, format!("Key changed on the server, but config.toml couldn't be updated: {e}"), Tone::Error);
            }
            set_status(d, "Restarting with the new key…", Tone::Ok);
            succeed(d, "Saved", After::Restart(true));
        }
        Done::CodeChanged { code } => {
            set_busy(d, None);
            let key = d.account.api_key.clone();
            if let Err(e) = crate::config::load::save_account(&code, &key) {
                return set_status(d, format!("Code changed on the server, but config.toml couldn't be updated: {e}"), Tone::Error);
            }
            d.account.code = code;
            go(d, Page::Home);
            set_status(d, "Code changed.", Tone::Ok);
        }
        Done::Deleted => {
            let _ = crate::config::load::clear_account();
            set_status(d, "Account deleted. Restarting…", Tone::Ok);
            succeed(d, "Deleted", After::Restart(false));
        }
    }
}

fn run_after(d: &mut Dlg) {
    unsafe {
        let _ = KillTimer(d.hwnd, T_AFTER);
    }
    match d.after.take() {
        Some(After::Finish(choice)) => finish(d, Some(choice)),
        Some(After::Restart(resume)) => restart(resume),
        None => {}
    }
}

/// After an account change the running app restarts so every part picks up the new key.
/// `resume` = skip the launch dialog and use the account key; otherwise show the dialog again.
fn restart(resume: bool) {
    use crate::lifecycle::self_restart;
    let r = if resume {
        std::env::set_var(self_restart::KEY_SOURCE_ENV, "account");
        self_restart::execute()
    } else {
        self_restart::execute_fresh()
    };
    if let Err(e) = r {
        log::error!("account: restart failed: {e}");
    }
}

/// Mouse-over highlight for the owner-drawn buttons. Fed from WM_SETCURSOR (which names the child
/// under the mouse) and a short timer that notices when the mouse has left.
fn set_hover(d: &mut Dlg, h: HWND) {
    if d.hover == h {
        return;
    }
    let old = std::mem::replace(&mut d.hover, h);
    unsafe {
        for w in [old, h] {
            if !w.is_invalid() {
                let _ = InvalidateRect(w, None, false);
            }
        }
        if h.is_invalid() {
            let _ = KillTimer(d.hwnd, T_HOVER);
        } else {
            SetTimer(d.hwnd, T_HOVER, 60, None);
        }
    }
}

// ---- painting -------------------------------------------------------------------------------------

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

unsafe fn fill(hdc: HDC, rc: RECT, color: COLORREF) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, &rc, brush);
    let _ = DeleteObject(HGDIOBJ(brush.0));
}

unsafe fn text(hdc: HDC, font: HFONT, color: COLORREF, s_: &str, mut rc: RECT, fmt: DRAW_TEXT_FORMAT) {
    SelectObject(hdc, HGDIOBJ(font.0));
    SetTextColor(hdc, color);
    SetBkMode(hdc, TRANSPARENT);
    let mut wide: Vec<u16> = s_.encode_utf16().collect();
    if !wide.is_empty() {
        DrawTextW(hdc, &mut wide, &mut rc, fmt);
    }
}

unsafe fn text_width(hdc: HDC, font: HFONT, s_: &str) -> i32 {
    let mut rc = RECT::default();
    SelectObject(hdc, HGDIOBJ(font.0));
    let mut wide: Vec<u16> = s_.encode_utf16().collect();
    if !wide.is_empty() {
        DrawTextW(hdc, &mut wide, &mut rc, DT_SINGLELINE | DT_CALCRECT);
    }
    rc.right - rc.left
}

/// A note line (status or field error) with its small icon hanging in the left indent.
unsafe fn note(d: &Dlg, hdc: HDC, rc: RECT, msg: &str, tone: Tone) {
    let (color, icon) = match tone {
        Tone::Info => (MUTED, None),
        Tone::Error => (ERROR_TEXT, Some(I_ERROR)),
        Tone::Ok => (OK_TEXT, Some(I_CHECK)),
    };
    if let Some(icon) = icon {
        let irc = RECT { left: rc.left - note_indent(), top: rc.top + s(2), right: rc.left, bottom: rc.top + s(15) };
        text(hdc, d.font_icon_small, color, &icon.to_string(), irc, DT_LEFT | DT_SINGLELINE);
    }
    text(hdc, d.font_small, color, msg, rc, DT_LEFT | DT_WORDBREAK);
}

unsafe fn paint(d: &Dlg) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(d.hwnd, &mut ps);
    let mut client = RECT::default();
    let _ = GetClientRect(d.hwnd, &mut client);
    FillRect(hdc, &client, d.bg_brush);

    // The launch screen shows a skull instead of the app's name (no branding anywhere on screen).
    let title_font = if d.title == SKULL.to_string() { d.font_skull } else { d.font_title };
    text(hdc, title_font, TEXT, &d.title, title_rc(), DT_LEFT | DT_SINGLELINE | DT_VCENTER);
    text(hdc, d.font_body, MUTED, &d.subtitle, d.subtitle_rc, DT_LEFT | DT_WORDBREAK);

    let focused = GetFocus();
    let busy = d.busy.is_some();
    for f in &d.fields {
        let label = RECT { left: s(PAD), top: f.label_y, right: s(PAD) + inner_w(), bottom: f.label_y + s(LABEL_H) };
        text(hdc, d.font_small, MUTED, f.label, label, DT_LEFT | DT_SINGLELINE);
        let border = if !f.error.is_empty() {
            DANGER
        } else if f.hwnd == focused && !busy {
            ACCENT
        } else {
            BORDER
        };
        round_rect(hdc, f.box_rc, Some(FIELD_BG), border, s(RADIUS));
        if !f.error.is_empty() {
            note(d, hdc, f.note_rc, &f.error, Tone::Error);
        } else if !f.hint.is_empty() {
            note(d, hdc, f.note_rc, f.hint, Tone::Info);
        }
    }
    if let Some(y) = d.divider_y {
        fill(hdc, RECT { left: s(PAD) + s(10), top: y, right: s(PAD) + inner_w() - s(10), bottom: y + s(1).max(1) }, BORDER);
    }
    if !d.status.is_empty() {
        note(d, hdc, d.status_rc, &d.status, d.tone);
    }
    let _ = EndPaint(d.hwnd, &ps);
}

/// Three dots, one lit at a time, moving left to right: "working on it".
unsafe fn working_dots(hdc: HDC, left: i32, mid_y: i32, tick: u32, on: COLORREF, off: COLORREF) -> i32 {
    let r = s(2);
    let step = s(7);
    for i in 0..3 {
        let c = if tick % 3 == i as u32 { on } else { off };
        let brush = CreateSolidBrush(c);
        let pen = CreatePen(PS_SOLID, 1, c);
        let ob = SelectObject(hdc, HGDIOBJ(brush.0));
        let op = SelectObject(hdc, HGDIOBJ(pen.0));
        let cx = left + r + i * step;
        let _ = Ellipse(hdc, cx - r, mid_y - r, cx + r + 1, mid_y + r + 1);
        SelectObject(hdc, ob);
        SelectObject(hdc, op);
        let _ = DeleteObject(HGDIOBJ(brush.0));
        let _ = DeleteObject(HGDIOBJ(pen.0));
    }
    2 * r + 2 * step
}

unsafe fn draw_button(d: &Dlg, item: &DrawItem) {
    let Some(b) = d.buttons.iter().find(|b| b.id == item.ctl_id as i32) else { return };
    let pressed = item.item_state & ODS_SELECTED != 0;
    let disabled = item.item_state & ODS_DISABLED != 0;
    let focus = item.item_state & ODS_FOCUS != 0 && item.item_state & ODS_NOFOCUSRECT == 0;
    let hover = d.hover == b.hwnd && !disabled && d.busy.is_none();
    let is_busy = d.busy == Some(b.id);
    let armed = b.style == Style::Danger && d.delete_armed;
    let hdc = item.hdc;
    let rc = item.rc;
    FillRect(hdc, &rc, d.bg_brush);

    // (fill, label colour, icon colour). `None` fill = flat until hovered.
    let (fill_c, fg, icon_fg) = match b.style {
        _ if b.id == B_X => (if pressed { Some(SURFACE_DOWN) } else if hover { Some(SURFACE) } else { None }, MUTED, if hover { TEXT } else { MUTED }),
        Style::Primary => (Some(if pressed { ACCENT_DOWN } else if hover { ACCENT_HOVER } else { ACCENT }), ON_ACCENT, ON_ACCENT),
        Style::Danger if armed || (is_busy) => (Some(if pressed { DANGER_DOWN } else { DANGER }), ON_ACCENT, ON_ACCENT),
        Style::Row | Style::Danger => {
            let f = if pressed { Some(SURFACE_DOWN) } else if hover { Some(SURFACE_HOVER) } else if focus { Some(SURFACE) } else { None };
            let fg = if b.style == Style::Danger { ERROR_TEXT } else { TEXT };
            (f, fg, if b.style == Style::Danger { ERROR_TEXT } else { MUTED })
        }
        Style::Plain => (Some(if pressed { SURFACE_DOWN } else if hover { SURFACE_HOVER } else { SURFACE }), TEXT, MUTED),
    };
    let (fg, icon_fg) = if disabled { (DIM, DIM) } else { (fg, icon_fg) };
    let fill_c = if disabled && b.style == Style::Primary { Some(SURFACE) } else { fill_c };
    if let Some(f) = fill_c {
        round_rect(hdc, rc, Some(f), f, s(RADIUS));
    }
    // Keyboard focus (only after Tab / arrow keys, like native dialogs): a thin ring.
    if focus && b.id != B_X {
        if b.style == Style::Primary || armed {
            let inset = RECT { left: rc.left + s(2), top: rc.top + s(2), right: rc.right - s(2), bottom: rc.bottom - s(2) };
            round_rect(hdc, inset, None, ACCENT_RING, s(RADIUS) - s(2));
        } else {
            round_rect(hdc, rc, None, ACCENT, s(RADIUS));
        }
    }

    if b.text.is_empty() {
        text(hdc, d.font_icon, icon_fg, &b.icon.to_string(), rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        return;
    }

    let label: &str = if is_busy {
        d.done_label.unwrap_or(d.busy_label)
    } else if armed {
        "Click again to delete"
    } else {
        &b.text
    };
    let gap = s(7);
    let mid_y = (rc.top + rc.bottom) / 2;
    let label_w = text_width(hdc, d.font_body, label);
    // Leading mark: working dots / success check / the row's icon.
    let lead_w = if is_busy && d.done_label.is_none() {
        s(18)
    } else if is_busy {
        text_width(hdc, d.font_icon, &I_CHECK.to_string())
    } else if b.icon != NO_ICON {
        text_width(hdc, d.font_icon, &b.icon.to_string())
    } else {
        0
    };
    let content_w = if lead_w > 0 { lead_w + gap + label_w } else { label_w };
    // Footer buttons are centred; list rows (and the main list button) line up on the left.
    let left = if b.footer {
        rc.left + ((rc.right - rc.left) - content_w).max(0) / 2
    } else {
        rc.left + s(10)
    };
    let mut x = left;
    if lead_w > 0 {
        if is_busy && d.done_label.is_none() {
            let dim = if b.style == Style::Row { DIM } else { DOT_OFF };
            working_dots(hdc, x, mid_y, d.tick, fg, dim);
        } else {
            let glyph = if is_busy { I_CHECK } else { b.icon };
            let irc = RECT { left: x, top: rc.top, right: x + lead_w + s(2), bottom: rc.bottom };
            text(hdc, d.font_icon, icon_fg, &glyph.to_string(), irc, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
        }
        x += lead_w + gap;
    }
    let label_rc = RECT { left: x, top: rc.top, right: rc.right - s(8), bottom: rc.bottom };
    text(hdc, d.font_body, fg, label, label_rc, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(d) = dlg().filter(|d| d.hwnd == hwnd) else {
        if msg == WM_DESTROY {
            PostQuitMessage(0);
        }
        if msg == WM_APP_DONE && lp.0 != 0 {
            // A result for a window that's already closed: just free it.
            drop(Box::from_raw(lp.0 as *mut Done));
            return LRESULT(0);
        }
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_PAINT => {
            paint(d);
            LRESULT(0)
        }
        WM_DRAWITEM => {
            draw_button(d, &*(lp.0 as *const DrawItem));
            LRESULT(1)
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            // Read-only (busy) fields come through as STATIC: same box, dimmer text.
            let hdc = HDC(wp.0 as *mut _);
            SetTextColor(hdc, if msg == WM_CTLCOLOREDIT { TEXT } else { MUTED });
            SetBkColor(hdc, FIELD_BG);
            LRESULT(d.field_brush.0 as isize)
        }
        // Drag the window by any empty spot.
        WM_NCHITTEST => {
            let hit = DefWindowProcW(hwnd, msg, wp, lp);
            if hit.0 == HTCLIENT { LRESULT(HTCAPTION) } else { hit }
        }
        WM_SETCURSOR => {
            let under = HWND(wp.0 as *mut _);
            let target = if d.buttons.iter().any(|b| b.hwnd == under) { under } else { HWND::default() };
            set_hover(d, target);
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_TIMER => {
            match wp.0 {
                T_SPIN => {
                    d.tick = d.tick.wrapping_add(1);
                    if let Some(b) = d.busy.and_then(|id| d.buttons.iter().find(|b| b.id == id)) {
                        let _ = InvalidateRect(b.hwnd, None, false);
                    }
                }
                T_HOVER => {
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);
                    if WindowFromPoint(pt) != d.hover {
                        set_hover(d, HWND::default());
                    }
                }
                T_AFTER => run_after(d),
                T_DISARM => set_armed(d, false),
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wp.0 & 0xFFFF) as i32;
            let code = ((wp.0 >> 16) & 0xFFFF) as u32;
            match id {
                IDOK => {
                    if let Some(p) = enter_target(d) {
                        on_action(d, p);
                    }
                }
                IDCANCEL => {
                    if d.busy.is_some() {
                        // A job is running; the × still closes if the user really wants out.
                    } else if d.delete_armed {
                        set_armed(d, false);
                    } else if d.page == Page::Home {
                        close(d)
                    } else {
                        on_action(d, B_BACK)
                    }
                }
                _ if code == EN_SETFOCUS || code == EN_KILLFOCUS => {
                    let _ = InvalidateRect(hwnd, None, false);
                }
                _ if code == EN_CHANGE => {
                    // Typing in a field clears its error (and a page error from the last try).
                    let had = d.fields.iter().any(|f| f.id == id && !f.error.is_empty());
                    if had || (d.tone == Tone::Error && !d.status.is_empty() && d.busy.is_none()) {
                        if let Some(f) = d.fields.iter_mut().find(|f| f.id == id) {
                            f.error.clear();
                        }
                        if d.tone == Tone::Error {
                            d.status.clear();
                        }
                        layout(d, Place::KeepTop);
                    }
                }
                _ if code == BN_CLICKED => on_action(d, id),
                _ => {}
            }
            LRESULT(0)
        }
        WM_APP_DONE => {
            // SAFETY: lp is the Box::into_raw from `start`'s worker, posted once to this window.
            let done = *Box::from_raw(lp.0 as *mut Done);
            on_done(d, done);
            LRESULT(0)
        }
        WM_CLOSE => {
            close(d);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dlg_for(mode: Mode, page: Page, logged_in: bool, config_key: &str) -> Dlg {
        Dlg {
            hwnd: HWND::default(),
            mode,
            page,
            account: if logged_in {
                AccountConfig { code: "2244644".into(), api_key: "sk-or-v1-x".into() }
            } else {
                AccountConfig::default()
            },
            config_key: config_key.into(),
            title: String::new(),
            subtitle: String::new(),
            status: String::new(),
            tone: Tone::Info,
            fields: vec![],
            buttons: vec![],
            busy: None,
            busy_label: "",
            done_label: None,
            tick: 0,
            hover: HWND::default(),
            after: None,
            delete_armed: false,
            font_title: HFONT::default(),
            font_skull: HFONT::default(),
            font_body: HFONT::default(),
            font_small: HFONT::default(),
            font_icon: HFONT::default(),
            font_icon_small: HFONT::default(),
            subtitle_rc: RECT::default(),
            status_rc: RECT::default(),
            divider_y: None,
            bg_brush: HBRUSH::default(),
            field_brush: HBRUSH::default(),
            result: None,
        }
    }

    fn ids(d: &Dlg) -> Vec<i32> {
        page_spec(d).3.iter().map(|b| b.0).collect()
    }

    #[test]
    fn launch_home_offers_the_right_choices() {
        assert_eq!(ids(&dlg_for(Mode::Launch, Page::Home, true, "sk-or-v1-cfg")), vec![B_CONT_ACCOUNT, B_CONT_CONFIG, B_GO_LOGIN, B_GO_SIGNUP]);
        assert_eq!(ids(&dlg_for(Mode::Launch, Page::Home, false, "sk-or-v1-cfg")), vec![B_CONT_CONFIG, B_GO_LOGIN, B_GO_SIGNUP]);
        assert_eq!(ids(&dlg_for(Mode::Launch, Page::Home, false, "")), vec![B_GO_LOGIN, B_GO_SIGNUP]);
        let spec = page_spec(&dlg_for(Mode::Launch, Page::Home, true, ""));
        assert_eq!(spec.3[0].1, "Continue as ••••644");
    }

    #[test]
    fn every_page_has_exactly_one_primary_button() {
        for mode in [Mode::Launch, Mode::Account] {
            for page in [Page::Home, Page::Login, Page::Signup, Page::ChangeKey, Page::ChangeCode] {
                for logged_in in [true, false] {
                    for key in ["", "sk-or-v1-cfg"] {
                        let d = dlg_for(mode, page, logged_in, key);
                        let primaries = page_spec(&d).3.iter().filter(|b| b.2 == Style::Primary).count();
                        assert_eq!(primaries, 1, "{mode:?} {page:?} logged_in={logged_in} key={key:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn signup_prefills_the_config_key() {
        let d = dlg_for(Mode::Launch, Page::Signup, false, "sk-or-v1-cfg");
        assert_eq!(page_spec(&d).2[1].3, "sk-or-v1-cfg");
    }
}
