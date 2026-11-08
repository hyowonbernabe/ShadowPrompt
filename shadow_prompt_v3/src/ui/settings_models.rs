// settings_models.rs — OpenRouter model picker for the Settings dialog.
// Owner-drawn popup with search, checkboxes, and badge pills. Same Win32 + GDI
// stack as account_dialog and settings_dialog: no framework, dark theme, DPI-scaled.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Once, OnceLock};

use serde::Deserialize;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetStockObject, InvalidateRect, RoundRect, SelectObject, SetBkColor, SetBkMode,
    SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DRAW_TEXT_FORMAT,
    DT_CALCRECT, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE,
    HBRUSH, HDC, HFONT, HGDIOBJ, NULL_BRUSH, OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, KillTimer, LoadCursorW, MoveWindow, RegisterClassExW, SendMessageW, SetTimer,
    ShowWindow, HMENU, IDC_ARROW, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_PAINT,
    WNDCLASSEXW, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
};

// ---- palette (same dark theme as settings_dialog / account_dialog) --------------------------------

const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}
const BG: COLORREF = rgb(0x1F, 0x1E, 0x1D);
const SURFACE_HOVER: COLORREF = rgb(0x38, 0x37, 0x35);
const FIELD_BG: COLORREF = rgb(0x26, 0x26, 0x24);
const BORDER: COLORREF = rgb(0x3E, 0x3D, 0x39);
const TEXT: COLORREF = rgb(0xF5, 0xF4, 0xEE);
const MUTED: COLORREF = rgb(0xA8, 0xA6, 0x9D);
const DIM: COLORREF = rgb(0x6E, 0x6C, 0x66);
const ACCENT: COLORREF = rgb(0xD9, 0x77, 0x57);
const ON_ACCENT: COLORREF = rgb(0xFF, 0xFF, 0xFF);

// Badge colours.
const BADGE_FREE: COLORREF = rgb(0x7E, 0xB8, 0xC2);
const BADGE_VISION: COLORREF = rgb(0xB8, 0x7E, 0xC2);
const BADGE_TOOLS: COLORREF = rgb(0xD9, 0x77, 0x57);
const BADGE_REASONING: COLORREF = rgb(0xC2, 0xB8, 0x7E);
const BADGE_RECOMMENDED: COLORREF = rgb(0xD9, 0xA7, 0x57);

// ---- layout (96-dpi reference; runtime values go through `s`) ------------------------------------

const POPUP_W: i32 = 420;
const POP_PAD: i32 = 8;
const SEARCH_H: i32 = 26;
const LIST_ROW_H: i32 = 26;
const CHECK_SIZE: i32 = 14;
const BADGE_W: i32 = 18;
const ARROW_W: i32 = 20;
const FOOTER_H: i32 = 38;
const MAX_VISIBLE_ROWS: i32 = 12;
const MIN_VISIBLE_ROWS: i32 = 3;

fn s(px: i32) -> i32 {
    let dpi = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() }.max(96) as i32;
    px * dpi / 96
}

// Win32 message constants not in our `windows` features.
const WM_SETFONT: u32 = 0x0030;
const WM_ACTIVATE: u32 = 0x0006;
const WM_COMMAND: u32 = 0x0111;
const WM_TIMER: u32 = 0x0113;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_MOUSEWHEEL: u32 = 0x020A;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_ERASEBKGND: u32 = 0x0014;
const WA_INACTIVE: u32 = 0;
const EN_CHANGE: u32 = 0x0300;
const ES_AUTOHSCROLL: u32 = 0x0080;
const CLEARTYPE_QUALITY: u32 = 5;


// Segoe MDL2 Assets glyphs.
const I_CHECK: char = '\u{E73E}';
const I_SEARCH: char = '\u{E721}';
const I_UP: char = '\u{E70E}';
const I_DOWN: char = '\u{E70D}';
const I_STAR: char = '\u{E734}';
const I_TAG: char = '\u{E8EC}';
const I_EYE: char = '\u{E890}';
const I_WRENCH: char = '\u{E90F}';
const I_BULB: char = '\u{EA80}';

// Group headers in the list.
const H_MAIN: &str = "MAIN \u{00B7} switch hotkey flips these two";
const H_FALLBACK: &str = "FALLBACKS \u{00B7} tried in order if main fails";
const H_IN_USE: &str = "IN USE \u{00B7} tried top to bottom";
const H_ALL: &str = "ALL MODELS";

