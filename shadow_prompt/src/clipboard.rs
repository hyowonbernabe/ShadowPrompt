use arboard::Clipboard;
use anyhow::Result;
use std::thread;
use std::time::Duration;

pub struct ClipboardManager {
    // arboard::Clipboard is not thread safe by default in all OS, but we can wrap it
    // Actually, creating a new instance per operation is often safer/easier for simple apps
    // but arboard recommends keeping the instance.
    // We'll use a Mutex to share it or lazy init.
}

impl ClipboardManager {
    pub fn read() -> Result<String> {
        // Retry logic for clipboard contention
        for _ in 0..3 {
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    return Ok(text);
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        anyhow::bail!("Failed to read clipboard")
    }

    pub fn write(text: &str) -> Result<()> {
        for _ in 0..3 {
            if let Ok(mut clipboard) = Clipboard::new() {
                if clipboard.set_text(text).is_ok() {
                    return Ok(());
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        anyhow::bail!("Failed to write to clipboard")
    }

    pub fn clear() -> Result<()> {
        // Clearing is writing a space or empty string
        // Windows doesn't like empty clipboard sometimes, but we can try empty string
        Self::write(" ") 
    }
}
