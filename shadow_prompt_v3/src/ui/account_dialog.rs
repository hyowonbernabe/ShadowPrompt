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
use std::sync::{Mutex, Once};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    InvalidateRect, RoundRect, SelectObject, SetBkColor, SetBkMode, SetTextColor, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DEFAULT_PITCH, DRAW_TEXT_FORMAT, DT_CENTER, DT_LEFT, DT_SINGLELINE, DT_VCENTER,
    DT_WORDBREAK, FF_DONTCARE, HBRUSH, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, GetFocus, SetFocus};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, GetWindowTextLengthW, GetWindowTextW, IsDialogMessageW, LoadCursorW, MoveWindow,
    PostMessageW, PostQuitMessage, RegisterClassExW, SendMessageW, SetForegroundWindow,
    SetWindowDisplayAffinity, ShowWindow, TranslateMessage, HMENU, IDC_ARROW, MSG, SM_CXSCREEN,
    SM_CYSCREEN, SW_SHOW, WDA_EXCLUDEFROMCAPTURE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_PAINT,
    WNDCLASSEXW, WS_CHILD, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_TABSTOP, WS_VISIBLE,
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
const BG: COLORREF = rgb(0x12, 0x14, 0x19);
const FIELD_BG: COLORREF = rgb(0x1B, 0x1E, 0x26);
const BORDER: COLORREF = rgb(0x2B, 0x30, 0x3B);
const TEXT: COLORREF = rgb(0xF1, 0xF3, 0xF7);
const MUTED: COLORREF = rgb(0x9A, 0xA3, 0xB2);
const ACCENT: COLORREF = rgb(0x63, 0x66, 0xF1);
const ACCENT_DOWN: COLORREF = rgb(0x4F, 0x46, 0xE5);
const SECONDARY: COLORREF = rgb(0x20, 0x24, 0x2D);
const SECONDARY_DOWN: COLORREF = rgb(0x2A, 0x2F, 0x3A);
const DANGER: COLORREF = rgb(0xE5, 0x48, 0x4D);
const DANGER_DOWN: COLORREF = rgb(0xC5, 0x34, 0x39);
const ERROR_TEXT: COLORREF = rgb(0xFF, 0x7A, 0x7A);
const OK_TEXT: COLORREF = rgb(0x6E, 0xE7, 0xA8);

const WIDTH: i32 = 420;
const PAD: i32 = 28;
const RADIUS: i32 = 12;

fn s(px: i32) -> i32 {
    // SAFETY: plain Win32 query, no arguments.
    let dpi = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() }.max(96) as i32;
    px * dpi / 96
}

// Win32 values used below, spelled out so no extra `windows` features are needed.
const WM_CLOSE: u32 = 0x0010;
const WM_SETFONT: u32 = 0x0030;
const WM_DRAWITEM: u32 = 0x002B;
const WM_NCHITTEST: u32 = 0x0084;
const WM_COMMAND: u32 = 0x0111;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_APP_DONE: u32 = 0x8000 + 1;
const HTCLIENT: isize = 1;
const HTCAPTION: isize = 2;
const IDOK: i32 = 1;
const IDCANCEL: i32 = 2;
const BN_CLICKED: u32 = 0;
const EN_SETFOCUS: u32 = 0x0100;
const EN_KILLFOCUS: u32 = 0x0200;
const BS_OWNERDRAW: u32 = 0x000B;
const ES_PASSWORD: u32 = 0x0020;
const ES_AUTOHSCROLL: u32 = 0x0080;
const ODS_SELECTED: u32 = 0x0001;
const ODS_DISABLED: u32 = 0x0004;
const DWMWA_USE_IMMERSIVE_DARK_MODE: i32 = 20;
const DWMWA_WINDOW_CORNER_PREFERENCE: i32 = 33;
const DWMWA_BORDER_COLOR: i32 = 34;
const DWMWCP_ROUND: u32 = 2;
const CLEARTYPE_QUALITY: u32 = 5;

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

#[derive(Clone, Copy, PartialEq)]
enum Style {
    Primary,
    Secondary,
    Danger,
    Ghost,
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
    hwnd: HWND,
    box_rc: RECT,
    label_y: i32,
}

