// Settings dialog — GUI editor for config.toml.
// Same Win32 + GDI stack as account_dialog: owner-drawn buttons, dark theme, no framework.
// Opens from the account dialog's Home page or (later) a dedicated hotkey.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetMonitorInfoW, GetStockObject, InvalidateRect, MonitorFromPoint,
    RoundRect, SelectObject, SetBkColor, SetBkMode, SetTextColor, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DEFAULT_PITCH, DRAW_TEXT_FORMAT, DT_CALCRECT, DT_CENTER, DT_END_ELLIPSIS,
    DT_LEFT, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, HBRUSH, HDC, HFONT, HGDIOBJ,
    MONITORINFO, MONITOR_DEFAULTTONEAREST, NULL_BRUSH, OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    GetClientRect, GetCursorPos, GetMessageW, GetWindowTextLengthW, GetWindowTextW,
    IsDialogMessageW, LoadCursorW, MoveWindow, PostMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW,
    SetForegroundWindow, SetWindowDisplayAffinity, ShowWindow, TranslateMessage,
    HMENU, IDC_ARROW, MSG, SW_SHOW, WDA_EXCLUDEFROMCAPTURE, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_DESTROY, WM_PAINT, WNDCLASSEXW, WS_CHILD, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    WS_TABSTOP, WS_VISIBLE,
};

use crate::config::schema::Config;
use super::{settings_controls, settings_models};

// ---- palette (same as account_dialog — same dark theme) -------------------------------------------

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}
const BG: COLORREF = rgb(0x1F, 0x1E, 0x1D);
const SURFACE: COLORREF = rgb(0x30, 0x30, 0x2E);
const SURFACE_DOWN: COLORREF = rgb(0x42, 0x41, 0x3D);
const FIELD_BG: COLORREF = rgb(0x26, 0x26, 0x24);
const BORDER: COLORREF = rgb(0x3E, 0x3D, 0x39);
const TEXT: COLORREF = rgb(0xF5, 0xF4, 0xEE);
const MUTED: COLORREF = rgb(0xA8, 0xA6, 0x9D);
const ACCENT: COLORREF = rgb(0xD9, 0x77, 0x57);
const ACCENT_HOVER: COLORREF = rgb(0xE0, 0x84, 0x65);
const ACCENT_DOWN: COLORREF = rgb(0xC2, 0x61, 0x3F);
const ACCENT_RING: COLORREF = rgb(0xF6, 0xC9, 0xB8);
const ON_ACCENT: COLORREF = rgb(0xFF, 0xFF, 0xFF);
const TAB_ACTIVE: COLORREF = ACCENT;

// ---- layout ---------------------------------------------------------------------------------------

const WIDTH: i32 = 400;
const PAD: i32 = 12;
const TAB_BAR_Y: i32 = 6;
const TAB_BAR_H: i32 = 28;
const CONTENT_TOP: i32 = TAB_BAR_Y + TAB_BAR_H + 6;
const ROW_H: i32 = 26;
const ROW_GAP: i32 = 2;
const FIELD_H: i32 = 24;
const SAVE_H: i32 = 28;
const SAVE_PAD: i32 = 8;
const RADIUS: i32 = 6;
const X_SIZE: i32 = 22;

// Segoe MDL2 Assets glyphs.
const I_SAVE: char = '\u{E74E}';
const I_CHECK: char = '\u{E73E}';
const I_BACK: char = '\u{E72B}';
const I_SETTINGS: char = '\u{E713}';

fn s(px: i32) -> i32 {
    let dpi = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() }.max(96) as i32;
    px * dpi / 96
}

// Win32 message constants not in our `windows` features.
const WM_CLOSE: u32 = 0x0010;
const WM_SETCURSOR: u32 = 0x0020;
const WM_SETFONT: u32 = 0x0030;
const WM_DRAWITEM: u32 = 0x002B;
const WM_NCHITTEST: u32 = 0x0084;
const WM_COMMAND: u32 = 0x0111;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WM_CHANGEUISTATE: u32 = 0x0127;
const WM_USER: u32 = 0x0400;
const WM_CLOUD_DONE: u32 = WM_USER + 1; // WPARAM: 1 = ok, 0 = error
const HTCLIENT: isize = 1;
const HTCAPTION: isize = 2;
const IDOK: i32 = 1;
const IDCANCEL: i32 = 2;
const BN_CLICKED: u32 = 0;
const BS_OWNERDRAW: u32 = 0x000B;
const ES_AUTOHSCROLL: u32 = 0x0080;
const ES_PASSWORD: u32 = 0x0020;
const ODS_SELECTED: u32 = 0x0001;
const ODS_FOCUS: u32 = 0x0010;
const ODS_NOFOCUSRECT: u32 = 0x0200;
const HIDE_FOCUS_CUES: usize = 0x0001_0001;
const DWMWA_USE_IMMERSIVE_DARK_MODE: i32 = 20;
const DWMWA_WINDOW_CORNER_PREFERENCE: i32 = 33;
const DWMWA_BORDER_COLOR: i32 = 34;
const DWMWCP_ROUND: u32 = 2;
const CLEARTYPE_QUALITY: u32 = 5;

#[repr(C)]
struct DrawItem {
    _ctl_type: u32,
    ctl_id: u32,
    _item_id: u32,
    _item_action: u32,
    item_state: u32,
    hwnd_item: HWND,
    hdc: HDC,
    rc: RECT,
    _item_data: usize,
}

// ---- tabs -----------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tab {
    Models,
    Hotkeys,
    Appearance,
    Advanced,
}

const TABS: [Tab; 4] = [Tab::Models, Tab::Hotkeys, Tab::Appearance, Tab::Advanced];

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Models => "Models",
            Tab::Hotkeys => "Hotkeys",
            Tab::Appearance => "Look",
            Tab::Advanced => "Advanced",
        }
    }
}

