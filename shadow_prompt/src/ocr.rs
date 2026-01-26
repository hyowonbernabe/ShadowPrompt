#![allow(unused)]
use windows::Media::Ocr::OcrEngine;
use windows::Graphics::Imaging::{BitmapDecoder, SoftwareBitmap};
use windows::Storage::Streams::{InMemoryRandomAccessStream, DataWriter};
use windows::Win32::Graphics::Gdi::{
    GetDC, GetDeviceCaps, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, 
    GetObjectW, DeleteObject, DeleteDC, ReleaseDC, BITMAP, SRCCOPY, HBITMAP, HDC,
    HORZRES, VERTRES, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, BI_RGB,
};
use windows::core::HSTRING;
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;
use windows::Win32::Foundation::HWND;
use windows::Foundation::IAsyncOperation;
use anyhow::{Result, Context};
use std::future::Future;

pub struct OcrManager;

impl OcrManager {
    pub async fn extract_from_screen(x: i32, y: i32, width: i32, height: i32) -> Result<String> {
        // Stub for compilation (Async WinRT issues)
        // let stream = capture_screen_area(x, y, width, height)?;
        // ... decoding ...
        
        Ok("MOCK OCR TEXT".to_string())
    }
}



fn capture_screen_area(x: i32, y: i32, width: i32, height: i32) -> Result<InMemoryRandomAccessStream> {
    unsafe {
        let hwnd_desktop = GetDesktopWindow();
        let hdc_screen = GetDC(hwnd_desktop);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        
        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        let h_old = SelectObject(hdc_mem, hbitmap);

        BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, x, y, SRCCOPY)
            .ok()
            .context("BitBlt failed")?;

        // Prepare info for GetDIBits
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down
                biPlanes: 1,
                biBitCount: 32, // Request 32-bit for easy alignment
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels = vec![0u8; (width * height * 4) as usize];
        
        GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI
        SelectObject(hdc_mem, h_old);
        DeleteObject(hbitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd_desktop, hdc_screen);

        // Create BMP in Memory
        let stream = InMemoryRandomAccessStream::new()?;
        let writer = DataWriter::CreateDataWriter(&stream)?;
        
        // Write BITMAPFILEHEADER (14 bytes)
        // Signature "BM"
        writer.WriteByte(0x42)?; writer.WriteByte(0x4D)?;
        // File Size (14 + 40 + pixels)
        let file_size = 14 + 40 + pixels.len() as u32;
        writer.WriteUInt32(file_size)?;
        // Reserved
        writer.WriteUInt16(0)?; writer.WriteUInt16(0)?;
        // Offset to data (14 + 40 = 54)
        writer.WriteUInt32(54)?;

        // Write BITMAPINFOHEADER (40 bytes) - matches bmiHeader
        let b = &bmi.bmiHeader;
        writer.WriteUInt32(b.biSize)?;
        writer.WriteInt32(b.biWidth)?;
        writer.WriteInt32(b.biHeight)?;
        writer.WriteUInt16(b.biPlanes)?;
        writer.WriteUInt16(b.biBitCount)?;
        writer.WriteUInt32(b.biCompression)?;
        writer.WriteUInt32(b.biSizeImage)?;
        writer.WriteInt32(b.biXPelsPerMeter)?;
        writer.WriteInt32(b.biYPelsPerMeter)?;
        writer.WriteUInt32(b.biClrUsed)?;
        writer.WriteUInt32(b.biClrImportant)?;

        // Write Pixels
        writer.WriteBytes(&pixels)?;

        // Flush and Detach
        // Flush and Detach (Async in Sync blocked - stubbed for now)
        // writer.StoreAsync()?.await?;
        // writer.DetachStream()?;
        
        stream.Seek(0)?;
        Ok(stream)
    }
}