struct Btn {
    id: i32,
    text: String,
    style: Style,
    hwnd: HWND,
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
    status_color: COLORREF,
    fields: Vec<Field>,
    buttons: Vec<Btn>,
    status_y: i32,
    busy: bool,
    /// Delete needs two clicks; this is set by the first.
    delete_armed: bool,
    font_title: HFONT,
    font_body: HFONT,
    font_small: HFONT,
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
    Fail(String),
}

static DONE: Mutex<Option<Done>> = Mutex::new(None);

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
                lpszClassName: w!("ShadowPromptAccount"),
                ..Default::default()
            };
            RegisterClassExW(&wc);
        });

        let font = |size: i32, weight: i32| {
            CreateFontW(
                -s(size), 0, 0, 0, weight, 0, 0, 0,
                DEFAULT_CHARSET.0 as u32, OUT_DEFAULT_PRECIS.0 as u32, CLIP_DEFAULT_PRECIS.0 as u32,
                CLEARTYPE_QUALITY, (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32, w!("Segoe UI"),
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
            status_color: MUTED,
            fields: Vec::new(),
            buttons: Vec::new(),
            status_y: 0,
            busy: false,
            delete_armed: false,
            font_title: font(22, 600),
            font_body: font(14, 400),
            font_small: font(12, 400),
            bg_brush: CreateSolidBrush(BG),
            field_brush: CreateSolidBrush(FIELD_BG),
            result: None,
        }));
        DLG.with(|c| c.set(state));

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("ShadowPromptAccount"),
            w!("ShadowPrompt"),
            WS_POPUP,
            0, 0, s(WIDTH), s(300),
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

        build_page(d);
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
        for obj in [HGDIOBJ(d.font_title.0), HGDIOBJ(d.font_body.0), HGDIOBJ(d.font_small.0), HGDIOBJ(d.bg_brush.0), HGDIOBJ(d.field_brush.0)] {
            let _ = DeleteObject(obj);
        }
        d.result
    }
}