// ---- field kinds ----------------------------------------------------------------------------------

#[derive(Clone)]
enum FieldKind {
    Text { password: bool },
    Hotkey,
    Color,
    Corner,
    Number,           // whole number
    Decimal,          // e.g. 0.85
    Toggle,
    ModelList,        // opens model picker popup
    Offset,           // pair of numbers "x, y"
}

// ---- field definitions per tab --------------------------------------------------------------------

/// (label, value, kind). Returned in display order.
fn tab_fields(config: &Config, tab: Tab) -> Vec<(&'static str, String, FieldKind)> {
    match tab {
        Tab::Models => vec![
            ("API Key", config.openrouter.api_key.clone(), FieldKind::Text { password: true }),
            ("Models", config.openrouter.models.join(", "), FieldKind::ModelList),
            ("Screen Answer Models", config.screen_answer.models.join(", "), FieldKind::ModelList),
            ("Match Threshold", config.screen_answer.match_threshold.to_string(), FieldKind::Decimal),
        ],
        Tab::Hotkeys => vec![
            ("Clipboard Query", config.hotkeys.clipboard_query.clone(), FieldKind::Hotkey),
            ("Screenshot Query", config.hotkeys.screenshot_query.clone(), FieldKind::Hotkey),
            ("Screen Answer", config.hotkeys.screen_answer.clone(), FieldKind::Hotkey),
            ("Clear Dots", config.hotkeys.clear_dots.clone(), FieldKind::Hotkey),
            ("Account", config.hotkeys.account.clone(), FieldKind::Hotkey),
            ("Test Model", config.hotkeys.test_model.clone(), FieldKind::Hotkey),
            ("Switch Model", config.hotkeys.switch_model.clone(), FieldKind::Hotkey),
            ("Add Context", config.hotkeys.add_context.clone(), FieldKind::Hotkey),
            ("Reset Conversation", config.hotkeys.reset_conversation.clone(), FieldKind::Hotkey),
            ("Launch Debugger", config.hotkeys.launch_debugger.clone(), FieldKind::Hotkey),
            ("Forms: Answer Page", config.hotkeys.forms_answer_page.clone(), FieldKind::Hotkey),
            ("Forms: Answer All", config.hotkeys.forms_answer_all.clone(), FieldKind::Hotkey),
            ("Forms: AX Page", config.hotkeys.forms_answer_page_axtree.clone(), FieldKind::Hotkey),
            ("Forms: AX All", config.hotkeys.forms_answer_all_axtree.clone(), FieldKind::Hotkey),
            ("Abort", config.hotkeys.abort.clone(), FieldKind::Hotkey),
            ("Hide Toggle", config.hotkeys.hide_toggle.clone(), FieldKind::Hotkey),
            ("Help Toggle", config.hotkeys.help_toggle.clone(), FieldKind::Hotkey),
            ("Restart", config.hotkeys.restart_daemon.clone(), FieldKind::Hotkey),
            ("Insta-Delete", config.hotkeys.insta_delete.clone(), FieldKind::Hotkey),
            ("Panic Kill", config.hotkeys.panic_kill.clone(), FieldKind::Hotkey),
            ("SEB Unpatch", config.hotkeys.seb_unpatch.clone(), FieldKind::Hotkey),
            ("Quit", config.hotkeys.quit.clone(), FieldKind::Hotkey),
            ("Debug: Open Tab", config.hotkeys.debug_open_tab.clone(), FieldKind::Hotkey),
        ],
        Tab::Appearance => vec![
            ("Indicator Corner", config.visuals.indicator_corner.clone(), FieldKind::Corner),
            ("Indicator Size", config.visuals.indicator_size.to_string(), FieldKind::Number),
            ("Indicator Offset", format!("{}, {}", config.visuals.indicator_offset[0], config.visuals.indicator_offset[1]), FieldKind::Offset),
            ("Ready Color", config.visuals.color_ready.clone(), FieldKind::Color),
            ("Processing Color", config.visuals.color_processing.clone(), FieldKind::Color),
            ("Error Color", config.visuals.color_error.clone(), FieldKind::Color),
            ("Form Corner", config.visuals.form_indicator_corner.clone(), FieldKind::Corner),
            ("Form Offset", format!("{}, {}", config.visuals.form_indicator_offset[0], config.visuals.form_indicator_offset[1]), FieldKind::Offset),
            ("Form Running", config.visuals.form_color_running.clone(), FieldKind::Color),
            ("Form Done", config.visuals.form_color_done.clone(), FieldKind::Color),
            ("Form Failed", config.visuals.form_color_failed.clone(), FieldKind::Color),
            ("Form Aborted", config.visuals.form_color_aborted.clone(), FieldKind::Color),
            ("Overlay Enabled", config.visuals.overlay_enabled.to_string(), FieldKind::Toggle),
            ("Overlay Corner", config.visuals.overlay_corner.clone(), FieldKind::Corner),
            ("Overlay Offset", format!("{}, {}", config.visuals.overlay_offset[0], config.visuals.overlay_offset[1]), FieldKind::Offset),
            ("Overlay Font Size", config.visuals.overlay_font_size.to_string(), FieldKind::Number),
            ("Crawl Lines", config.visuals.crawl_trigger_lines.to_string(), FieldKind::Number),
            ("Crawl Speed (ms)", config.visuals.crawl_ms_per_line.to_string(), FieldKind::Number),
            ("Dot Color", config.visuals.dot_color.clone(), FieldKind::Color),
            ("Dot Size", config.visuals.dot_size.to_string(), FieldKind::Number),
        ],
        Tab::Advanced => vec![
            ("Connect Timeout (s)", config.http.connect_timeout_secs.to_string(), FieldKind::Number),
            ("Read Timeout (s)", config.http.read_timeout_secs.to_string(), FieldKind::Number),
            ("Max Pages", config.forms.max_pages.to_string(), FieldKind::Number),
            ("Max Images / Req", config.forms.max_images_per_request.to_string(), FieldKind::Number),
            ("Max Image Edge (px)", config.forms.max_image_long_edge_px.to_string(), FieldKind::Number),
            ("Knowledge Enabled", config.knowledge.enabled.to_string(), FieldKind::Toggle),
            ("Max Doc Bytes", config.knowledge.max_doc_bytes.to_string(), FieldKind::Number),
            ("Cloud URL", config.knowledge.cloud_url.clone(), FieldKind::Text { password: false }),
        ],
    }
}

