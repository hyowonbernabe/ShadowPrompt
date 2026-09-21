// UI thread: registers window classes, creates indicator + form-indicator + overlay + help +
// debug-rect windows, runs the Win32 message loop, drains UICommand via WM_USER.
//
// Ported from v2's proven structure, then extended for v3:
//   1. SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE) on indicator/overlay/help, gated on
//      Windows build >= 19041 (design doc §6) — landed in M9. See `apply_capture_exclusion`.
//   2. Gradient-crawl overlay rendering (design doc §6, M10) — the answer overlay
//      (`hwnd_overlay`) now renders through `UpdateLayeredWindow` with a real per-pixel-alpha
//      ARGB bitmap built offscreen in `render_overlay`/`draw_and_present`, instead of the old
//      `LWA_COLORKEY` + `WM_PAINT` static paint. Short text (<= `crawl_trigger_lines` wrapped
//      lines) still paints statically; longer text crawls via a `WM_TIMER`-driven ~1.5-line
//      moving window (line 1 opaque, line 2 faded), looping back to the top. The help panel
//      (`hwnd_help`) shares this exact same rendering path via `render_help_panel` (real bug
//      fixed post-M10: it used to render through a separate static `WM_PAINT` + `LWA_COLORKEY`
//      path with its own font, which is why the answer overlay and the cheat sheet looked like
//      they were built by two different people).
// Window class/title below is intentionally generic/unbranded — design doc §6.

use std::sync::Mutex;
use std::sync::mpsc;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, DrawTextW, EndPaint, FillRect, GetTextExtentPoint32W, InvalidateRect,
    SelectObject, SetBkMode, SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    BLENDFUNCTION, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, DIB_RGB_COLORS,
    DT_LEFT, DT_NOCLIP, FF_DONTCARE, HFONT, OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetSystemMetrics, GetWindowDisplayAffinity, GetWindowLongPtrW, KillTimer, LoadCursorW, PostQuitMessage,
    PostThreadMessageW, RegisterClassExW, SetLayeredWindowAttributes, SetTimer, SetWindowDisplayAffinity,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, UpdateLayeredWindow,
    GWLP_USERDATA, HWND_TOPMOST, IDC_ARROW, LWA_ALPHA, MSG, SM_CXSCREEN, SM_CYSCREEN,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_HIDE, SW_SHOWNOACTIVATE, ULW_ALPHA,
    WDA_EXCLUDEFROMCAPTURE, WM_DESTROY, WM_PAINT, WM_TIMER, WM_USER, WNDCLASSEXW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
    WINDOW_EX_STYLE, WINDOW_STYLE,
};
use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::System::SystemInformation::OSVERSIONINFOW;

use super::commands::{FormIndicatorState, IndicatorState, UICommand};
use crate::config::schema::VisualsConfig;

const WM_UI_COMMAND: u32 = WM_USER + 1;

// Generic, unbranded window class/title — design doc §6. Deliberately reads like an ordinary
// system utility window, nothing that hints at this project's name or purpose.
const WINDOW_CLASS_NAME: &str = "SysIndicatorHost\0";
const WINDOW_TITLE: &str = "System Notification\0";

// Gradient-crawl overlay tuning (design doc §6, M10). `crawl_trigger_lines` and
// `crawl_ms_per_line` come from VisualsConfig; the rest has no config field (VisualsConfig has
// no `overlay_width`, and schema.rs is out of this task's scope), so it's a fixed constant here.
const OVERLAY_MAX_WIDTH_PX: i32 = 420;
const OVERLAY_LINE_PAD: i32 = 2;
const CRAWL_SECOND_LINE_FADE: f32 = 0.35;
const CRAWL_TIMER_ID: usize = 1;
const FLASH_TIMER_ID: usize = 2;
const FLASH_NOTICE_MS: u32 = 3000;
/// How long the form indicator stays green on `FormIndicatorState::Done` before auto-hiding,
/// same "flash then clear" shape as the overlay's own `FLASH_TIMER_ID` — a run finishing used to
/// jump straight from Running to Hidden with no visible success state at all.
const FORM_DONE_TIMER_ID: usize = 3;
const FORM_DONE_MS: u32 = 3000;
/// Windows' topmost band is a stack, not a guarantee: whichever window last called
/// `SetWindowPos(HWND_TOPMOST, ...)` sits highest. Task Manager (or any other always-on-top
/// window) asserting topmost after our one-shot calls at window creation would otherwise float
/// above us until we happen to re-render for an unrelated reason. This timer keeps re-asserting
/// topmost on all our windows so we're never the stale entry at the bottom of that stack for
/// long — see `reassert_topmost`.
const TOPMOST_REASSERT_TIMER_ID: usize = 4;
const TOPMOST_REASSERT_MS: u32 = 1500;
/// The help cheat-sheet's fixed anchor — bottom-right, its own small offset, distinct from the
/// answer overlay's *configured* corner (`VisualsConfig::overlay_corner`, no config field of its
/// own exists for this panel). Same rendering technique as the answer overlay either way — this
/// only affects *where* it sits, not what it looks like.
const HELP_CORNER: &str = "bottom_right";
const HELP_OFFSET: [i32; 2] = [8, 8];

static PENDING: Mutex<Vec<UICommand>> = Mutex::new(Vec::new());
static UI_TID: Mutex<Option<u32>> = Mutex::new(None);