/// Page content: title, subtitle, fields and buttons for the current mode/page.
/// (title, subtitle, fields as (id, label, password, initial text), buttons as (id, text, style)).
type PageSpec = (String, String, Vec<(i32, &'static str, bool, String)>, Vec<(i32, String, Style)>);

fn page_spec(d: &Dlg) -> PageSpec {
    let logged_in = d.account.logged_in();
    let who = account::masked(&d.account.code);
    match (d.mode, d.page) {
        (Mode::Launch, Page::Home) => {
            let mut b = Vec::new();
            if logged_in {
                b.push((B_CONT_ACCOUNT, format!("Continue as {who}"), Style::Primary));
            }
            if !d.config_key.is_empty() {
                let style = if logged_in { Style::Secondary } else { Style::Primary };
                b.push((B_CONT_CONFIG, "Continue with my key".to_string(), style));
            }
            let style = if b.is_empty() { Style::Primary } else { Style::Secondary };
            b.push((B_GO_LOGIN, "Log in with a code".to_string(), style));
            b.push((B_GO_SIGNUP, "Sign up".to_string(), Style::Secondary));
            ("ShadowPrompt".into(), "How do you want to start?".into(), vec![], b)
        }
        (Mode::Account, Page::Home) if logged_in => (
            "Account".into(),
            format!("Logged in as {who}."),
            vec![],
            vec![
                (B_GO_CHKEY, "Change API key".into(), Style::Secondary),
                (B_GO_CHCODE, "Change code".into(), Style::Secondary),
                (B_LOGOUT, "Log out of this computer".into(), Style::Secondary),
                (B_DELETE, "Delete account".into(), Style::Danger),
                (B_CLOSE, "Close".into(), Style::Primary),
            ],
        ),
        (Mode::Account, Page::Home) => (
            "Account".into(),
            "Not logged in. Using the key in config.toml.".into(),
            vec![],
            vec![
                (B_GO_LOGIN, "Log in with a code".into(), Style::Primary),
                (B_GO_SIGNUP, "Sign up".into(), Style::Secondary),
                (B_CLOSE, "Close".into(), Style::Secondary),
            ],
        ),
        (_, Page::Login) => (
            "Log in".into(),
            "Enter your code to load your API key.".into(),
            vec![(F_CODE, "Code", true, String::new())],
            vec![(B_DO_LOGIN, "Log in".into(), Style::Primary), (B_BACK, "Back".into(), Style::Ghost)],
        ),
        (_, Page::Signup) => (
            "Sign up".into(),
            "Your key is stored under this code. Anyone with the code can use the key, so pick one that's hard to guess.".into(),
            vec![(F_CODE, "Choose a code", false, String::new()), (F_KEY, "OpenRouter API key", true, d.config_key.clone())],
            vec![(B_DO_SIGNUP, "Create account".into(), Style::Primary), (B_BACK, "Back".into(), Style::Ghost)],
        ),
        (_, Page::ChangeKey) => (
            "Change API key".into(),
            format!("The new key replaces the one stored under {who}, on every computer that uses this code."),
            vec![(F_KEY, "New OpenRouter API key", true, String::new())],
            vec![(B_SAVE_KEY, "Save key".into(), Style::Primary), (B_BACK, "Back".into(), Style::Ghost)],
        ),
        (_, Page::ChangeCode) => (
            "Change code".into(),
            format!("{who} stops working. Your key moves to the new code."),
            vec![(F_CODE, "New code", false, String::new())],
            vec![(B_SAVE_CODE, "Save code".into(), Style::Primary), (B_BACK, "Back".into(), Style::Ghost)],
        ),
    }
}

/// (Re)creates the controls for the current page and sizes the window to fit.
fn build_page(d: &mut Dlg) {
    unsafe {
        for f in d.fields.drain(..) {
            let _ = DestroyWindow(f.hwnd);
        }
        for b in d.buttons.drain(..) {
            let _ = DestroyWindow(b.hwnd);
        }
        let (title, subtitle, fields, buttons) = page_spec(d);
        d.title = title;
        d.subtitle = subtitle;
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let w = s(WIDTH) - 2 * s(PAD);
        let mut y = s(PAD) + s(34) + subtitle_height(d) + s(20);

        for (id, label, password, initial) in fields {
            let label_y = y;
            y += s(20);
            let box_rc = RECT { left: s(PAD), top: y, right: s(PAD) + w, bottom: y + s(42) };
            let style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL | if password { ES_PASSWORD } else { 0 });
            let text: Vec<u16> = initial.encode_utf16().chain([0]).collect();
            let edit_h = s(22);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0), w!("EDIT"), PCWSTR(text.as_ptr()), style,
                box_rc.left + s(14), box_rc.top + (s(42) - edit_h) / 2, w - s(28), edit_h,
                d.hwnd, HMENU(id as isize as *mut _), hinstance, None,
            )
            .unwrap_or_default();
            SendMessageW(hwnd, WM_SETFONT, WPARAM(d.font_body.0 as usize), LPARAM(1));
            d.fields.push(Field { id, label, hwnd, box_rc, label_y });
            y = box_rc.bottom + s(16);
        }
        y += s(4);
        for (id, text, style) in buttons {
            let wide: Vec<u16> = text.encode_utf16().chain([0]).collect();
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0), w!("BUTTON"), PCWSTR(wide.as_ptr()),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW),
                s(PAD), y, w, s(44),
                d.hwnd, HMENU(id as isize as *mut _), hinstance, None,
            )
            .unwrap_or_default();
            d.buttons.push(Btn { id, text, style, hwnd });
            y += s(44) + s(10);
        }
        // The small × in the corner.
        let x_btn = CreateWindowExW(
            WINDOW_EX_STYLE(0), w!("BUTTON"), w!("✕"), WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_OWNERDRAW),
            s(WIDTH) - s(PAD) - s(28), s(PAD) - s(6), s(28), s(28),
            d.hwnd, HMENU(B_X as isize as *mut _), hinstance, None,
        )
        .unwrap_or_default();
        d.buttons.push(Btn { id: B_X, text: "✕".into(), style: Style::Ghost, hwnd: x_btn });

        d.status_y = y;
        let height = y + s(26) + s(PAD) - s(4);
        let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        let _ = MoveWindow(d.hwnd, (sw - s(WIDTH)) / 2, (sh - height) / 2, s(WIDTH), height, true);
        let _ = InvalidateRect(d.hwnd, None, true);
    }
}

