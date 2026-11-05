// GDI bitmap capture of a screen region, for Screenshot Query. `unsafe` confined to this file's
// Win32/GDI calls only, per project convention.

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject,
    EnumDisplayMonitors, GetDC, GetMonitorInfoW, ReleaseDC, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC, HMONITOR, MONITORINFO, SRCCOPY,
};

/// One monitor's capture. `left`/`top` are virtual-screen coordinates (can be negative for a
/// monitor left of / above the primary); physical pixels, since the process is per-monitor-DPI
/// aware (lib.rs).
pub struct MonitorShot {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub rgba: Vec<u8>,
}

/// Captures every attached monitor separately, in `EnumDisplayMonitors` order.
pub fn capture_monitors() -> anyhow::Result<Vec<MonitorShot>> {
    monitor_rects()
        .into_iter()
        .map(|r| {
            let (w, h) = (r.right - r.left, r.bottom - r.top);
            let rgba = unsafe { capture_region_raw(r.left, r.top, w, h)? };
            Ok(MonitorShot { left: r.left, top: r.top, width: w, height: h, rgba })
        })
        .collect()
}

fn monitor_rects() -> Vec<RECT> {
    unsafe extern "system" fn cb(hmon: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
        let out = &mut *(data.0 as *mut Vec<RECT>);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(hmon, &mut mi).as_bool() {
            out.push(mi.rcMonitor);
        }
        BOOL(1)
    }
    let mut rects: Vec<RECT> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(HDC::default(), None, Some(cb), LPARAM(&mut rects as *mut _ as isize));
    }
    rects
}

/// The given screen-space rectangle as raw RGBA pixels (no encoding). Used by dot tracking, which
/// grabs small areas many times a second.
pub fn capture_region_rgba(x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<Vec<u8>> {
    if w <= 0 || h <= 0 {
        anyhow::bail!("capture_region: non-positive dimensions {w}x{h}");
    }
    unsafe { capture_region_raw(x, y, w, h) }
}

/// Captures the given screen-space rectangle to a raw BGRA buffer, then encodes it to PNG bytes.
pub fn capture_region(x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<Vec<u8>> {
    if w <= 0 || h <= 0 {
        anyhow::bail!("capture_region: non-positive dimensions {w}x{h}");
    }
    let rgba = unsafe { capture_region_raw(x, y, w, h)? };
    crate::capture::image::encode_png(w as u32, h as u32, &rgba)
}

unsafe fn capture_region_raw(x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<Vec<u8>> {
    let screen_dc = GetDC(HWND::default());
    if screen_dc.is_invalid() {
        anyhow::bail!("GetDC failed");
    }
    let mem_dc = CreateCompatibleDC(screen_dc);
    let bitmap = CreateCompatibleBitmap(screen_dc, w, h);
    let old = SelectObject(mem_dc, bitmap);

    // Checked, not ignored: a failed copy (locked screen, UAC, out of memory) would otherwise
    // come back as an all-black image reported as success.
    let mut copied = !bitmap.is_invalid() && BitBlt(mem_dc, 0, 0, w, h, screen_dc, x, y, SRCCOPY).is_ok();

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h, // negative = top-down DIB, matches screen row order
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
            // Release everything acquired so far — screen_answer's tracking loop calls this
            // every 500ms, so a leak here would pile up handles fast.
            SelectObject(mem_dc, old);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_dc);
            ReleaseDC(HWND::default(), screen_dc);
            return Err(e.into());
        }
    };
    let dib_dc = CreateCompatibleDC(screen_dc);
    let old_dib = SelectObject(dib_dc, dib);
    copied &= BitBlt(dib_dc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY).is_ok();

    let buf_len = (w as usize) * (h as usize) * 4;
    let mut out = vec![0u8; buf_len];
    std::ptr::copy_nonoverlapping(bits_ptr as *const u8, out.as_mut_ptr(), buf_len);

    // BGRA -> RGBA (GDI DIBs are BGR order)
    for px in out.as_chunks_mut::<4>().0 {
        px.swap(0, 2);
    }

    SelectObject(dib_dc, old_dib);
    SelectObject(mem_dc, old);
    let _ = DeleteObject(dib);
    let _ = DeleteObject(bitmap);
    let _ = DeleteDC(dib_dc);
    let _ = DeleteDC(mem_dc);
    ReleaseDC(HWND::default(), screen_dc);

    let _ = &mut bmi; // silence unused-mut warning from the struct-update pattern above

    if !copied {
        anyhow::bail!("screen copy failed (locked screen, secure desktop or out of memory)");
    }
    Ok(out)
}