pub fn start(visuals: VisualsConfig) -> anyhow::Result<tokio::sync::mpsc::UnboundedSender<UICommand>> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UICommand>();
    let (ready_tx, ready_rx) = mpsc::channel::<u32>();

    let visuals_for_thread = visuals.clone();
    std::thread::Builder::new()
        .name("ui-thread".into())
        .spawn(move || {
            if let Err(e) = run_ui_thread(visuals_for_thread, ready_tx) {
                log::error!("ui thread fatal: {e}");
            }
        })?;

    let tid = ready_rx
        .recv()
        .map_err(|_| anyhow::anyhow!("ui thread failed to start"))?;
    *UI_TID.lock().unwrap() = Some(tid);

    std::thread::Builder::new()
        .name("ui-forwarder".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("forwarder runtime");
            rt.block_on(async move {
                while let Some(cmd) = rx.recv().await {
                    PENDING.lock().unwrap().push(cmd);
                    if let Some(tid) = *UI_TID.lock().unwrap() {
                        unsafe {
                            let _ = PostThreadMessageW(tid, WM_UI_COMMAND, WPARAM(0), LPARAM(0));
                        }
                    }
                }
            });
        })?;

    Ok(tx)
}

struct UiCtx {
    visuals: VisualsConfig,
    hwnd_indicator: HWND,
    hwnd_form: HWND,
    hwnd_overlay: HWND,
    hwnd_debug: HWND,
    hwnd_help: HWND,
    indicator: IndicatorState,
    form: FormIndicatorState,
    /// The last "real" answer text set via `SetOverlayText` (design doc §6 primary data flow).
    overlay_text: String,
    /// Transient notice text set via `FlashNotice` (e.g. `switch_model`). Takes display
    /// priority over `overlay_text` while `Some`, and auto-clears after `FLASH_NOTICE_MS` via
    /// `FLASH_TIMER_ID` — see `drain_commands` and the `WM_TIMER` handling in `wnd_proc`.
    flash_text: Option<String>,
    /// Whichever of `flash_text`/`overlay_text` was actually rendered last, used to detect a
    /// real content change (vs. a mere crawl tick) so the crawl restarts at the top for new text.
    overlay_active_cache: String,
    /// Index of the wrapped line currently shown fully opaque at the top of the crawl window.
    overlay_crawl_line: usize,
    help_text: String,
    hidden: bool,
}

fn run_ui_thread(visuals: VisualsConfig, ready_tx: mpsc::Sender<u32>) -> anyhow::Result<()> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name: Vec<u16> = WINDOW_CLASS_NAME.encode_utf16().collect();

        // Real bug found live: with no `hCursor` here, Windows never resets the pointer while
        // it's over any of these windows, so hovering the indicator showed whatever cursor
        // happened to be active beforehand (often the "busy"/loading one) — making the app look
        // unresponsive even while it was idle. `LoadCursorW(None, IDC_ARROW)` is the normal
        // system arrow, same as every ordinary window gets by default.
        let cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
            hCursor: cursor,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        let ind_ex = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
        let overlay_ex = ind_ex | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT;

        let ind_size = visuals.indicator_size.max(1) as i32;
        let (ix, iy) = corner_pos(&visuals.indicator_corner, visuals.indicator_offset, ind_size, ind_size, screen_w, screen_h);
        let hwnd_indicator = create_window(&class_name, ind_ex, WS_POPUP | WS_VISIBLE, ix, iy, ind_size, ind_size, hinstance.into());
        let _ = SetLayeredWindowAttributes(hwnd_indicator, COLORREF(0), 255, LWA_ALPHA);

        let (fx, fy) = corner_pos(&visuals.form_indicator_corner, visuals.form_indicator_offset, ind_size, ind_size, screen_w, screen_h);
        let hwnd_form = create_window(&class_name, ind_ex, WS_POPUP, fx, fy, ind_size, ind_size, hinstance.into());
        let _ = SetLayeredWindowAttributes(hwnd_form, COLORREF(0), 255, LWA_ALPHA);

        // hwnd_overlay is driven entirely by UpdateLayeredWindow (render_overlay /
        // draw_and_present below) — no SetLayeredWindowAttributes here; mixing the two alpha
        // mechanisms on the same window is redundant at best. Initial size is a placeholder;
        // real position/size come from the first render_overlay call once content exists.
        let (ox, oy) = (visuals.overlay_offset[0], visuals.overlay_offset[1]);
        let hwnd_overlay = create_window(&class_name, overlay_ex, WS_POPUP, ox, oy, 10, visuals.overlay_font_size.max(1) as i32, hinstance.into());

        let hwnd_debug = create_window(&class_name, overlay_ex, WS_POPUP, 0, 0, 1, 1, hinstance.into());
        let _ = SetLayeredWindowAttributes(hwnd_debug, COLORREF(0), 180, LWA_ALPHA);

        // hwnd_help now shares the exact same UpdateLayeredWindow rendering as hwnd_overlay
        // (render_help_panel / draw_and_present below) — same font, same alpha technique, same
        // "everything" the answer overlay uses, differing only in content and anchor position.
        // Real, confirmed inconsistency this replaces: help used to be a completely separate
        // WM_PAINT + LWA_COLORKEY static-text path with its own font-creation call — visually a
        // different UI, not the same style at a different location, exactly as reported. No
        // SetLayeredWindowAttributes here for the same reason hwnd_overlay has none: real size
        // comes from the first render once content exists.
        let hwnd_help = create_window(&class_name, overlay_ex, WS_POPUP, HELP_OFFSET[0], HELP_OFFSET[1], 10, visuals.overlay_font_size.max(1) as i32, hinstance.into());

        apply_capture_exclusion(&[hwnd_indicator, hwnd_form, hwnd_overlay, hwnd_help]);

        let mut ctx = Box::new(UiCtx {
            visuals: visuals.clone(),
            hwnd_indicator,
            hwnd_form,
            hwnd_overlay,
            hwnd_debug,
            hwnd_help,
            indicator: IndicatorState::Ready,
            form: FormIndicatorState::Hidden,
            overlay_text: String::new(),
            flash_text: None,
            overlay_active_cache: String::new(),
            overlay_crawl_line: 0,
            help_text: String::new(),
            hidden: false,
        });
        let ctx_ptr: *mut UiCtx = &mut *ctx;
        SetWindowLongPtrW(hwnd_indicator, GWLP_USERDATA, ctx_ptr as isize);
        SetWindowLongPtrW(hwnd_form, GWLP_USERDATA, ctx_ptr as isize);
        SetWindowLongPtrW(hwnd_overlay, GWLP_USERDATA, ctx_ptr as isize);
        SetWindowLongPtrW(hwnd_debug, GWLP_USERDATA, ctx_ptr as isize);
        SetWindowLongPtrW(hwnd_help, GWLP_USERDATA, ctx_ptr as isize);

        let _ = ShowWindow(hwnd_indicator, SW_SHOWNOACTIVATE);
        topmost(hwnd_indicator);
        // Establishes the overlay's initial (hidden, empty-content) layered surface for real,
        // rather than force-showing it — render_overlay decides visibility from content+hidden.
        render_overlay(&mut ctx, false);

        if SetTimer(hwnd_indicator, TOPMOST_REASSERT_TIMER_ID, TOPMOST_REASSERT_MS, None) == 0 {
            log::warn!("topmost reassert: SetTimer failed");
        }

        let tid = GetCurrentThreadId();
        let _ = ready_tx.send(tid);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_UI_COMMAND {
                drain_commands(&mut ctx);
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        std::mem::forget(ctx);
    }
    Ok(())
}

/// SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE), gated on Windows build >= 19041 (checked via
/// RtlGetVersion) to avoid the WDA_MONITOR black-box degrade on older builds. Excludes the given
/// windows from screen-capture/screen-share APIs (design doc §6): defeats passive software
/// screen-monitoring (BitBlt/PrintWindow/Desktop Duplication/WGC-based capture pipelines all
/// respect this DWM-level flag). Deliberately gated on the real OS build via `RtlGetVersion` (not
/// `GetVersionExW`, which lies about the version above Windows 8.1 unless the exe carries a
/// manifest declaring Windows 10 support) — below build 19041 (May 2020 Update),
/// `WDA_EXCLUDEFROMCAPTURE` silently degrades to `WDA_MONITOR`, a visible black rectangle in any
/// capture, which is worse than doing nothing. Skipping the call entirely on old builds avoids
/// that regression; this is not a security guarantee (does not defend against a phone camera,
/// and `GetWindowDisplayAffinity` is itself a public API any process can query to detect the
/// flag's presence — see docs/COMPETITORS.md Part 3).
fn apply_capture_exclusion(windows: &[HWND]) {
    if !os_build_supports_capture_exclusion() {
        log::info!("capture exclusion: OS build below 19041, skipping (would degrade to a visible black box)");
        return;
    }
    for &hwnd in windows {
        unsafe {
            if let Err(e) = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) {
                log::warn!("capture exclusion: SetWindowDisplayAffinity failed: {e}");
                continue;
            }
            // Read back what Windows actually recorded, rather than assuming a success return
            // means the flag stuck — this was never actually visually confirmed (build plan M9),
            // and a user report of a stealth window showing up in a real screenshot needs a way
            // to tell "the OS silently didn't apply this" apart from "the flag is set but this
            // particular capture method/tool doesn't honor it anyway."
            let mut affinity: u32 = 0;
            match GetWindowDisplayAffinity(hwnd, &mut affinity) {
                Ok(()) if affinity == WDA_EXCLUDEFROMCAPTURE.0 => {}
                Ok(()) => log::warn!(
                    "capture exclusion: set on {hwnd:?} but readback reports affinity {affinity}, not WDA_EXCLUDEFROMCAPTURE — Windows did not actually apply it"
                ),
                Err(e) => log::warn!("capture exclusion: GetWindowDisplayAffinity readback failed: {e}"),
            }
        }
    }
}

fn os_build_supports_capture_exclusion() -> bool {
    const MIN_BUILD: u32 = 19041;
    unsafe {
        let mut info = OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
            ..Default::default()
        };
        let status = RtlGetVersion(&mut info as *mut OSVERSIONINFOW as *mut _);
        if status.is_err() {
            log::warn!("RtlGetVersion failed ({status:?}); assuming capture exclusion unsupported");
            return false;
        }
        info.dwBuildNumber >= MIN_BUILD
    }
}