/// "x, y" → [x, y]; anything unparseable keeps `current`.
fn parse_offset(s: &str, current: [i32; 2]) -> [i32; 2] {
    match s.split(',').map(|p| p.trim().parse::<i32>()).collect::<Vec<_>>().as_slice() {
        [Ok(x), Ok(y)] => [*x, *y],
        _ => current,
    }
}

/// Writes EDIT control values back into the Config struct for the given tab.
fn save_tab(config: &mut Config, tab: Tab, values: &[String]) {
    match tab {
        Tab::Models => {
            config.openrouter.api_key = values[0].clone();
            config.openrouter.models = values[1].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            config.screen_answer.models = values[2].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            config.screen_answer.match_threshold = values[3].trim().parse().unwrap_or(config.screen_answer.match_threshold);
        }
        Tab::Hotkeys => {
            let h = &mut config.hotkeys;
            h.clipboard_query = values[0].clone();
            h.screenshot_query = values[1].clone();
            h.screen_answer = values[2].clone();
            h.clear_dots = values[3].clone();
            h.account = values[4].clone();
            h.test_model = values[5].clone();
            h.switch_model = values[6].clone();
            h.add_context = values[7].clone();
            h.reset_conversation = values[8].clone();
            h.launch_debugger = values[9].clone();
            h.forms_answer_page = values[10].clone();
            h.forms_answer_all = values[11].clone();
            h.forms_answer_page_axtree = values[12].clone();
            h.forms_answer_all_axtree = values[13].clone();
            h.abort = values[14].clone();
            h.hide_toggle = values[15].clone();
            h.help_toggle = values[16].clone();
            h.restart_daemon = values[17].clone();
            h.insta_delete = values[18].clone();
            h.panic_kill = values[19].clone();
            h.seb_unpatch = values[20].clone();
            h.quit = values[21].clone();
            h.debug_open_tab = values[22].clone();
        }
        Tab::Appearance => {
            let v = &mut config.visuals;
            v.indicator_corner = values[0].clone();
            v.indicator_size = values[1].trim().parse().unwrap_or(v.indicator_size);
            v.indicator_offset = parse_offset(&values[2], v.indicator_offset);
            v.color_ready = values[3].clone();
            v.color_processing = values[4].clone();
            v.color_error = values[5].clone();
            v.form_indicator_corner = values[6].clone();
            v.form_indicator_offset = parse_offset(&values[7], v.form_indicator_offset);
            v.form_color_running = values[8].clone();
            v.form_color_done = values[9].clone();
            v.form_color_failed = values[10].clone();
            v.form_color_aborted = values[11].clone();
            v.overlay_enabled = values[12].trim().parse().unwrap_or(v.overlay_enabled);
            v.overlay_corner = values[13].clone();
            v.overlay_offset = parse_offset(&values[14], v.overlay_offset);
            v.overlay_font_size = values[15].trim().parse().unwrap_or(v.overlay_font_size);
            v.crawl_trigger_lines = values[16].trim().parse().unwrap_or(v.crawl_trigger_lines);
            v.crawl_ms_per_line = values[17].trim().parse().unwrap_or(v.crawl_ms_per_line);
            v.dot_color = values[18].clone();
            v.dot_size = values[19].trim().parse().unwrap_or(v.dot_size);
        }
        Tab::Advanced => {
            config.http.connect_timeout_secs = values[0].trim().parse().unwrap_or(config.http.connect_timeout_secs);
            config.http.read_timeout_secs = values[1].trim().parse().unwrap_or(config.http.read_timeout_secs);
            config.forms.max_pages = values[2].trim().parse().unwrap_or(config.forms.max_pages);
            config.forms.max_images_per_request = values[3].trim().parse().unwrap_or(config.forms.max_images_per_request);
            config.forms.max_image_long_edge_px = values[4].trim().parse().unwrap_or(config.forms.max_image_long_edge_px);
            config.knowledge.enabled = values[5].trim().parse().unwrap_or(config.knowledge.enabled);
            config.knowledge.max_doc_bytes = values[6].trim().parse().unwrap_or(config.knowledge.max_doc_bytes);
            config.knowledge.cloud_url = values[7].clone();
        }
    }
}

// ---- state ----------------------------------------------------------------------------------------

const B_SAVE: i32 = 101;
const B_X: i32 = 102;
const F_FIRST: i32 = 301;

struct SField {
    label: &'static str,
    hwnd: HWND,
    row_y: i32,
    kind: FieldKind,
}

#[derive(Clone, Copy, PartialEq)]
enum CloudStatus {
    None,
    Saving,
    Saved,
    Error,
}

struct Sett {
    hwnd: HWND,
    tab: Tab,
    config: Config,
    fields: Vec<SField>,
    scroll_y: i32,
    max_scroll: i32,
    content_hwnd: HWND,
    hover: HWND,
    saved: bool,
    save_failed: bool,
    cloud: CloudStatus,
    font_title: HFONT,
    font_body: HFONT,
    font_small: HFONT,
    font_icon: HFONT,
    bg_brush: HBRUSH,
    field_brush: HBRUSH,
}

thread_local! {
    static SETT: std::cell::Cell<*mut Sett> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

fn sett() -> Option<&'static mut Sett> {
    unsafe { SETT.with(|c| c.get()).as_mut() }
}

// ---- window ---------------------------------------------------------------------------------------