// Timer IDs.
const T_CHECK_MODELS: usize = 1;

// Control IDs.
const ID_SEARCH: i32 = 201;

// ---- public types ---------------------------------------------------------------------------------

const RECOMMENDED_MODELS: &[&str] = &[
    "anthropic/claude-sonnet-5.5",
    "anthropic/claude-opus-5.5",
    "google/gemini-3.8-flash",
];

#[derive(Clone, Debug)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub compatible: bool,
    pub free: bool,
    pub vision: bool,
    pub tools: bool,
    pub reasoning: bool,
    pub recommended: bool,
}

/// Icon-only badge; `meaning` is spelled out in the picker's hover footer.
#[derive(Clone, Debug)]
pub struct Badge {
    pub icon: char,
    pub meaning: &'static str,
    pub color: COLORREF,
}

impl ModelInfo {
    // Compatibility isn't a badge: incompatible rows are greyed out and the footer says why.
    pub fn badges(&self) -> Vec<Badge> {
        let mut b = Vec::new();
        if self.recommended {
            b.push(Badge { icon: I_STAR, meaning: "Recommended", color: BADGE_RECOMMENDED });
        }
        if self.free {
            b.push(Badge { icon: I_TAG, meaning: "Free", color: BADGE_FREE });
        }
        if self.vision {
            b.push(Badge { icon: I_EYE, meaning: "Reads images", color: BADGE_VISION });
        }
        if self.tools {
            b.push(Badge { icon: I_WRENCH, meaning: "Uses tools", color: BADGE_TOOLS });
        }
        if self.reasoning {
            b.push(Badge { icon: I_BULB, meaning: "Thinks first", color: BADGE_REASONING });
        }
        b
    }

    pub fn is_incompatible(&self) -> bool {
        !self.compatible
    }
}

// ---- API response types (only the fields we need) -------------------------------------------------

#[derive(Deserialize)]
struct ApiResponse {
    data: Vec<ApiModel>,
}

#[derive(Deserialize)]
struct ApiModel {
    id: String,
    name: String,
    pricing: Option<ApiPricing>,
    architecture: Option<ApiArchitecture>,
    supported_parameters: Option<Vec<String>>,
    #[allow(dead_code)]
    top_provider: Option<ApiTopProvider>,
}

#[derive(Deserialize)]
struct ApiPricing {
    prompt: String,
    completion: String,
}

#[derive(Deserialize)]
struct ApiArchitecture {
    input_modalities: Vec<String>,
    #[allow(dead_code)]
    output_modalities: Vec<String>,
}

#[derive(Deserialize)]
struct ApiTopProvider {
    #[allow(dead_code)]
    is_moderated: bool,
}

// ---- model cache ----------------------------------------------------------------------------------

static MODELS: OnceLock<Vec<ModelInfo>> = OnceLock::new();
static LOADING: AtomicBool = AtomicBool::new(false);
static FETCH_FAILED: AtomicBool = AtomicBool::new(false);

fn is_zero_price(s: &str) -> bool {
    s.trim().parse::<f64>().is_ok_and(|v| v == 0.0)
}

async fn fetch_models(api_key: &str) -> anyhow::Result<Vec<ModelInfo>> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp: ApiResponse = client
        .get("https://openrouter.ai/api/v1/models")
        .bearer_auth(api_key.trim())
        .send()
        .await?
        .json()
        .await?;

    let models = resp
        .data
        .into_iter()
        .map(|m| {
            let input_mods = m
                .architecture
                .as_ref()
                .map(|a| a.input_modalities.as_slice())
                .unwrap_or(&[]);
            let params = m.supported_parameters.as_deref().unwrap_or(&[]);

            let vision = input_mods.iter().any(|s| s == "image");
            let tools = params.iter().any(|s| s == "tools");
            let reasoning = params.iter().any(|s| s == "reasoning");

            let free = m.id.ends_with(":free")
                || m.pricing
                    .as_ref()
                    .is_some_and(|p| is_zero_price(&p.prompt) && is_zero_price(&p.completion));

            let recommended = RECOMMENDED_MODELS.contains(&m.id.as_str());

            ModelInfo {
                id: m.id,
                name: m.name,
                compatible: vision && tools,
                free,
                vision,
                tools,
                reasoning,
                recommended,
            }
        })
        .collect();

    Ok(models)
}