fn subtitle_height(d: &Dlg) -> i32 {
    // Long subtitles wrap to two lines; short ones take one.
    if d.subtitle.chars().count() > 46 { s(42) } else { s(22) }
}

fn focus_first(d: &Dlg) {
    let target = d.fields.first().map(|f| f.hwnd).or_else(|| d.buttons.first().map(|b| b.hwnd));
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

fn set_status(d: &mut Dlg, text: impl Into<String>, color: COLORREF) {
    d.status = text.into();
    d.status_color = color;
    unsafe {
        let _ = InvalidateRect(d.hwnd, None, true);
    }
}

fn set_busy(d: &mut Dlg, busy: bool) {
    d.busy = busy;
    unsafe {
        for h in d.fields.iter().map(|f| f.hwnd).chain(d.buttons.iter().filter(|b| b.id != B_X).map(|b| b.hwnd)) {
            let _ = EnableWindow(h, !busy);
        }
    }
}

fn go(d: &mut Dlg, page: Page) {
    d.page = page;
    d.delete_armed = false;
    d.status.clear();
    build_page(d);
    focus_first(d);
}

fn finish(d: &mut Dlg, result: Option<LaunchChoice>) {
    d.result = result;
    unsafe {
        let _ = DestroyWindow(d.hwnd);
    }
}

/// The page's main action, for Enter.
fn primary_id(d: &Dlg) -> Option<i32> {
    d.buttons.iter().find(|b| b.style == Style::Primary).map(|b| b.id)
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
    if d.busy {
        return;
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
            let code = text_of(d, F_CODE);
            if code.is_empty() {
                return set_status(d, "Type your code first.", ERROR_TEXT);
            }
            start(d, Job::Login(code), "Logging in…");
        }
        B_DO_SIGNUP => {
            let (code, key) = (text_of(d, F_CODE), text_of(d, F_KEY));
            if code.is_empty() || key.is_empty() {
                return set_status(d, "Fill in both the code and the API key.", ERROR_TEXT);
            }
            start(d, Job::Signup(code, key), "Checking the key and creating your account…");
        }
        B_SAVE_KEY => {
            let key = text_of(d, F_KEY);
            if key.is_empty() {
                return set_status(d, "Paste the new API key first.", ERROR_TEXT);
            }
            start(d, Job::ChangeKey(d.account.code.clone(), key), "Checking the key and saving…");
        }
        B_SAVE_CODE => {
            let code = text_of(d, F_CODE);
            if code.is_empty() {
                return set_status(d, "Type the new code first.", ERROR_TEXT);
            }
            start(d, Job::ChangeCode(d.account.code.clone(), code), "Saving…");
        }
        B_LOGOUT => {
            // Not destructive: the account stays on the server; logging in again restores it.
            if let Err(e) = crate::config::load::clear_account() {
                return set_status(d, format!("Couldn't update config.toml: {e}"), ERROR_TEXT);
            }
            restart(false);
        }
        B_DELETE => {
            // GUARD: destructive and irreversible (removes the code and key from the server), so
            // it takes two clicks — the first only arms it and says what will happen.
            if !d.delete_armed {
                d.delete_armed = true;
                return set_status(d, "This permanently deletes your code and its key from the server. Click Delete account again to confirm.", ERROR_TEXT);
            }
            start(d, Job::Delete(d.account.code.clone()), "Deleting…");
        }
        _ => {}
    }
}

fn close(d: &mut Dlg) {
    match (d.mode, d.page) {
        (Mode::Launch, _) => finish(d, Some(LaunchChoice::Quit)),
        (Mode::Account, _) => finish(d, None),
    }
}

fn start(d: &mut Dlg, job: Job, working: &str) {
    set_busy(d, true);
    set_status(d, working, MUTED);
    let hwnd = d.hwnd.0 as isize;
    std::thread::spawn(move || {
        let done = run_job(job);
        *DONE.lock().unwrap() = Some(done);
        // SAFETY: posting to a window owned by another thread is what PostMessage is for.
        unsafe {
            let _ = PostMessageW(HWND(hwnd as *mut _), WM_APP_DONE, WPARAM(0), LPARAM(0));
        }
    });
}