fn run(config: Config) {
    static REGISTER: Once = Once::new();
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap();
        REGISTER.call_once(|| {
            for (name, proc) in [
                (w!("SysSettHost"), wnd_proc as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT),
                (w!("SysSettContent"), content_proc),
            ] {
                let wc = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    lpfnWndProc: Some(proc),
                    hInstance: hinstance.into(),
                    hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                    lpszClassName: name,
                    ..Default::default()
                };
                RegisterClassExW(&wc);
            }
            settings_controls::register_classes();
            settings_models::register_class();
        });

        let font = |size: i32, weight: i32, face: PCWSTR| {
            CreateFontW(
                -s(size), 0, 0, 0, weight, 0, 0, 0,
                DEFAULT_CHARSET.0 as u32, OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32, CLEARTYPE_QUALITY,
                (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32, face,
            )
        };

        let state = Box::into_raw(Box::new(Sett {
            hwnd: HWND::default(),
            tab: Tab::Models,
            config,
            fields: Vec::new(),
            scroll_y: 0,
            max_scroll: 0,
            content_hwnd: HWND::default(),
            hover: HWND::default(),
            saved: false,
            save_failed: false,
            cloud: CloudStatus::None,
            font_title: font(14, 600, w!("Segoe UI")),
            font_body: font(12, 400, w!("Segoe UI")),
            font_small: font(11, 400, w!("Segoe UI")),
            font_icon: font(12, 400, w!("Segoe MDL2 Assets")),
            bg_brush: CreateSolidBrush(BG),
            field_brush: CreateSolidBrush(FIELD_BG),
        }));
        SETT.with(|c| c.set(state));

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("SysSettHost"), w!(""),
            WS_POPUP,
            0, 0, s(WIDTH), s(400),
            None, None, hinstance, None,
        )
        .unwrap();
        let d = sett().unwrap();
        d.hwnd = hwnd;

        // Content container — clipped child that holds the EDIT controls.
        d.content_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("SysSettContent"), w!(""),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x04000000), // WS_CLIPSIBLINGS
            0, 0, 0, 0,
            hwnd, None, hinstance, None,
        )
        .unwrap();

        let set = |attr: i32, val: u32| {
            let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(attr), &val as *const u32 as *const _, 4);
        };
        set(DWMWA_USE_IMMERSIVE_DARK_MODE, 1);
        set(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND);
        set(DWMWA_BORDER_COLOR, BORDER.0);
        let capturable = cfg!(feature = "debug") && std::env::var_os("SHADOWPROMPT_CAPTURABLE").is_some();
        if !capturable {
            let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
        }
        SendMessageW(hwnd, WM_CHANGEUISTATE, WPARAM(HIDE_FOCUS_CUES), LPARAM(0));

        // Create Save + X buttons (children of main window, not the content container).
        create_button(hwnd, B_X, hinstance.into());
        create_button(hwnd, B_SAVE, hinstance.into());

        build_tab(d);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        let d = Box::from_raw(SETT.with(|c| c.replace(std::ptr::null_mut())));
        for obj in [
            HGDIOBJ(d.font_title.0), HGDIOBJ(d.font_body.0),
            HGDIOBJ(d.font_small.0), HGDIOBJ(d.font_icon.0),
            HGDIOBJ(d.bg_brush.0), HGDIOBJ(d.field_brush.0),
        ] {
            let _ = DeleteObject(obj);
        }
    }
}

unsafe fn create_button(parent: HWND, id: i32, hinstance: windows::Win32::Foundation::HINSTANCE) {
    CreateWindowExW(
        WINDOW_EX_STYLE(0), w!("BUTTON"), PCWSTR::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW),
        0, 0, 0, 0,
        parent, HMENU(id as isize as *mut _), hinstance, None,
    )
    .unwrap_or_default();
}

/// Tabs start right of the Back button.
fn tabs_left() -> i32 {
    s(PAD) + s(X_SIZE) + s(4)
}

fn inner_w() -> i32 {
    s(WIDTH) - 2 * s(PAD)
}

// ---- build / layout -------------------------------------------------------------------------------

/// Destroys current field controls and creates new ones for the active tab.
fn build_tab(d: &mut Sett) {
    unsafe {
        for f in d.fields.drain(..) {
            match &f.kind {
                FieldKind::Color | FieldKind::Toggle | FieldKind::Corner => settings_controls::cleanup(f.hwnd),
                _ => {}
            }
            let _ = DestroyWindow(f.hwnd);
        }
        d.scroll_y = 0;
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let specs = tab_fields(&d.config, d.tab);
        for (i, (label, value, kind)) in specs.into_iter().enumerate() {
            let id = F_FIRST + i as i32;
            let hwnd = match &kind {
                FieldKind::Text { password } => {
                    let style = WS_CHILD | WS_VISIBLE | WS_TABSTOP
                        | WINDOW_STYLE(ES_AUTOHSCROLL | if *password { ES_PASSWORD } else { 0 });
                    let text: Vec<u16> = value.encode_utf16().chain([0]).collect();
                    let h = CreateWindowExW(
                        WINDOW_EX_STYLE(0), w!("EDIT"), PCWSTR(text.as_ptr()), style,
                        0, 0, 0, 0,
                        d.content_hwnd, HMENU(id as isize as *mut _), hinstance, None,
                    )
                    .unwrap_or_default();
                    SendMessageW(h, WM_SETFONT, WPARAM(d.font_body.0 as usize), LPARAM(1));
                    h
                }
                FieldKind::Hotkey => {
                    settings_controls::create_hotkey(d.content_hwnd, id, 0, 0, 0, 0, &value, d.font_body)
                }
                FieldKind::Color => {
                    settings_controls::create_color(d.content_hwnd, id, 0, 0, 0, 0, &value, d.font_body)
                }
                FieldKind::Corner => {
                    settings_controls::create_corner(d.content_hwnd, id, 0, 0, 0, 0, &value, d.font_body)
                }
                FieldKind::Number | FieldKind::Decimal | FieldKind::Offset => {
                    let digits_only = matches!(kind, FieldKind::Number);
                    settings_controls::create_number(d.content_hwnd, id, 0, 0, 0, 0, &value, digits_only, d.font_body)
                }
                FieldKind::Toggle => {
                    settings_controls::create_toggle(d.content_hwnd, id, 0, 0, 0, 0, value == "true", d.font_body)
                }
                FieldKind::ModelList => {
                    // Owner-drawn button holding the comma-separated IDs as its text; click opens the picker.
                    let style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW);
                    let text: Vec<u16> = value.encode_utf16().chain([0]).collect();
                    let h = CreateWindowExW(
                        WINDOW_EX_STYLE(0), w!("BUTTON"), PCWSTR(text.as_ptr()), style,
                        0, 0, 0, 0,
                        d.content_hwnd, HMENU(id as isize as *mut _), hinstance, None,
                    )
                    .unwrap_or_default();
                    SendMessageW(h, WM_SETFONT, WPARAM(d.font_body.0 as usize), LPARAM(1));
                    h
                }
            };
            d.fields.push(SField { label, hwnd, row_y: 0, kind });
        }

        // Kick off model fetch when Models tab is active.
        if d.tab == Tab::Models {
            let key = d.config.openrouter.api_key.clone();
            settings_models::ensure_models(&key);
        }
    }
    layout(d);
    if let Some(f) = d.fields.first() {
        unsafe { let _ = SetFocus(f.hwnd); }
    }
}

