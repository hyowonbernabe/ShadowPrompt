// UI thread: registers window classes, creates indicator + form-indicator + overlay
// + debug-rect windows, runs Win32 message loop, drains UICommand channel via WM_USER.

use std::sync::Mutex;
use std::sync::mpsc;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
    GetTextExtentPoint32W, InvalidateRect, SelectObject, SetBkMode, SetTextColor,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, DT_LEFT, DT_NOCLIP,
    FF_DONTCARE, OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::Foundation::SIZE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetSystemMetrics, GetWindowLongPtrW, PostQuitMessage, PostThreadMessageW, RegisterClassExW,
    SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage,
    GWLP_USERDATA, HWND_TOPMOST, LWA_ALPHA, LWA_COLORKEY, MSG, SM_CXSCREEN, SM_CYSCREEN,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_HIDE, SW_SHOWNOACTIVATE, WM_DESTROY,
    WM_PAINT, WM_USER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE, WINDOW_EX_STYLE, WINDOW_STYLE,
};

use super::commands::{FormIndicatorState, IndicatorState, UICommand};
use crate::config::schema::VisualsConfig;

const WM_UI_COMMAND: u32 = WM_USER + 1;

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
    indicator: IndicatorState,
    form: FormIndicatorState,
    overlay_text: String,
    hidden: bool,
}

