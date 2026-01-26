#![allow(unused)]
use windows::Media::Ocr::OcrEngine;
use windows::Graphics::Imaging::{SoftwareBitmap, BitmapPixelFormat, BitmapAlphaMode};
use windows::Storage::Streams::DataWriter;
use windows::Win32::Graphics::Gdi::{
    GetDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, 
    DeleteObject, DeleteDC, ReleaseDC, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, 
    DIB_RGB_COLORS, BI_RGB, GetDIBits,
};
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;
use windows::Win32::Foundation::HWND;
use windows::Foundation::AsyncStatus;
use anyhow::{Result, Context};

// Blocking helper removed. We rely on async/await support in windows-rs.

pub struct OcrManager;

impl OcrManager {
    pub async fn extract_from_screen(x: i32, y: i32, width: i32, height: i32) -> Result<String> {
        // 1. Capture Pixels
        let pixels = capture_pixels(x, y, width, height)?;
        
        // 2. Create IBuffer via DataWriter
        let writer = DataWriter::new()?;
        writer.WriteBytes(&pixels)?;
        let buffer = writer.DetachBuffer()?;

        // 3. Create SoftwareBitmap
        let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
            &buffer, 
            BitmapPixelFormat::Bgra8, 
            width, 
            height
        )?;

        // 4. Init Engine
        let engine = OcrEngine::TryCreateFromUserProfileLanguages().unwrap_or_else(|_| {
             panic!("Failed to create OCR engine from profile languages.");
        });

        // 5. Recognize
        let operation = engine.RecognizeAsync(&bitmap)?;
        
        // Manual blocking wait
        while operation.Status()? == AsyncStatus::Started {
             std::thread::yield_now();
        }
        
        let result = operation.GetResults()?;
        let text = result.Text()?.to_string();

        Ok(text)
    }
}

fn capture_pixels(x: i32, y: i32, width: i32, height: i32) -> Result<Vec<u8>> {
    unsafe {
        let hwnd_desktop = GetDesktopWindow();
        let hdc_screen = GetDC(hwnd_desktop);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        
        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        let h_old = SelectObject(hdc_mem, hbitmap);

        if BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, x, y, SRCCOPY).is_err() {
            SelectObject(hdc_mem, h_old); DeleteObject(hbitmap); DeleteDC(hdc_mem); ReleaseDC(hwnd_desktop, hdc_screen);
            anyhow::bail!("BitBlt failed");
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down
                biPlanes: 1,
                biBitCount: 32, 
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels = vec![0u8; (width * height * 4) as usize];
        GetDIBits(hdc_mem, hbitmap, 0, height as u32, Some(pixels.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);

        SelectObject(hdc_mem, h_old); DeleteObject(hbitmap); DeleteDC(hdc_mem); ReleaseDC(hwnd_desktop, hdc_screen);
        
        Ok(pixels)
    }
}