/// Fetch models from OpenRouter (blocking). Caches in a static.
/// Pass the user's API key for authenticated access (gets their specific model list).
pub fn ensure_models(api_key: &str) {
    if MODELS.get().is_some() || LOADING.load(Ordering::SeqCst) {
        return;
    }
    LOADING.store(true, Ordering::SeqCst);
    FETCH_FAILED.store(false, Ordering::SeqCst);
    let key = api_key.to_string();
    std::thread::spawn(move || {
        let result = crate::account::block_on(fetch_models(&key));
        match result {
            // An empty answer counts as a failure so the next open tries again.
            Ok(m) if !m.is_empty() => {
                let _ = MODELS.set(m);
            }
            Ok(_) => {
                log::error!("settings_models: fetch returned no models");
                FETCH_FAILED.store(true, Ordering::SeqCst);
            }
            Err(e) => {
                log::error!("settings_models: fetch failed: {e}");
                FETCH_FAILED.store(true, Ordering::SeqCst);
            }
        }
        LOADING.store(false, Ordering::SeqCst);
    });
}

/// Get cached models. Returns empty slice if not fetched yet.
pub fn models() -> &'static [ModelInfo] {
    MODELS.get().map(|v| v.as_slice()).unwrap_or(&[])
}

/// Returns true if models are still being fetched.
pub fn is_loading() -> bool {
    LOADING.load(Ordering::SeqCst)
}

// ---- display list ---------------------------------------------------------------------------------

/// Row label: OpenRouter's name minus its "Company: " prefix (the full id is in the footer).
fn display_name(m: &ModelInfo) -> &str {
    if m.name.is_empty() || m.name == m.id {
        return &m.id;
    }
    m.name.split_once(": ").map_or(&m.name, |(_, rest)| rest)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Row {
    Header(&'static str),
    Model(usize), // index into models()
    /// A chosen id that isn't in the fetched list (retired, misspelled). Index into `selected`.
    Missing(usize),
}

/// Chosen models first, grouped by role and kept in chain order, then everything else.
/// `switchable`: the list's first two entries are the pair the switch hotkey flips.
fn build_display(all: &[ModelInfo], selected: &[String], search: &str, switchable: bool) -> Vec<Row> {
    let q = search.to_lowercase();
    let matches: Vec<usize> = all
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            q.is_empty()
                || m.name.to_lowercase().contains(&q)
                || m.id.to_lowercase().contains(&q)
        })
        .map(|(i, _)| i)
        .collect();

    let mut rows = Vec::new();
    let mut picked = std::collections::HashSet::new();
    let mut group = None;
    for (pos, sid) in selected.iter().enumerate() {
        let row = match all.iter().position(|m| m.id == *sid) {
            Some(idx) if matches.contains(&idx) => Row::Model(idx),
            Some(_) => continue, // hidden by the search
            None if q.is_empty() || sid.to_lowercase().contains(&q) => Row::Missing(pos),
            None => continue,
        };
        let g = match (switchable, pos < 2) {
            (true, true) => H_MAIN,
            (true, false) => H_FALLBACK,
            (false, _) => H_IN_USE,
        };
        if group != Some(g) {
            rows.push(Row::Header(g));
            group = Some(g);
        }
        if let Row::Model(idx) = row {
            picked.insert(idx);
        }
        rows.push(row);
    }

    let mut rest: Vec<usize> = matches.into_iter().filter(|i| !picked.contains(i)).collect();

    // Sort: recommended first, then compatible, then alphabetical.
    rest.sort_by(|&a, &b| {
        let (ma, mb) = (&all[a], &all[b]);
        mb.recommended
            .cmp(&ma.recommended)
            .then_with(|| mb.compatible.cmp(&ma.compatible))
            .then_with(|| ma.name.to_lowercase().cmp(&mb.name.to_lowercase()))
    });

    if !picked.is_empty() && !rest.is_empty() {
        rows.push(Row::Header(H_ALL));
    }
    rows.extend(rest.into_iter().map(Row::Model));
    rows
}

// ---- GDI helpers (same as account_dialog / settings_dialog) ---------------------------------------

unsafe fn round_rect_fill(
    hdc: HDC,
    rc: RECT,
    fill: Option<COLORREF>,
    border: COLORREF,
    radius: i32,
) {
    let brush = fill.map(|c| CreateSolidBrush(c));
    let pen = CreatePen(PS_SOLID, s(1).max(1), border);
    let old_b = SelectObject(
        hdc,
        brush
            .map(|b| HGDIOBJ(b.0))
            .unwrap_or_else(|| GetStockObject(NULL_BRUSH)),
    );
    let old_p = SelectObject(hdc, HGDIOBJ(pen.0));
    let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
    if let Some(b) = brush {
        let _ = DeleteObject(HGDIOBJ(b.0));
    }
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

unsafe fn fill(hdc: HDC, rc: RECT, color: COLORREF) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, &rc, brush);
    let _ = DeleteObject(HGDIOBJ(brush.0));
}

