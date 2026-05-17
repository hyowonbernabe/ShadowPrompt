// GDI screen capture → PNG bytes.

use image::{ImageBuffer, Rgba};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC,
    SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub fn capture_fullscreen() -> anyhow::Result<Vec<u8>> {
    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN);
        let h = GetSystemMetrics(SM_CYSCREEN);
        capture_region(0, 0, w, h)
    }
}

pub fn capture_region(x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<Vec<u8>> {
    if w <= 0 || h <= 0 {
        anyhow::bail!("invalid region {w}x{h}");
    }
    unsafe {
        let screen_dc: HDC = GetDC(HWND::default());
        if screen_dc.is_invalid() {
            anyhow::bail!("GetDC failed");
        }
        let mem_dc: HDC = CreateCompatibleDC(screen_dc);
        let bmp: HBITMAP = CreateCompatibleBitmap(screen_dc, w, h);
        if bmp.is_invalid() {
            ReleaseDC(HWND::default(), screen_dc);
            let _ = DeleteDC(mem_dc);
            anyhow::bail!("CreateCompatibleBitmap failed");
        }
        let prev = SelectObject(mem_dc, bmp);
        let _ = BitBlt(mem_dc, 0, 0, w, h, screen_dc, x, y, SRCCOPY);

        let mut bi = BITMAPINFO {
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
        let mut buf = vec![0u8; (w as usize) * (h as usize) * 4];
        let rows = GetDIBits(
            mem_dc,
            bmp,
            0,
            h as u32,
            Some(buf.as_mut_ptr() as *mut _),
            &mut bi,
            DIB_RGB_COLORS,
        );
        SelectObject(mem_dc, prev);
        let _ = DeleteObject(bmp);
        let _ = DeleteDC(mem_dc);
        ReleaseDC(HWND::default(), screen_dc);

        if rows == 0 {
            anyhow::bail!("GetDIBits returned 0 rows");
        }

        for chunk in buf.chunks_exact_mut(4) {
            chunk.swap(0, 2);
            chunk[3] = 255;
        }

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(w as u32, h as u32, buf)
            .ok_or_else(|| anyhow::anyhow!("buffer size mismatch"))?;

        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)?;
        Ok(out)
    }
}