/// Positions everything and sizes the main window.
fn layout(d: &mut Sett) {
    let x0 = s(PAD);
    let w = inner_w();
    let label_w = w * 42 / 100;
    let field_w = w - label_w - s(4);
    let field_x = x0 + label_w + s(4);

    // Content area geometry.
    let content_top = s(CONTENT_TOP);
    let row_step = s(ROW_H) + s(ROW_GAP);
    let content_h = d.fields.len() as i32 * row_step;

    // Limit window height to 85% of screen work area.
    let max_h = unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        let _ = GetMonitorInfoW(MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST), &mut mi);
        (mi.rcWork.bottom - mi.rcWork.top) * 85 / 100
    };

    let ideal_h = content_top + content_h + s(SAVE_PAD) + s(SAVE_H) + s(PAD);
    let win_h = ideal_h.min(max_h);
    let visible_content_h = win_h - content_top - s(SAVE_PAD) - s(SAVE_H) - s(PAD);
    d.max_scroll = (content_h - visible_content_h).max(0);
    d.scroll_y = d.scroll_y.clamp(0, d.max_scroll);

    // Position content container.
    unsafe {
        let _ = MoveWindow(d.content_hwnd, 0, content_top, s(WIDTH), visible_content_h, true);
    }

    // Position each field's EDIT control within the content container.
    let edit_h = s(FIELD_H);
    for (i, f) in d.fields.iter_mut().enumerate() {
        let y = i as i32 * row_step - d.scroll_y;
        f.row_y = y;
        let edit_y = y + (s(ROW_H) - edit_h) / 2;
        unsafe {
            let _ = MoveWindow(f.hwnd, field_x, edit_y, field_w, edit_h, true);
        }
    }

    // Save button at the bottom of the main window.
    let save_y = content_top + visible_content_h + s(SAVE_PAD);
    let save_w = s(90);
    unsafe {
        // Find the save button by iterating child windows... or just use FindWindowEx.
        // Simpler: save button HWND stored? No — we use the button ID approach.
        // Let's use GetDlgItem equivalent: the button is a child of the main window with id B_SAVE.
        use windows::Win32::UI::WindowsAndMessaging::GetDlgItem;
        if let Ok(save_btn) = GetDlgItem(d.hwnd, B_SAVE) {
            let _ = MoveWindow(save_btn, s(WIDTH) - s(PAD) - save_w, save_y, save_w, s(SAVE_H), true);
        }

        // Back button in the top-left corner, left of the tabs.
        if let Ok(x_btn) = GetDlgItem(d.hwnd, B_X) {
            let y = s(TAB_BAR_Y) + (s(TAB_BAR_H) - s(X_SIZE)) / 2;
            let _ = MoveWindow(x_btn, s(PAD), y, s(X_SIZE), s(X_SIZE), true);
        }

        // Size + centre the main window.
        let mut rc = RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(d.hwnd, &mut rc);
        let first_show = !windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(d.hwnd).as_bool();
        let (cx, cy) = if first_show {
            // First show — centre on monitor.
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
            let _ = GetMonitorInfoW(MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST), &mut mi);
            let wa = mi.rcWork;
            (wa.left + (wa.right - wa.left - s(WIDTH)) / 2,
             wa.top + (wa.bottom - wa.top - win_h) * 9 / 20)
        } else {
            // Keep centre.
            let mid_x = (rc.left + rc.right) / 2;
            let mid_y = (rc.top + rc.bottom) / 2;
            (mid_x - s(WIDTH) / 2, mid_y - win_h / 2)
        };
        let _ = MoveWindow(d.hwnd, cx, cy, s(WIDTH), win_h, true);
        let _ = InvalidateRect(d.hwnd, None, true);
        let _ = InvalidateRect(d.content_hwnd, None, true);
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
    if let Some(b) = brush { let _ = DeleteObject(HGDIOBJ(b.0)); }
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

unsafe fn text_width(hdc: HDC, font: HFONT, s_: &str) -> i32 {
    let mut rc = RECT::default();
    SelectObject(hdc, HGDIOBJ(font.0));
    let mut wide: Vec<u16> = s_.encode_utf16().collect();
    if !wide.is_empty() {
        DrawTextW(hdc, &mut wide, &mut rc, DT_SINGLELINE | DT_CALCRECT);
    }
    rc.right - rc.left
}

