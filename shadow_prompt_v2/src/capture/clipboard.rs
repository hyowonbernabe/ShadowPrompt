// arboard wrapper. 3-attempt retry on transient Win32 failures.

use arboard::Clipboard;
use std::thread::sleep;
use std::time::Duration;

const RETRIES: u32 = 3;
const BACKOFF_MS: u64 = 50;

fn with_retry<F, T>(mut op: F) -> anyhow::Result<T>
where
    F: FnMut() -> Result<T, arboard::Error>,
{
    let mut last: Option<arboard::Error> = None;
    for i in 0..RETRIES {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) => {
                last = Some(e);
                sleep(Duration::from_millis(BACKOFF_MS * (1 << i)));
            }
        }
    }
    Err(anyhow::anyhow!("clipboard error: {:?}", last))
}

pub fn read_text() -> anyhow::Result<String> {
    with_retry(|| {
        let mut cb = Clipboard::new()?;
        cb.get_text()
    })
}

pub fn write_text(text: &str) -> anyhow::Result<()> {
    with_retry(|| {
        let mut cb = Clipboard::new()?;
        cb.set_text(text.to_string())
    })
}

pub fn clear() -> anyhow::Result<()> {
    with_retry(|| {
        let mut cb = Clipboard::new()?;
        cb.clear()
    })
}