fn drain_commands(ctx: &mut UiCtx) {
    let cmds: Vec<UICommand> = std::mem::take(&mut *PENDING.lock().unwrap());
    for cmd in cmds {
        match cmd {
            UICommand::SetIndicatorState(s) => {
                ctx.indicator = s;
                invalidate(ctx.hwnd_indicator);
            }
            UICommand::SetFormIndicator(s) => {
                ctx.form = s;
                unsafe {
                    // Any new state cancels a pending auto-hide from a previous `Done` — most
                    // relevantly, a run starting again right after the last one's green flash.
                    let _ = KillTimer(ctx.hwnd_form, FORM_DONE_TIMER_ID);
                    let _ = ShowWindow(ctx.hwnd_form, if matches!(s, FormIndicatorState::Hidden) { SW_HIDE } else { SW_SHOWNOACTIVATE });
                    if matches!(s, FormIndicatorState::Done) && SetTimer(ctx.hwnd_form, FORM_DONE_TIMER_ID, FORM_DONE_MS, None) == 0 {
                        log::warn!("form indicator: SetTimer for the Done auto-hide failed");
                    }
                }
                invalidate(ctx.hwnd_form);
            }
            UICommand::SetOverlayText(t) => {
                ctx.overlay_text = t;
                ctx.flash_text = None;
                stop_flash_timer(ctx);
                render_overlay(ctx, false);
            }
            UICommand::ClearOverlay => {
                ctx.overlay_text.clear();
                ctx.flash_text = None;
                stop_flash_timer(ctx);
                render_overlay(ctx, false);
            }
            // Dev/testing aid only, never in production — a visible selection box on screen is
            // exactly the kind of artifact stealth requires not shipping (CLAUDE.md: "no visible
            // window... any new UI surface must justify itself against this"), and the user
            // explicitly doesn't want it in a real build ("ugly purple box... that should just be
            // for testing"). Gated on the same `debug` feature that already controls the console
            // window, not `debug_assertions` — matches the existing dev/prod split in this repo.
            #[cfg(feature = "debug")]
            UICommand::ShowDebugRect { x, y, w, h } => unsafe {
                let _ = SetWindowPos(ctx.hwnd_debug, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE);
                let _ = ShowWindow(ctx.hwnd_debug, SW_SHOWNOACTIVATE);
                invalidate(ctx.hwnd_debug);
            },
            #[cfg(not(feature = "debug"))]
            UICommand::ShowDebugRect { .. } => {}
            #[cfg(feature = "debug")]
            UICommand::HideDebugRect => unsafe {
                let _ = ShowWindow(ctx.hwnd_debug, SW_HIDE);
            },
            #[cfg(not(feature = "debug"))]
            UICommand::HideDebugRect => {}
            UICommand::HideOverlaysForCapture => unsafe {
                let _ = ShowWindow(ctx.hwnd_overlay, SW_HIDE);
                let _ = ShowWindow(ctx.hwnd_help, SW_HIDE);
            },
            UICommand::RestoreOverlaysAfterCapture => {
                // Re-render rather than a plain `ShowWindow` — this naturally respects whatever
                // `ctx.hidden`/content state was actually true before the capture instead of
                // unconditionally re-showing a panel the user had hidden on purpose.
                render_overlay(ctx, false);
                render_help_panel(ctx);
            }
            UICommand::ToggleHide => {
                ctx.hidden = !ctx.hidden;
                unsafe {
                    let sw = if ctx.hidden { SW_HIDE } else { SW_SHOWNOACTIVATE };
                    let _ = ShowWindow(ctx.hwnd_indicator, sw);
                    if !matches!(ctx.form, FormIndicatorState::Hidden) {
                        let _ = ShowWindow(ctx.hwnd_form, sw);
                    }
                }
                // Re-applies the current state for the new `hidden` value without resetting the
                // crawl position or clearing the help text — both the answer overlay and the
                // help panel respect the global hide-toggle (a real gap fixed here: this never
                // touched hwnd_help before, so "hide everything" didn't hide an open cheat sheet).
                render_overlay(ctx, false);
                render_help_panel(ctx);
            }
            UICommand::ShowHelp(t) => {
                ctx.help_text = t;
                render_help_panel(ctx);
            }
            UICommand::HideHelp => {
                ctx.help_text.clear();
                render_help_panel(ctx);
            }
            UICommand::FlashNotice(t) => {
                ctx.flash_text = Some(t);
                start_flash_timer(ctx);
                render_overlay(ctx, false);
            }
            UICommand::Shutdown => unsafe {
                PostQuitMessage(0);
            },
        }
    }
}

fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(hwnd, None, true);
    }
}