unsafe fn paint_main(d: &Sett) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(d.hwnd, &mut ps);
    let mut client = RECT::default();
    let _ = GetClientRect(d.hwnd, &mut client);
    FillRect(hdc, &client, d.bg_brush);

    // Tab bar.
    let tab_count = TABS.len() as i32;
    let tab_w = (s(WIDTH) - s(PAD) * 2 - s(X_SIZE) - s(4)) / tab_count;
    for (i, &tab) in TABS.iter().enumerate() {
        let x = tabs_left() + i as i32 * tab_w;
        let y = s(TAB_BAR_Y);
        let rc = RECT { left: x, top: y, right: x + tab_w, bottom: y + s(TAB_BAR_H) };
        let active = tab == d.tab;
        if active {
            let fill_rc = RECT { left: rc.left, top: rc.bottom - s(2), right: rc.right, bottom: rc.bottom };
            let brush = CreateSolidBrush(TAB_ACTIVE);
            FillRect(hdc, &fill_rc, brush);
            let _ = DeleteObject(HGDIOBJ(brush.0));
        }
        let color = if active { TEXT } else { MUTED };
        text(hdc, d.font_body, color, tab.label(), rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    }

    let _ = EndPaint(d.hwnd, &ps);
}

/// Paints labels for the field rows inside the content container.
unsafe fn paint_content(d: &Sett) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(d.content_hwnd, &mut ps);
    let mut client = RECT::default();
    let _ = GetClientRect(d.content_hwnd, &mut client);
    FillRect(hdc, &client, d.bg_brush);

    let x0 = s(PAD);
    let label_w = inner_w() * 42 / 100;
    for f in &d.fields {
        let rc = RECT {
            left: x0,
            top: f.row_y,
            right: x0 + label_w,
            bottom: f.row_y + s(ROW_H),
        };
        text(hdc, d.font_small, MUTED, f.label, rc, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);
    }

    let _ = EndPaint(d.content_hwnd, &ps);
}

unsafe fn draw_button(d: &Sett, item: &DrawItem) {
    let hdc = item.hdc;
    let rc = item.rc;
    let pressed = item.item_state & ODS_SELECTED != 0;
    let focus = item.item_state & ODS_FOCUS != 0 && item.item_state & ODS_NOFOCUSRECT == 0;
    let hover = d.hover == item.hwnd_item;

    FillRect(hdc, &rc, d.bg_brush);

    match item.ctl_id as i32 {
        B_X => {
            if pressed {
                let brush = CreateSolidBrush(SURFACE_DOWN);
                FillRect(hdc, &rc, brush);
                let _ = DeleteObject(HGDIOBJ(brush.0));
            } else if hover {
                let brush = CreateSolidBrush(SURFACE);
                FillRect(hdc, &rc, brush);
                let _ = DeleteObject(HGDIOBJ(brush.0));
            }
            let fg = if hover { TEXT } else { MUTED };
            text(hdc, d.font_icon, fg, &I_BACK.to_string(), rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        }
        B_SAVE => {
            let bg = if pressed { ACCENT_DOWN } else if hover { ACCENT_HOVER } else { ACCENT };
            round_rect(hdc, rc, Some(bg), bg, s(RADIUS));
            if focus {
                let inset = RECT { left: rc.left + s(2), top: rc.top + s(2), right: rc.right - s(2), bottom: rc.bottom - s(2) };
                round_rect(hdc, inset, None, ACCENT_RING, s(RADIUS) - s(2));
            }
            let label = match (d.saved, d.cloud) {
                _ if d.save_failed => "Couldn't save",
                (_, CloudStatus::Saving) => "Saving to cloud\u{2026}",
                (_, CloudStatus::Saved) => "Saved to cloud",
                (_, CloudStatus::Error) => "Saved locally",
                (true, _) => "Saved",
                _ => "Save",
            };
            let icon = if d.saved || d.cloud == CloudStatus::Saved { I_CHECK } else { I_SAVE };
            let icon_s = icon.to_string();
            let icon_w = text_width(hdc, d.font_icon, &icon_s);
            let label_w = text_width(hdc, d.font_body, label);
            let gap = s(6);
            let total = icon_w + gap + label_w;
            let left = rc.left + (rc.right - rc.left - total) / 2;
            let irc = RECT { left, top: rc.top, right: left + icon_w + s(2), bottom: rc.bottom };
            text(hdc, d.font_icon, ON_ACCENT, &icon_s, irc, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
            let lrc = RECT { left: left + icon_w + gap, top: rc.top, right: rc.right - s(4), bottom: rc.bottom };
            text(hdc, d.font_body, ON_ACCENT, label, lrc, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
        }
        _ => {}
    }
}

/// Model-list field: shared field box, chosen IDs (ellipsized).
unsafe fn draw_model_field(d: &Sett, item: &DrawItem) {
    FillRect(item.hdc, &item.rc, d.bg_brush);
    let pressed = item.item_state & ODS_SELECTED != 0;
    let inner = settings_controls::draw_field_box(item.hdc, item.rc, pressed);
    let value = window_text(item.hwnd_item);
    let (label, color) = if value.is_empty() { ("Choose models\u{2026}", MUTED) } else { (value.as_str(), TEXT) };
    text(item.hdc, d.font_body, color, label, inner, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);
}

/// Opens the model picker under `field`; writes the new selection back on confirm.
unsafe fn open_model_picker(d: &Sett, field: HWND) {
    // Prefer the key typed in this tab (it may not be saved yet) for the model fetch.
    let key = d.fields.iter()
        .find(|f| f.label == "API Key")
        .map(read_field)
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| d.config.openrouter.api_key.clone());
    let current = window_text(field);
    // Only the main list has a switchable pair; Screen Answer is a plain fallback chain.
    let switchable = d.fields.iter().any(|f| f.hwnd == field && f.label == "Models");
    let (owner, font) = (d.hwnd, d.font_body);
    let mut anchor = RECT::default();
    let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(field, &mut anchor);
    let picked = settings_models::open_picker(owner, anchor, &current, switchable, &key, font);
    // The field could have been rebuilt while the picker's loop ran; don't write to a dead window.
    if let (Some(v), true) = (picked, windows::Win32::UI::WindowsAndMessaging::IsWindow(field).as_bool()) {
        let wide: Vec<u16> = v.encode_utf16().chain([0]).collect();
        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowTextW(field, PCWSTR(wide.as_ptr()));
        let _ = InvalidateRect(field, None, true);
    }
}

fn window_text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let n = GetWindowTextW(hwnd, &mut buf).max(0) as usize;
        String::from_utf16_lossy(&buf[..n]).trim().to_string()
    }
}