fn run_job(job: Job) -> Done {
    let r: anyhow::Result<Done> = account::block_on(async move {
        Ok(match job {
            Job::Login(code) => match account::login(&code).await? {
                Some(key) => Done::Saved { code, key },
                None => Done::Fail("No account with that code.".into()),
            },
            Job::Signup(code, key) => {
                if !account::openrouter_key_works(&key).await? {
                    return Ok(Done::Fail("OpenRouter doesn't accept that API key.".into()));
                }
                match account::signup(&code, &key).await?.as_str() {
                    "ok" => Done::Saved { code, key },
                    "taken" => Done::Fail("That code is already in use. Pick another one.".into()),
                    other => Done::Fail(format!("Couldn't sign up ({other}).")),
                }
            }
            Job::ChangeKey(code, key) => {
                if !account::openrouter_key_works(&key).await? {
                    return Ok(Done::Fail("OpenRouter doesn't accept that API key.".into()));
                }
                match account::update(&code, None, Some(&key)).await?.as_str() {
                    "ok" => Done::KeyChanged { key },
                    "not_found" => Done::Fail("This code no longer exists on the server. Log out and sign up again.".into()),
                    other => Done::Fail(format!("Couldn't change the key ({other}).")),
                }
            }
            Job::ChangeCode(code, new_code) => match account::update(&code, Some(&new_code), None).await?.as_str() {
                "ok" => Done::CodeChanged { code: new_code },
                "taken" => Done::Fail("That code is already in use. Pick another one.".into()),
                "not_found" => Done::Fail("This code no longer exists on the server. Log out and sign up again.".into()),
                other => Done::Fail(format!("Couldn't change the code ({other}).")),
            },
            Job::Delete(code) => {
                account::delete(&code).await?;
                Done::Deleted
            }
        })
    });
    r.unwrap_or_else(|e| Done::Fail(format!("{e}")))
}

