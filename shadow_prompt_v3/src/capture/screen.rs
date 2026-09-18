// GDI bitmap capture of a screen region, for Screenshot Query. `unsafe` confined to this file's
// Win32/GDI calls only, per project convention.

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject,
    GetDC, ReleaseDC, SelectObject, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};

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

    let _ = BitBlt(mem_dc, 0, 0, w, h, screen_dc, x, y, SRCCOPY);

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
    let dib = CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits_ptr, None, 0)?;
    let dib_dc = CreateCompatibleDC(screen_dc);
    let old_dib = SelectObject(dib_dc, dib);
    let _ = BitBlt(dib_dc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY);

    let buf_len = (w as usize) * (h as usize) * 4;
    let mut out = vec![0u8; buf_len];
    std::ptr::copy_nonoverlapping(bits_ptr as *const u8, out.as_mut_ptr(), buf_len);

    // BGRA -> RGBA (GDI DIBs are BGR order)
    for px in out.chunks_exact_mut(4) {
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

    Ok(out)
}