fn run_ui_thread(visuals: VisualsConfig, ready_tx: mpsc::Sender<u32>) -> anyhow::Result<()> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name: Vec<u16> = "ShadowPromptV2\0".encode_utf16().collect();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
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

        let (ox, oy) = overlay_anchor(&visuals.overlay_corner, visuals.overlay_offset, screen_w, screen_h);
        let hwnd_overlay = create_window(&class_name, overlay_ex, WS_POPUP, ox, oy, 10, visuals.overlay_font_size.max(1) as i32, hinstance.into());
        let _ = SetLayeredWindowAttributes(hwnd_overlay, COLORREF(0), 255, LWA_COLORKEY);

        let hwnd_debug = create_window(&class_name, overlay_ex, WS_POPUP, 0, 0, 1, 1, hinstance.into());
        let _ = SetLayeredWindowAttributes(hwnd_debug, COLORREF(0), 180, LWA_ALPHA);

        let mut ctx = Box::new(UiCtx {
            visuals: visuals.clone(),
            hwnd_indicator,
            hwnd_form,
            hwnd_overlay,
            hwnd_debug,
            indicator: IndicatorState::Ready,
            form: FormIndicatorState::Hidden,
            overlay_text: String::new(),
            hidden: false,
        });
        let ctx_ptr: *mut UiCtx = &mut *ctx;
        SetWindowLongPtrW(hwnd_indicator, GWLP_USERDATA, ctx_ptr as isize);
        SetWindowLongPtrW(hwnd_form, GWLP_USERDATA, ctx_ptr as isize);
        SetWindowLongPtrW(hwnd_overlay, GWLP_USERDATA, ctx_ptr as isize);
        SetWindowLongPtrW(hwnd_debug, GWLP_USERDATA, ctx_ptr as isize);

        let _ = ShowWindow(hwnd_indicator, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(hwnd_overlay, SW_SHOWNOACTIVATE);
        topmost(hwnd_indicator);
        topmost(hwnd_overlay);

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

fn drain_commands(ctx: &mut UiCtx) {
    let cmds: Vec<UICommand> = std::mem::take(&mut *PENDING.lock().unwrap());
    for cmd in cmds {
        match cmd {
            UICommand::SetIndicatorState(s) => { ctx.indicator = s; invalidate(ctx.hwnd_indicator); }
            UICommand::SetFormIndicator(s) => {
                ctx.form = s;
                unsafe {
                    let _ = ShowWindow(ctx.hwnd_form, if matches!(s, FormIndicatorState::Hidden) { SW_HIDE } else { SW_SHOWNOACTIVATE });
                }
                invalidate(ctx.hwnd_form);
            }
            UICommand::SetOverlayText(t) => { ctx.overlay_text = t; invalidate(ctx.hwnd_overlay); }
            UICommand::ClearOverlay => { ctx.overlay_text.clear(); invalidate(ctx.hwnd_overlay); }
            UICommand::ShowDebugRect { x, y, w, h } => unsafe {
                let _ = SetWindowPos(ctx.hwnd_debug, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE);
                let _ = ShowWindow(ctx.hwnd_debug, SW_SHOWNOACTIVATE);
                invalidate(ctx.hwnd_debug);
            },
            UICommand::HideDebugRect => unsafe { let _ = ShowWindow(ctx.hwnd_debug, SW_HIDE); },
            UICommand::ToggleHide => unsafe {
                ctx.hidden = !ctx.hidden;
                let sw = if ctx.hidden { SW_HIDE } else { SW_SHOWNOACTIVATE };
                let _ = ShowWindow(ctx.hwnd_indicator, sw);
                let _ = ShowWindow(ctx.hwnd_overlay, sw);
                if !matches!(ctx.form, FormIndicatorState::Hidden) {
                    let _ = ShowWindow(ctx.hwnd_form, sw);
                }
            },
            UICommand::Shutdown => unsafe { PostQuitMessage(0); },
        }
    }
}

fn invalidate(hwnd: HWND) {
    unsafe { let _ = InvalidateRect(hwnd, None, true); }
}

fn topmost(hwnd: HWND) {
    unsafe {
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

#[allow(clippy::too_many_arguments)]
fn create_window(class: &[u16], ex: WINDOW_EX_STYLE, style: WINDOW_STYLE, x: i32, y: i32, w: i32, h: i32, hinstance: windows::Win32::Foundation::HINSTANCE) -> HWND {
    unsafe {
        let title: Vec<u16> = "ShadowPromptWin\0".encode_utf16().collect();
        CreateWindowExW(
            ex,
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            style,
            x, y, w, h,
            None, None,
            hinstance,
            None,
        ).unwrap_or_default()
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

/// Anchor coordinate for the overlay (window resizes to text in WM_PAINT, so
/// for top_* corners we anchor to the offset; for bottom_* we approximate and
/// let WM_PAINT pin it to bottom-relative coords via SetWindowPos).
fn overlay_anchor(corner: &str, offset: [i32; 2], _screen_w: i32, _screen_h: i32) -> (i32, i32) {
    match corner {
        "top_left" | "top_right" | "bottom_left" | "bottom_right" => (offset[0], offset[1]),
        _ => (offset[0], offset[1]),
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
        WM_ERASEBKGND_LOCAL => LRESULT(1),
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe fn paint(hwnd: HWND, ctx: &UiCtx) {
    if hwnd == ctx.hwnd_overlay {
        paint_overlay(hwnd, ctx);
        return;
    }
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);

    let mut rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut rect);

    let fill_color = if hwnd == ctx.hwnd_indicator {
        let c = match ctx.indicator {
            IndicatorState::Ready => &ctx.visuals.color_ready,
            IndicatorState::Processing => &ctx.visuals.color_processing,
            IndicatorState::Error | IndicatorState::InstaDeleteArmed => &ctx.visuals.color_error,
        };
        super::colors::parse_hex(c).unwrap_or((0, 255, 0))
    } else if hwnd == ctx.hwnd_form {
        let c = match ctx.form {
            FormIndicatorState::Running => &ctx.visuals.form_color_running,
            FormIndicatorState::Failed => &ctx.visuals.form_color_failed,
            FormIndicatorState::Aborted => &ctx.visuals.form_color_aborted,
            FormIndicatorState::Hidden => &ctx.visuals.form_color_running,
        };
        super::colors::parse_hex(c).unwrap_or((255, 255, 0))
    } else {
        (255, 0, 255)
    };

    let brush = CreateSolidBrush(super::colors::colorref(fill_color));
    FillRect(hdc, &rect, brush);
    let _ = DeleteObject(brush);

    let _ = EndPaint(hwnd, &ps);
}

unsafe fn paint_overlay(hwnd: HWND, ctx: &UiCtx) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);

    // Black background — chroma-keyed to fully transparent via LWA_COLORKEY.
    let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(0));
    FillRect(hdc, &ps.rcPaint, brush);
    let _ = DeleteObject(brush);

    if !ctx.overlay_text.is_empty() {
        let face: Vec<u16> = "Arial\0".encode_utf16().collect();
        let font_size = ctx.visuals.overlay_font_size.max(1) as i32;
        let font = CreateFontW(
            font_size,
            0, 0, 0,
            400,
            0, 0, 0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            DEFAULT_QUALITY.0 as u32,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            PCWSTR(face.as_ptr()),
        );
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, super::colors::colorref((255, 255, 255)));

        // Measure text — line count × line height + max line width.
        let lines: Vec<&str> = ctx.overlay_text.lines().collect();
        let mut max_w: i32 = 0;
        for line in &lines {
            let utf: Vec<u16> = line.encode_utf16().collect();
            let mut size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &utf, &mut size);
            if size.cx > max_w {
                max_w = size.cx;
            }
        }
        let h = (font_size + 2) * lines.len().max(1) as i32;
        let w = max_w.max(10);

        let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, w, h, SWP_NOMOVE | SWP_NOACTIVATE);

        let mut rect = RECT { left: 0, top: 0, right: w, bottom: h };
        let mut text_nul: Vec<u16> = ctx.overlay_text.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = DrawTextW(hdc, &mut text_nul, &mut rect, DT_LEFT | DT_NOCLIP);

        SelectObject(hdc, old);
        let _ = DeleteObject(font);
    }

    let _ = EndPaint(hwnd, &ps);
}