fn on_done(d: &mut Dlg) {
    let Some(done) = DONE.lock().unwrap().take() else { return };
    set_busy(d, false);
    match done {
        Done::Fail(msg) => set_status(d, msg, ERROR_TEXT),
        Done::Saved { code, key } => {
            if let Err(e) = crate::config::load::save_account(&code, &key) {
                log::warn!("account: couldn't save login to config.toml: {e}");
            }
            match d.mode {
                Mode::Launch => finish(d, Some(LaunchChoice::Use { key, from_account: true })),
                Mode::Account => restart(true),
            }
        }
        Done::KeyChanged { key } => {
            let code = d.account.code.clone();
            if let Err(e) = crate::config::load::save_account(&code, &key) {
                return set_status(d, format!("Key changed on the server, but config.toml couldn't be updated: {e}"), ERROR_TEXT);
            }
            restart(true);
        }
        Done::CodeChanged { code } => {
            let key = d.account.api_key.clone();
            if let Err(e) = crate::config::load::save_account(&code, &key) {
                return set_status(d, format!("Code changed on the server, but config.toml couldn't be updated: {e}"), ERROR_TEXT);
            }
            d.account.code = code;
            go(d, Page::Home);
            set_status(d, "Code changed. Use the new one from now on.", OK_TEXT);
        }
        Done::Deleted => {
            let _ = crate::config::load::clear_account();
            restart(false);
        }
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

// ---- painting -------------------------------------------------------------------------------------

unsafe fn round_rect(hdc: HDC, rc: RECT, fill: COLORREF, border: COLORREF, radius: i32) {
    let brush = CreateSolidBrush(fill);
    let pen = CreatePen(PS_SOLID, s(1).max(1), border);
    let old_b = SelectObject(hdc, HGDIOBJ(brush.0));
    let old_p = SelectObject(hdc, HGDIOBJ(pen.0));
    let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
    let _ = DeleteObject(HGDIOBJ(brush.0));
    let _ = DeleteObject(HGDIOBJ(pen.0));
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

unsafe fn paint(d: &Dlg) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(d.hwnd, &mut ps);
    let mut client = RECT::default();
    let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(d.hwnd, &mut client);
    FillRect(hdc, &client, d.bg_brush);
    let w = client.right - 2 * s(PAD);
    let line = |top: i32, h: i32| RECT { left: s(PAD), top, right: s(PAD) + w, bottom: top + h };

    text(hdc, d.font_title, TEXT, &d.title, line(s(PAD) - s(4), s(34)), DT_LEFT | DT_SINGLELINE | DT_VCENTER);
    text(hdc, d.font_body, MUTED, &d.subtitle, line(s(PAD) + s(34), subtitle_height(d)), DT_LEFT | DT_WORDBREAK);

    let focused = GetFocus();
    for f in &d.fields {
        text(hdc, d.font_small, MUTED, f.label, line(f.label_y, s(18)), DT_LEFT | DT_SINGLELINE);
        let border = if f.hwnd == focused { ACCENT } else { BORDER };
        round_rect(hdc, f.box_rc, FIELD_BG, border, s(RADIUS));
    }
    if !d.status.is_empty() {
        let mut rc = line(d.status_y - s(2), s(40));
        rc.right = client.right - s(PAD);
        text(hdc, d.font_small, d.status_color, &d.status, rc, DT_LEFT | DT_WORDBREAK);
    }
    let _ = EndPaint(d.hwnd, &ps);
}

unsafe fn draw_button(d: &Dlg, item: &DrawItem) {
    let Some(b) = d.buttons.iter().find(|b| b.id == item.ctl_id as i32) else { return };
    let pressed = item.item_state & ODS_SELECTED != 0;
    let disabled = item.item_state & ODS_DISABLED != 0;
    FillRect(item.hdc, &item.rc, d.bg_brush);
    let (fill, border, fg) = match b.style {
        Style::Primary => (if pressed { ACCENT_DOWN } else { ACCENT }, if pressed { ACCENT_DOWN } else { ACCENT }, TEXT),
        Style::Secondary => (if pressed { SECONDARY_DOWN } else { SECONDARY }, BORDER, TEXT),
        Style::Danger => {
            let armed = d.delete_armed;
            let c = if pressed { DANGER_DOWN } else if armed { DANGER } else { SECONDARY };
            (c, if armed { DANGER } else { BORDER }, if armed { TEXT } else { DANGER })
        }
        Style::Ghost => (if pressed { SECONDARY } else { BG }, if pressed { BORDER } else { BG }, MUTED),
    };
    let fg = if disabled { MUTED } else { fg };
    round_rect(item.hdc, item.rc, fill, border, s(RADIUS));
    let font = if b.id == B_X { d.font_small } else { d.font_body };
    text(item.hdc, font, fg, &b.text, item.rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(d) = dlg().filter(|d| d.hwnd == hwnd) else {
        if msg == WM_DESTROY {
            PostQuitMessage(0);
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
        WM_CTLCOLOREDIT => {
            let hdc = HDC(wp.0 as *mut _);
            SetTextColor(hdc, TEXT);
            SetBkColor(hdc, FIELD_BG);
            LRESULT(d.field_brush.0 as isize)
        }
        // Drag the window by any empty spot.
        WM_NCHITTEST => {
            let hit = DefWindowProcW(hwnd, msg, wp, lp);
            if hit.0 == HTCLIENT { LRESULT(HTCAPTION) } else { hit }
        }
        WM_COMMAND => {
            let id = (wp.0 & 0xFFFF) as i32;
            let code = ((wp.0 >> 16) & 0xFFFF) as u32;
            match id {
                IDOK => {
                    if let Some(p) = primary_id(d) {
                        on_action(d, p);
                    }
                }
                IDCANCEL => {
                    if d.page == Page::Home { close(d) } else { on_action(d, B_BACK) }
                }
                _ if code == EN_SETFOCUS || code == EN_KILLFOCUS => {
                    let _ = InvalidateRect(hwnd, None, false);
                }
                _ if code == BN_CLICKED => on_action(d, id),
                _ => {}
            }
            LRESULT(0)
        }
        WM_APP_DONE => {
            on_done(d);
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
            status_color: MUTED,
            fields: vec![],
            buttons: vec![],
            status_y: 0,
            busy: false,
            delete_armed: false,
            font_title: HFONT::default(),
            font_body: HFONT::default(),
            font_small: HFONT::default(),
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