// ---- reading values -------------------------------------------------------------------------------

fn read_field(f: &SField) -> String {
    match &f.kind {
        FieldKind::Hotkey => settings_controls::read_hotkey(f.hwnd),
        FieldKind::Color => settings_controls::read_color(f.hwnd),
        FieldKind::Corner => settings_controls::read_corner(f.hwnd),
        FieldKind::Toggle => settings_controls::read_toggle(f.hwnd),
        FieldKind::Number => settings_controls::read_number(f.hwnd),
        _ => window_text(f.hwnd),
    }
}

fn read_all_fields(d: &Sett) -> Vec<String> {
    d.fields.iter().map(read_field).collect()
}

// ---- save -----------------------------------------------------------------------------------------

fn do_save(d: &mut Sett) {
    // Collect current tab's values and apply them.
    let values = read_all_fields(d);
    save_tab(&mut d.config, d.tab, &values);

    // Serialize the full config and write it (temp file + rename, like every other config write).
    let header = "# ShadowPrompt config — edited via Settings.
                  # See config.example.toml for documentation on each field.

";
    let written = toml::to_string_pretty(&d.config)
        .map_err(anyhow::Error::from)
        .and_then(|body| {
            let path = crate::config::paths::config_path()?;
            crate::config::load::write_atomic(&path, &format!("{header}{body}"))
        });
    d.save_failed = written.is_err();
    if let Err(e) = written {
        log::error!("settings: save failed: {e}");
        d.saved = false;
        unsafe {
            if let Ok(btn) = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(d.hwnd, B_SAVE) {
                let _ = InvalidateRect(btn, None, false);
            }
        }
        return;
    }
    d.saved = true;

    // Push to cloud if logged in.
    if d.config.account.logged_in() {
        d.cloud = CloudStatus::Saving;
        let mut cloud_cfg = d.config.clone();
        cloud_cfg.account = Default::default();
        // Each PC keeps its own OpenRouter key; the account's key is already stored separately.
        cloud_cfg.openrouter.api_key = String::new();
        let code = d.config.account.code.clone();
        let hwnd_raw = d.hwnd.0 as usize;
        std::thread::spawn(move || {
            // Always report back, or the button would sit on "Saving to cloud…" forever.
            let ok = match toml::to_string_pretty(&cloud_cfg) {
                Ok(toml) => crate::account::block_on(crate::account::save_config(&code, &toml))
                    .map(|s| s == "ok")
                    .unwrap_or_else(|e| { log::warn!("settings: cloud save: {e}"); false }),
                Err(e) => { log::error!("settings: cloud serialize: {e}"); false }
            };
            unsafe {
                let hwnd = HWND(hwnd_raw as *mut _);
                let _ = PostMessageW(hwnd, WM_CLOUD_DONE, WPARAM(ok as usize), LPARAM(0));
            }
        });
    }

    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::GetDlgItem;
        if let Ok(btn) = GetDlgItem(d.hwnd, B_SAVE) {
            let _ = InvalidateRect(btn, None, false);
        }
    }
}

// ---- tab switching --------------------------------------------------------------------------------

fn switch_tab(d: &mut Sett, tab: Tab) {
    if d.tab == tab { return; }
    // Save current tab's values into config before switching.
    let values = read_all_fields(d);
    save_tab(&mut d.config, d.tab, &values);
    d.tab = tab;
    d.saved = false;
    d.cloud = CloudStatus::None;
    build_tab(d);
}

fn tab_at(x: i32, _y: i32) -> Option<Tab> {
    let tab_count = TABS.len() as i32;
    let tab_w = (s(WIDTH) - s(PAD) * 2 - s(X_SIZE) - s(4)) / tab_count;
    let rel = x - tabs_left();
    if rel < 0 { return None; }
    let idx = (rel / tab_w) as usize;
    TABS.get(idx).copied()
}

// ---- hover tracking -------------------------------------------------------------------------------

fn set_hover(d: &mut Sett, h: HWND) {
    if d.hover == h { return; }
    let old = std::mem::replace(&mut d.hover, h);
    unsafe {
        for w in [old, h] {
            if !w.is_invalid() { let _ = InvalidateRect(w, None, false); }
        }
    }
}