// ---- popup state ----------------------------------------------------------------------------------

struct Picker {
    hwnd: HWND,
    search_hwnd: HWND,
    font: HFONT,       // caller's body font — model names
    font_small: HFONT, // badge labels
    font_icon: HFONT,  // checkmarks (Segoe MDL2 Assets)
    bg_brush: HBRUSH,
    field_brush: HBRUSH,
    selected: Vec<String>, // chain order
    switchable: bool,
    display: Vec<Row>,
    scroll_y: i32,
    max_scroll: i32,
    hover_row: i32, // -1 = none
    list_top: i32,
    list_height: i32,
    done: bool,
}

thread_local! {
    static PICKER: std::cell::Cell<*mut Picker> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

fn picker() -> Option<&'static mut Picker> {
    unsafe { PICKER.with(|c| c.get()).as_mut() }
}

fn is_selected(p: &Picker, id: &str) -> bool {
    p.selected.iter().any(|s| s == id)
}

fn toggle_selection(p: &mut Picker, id: &str) {
    if let Some(pos) = p.selected.iter().position(|s| s == id) {
        p.selected.remove(pos);
    } else {
        p.selected.push(id.to_string());
    }
}

/// Moves a chosen model one place up (-1) or down (+1) in the chain.
fn move_selection(selected: &mut [String], id: &str, delta: isize) {
    let Some(pos) = selected.iter().position(|s| s == id) else { return };
    let to = pos as isize + delta;
    if to >= 0 && (to as usize) < selected.len() {
        selected.swap(pos, to as usize);
    }
}

/// Up/down arrow hit boxes at the right end of a chosen model's row.
fn arrow_rects(y: i32) -> (RECT, RECT) {
    let right = s(POPUP_W) - s(POP_PAD);
    let aw = s(ARROW_W);
    let h = s(LIST_ROW_H);
    let down = RECT { left: right - aw, top: y, right, bottom: y + h };
    let up = RECT { left: right - 2 * aw, top: y, right: right - aw, bottom: y + h };
    (up, down)
}

fn popup_height(list_h: i32) -> i32 {
    s(POP_PAD) + s(SEARCH_H) + s(4) + list_h + s(FOOTER_H) + s(POP_PAD)
}

fn row_y(p: &Picker, i: usize) -> i32 {
    p.list_top + i as i32 * s(LIST_ROW_H) - p.scroll_y
}

fn row_at_y(p: &Picker, client_y: i32) -> Option<usize> {
    if client_y < p.list_top || client_y >= p.list_top + p.list_height {
        return None;
    }
    let rel_y = client_y - p.list_top + p.scroll_y;
    if rel_y < 0 {
        return None;
    }
    let row = (rel_y / s(LIST_ROW_H)) as usize;
    if row < p.display.len() {
        Some(row)
    } else {
        None
    }
}

fn total_content_height(p: &Picker) -> i32 {
    p.display.len() as i32 * s(LIST_ROW_H)
}

fn recompute_scroll(p: &mut Picker) {
    p.max_scroll = (total_content_height(p) - p.list_height).max(0);
    p.scroll_y = p.scroll_y.clamp(0, p.max_scroll);
}

fn rebuild_display(p: &mut Picker) {
    let search = read_edit(p.search_hwnd);
    let all = models();
    p.display = build_display(all, &p.selected, &search, p.switchable);
    p.hover_row = -1;
    recompute_scroll(p);
}

fn read_edit(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let n = GetWindowTextW(hwnd, &mut buf).max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }
}

// ---- painting -------------------------------------------------------------------------------------