fn topmost(hwnd: HWND) {
    unsafe {
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

/// Re-asserts topmost on every one of our windows, hidden or not — `SetWindowPos` on a hidden
/// window just reorders it in the z-stack without showing it, so no visibility check is needed
/// here. Called on a timer (see `TOPMOST_REASSERT_TIMER_ID`) since a one-shot topmost call gets
/// silently outranked the moment anything else asserts topmost afterward.
fn reassert_topmost(ctx: &UiCtx) {
    topmost(ctx.hwnd_indicator);
    topmost(ctx.hwnd_form);
    topmost(ctx.hwnd_overlay);
    topmost(ctx.hwnd_help);
    #[cfg(feature = "debug")]
    topmost(ctx.hwnd_debug);
}

fn start_crawl_timer(ctx: &UiCtx) {
    let ms = ctx.visuals.crawl_ms_per_line.clamp(100, u32::MAX as u64) as u32;
    unsafe {
        if SetTimer(ctx.hwnd_overlay, CRAWL_TIMER_ID, ms, None) == 0 {
            log::warn!("overlay crawl: SetTimer failed");
        }
    }
}

fn stop_crawl_timer(ctx: &UiCtx) {
    unsafe {
        let _ = KillTimer(ctx.hwnd_overlay, CRAWL_TIMER_ID);
    }
}

fn start_flash_timer(ctx: &UiCtx) {
    unsafe {
        if SetTimer(ctx.hwnd_overlay, FLASH_TIMER_ID, FLASH_NOTICE_MS, None) == 0 {
            log::warn!("overlay flash notice: SetTimer failed");
        }
    }
}

fn stop_flash_timer(ctx: &UiCtx) {
    unsafe {
        let _ = KillTimer(ctx.hwnd_overlay, FLASH_TIMER_ID);
    }
}

/// The text that should currently be visible on the overlay: a live `FlashNotice` takes
/// priority over the standing answer text (design doc §6/§8/§10, `FlashNotice` doc comment).
fn active_overlay_text(ctx: &UiCtx) -> String {
    ctx.flash_text.clone().unwrap_or_else(|| ctx.overlay_text.clone())
}

/// Re-renders the answer overlay for real (design doc §6, M10): word-wraps the active text,
/// decides static-vs-crawl from `crawl_trigger_lines`, advances the crawl window by one line
/// when `advance_crawl` is true (i.e. called from the `CRAWL_TIMER_ID` tick), and (re)builds a
/// per-pixel-alpha ARGB bitmap via `UpdateLayeredWindow`. Also starts/stops the crawl timer as
/// needed and applies the current `hidden`/empty-content visibility.
fn render_overlay(ctx: &mut UiCtx, advance_crawl: bool) {
    let active = active_overlay_text(ctx);
    if active != ctx.overlay_active_cache {
        ctx.overlay_active_cache = active.clone();
        ctx.overlay_crawl_line = 0;
    }

    if active.is_empty() {
        stop_crawl_timer(ctx);
        unsafe {
            let _ = ShowWindow(ctx.hwnd_overlay, SW_HIDE);
        }
        return;
    }

    let font_size = ctx.visuals.overlay_font_size.max(1) as i32;
    unsafe {
        let mem_dc = CreateCompatibleDC(None);
        let font = make_overlay_font(font_size);
        let old_font = SelectObject(mem_dc, font);

        let lines = wrap_lines(mem_dc, &active, OVERLAY_MAX_WIDTH_PX);
        let trigger = ctx.visuals.crawl_trigger_lines.max(1) as usize;
        let crawling = !lines.is_empty() && lines.len() > trigger;

        if crawling {
            if advance_crawl {
                ctx.overlay_crawl_line = (ctx.overlay_crawl_line + 1) % lines.len();
            } else if ctx.overlay_crawl_line >= lines.len() {
                ctx.overlay_crawl_line = 0;
            }
            start_crawl_timer(ctx);
        } else {
            stop_crawl_timer(ctx);
            ctx.overlay_crawl_line = 0;
        }

        draw_and_present(ctx, mem_dc, &lines, crawling, font_size, ctx.hwnd_overlay, &ctx.visuals.overlay_corner, ctx.visuals.overlay_offset);

        SelectObject(mem_dc, old_font);
        let _ = DeleteObject(font);
        let _ = DeleteDC(mem_dc);
    }
}

/// Re-renders the help cheat-sheet — same rendering technique as `render_overlay` (same font,
/// same real per-pixel-alpha `UpdateLayeredWindow` bitmap, same word-wrap), deliberately never
/// crawling: this is a reference sheet meant to be read in full, not glanced at, so it always
/// shows every line statically regardless of length. This is the fix for the reported UI
/// inconsistency — before this, `hwnd_help` rendered through a completely separate WM_PAINT +
/// LWA_COLORKEY path with its own font-creation call (`paint_static_panel`, now removed).
fn render_help_panel(ctx: &UiCtx) {
    if ctx.help_text.is_empty() {
        unsafe {
            let _ = ShowWindow(ctx.hwnd_help, SW_HIDE);
        }
        return;
    }
    let font_size = ctx.visuals.overlay_font_size.max(1) as i32;
    unsafe {
        let mem_dc = CreateCompatibleDC(None);
        let font = make_overlay_font(font_size);
        let old_font = SelectObject(mem_dc, font);

        let lines = wrap_lines(mem_dc, &ctx.help_text, OVERLAY_MAX_WIDTH_PX);
        draw_and_present(ctx, mem_dc, &lines, false, font_size, ctx.hwnd_help, HELP_CORNER, HELP_OFFSET);

        SelectObject(mem_dc, old_font);
        let _ = DeleteObject(font);
        let _ = DeleteDC(mem_dc);
    }
}

unsafe fn make_overlay_font(font_size: i32) -> HFONT {
    let face: Vec<u16> = "Arial\0".encode_utf16().collect();
    CreateFontW(
        font_size, 0, 0, 0, 400, 0, 0, 0,
        DEFAULT_CHARSET.0 as u32, OUT_DEFAULT_PRECIS.0 as u32, CLIP_DEFAULT_PRECIS.0 as u32,
        DEFAULT_QUALITY.0 as u32, (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32, PCWSTR(face.as_ptr()),
    )
}

unsafe fn measure_width(hdc: windows::Win32::Graphics::Gdi::HDC, text: &str) -> i32 {
    if text.is_empty() {
        return 0;
    }
    let utf: Vec<u16> = text.encode_utf16().collect();
    let mut size = SIZE::default();
    let _ = GetTextExtentPoint32W(hdc, &utf, &mut size);
    size.cx
}

/// Greedy word-wrap against `max_w` pixels, measured with whatever font is currently selected
/// into `hdc`. Explicit newlines in `text` start new paragraphs (each wrapped independently); a
/// single "word" wider than `max_w` on its own (long URL/token with no spaces) is hard-split by
/// character as a fallback, so one pathological run can't blow out the overlay's width.
unsafe fn wrap_lines(hdc: windows::Win32::Graphics::Gdi::HDC, text: &str, max_w: i32) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.trim().is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            push_word(hdc, &mut current, &mut out, word, max_w);
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

unsafe fn push_word(hdc: windows::Win32::Graphics::Gdi::HDC, current: &mut String, out: &mut Vec<String>, word: &str, max_w: i32) {
    let candidate = if current.is_empty() { word.to_string() } else { format!("{current} {word}") };
    if measure_width(hdc, &candidate) <= max_w {
        *current = candidate;
        return;
    }
    if !current.is_empty() {
        out.push(std::mem::take(current));
    }
    // `word` alone may still be too wide (no spaces to break on) — hard-split by character.
    let mut chunk = String::new();
    for ch in word.chars() {
        let attempt = format!("{chunk}{ch}");
        if chunk.is_empty() || measure_width(hdc, &attempt) <= max_w {
            chunk = attempt;
        } else {
            out.push(std::mem::take(&mut chunk));
            chunk.push(ch);
        }
    }
    *current = chunk;
}

/// Renders `rows` into a throwaway DIB and returns the raw per-pixel intensity (0-255, before
/// `fade`/color is applied) as a flat `w * h` buffer. This is GDI's classic "text alpha" trick —
/// draw white glyphs on a zeroed background, then treat the resulting intensity as alpha — run
/// through a small helper so `draw_and_present` can call it twice: once undisplaced for the
/// glyph itself, once offset by a pixel or two to get its drop-shadow silhouette.
unsafe fn render_intensity_layer(
    mem_dc: windows::Win32::Graphics::Gdi::HDC,
    w: i32,
    h: i32,
    rows: &[(&str, i32, f32)],
    line_h: i32,
    margin: i32,
    offsets: &[(i32, i32)],
) -> Vec<u8> {
    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits_ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let dib = match CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("overlay render: intensity layer CreateDIBSection failed: {e}");
            return vec![0u8; (w as usize) * (h as usize)];
        }
    };
    let old_bmp = SelectObject(mem_dc, dib);

    let stride = (w as usize) * 4;
    std::ptr::write_bytes(bits_ptr as *mut u8, 0, stride * h as usize);
    SetBkMode(mem_dc, TRANSPARENT);
    SetTextColor(mem_dc, super::colors::colorref((255, 255, 255)));
    for (text, y, _fade) in rows {
        for (dx, dy) in offsets {
            let mut buf: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let mut rect = RECT {
                left: margin + dx,
                top: *y + margin + dy,
                right: w - margin + dx,
                bottom: *y + margin + line_h + dy,
            };
            let _ = DrawTextW(mem_dc, &mut buf, &mut rect, DT_LEFT | DT_NOCLIP);
        }
    }

    let src: &[u8] = std::slice::from_raw_parts(bits_ptr as *const u8, stride * h as usize);
    let mut out = vec![0u8; (w as usize) * (h as usize)];
    for (i, px) in src.as_chunks::<4>().0.iter().enumerate() {
        out[i] = px[0].max(px[1]).max(px[2]);
    }

    SelectObject(mem_dc, old_bmp);
    let _ = DeleteObject(dib);
    out
}

/// Builds the offscreen 32bpp ARGB DIB for the current frame, fixes up its alpha channel from
/// GDI's white-on-black text rendering (see inline comment below), and presents it with
/// `UpdateLayeredWindow`. Static mode paints every wrapped line fully opaque; crawl mode paints
/// only the current top line (opaque) and the next line (faded, and vertically clipped by the
/// ~1.5-line bitmap height), which is what actually produces the "gradient-crawl" look. Every
/// glyph also gets a soft black drop-shadow (see `render_intensity_layer`), enough contrast to
/// stay legible on light backgrounds without reading as an obvious on-screen box.
#[allow(clippy::too_many_arguments)]
unsafe fn draw_and_present(
    ctx: &UiCtx,
    mem_dc: windows::Win32::Graphics::Gdi::HDC,
    lines: &[String],
    crawling: bool,
    font_size: i32,
    hwnd: HWND,
    corner: &str,
    offset: [i32; 2],
) {
    if lines.is_empty() {
        let _ = ShowWindow(hwnd, SW_HIDE);
        return;
    }
    let line_h = font_size + OVERLAY_LINE_PAD;

    // (text, y-offset within the bitmap, fade factor 0.0-1.0) for each row band actually drawn
    // this frame.
    let rows: Vec<(&str, i32, f32)> = if crawling {
        let n = lines.len();
        let top = ctx.overlay_crawl_line % n;
        let next = (top + 1) % n;
        vec![
            (lines[top].as_str(), 0, 1.0_f32),
            (lines[next].as_str(), line_h, CRAWL_SECOND_LINE_FADE),
        ]
    } else {
        lines
            .iter()
            .enumerate()
            .map(|(i, l)| (l.as_str(), i as i32 * line_h, 1.0_f32))
            .collect()
    };

    let content_h: i32 = if crawling {
        ((line_h as f32) * 1.5).ceil() as i32
    } else {
        line_h * lines.len() as i32
    };
    let content_w: i32 = lines.iter().map(|l| measure_width(mem_dc, l)).max().unwrap_or(4).max(4);

    // Soft drop-shadow behind the glyphs, not a full outline — a full black ring around every
    // letter read as an obvious on-screen box, too conspicuous for something meant to be
    // glanced at, not noticed (user feedback: "way too visible"). A single offset shadow at
    // partial opacity gives just enough contrast on light backgrounds without looking like an
    // overlay. `BORDER_MARGIN` pads the bitmap so the shadow isn't clipped at its edge.
    const BORDER_MARGIN: i32 = 2;
    const SHADOW_OFFSET: [(i32, i32); 1] = [(1, 1)];
    const SHADOW_OPACITY: f32 = 0.55;
    let w = content_w + BORDER_MARGIN * 2;
    let h = (content_h + BORDER_MARGIN * 2).max(1);

    // Two intensity passes: the glyph itself, and the same glyph shifted down-right by 1px (the
    // shadow silhouette). Composited below via a premultiplied "over" blend (glyph on top of
    // shadow) at `SHADOW_OPACITY`, so a partially-covered edge pixel still has a hint of dark
    // backing instead of fading straight into a light background — a plain white-on-black GDI
    // trick can't produce a colored (black) shadow directly, since alpha there is derived from
    // intensity and black has none, so the two need separate layers.
    let shadow_raw = render_intensity_layer(mem_dc, w, h, &rows, line_h, BORDER_MARGIN, &SHADOW_OFFSET);
    let main_raw = render_intensity_layer(mem_dc, w, h, &rows, line_h, BORDER_MARGIN, &[(0, 0)]);

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h, // negative = top-down DIB
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits_ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let dib = match CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("overlay render: CreateDIBSection failed: {e}");
            return;
        }
    };
    let old_bmp = SelectObject(mem_dc, dib);
    let _ = &mut bmi; // silence unused-mut from the struct-update pattern above

    let stride = (w as usize) * 4;
    std::ptr::write_bytes(bits_ptr as *mut u8, 0, stride * h as usize);

    // Premultiplied "over" compositing: glyph (white, premultiplied color == alpha == `a_m`) on
    // top of shadow (black, premultiplied color == 0, alpha == `a_o`, capped at `SHADOW_OPACITY`
    // so it stays a soft tint rather than a solid block). Since the shadow's color channels are
    // 0, they never change the glyph's own RGB — only alpha increases where the shadow extends
    // past a partially-covered glyph edge. The per-row `fade` factor (crawl's second line)
    // uniformly scales both layers before compositing.
    let buf: &mut [u8] = std::slice::from_raw_parts_mut(bits_ptr as *mut u8, stride * h as usize);
    for (_text, y0, fade) in &rows {
        let y_start = (*y0 + BORDER_MARGIN).max(0);
        let y_end = (*y0 + BORDER_MARGIN + line_h).min(h);
        for row in y_start..y_end {
            let row_off = row as usize * stride;
            for col in 0..w as usize {
                let px_idx = row as usize * w as usize + col;
                let byte_off = row_off + col * 4;
                let a_m = ((main_raw[px_idx] as f32) * fade).round().clamp(0.0, 255.0);
                let a_o = ((shadow_raw[px_idx] as f32) * fade * SHADOW_OPACITY).round().clamp(0.0, 255.0);
                if a_m == 0.0 && a_o == 0.0 {
                    continue;
                }
                let out_alpha = (a_m + a_o * (255.0 - a_m) / 255.0).round().clamp(0.0, 255.0) as u8;
                let out_rgb = a_m.round().clamp(0.0, 255.0) as u8;
                buf[byte_off] = out_rgb;
                buf[byte_off + 1] = out_rgb;
                buf[byte_off + 2] = out_rgb;
                buf[byte_off + 3] = out_alpha;
            }
        }
    }

    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    let (x, y) = corner_pos(corner, offset, w, h, screen_w, screen_h);
    let pt_dst = POINT { x, y };
    let size = SIZE { cx: w, cy: h };
    let pt_src = POINT { x: 0, y: 0 };
    let blend = BLENDFUNCTION {
        BlendOp: windows::Win32::Graphics::Gdi::AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: windows::Win32::Graphics::Gdi::AC_SRC_ALPHA as u8,
    };
    if let Err(e) = UpdateLayeredWindow(
        hwnd,
        None,
        Some(&pt_dst as *const POINT),
        Some(&size as *const SIZE),
        mem_dc,
        Some(&pt_src as *const POINT),
        COLORREF(0),
        Some(&blend as *const BLENDFUNCTION),
        ULW_ALPHA,
    ) {
        log::warn!("panel render: UpdateLayeredWindow failed: {e}");
    }

    // User report: the overlay showed up in a Windows Snipping Tool capture specifically while
    // the gradient-crawl effect was animating — i.e. specifically while `UpdateLayeredWindow` was
    // being called repeatedly on a timer, not while the window was static. `SetWindowDisplayAffinity`
    // is only ever set once, at window creation; reasserting it after every single frame this
    // window presents is cheap and removes any possibility that repeatedly resizing/repositioning
    // the layered surface (crawl mode changes both as it advances) silently drops the exclusion on
    // whatever new backing surface each `UpdateLayeredWindow` call allocates.
    if os_build_supports_capture_exclusion() {
        if let Err(e) = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) {
            log::warn!("panel render: re-asserting capture exclusion failed: {e}");
        }
    }

    SelectObject(mem_dc, old_bmp);
    let _ = DeleteObject(dib);

    // Both the answer overlay and the help panel respect the global hide-toggle now — a real,
    // separate consistency gap this also fixes: ToggleHide previously never touched hwnd_help
    // at all, so hiding "everything" didn't actually hide the cheat sheet if it was open.
    let sw = if ctx.hidden { SW_HIDE } else { SW_SHOWNOACTIVATE };
    let _ = ShowWindow(hwnd, sw);
    topmost(hwnd);
}