// ---- wnd_proc -------------------------------------------------------------------------------------

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(d) = sett().filter(|d| d.hwnd == hwnd) else {
        if msg == WM_DESTROY { PostQuitMessage(0); }
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_PAINT => {
            paint_main(d);
            LRESULT(0)
        }
        WM_DRAWITEM => {
            draw_button(d, &*(lp.0 as *const DrawItem));
            LRESULT(1)
        }
        WM_NCHITTEST => {
            let hit = DefWindowProcW(hwnd, msg, wp, lp);
            if hit.0 == HTCLIENT { LRESULT(HTCAPTION) } else { hit }
        }
        WM_SETCURSOR => {
            let under = HWND(wp.0 as *mut _);
            let is_button = [B_X, B_SAVE].iter().any(|&id| {
                windows::Win32::UI::WindowsAndMessaging::GetDlgItem(hwnd, id)
                    .is_ok_and(|btn| btn == under)
            });
            set_hover(d, if is_button { under } else { HWND::default() });
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        0x020A /* WM_MOUSEWHEEL */ => {
            let delta = ((wp.0 >> 16) & 0xFFFF) as i16;
            let scroll_step = s(ROW_H) * 3;
            if delta > 0 {
                d.scroll_y = (d.scroll_y - scroll_step).max(0);
            } else {
                d.scroll_y = (d.scroll_y + scroll_step).min(d.max_scroll);
            }
            layout(d);
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wp.0 & 0xFFFF) as i32;
            let code = ((wp.0 >> 16) & 0xFFFF) as u32;
            match id {
                IDOK => do_save(d),
                IDCANCEL => { let _ = DestroyWindow(hwnd); }
                _ if code == BN_CLICKED && id == B_SAVE => do_save(d),
                _ if code == BN_CLICKED && id == B_X => { let _ = DestroyWindow(hwnd); }
                _ => {}
            }
            LRESULT(0)
        }
        // Tab clicks: the NCHITTEST override reports HTCAPTION for the client area (so the window
        // is draggable), but that swallows normal clicks. To let the tab bar respond, intercept
        // WM_NCLBUTTONDOWN (0x00A1) on HTCAPTION and check if it's in the tab bar.
        0x00A1 /* WM_NCLBUTTONDOWN */ => {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let mut rc = RECT::default();
            let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rc);
            let local_y = pt.y - rc.top;
            let local_x = pt.x - rc.left;
            if local_y >= s(TAB_BAR_Y) && local_y < s(TAB_BAR_Y) + s(TAB_BAR_H) {
                if let Some(tab) = tab_at(local_x, local_y) {
                    switch_tab(d, tab);
                    return LRESULT(0);
                }
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_CLOUD_DONE => {
            d.cloud = if wp.0 == 1 { CloudStatus::Saved } else { CloudStatus::Error };
            if let Ok(btn) = windows::Win32::UI::WindowsAndMessaging::GetDlgItem(hwnd, B_SAVE) {
                let _ = InvalidateRect(btn, None, false);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            // Children are still alive here (they're destroyed after the parent's WM_DESTROY),
            // so their boxed data can be freed now.
            for f in d.fields.drain(..) {
                if matches!(f.kind, FieldKind::Color | FieldKind::Toggle | FieldKind::Corner) {
                    settings_controls::cleanup(f.hwnd);
                }
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

/// Content container: forwards EDIT background messages, paints labels, routes owner-drawn controls.
unsafe extern "system" fn content_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            if let Some(d) = sett() {
                paint_content(d);
                return LRESULT(0);
            }
        }
        WM_DRAWITEM => {
            if let Some(d) = sett() {
                let di = &*(lp.0 as *const DrawItem);
                if d.fields.iter().any(|f| f.hwnd == di.hwnd_item && matches!(f.kind, FieldKind::ModelList)) {
                    draw_model_field(d, di);
                } else {
                    settings_controls::draw_control(di.hdc, di.hwnd_item, di.rc, di.item_state, d.bg_brush);
                }
                return LRESULT(1);
            }
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            if let Some(d) = sett() {
                let hdc = HDC(wp.0 as *mut _);
                SetTextColor(hdc, if msg == WM_CTLCOLOREDIT { TEXT } else { MUTED });
                SetBkColor(hdc, FIELD_BG);
                return LRESULT(d.field_brush.0 as isize);
            }
        }
        WM_COMMAND => {
            let id = (wp.0 & 0xFFFF) as i32;
            let code = ((wp.0 >> 16) & 0xFFFF) as u32;
            if let Some(d) = sett() {
                // Route BN_CLICKED to owner-drawn controls (color picker, toggle).
                if code == BN_CLICKED {
                    if let Some(f) = d.fields.iter().find(|f| {
                        windows::Win32::UI::WindowsAndMessaging::GetDlgItem(hwnd, id)
                            .is_ok_and(|h| h == f.hwnd)
                    }) {
                        match &f.kind {
                            FieldKind::Color | FieldKind::Toggle | FieldKind::Corner => {
                                settings_controls::on_click(f.hwnd);
                                return LRESULT(0);
                            }
                            FieldKind::ModelList => {
                                open_model_picker(d, f.hwnd);
                                return LRESULT(0);
                            }
                            _ => {}
                        }
                    }
                }
                // Forward Enter/Escape to main window for Save/Close.
                return SendMessageW(d.hwnd, msg, wp, lp);
            }
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

// ---- public API -----------------------------------------------------------------------------------

static SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);

/// Opens Settings in place of `menu`: the menu is hidden while Settings is up and
/// shown again when Settings closes (Back, Escape, or any other way).
///
/// Refuses (Err with a message for the menu) if config.toml can't be read: Settings rewrites the
/// whole file on Save, so editing a stand-in would wipe the user's real settings and login.
pub fn open(menu: HWND) -> Result<(), String> {
    let config = crate::config::load::load().map_err(|e| {
        log::error!("settings: couldn't load config: {e}");
        "config.toml has an error, so Settings can't open. Fix or delete it, then try again.".to_string()
    })?;
    if SETTINGS_OPEN.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    unsafe { let _ = ShowWindow(menu, windows::Win32::UI::WindowsAndMessaging::SW_HIDE); }
    let menu_raw = menu.0 as usize;
    std::thread::spawn(move || {
        run(config);
        SETTINGS_OPEN.store(false, Ordering::SeqCst);
        unsafe {
            let menu = HWND(menu_raw as *mut _);
            let _ = ShowWindow(menu, SW_SHOW);
            let _ = SetForegroundWindow(menu);
        }
    });
    Ok(())
}

/// Icon for the Settings button on the account dialog's Home page.
pub const ICON: char = I_SETTINGS;
