// arboard wrapper, text only — design doc §3: an image-on-clipboard check was proposed and
// explicitly declined, clipboard stays text-only, use Screenshot Query for anything visual.

use arboard::Clipboard;

const RETRIES: u32 = 3;

pub fn read_text() -> anyhow::Result<String> {
    with_retries(|cb| cb.get_text().map_err(anyhow::Error::from))
}

pub fn write_text(text: &str) -> anyhow::Result<()> {
    with_retries(|cb| cb.set_text(text.to_string()).map_err(anyhow::Error::from))
}

pub fn clear() -> anyhow::Result<()> {
    write_text("")
}

fn with_retries<T>(mut op: impl FnMut(&mut Clipboard) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let mut last_err: Option<anyhow::Error> = None;
    for _ in 0..RETRIES {
        let attempt = Clipboard::new().map_err(anyhow::Error::from).and_then(|mut cb| op(&mut cb));
        match attempt {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = Some(e);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("clipboard access failed after {RETRIES} retries")))
}