#[allow(clippy::too_many_arguments)]
fn create_window(class: &[u16], ex: WINDOW_EX_STYLE, style: WINDOW_STYLE, x: i32, y: i32, w: i32, h: i32, hinstance: windows::Win32::Foundation::HINSTANCE) -> HWND {
    unsafe {
        let title: Vec<u16> = WINDOW_TITLE.encode_utf16().collect();
        CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, x, y, w, h, None, None, hinstance, None)
            .unwrap_or_default()
    }
}

fn corner_pos(corner: &str, offset: [i32; 2], w: i32, h: i32, screen_w: i32, screen_h: i32) -> (i32, i32) {
    match corner {
        "top_left" => (offset[0], offset[1]),
        "top_right" => (screen_w - w - offset[0], offset[1]),
        "bottom_left" => (offset[0], screen_h - h - offset[1]),
        _ => (screen_w - w - offset[0], screen_h - h - offset[1]),
    }
}

const WM_ERASEBKGND_LOCAL: u32 = 0x0014;

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let ctx_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiCtx;
            if !ctx_ptr.is_null() {
                paint(hwnd, &*ctx_ptr);
            }
            LRESULT(0)
        }
        WM_TIMER => {
            let ctx_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiCtx;
            if !ctx_ptr.is_null() {
                let ctx = &mut *ctx_ptr;
                match wp.0 {
                    CRAWL_TIMER_ID => render_overlay(ctx, true),
                    FLASH_TIMER_ID => {
                        stop_flash_timer(ctx);
                        ctx.flash_text = None;
                        render_overlay(ctx, false);
                    }
                    FORM_DONE_TIMER_ID => {
                        let _ = KillTimer(ctx.hwnd_form, FORM_DONE_TIMER_ID);
                        ctx.form = FormIndicatorState::Hidden;
                        let _ = ShowWindow(ctx.hwnd_form, SW_HIDE);
                        invalidate(ctx.hwnd_form);
                    }
                    TOPMOST_REASSERT_TIMER_ID => reassert_topmost(ctx),
                    _ => {}
                }
            }
            LRESULT(0)
        }
        WM_ERASEBKGND_LOCAL => LRESULT(1),
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