unsafe fn paint(p: &Picker) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(p.hwnd, &mut ps);
    let mut client = RECT::default();
    let _ = GetClientRect(p.hwnd, &mut client);
    FillRect(hdc, &client, p.bg_brush);

    // Search icon to the left of the EDIT.
    let icon_rc = RECT {
        left: s(POP_PAD) - s(1),
        top: s(POP_PAD),
        right: s(POP_PAD) + s(16),
        bottom: s(POP_PAD) + s(SEARCH_H),
    };
    text(
        hdc,
        p.font_icon,
        DIM,
        &I_SEARCH.to_string(),
        icon_rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    let all = models();

    if all.is_empty() {
        // Status message while loading or on failure.
        let msg = if is_loading() {
            "Loading models\u{2026}"
        } else if FETCH_FAILED.load(Ordering::SeqCst) {
            "Could not load models"
        } else {
            "No models available"
        };
        let rc = RECT {
            left: s(POP_PAD),
            top: p.list_top,
            right: client.right - s(POP_PAD),
            bottom: p.list_top + s(LIST_ROW_H) * 2,
        };
        text(hdc, p.font, MUTED, msg, rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        let _ = EndPaint(p.hwnd, &ps);
        return;
    }

    if p.display.is_empty() {
        let rc = RECT {
            left: s(POP_PAD),
            top: p.list_top,
            right: client.right - s(POP_PAD),
            bottom: p.list_top + s(LIST_ROW_H) * 2,
        };
        text(hdc, p.font, MUTED, "No matches", rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        let _ = EndPaint(p.hwnd, &ps);
        return;
    }

    let row_h = s(LIST_ROW_H);
    let check_s = s(CHECK_SIZE);
    let x_pad = s(POP_PAD);
    let list_right = client.right - s(POP_PAD);

    // Rows are clipped to the list band so partial rows don't paint over the search box or footer.
    let clip = windows::Win32::Graphics::Gdi::CreateRectRgn(0, p.list_top, client.right, p.list_top + p.list_height);
    windows::Win32::Graphics::Gdi::SelectClipRgn(hdc, clip);

    for (i, &row) in p.display.iter().enumerate() {
        let y = row_y(p, i);

        // Skip rows outside visible area.
        if y + row_h <= p.list_top {
            continue;
        }
        if y >= p.list_top + p.list_height {
            break;
        }

        let (id, name, badges, dim) = match row {
            Row::Header(label) => {
                let rc = RECT { left: x_pad, top: y, right: list_right, bottom: y + row_h };
                text(hdc, p.font_small, ACCENT, label, rc, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);
                fill(hdc, RECT { left: x_pad, top: y + row_h - s(1), right: list_right, bottom: y + row_h }, BORDER);
                continue;
            }
            Row::Model(idx) => {
                let m = &all[idx];
                (m.id.as_str(), display_name(m), m.badges(), m.is_incompatible())
            }
            Row::Missing(pos) => (p.selected[pos].as_str(), p.selected[pos].as_str(), Vec::new(), true),
        };
        let sel = is_selected(p, id);
        let hover = p.hover_row == i as i32;

        // Row background.
        if hover {
            let rc = RECT {
                left: 0,
                top: y,
                right: client.right,
                bottom: y + row_h,
            };
            fill(hdc, rc, SURFACE_HOVER);
        }

        // Checkbox.
        let ck_x = x_pad;
        let ck_y = y + (row_h - check_s) / 2;
        let ck_rc = RECT {
            left: ck_x,
            top: ck_y,
            right: ck_x + check_s,
            bottom: ck_y + check_s,
        };
        if sel {
            round_rect_fill(hdc, ck_rc, Some(ACCENT), ACCENT, s(3));
            text(
                hdc,
                p.font_icon,
                ON_ACCENT,
                &I_CHECK.to_string(),
                ck_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
        } else {
            round_rect_fill(hdc, ck_rc, Some(FIELD_BG), BORDER, s(3));
        }

        // Reorder arrows on chosen rows (always reserved there, so badges don't jump on hover).
        let mut badge_right = list_right;
        if sel {
            let (up, down) = arrow_rects(y);
            let c = if hover { TEXT } else { DIM };
            text(hdc, p.font_icon, c, &I_UP.to_string(), up, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            text(hdc, p.font_icon, c, &I_DOWN.to_string(), down, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            badge_right = up.left - s(4);
        }

        // Icon-only badges, right-to-left so we know how much space the name gets.
        for badge in badges.iter().rev() {
            let brc = RECT { left: badge_right - s(BADGE_W), top: y, right: badge_right, bottom: y + row_h };
            let c = if dim { DIM } else { badge.color };
            text(hdc, p.font_icon, c, &badge.icon.to_string(), brc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            badge_right = brc.left;
        }

        // Model name.
        let name_x = ck_x + check_s + s(6);
        let name_right = badge_right - s(4);
        if name_right > name_x {
            let name_rc = RECT {
                left: name_x,
                top: y,
                right: name_right,
                bottom: y + row_h,
            };
            let name_color = if dim { DIM } else { TEXT };
            text(
                hdc,
                p.font,
                name_color,
                name,
                name_rc,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
        }

    }

    windows::Win32::Graphics::Gdi::SelectClipRgn(hdc, None);
    let _ = DeleteObject(HGDIOBJ(clip.0));
    paint_footer(p, hdc, client);
    let _ = EndPaint(p.hwnd, &ps);
}

/// Bottom strip: full id + badge meanings for the hovered model, or how-to text.
unsafe fn paint_footer(p: &Picker, hdc: HDC, client: RECT) {
    let top = p.list_top + p.list_height + s(4);
    let x = s(POP_PAD);
    let right = client.right - s(POP_PAD);
    fill(hdc, RECT { left: x, top, right, bottom: top + s(1) }, BORDER);
    let hovered = usize::try_from(p.hover_row).ok().and_then(|i| p.display.get(i)).and_then(|r| match *r {
        Row::Model(idx) => models().get(idx).map(|m| {
            let mut what: Vec<&str> = m.badges().iter().map(|b| b.meaning).collect();
            if m.is_incompatible() {
                what.insert(0, "Won't work here (needs images + tools)");
            }
            (m.id.clone(), what.join(" \u{00B7} "))
        }),
        Row::Missing(pos) => p.selected.get(pos).map(|id| {
            (id.clone(), "Not in OpenRouter's list (retired or misspelled) \u{00B7} untick to remove".to_string())
        }),
        Row::Header(_) => None,
    });
    let has_hover = hovered.is_some();
    let (line1, line2) = hovered.unwrap_or_else(|| {
        ("Click a model to add or remove it. Arrows change the order.".to_string(), String::new())
    });
    let h = s(FOOTER_H) / 2 - s(1);
    let r1 = RECT { left: x, top: top + s(2), right, bottom: top + s(2) + h };
    let r2 = RECT { left: x, top: r1.bottom, right, bottom: r1.bottom + h };
    let fmt = DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
    text(hdc, p.font_small, TEXT, &line1, r1, fmt);
    if has_hover {
        text(hdc, p.font_small, MUTED, &line2, r2, fmt);
    } else {
        paint_legend(p, hdc, r2);
    }
}

/// Legend: each badge icon in its colour followed by its meaning.
unsafe fn paint_legend(p: &Picker, hdc: HDC, rc: RECT) {
    let all = ModelInfo {
        id: String::new(), name: String::new(), compatible: true,
        free: true, vision: true, tools: true, reasoning: true, recommended: true,
    };
    let mut x = rc.left;
    for b in all.badges() {
        let irc = RECT { left: x, top: rc.top, right: x + s(BADGE_W), bottom: rc.bottom };
        text(hdc, p.font_icon, b.color, &b.icon.to_string(), irc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        x = irc.right + s(2);
        let w = text_width(hdc, p.font_small, b.meaning);
        let trc = RECT { left: x, top: rc.top, right: (x + w).min(rc.right), bottom: rc.bottom };
        text(hdc, p.font_small, MUTED, b.meaning, trc, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
        x = trc.right + s(8);
        if x >= rc.right {
            break;
        }
    }
}

// ---- window proc ----------------------------------------------------------------------------------

unsafe extern "system" fn picker_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    let Some(p) = picker().filter(|p| p.hwnd == hwnd) else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };

    match msg {
        WM_PAINT => {
            paint(p);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1), // we fill background in WM_PAINT

        WM_ACTIVATE => {
            let state = (wp.0 & 0xFFFF) as u32;
            if state == WA_INACTIVE {
                // Click outside — confirm current selection and close.
                p.done = true;
            }
            LRESULT(0)
        }

        WM_CTLCOLOREDIT => {
            let hdc = HDC(wp.0 as *mut _);
            SetTextColor(hdc, TEXT);
            SetBkColor(hdc, FIELD_BG);
            LRESULT(p.field_brush.0 as isize)
        }

        WM_COMMAND => {
            let code = ((wp.0 >> 16) & 0xFFFF) as u32;
            if code == EN_CHANGE {
                rebuild_display(p);
                let _ = InvalidateRect(hwnd, None, true);
            }
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            let x = (lp.0 & 0xFFFF) as i16 as i32;
            let y = ((lp.0 >> 16) & 0xFFFF) as i16 as i32;
            let id = match row_at_y(p, y).and_then(|r| p.display.get(r).copied()) {
                Some(Row::Model(idx)) => Some(models()[idx].id.clone()),
                Some(Row::Missing(pos)) => p.selected.get(pos).cloned(),
                _ => None,
            };
            if let Some(id) = id {
                let (up, down) = arrow_rects(y);
                let on_arrow = is_selected(p, &id) && x >= up.left;
                if on_arrow {
                    move_selection(&mut p.selected, &id, if x >= down.left { 1 } else { -1 });
                } else {
                    toggle_selection(p, &id);
                }
                let hover = p.hover_row;
                rebuild_display(p);
                p.hover_row = hover;
                let _ = InvalidateRect(hwnd, None, true);
            }
            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            let y = ((lp.0 >> 16) & 0xFFFF) as i16 as i32;
            let new_hover = row_at_y(p, y).map(|r| r as i32).unwrap_or(-1);
            if new_hover != p.hover_row {
                p.hover_row = new_hover;
                let _ = InvalidateRect(hwnd, None, true);
            }
            LRESULT(0)
        }

        WM_MOUSEWHEEL => {
            let delta = ((wp.0 >> 16) & 0xFFFF) as i16;
            let step = s(LIST_ROW_H) * 3;
            if delta > 0 {
                p.scroll_y = (p.scroll_y - step).max(0);
            } else {
                p.scroll_y = (p.scroll_y + step).min(p.max_scroll);
            }
            let _ = InvalidateRect(hwnd, None, true);
            LRESULT(0)
        }

        WM_TIMER => {
            if wp.0 == T_CHECK_MODELS {
                if !models().is_empty() || (!is_loading() && FETCH_FAILED.load(Ordering::SeqCst)) {
                    let _ = KillTimer(hwnd, T_CHECK_MODELS);
                    rebuild_display(p);
                    // Resize the popup now that we know how many rows there are.
                    resize_popup(p);
                }
                let _ = InvalidateRect(hwnd, None, true);
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            p.done = true;
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

/// Resizes the popup to fit the current number of display rows.
fn resize_popup(p: &mut Picker) {
    let row_count = p.display.len() as i32;
    let visible_rows = row_count.clamp(MIN_VISIBLE_ROWS, MAX_VISIBLE_ROWS);
    let list_h = visible_rows * s(LIST_ROW_H);
    let popup_h = popup_height(list_h);

    p.list_height = list_h;
    recompute_scroll(p);

    unsafe {
        let mut rc = RECT::default();
        let _ = GetWindowRect(p.hwnd, &mut rc);
        let _ = MoveWindow(p.hwnd, rc.left, rc.top, s(POPUP_W), popup_h, true);
        let _ = InvalidateRect(p.hwnd, None, true);
    }
}

// ---- registration ---------------------------------------------------------------------------------

/// Register the popup window class. Call once.
pub fn register_class() {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| unsafe {
        let hinstance = GetModuleHandleW(None).unwrap();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(picker_proc),
            hInstance: hinstance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            lpszClassName: w!("SysModelPicker"),
            ..Default::default()
        };
        RegisterClassExW(&wc);
    });
}

// ---- public picker entry point --------------------------------------------------------------------

/// Open the model picker popup below `anchor_rect` on screen.
/// `selected` is the current comma-separated model IDs, in chain order.
/// `switchable`: the first two are the pair the switch hotkey flips (main list only).
/// Returns the new selection as comma-separated IDs, or None if cancelled.
pub(crate) unsafe fn open_picker(
    parent: HWND,
    anchor_rect: RECT,
    selected: &str,
    switchable: bool,
    api_key: &str,
    font: HFONT,
) -> Option<String> {
    register_class();

    // Start fetch if needed.
    if models().is_empty() && !is_loading() {
        ensure_models(api_key);
    }

    // Parse current selection.
    let selected_vec: Vec<String> = selected
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Build display list from whatever models are available now.
    let display = build_display(models(), &selected_vec, "", switchable);

    // Create fonts.
    let mk_font = |size: i32, weight: i32, face: PCWSTR| {
        CreateFontW(
            -s(size),
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            face,
        )
    };
    let font_small = mk_font(9, 400, w!("Segoe UI"));
    let font_icon = mk_font(10, 400, w!("Segoe MDL2 Assets"));

    // Compute initial popup size.
    let row_count = if display.is_empty() && models().is_empty() {
        MIN_VISIBLE_ROWS
    } else {
        (display.len() as i32).clamp(MIN_VISIBLE_ROWS, MAX_VISIBLE_ROWS)
    };
    let list_h = row_count * s(LIST_ROW_H);
    let popup_w = s(POPUP_W);
    let popup_h = popup_height(list_h);
    let list_top = s(POP_PAD) + s(SEARCH_H) + s(4);

    let (x, y) = super::settings_controls::place_below(anchor_rect, popup_w, popup_h);

    // Allocate state.
    let state = Box::into_raw(Box::new(Picker {
        hwnd: HWND::default(),
        search_hwnd: HWND::default(),
        font,
        font_small,
        font_icon,
        bg_brush: CreateSolidBrush(BG),
        field_brush: CreateSolidBrush(FIELD_BG),
        selected: selected_vec,
        switchable,
        display,
        scroll_y: 0,
        max_scroll: 0,
        hover_row: -1,
        list_top,
        list_height: list_h,
        done: false,
    }));
    PICKER.with(|c| c.set(state));

    let hinstance = GetModuleHandleW(None).unwrap();
    let hwnd = super::settings_controls::make_popup(w!("SysModelPicker"), parent, x, y, popup_w, popup_h);
    let p = &mut *state;
    p.hwnd = hwnd;

    // Search EDIT — positioned with a left indent for the search icon.
    let search_left = s(POP_PAD) + s(16) + s(2);
    let search_w = popup_w - search_left - s(POP_PAD);
    let search_hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("EDIT"),
        w!(""),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL),
        search_left,
        s(POP_PAD),
        search_w,
        s(SEARCH_H),
        hwnd,
        HMENU(ID_SEARCH as isize as *mut _),
        hinstance,
        None,
    )
    .unwrap();
    p.search_hwnd = search_hwnd;
    SendMessageW(search_hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));

    // Compute initial scroll.
    recompute_scroll(p);

    // Show and focus.
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = SetFocus(search_hwnd);

    // Start polling for model data if not loaded yet.
    if models().is_empty() {
        let _ = SetTimer(hwnd, T_CHECK_MODELS, 200, None);
    }

    let end = super::settings_controls::run_popup(hwnd, || (*state).done);

    // Extract result before freeing state.
    let p = Box::from_raw(PICKER.with(|c| c.replace(std::ptr::null_mut())));

    // Clean up GDI objects we created (the caller's `font` is not ours to delete).
    for obj in [
        HGDIOBJ(p.font_small.0),
        HGDIOBJ(p.font_icon.0),
        HGDIOBJ(p.bg_brush.0),
        HGDIOBJ(p.field_brush.0),
    ] {
        let _ = DeleteObject(obj);
    }

    // Clicking away or Enter keeps the edits; only Escape throws them away.
    (end != super::settings_controls::PopupEnd::Escape).then(|| p.selected.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(id: &str) -> ModelInfo {
        ModelInfo {
            id: id.into(), name: format!("Co: {id}"), compatible: true,
            free: false, vision: true, tools: true, reasoning: false, recommended: false,
        }
    }

    #[test]
    fn chosen_models_are_grouped_by_role_in_chain_order() {
        let all = [m("a"), m("b"), m("c"), m("d")];
        let sel: Vec<String> = ["c", "a", "b"].map(String::from).to_vec();
        let rows = build_display(&all, &sel, "", true);
        assert_eq!(rows, vec![
            Row::Header(H_MAIN), Row::Model(2), Row::Model(0),
            Row::Header(H_FALLBACK), Row::Model(1),
            Row::Header(H_ALL), Row::Model(3),
        ]);
        let rows = build_display(&all, &sel, "", false);
        assert_eq!(rows[0], Row::Header(H_IN_USE));
        // A chosen id OpenRouter doesn't list stays visible (so it can be removed or moved).
        let sel: Vec<String> = ["gone/model", "a"].map(String::from).to_vec();
        let rows = build_display(&all, &sel, "", true);
        assert_eq!(&rows[..3], &[Row::Header(H_MAIN), Row::Missing(0), Row::Model(0)]);
        assert_eq!(display_name(&all[0]), "a");
    }

    #[test]
    fn move_selection_swaps_neighbours_and_stops_at_ends() {
        let mut sel: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
        move_selection(&mut sel, "c", -1);
        assert_eq!(sel, ["a", "c", "b"]);
        move_selection(&mut sel, "a", -1);
        move_selection(&mut sel, "b", 1);
        assert_eq!(sel, ["a", "c", "b"]);
    }
}