/// Dispatches `WM_PAINT` for the indicator/form/debug/help windows. `hwnd_overlay` and
/// `hwnd_help` are both excluded — their content is driven entirely by `UpdateLayeredWindow` via
/// `render_overlay`/`render_help_panel`, not by `WM_PAINT`; if Windows ever sends either one
/// anyway (rare for a `ULW`-managed layered window), it just needs a `BeginPaint`/`EndPaint` pair
/// to ack the request, no drawing.
unsafe fn paint(hwnd: HWND, ctx: &UiCtx) {
    if hwnd == ctx.hwnd_overlay || hwnd == ctx.hwnd_help {
        let mut ps = PAINTSTRUCT::default();
        let _ = BeginPaint(hwnd, &mut ps);
        let _ = EndPaint(hwnd, &ps);
        return;
    }
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);

    let mut rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut rect);

    // Real bug found in live testing: this used to `FillRect` the *entire* selection rectangle
    // solid magenta at 180/255 alpha, which both looked like the screen was darkening/tinting
    // under whatever app was behind it and hid the exact content being selected. Drawing a thin
    // border instead of a filled block fixes both: only ~3px of the region is ever painted, and
    // the content underneath stays fully visible.
    if hwnd == ctx.hwnd_debug {
        const BORDER_PX: i32 = 3;
        let brush = CreateSolidBrush(super::colors::colorref((255, 0, 255)));
        let top = RECT { left: rect.left, top: rect.top, right: rect.right, bottom: (rect.top + BORDER_PX).min(rect.bottom) };
        let bottom = RECT { left: rect.left, top: (rect.bottom - BORDER_PX).max(rect.top), right: rect.right, bottom: rect.bottom };
        let left = RECT { left: rect.left, top: rect.top, right: (rect.left + BORDER_PX).min(rect.right), bottom: rect.bottom };
        let right = RECT { left: (rect.right - BORDER_PX).max(rect.left), top: rect.top, right: rect.right, bottom: rect.bottom };
        for edge in [&top, &bottom, &left, &right] {
            FillRect(hdc, edge, brush);
        }
        let _ = DeleteObject(brush);
        let _ = EndPaint(hwnd, &ps);
        return;
    }

    let fill_color = if hwnd == ctx.hwnd_indicator {
        let c = match ctx.indicator {
            IndicatorState::Ready => &ctx.visuals.color_ready,
            IndicatorState::Processing => &ctx.visuals.color_processing,
            IndicatorState::Error | IndicatorState::InstaDeleteArmed => &ctx.visuals.color_error,
        };
        super::colors::parse_hex(c).unwrap_or((0, 255, 0))
    } else {
        let c = match ctx.form {
            FormIndicatorState::Running => &ctx.visuals.form_color_running,
            FormIndicatorState::Done => &ctx.visuals.form_color_done,
            FormIndicatorState::Failed => &ctx.visuals.form_color_failed,
            FormIndicatorState::Aborted => &ctx.visuals.form_color_aborted,
            FormIndicatorState::Hidden => &ctx.visuals.form_color_running,
        };
        super::colors::parse_hex(c).unwrap_or((255, 255, 0))
    };

    let brush = CreateSolidBrush(super::colors::colorref(fill_color));
    FillRect(hdc, &rect, brush);
    let _ = DeleteObject(brush);

    let _ = EndPaint(hwnd, &ps);
}
