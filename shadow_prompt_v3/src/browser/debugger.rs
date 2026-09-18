// Launch Chrome/Edge with --remote-debugging-port. Detached, visible incognito window — design
// doc §7.3: the user manually logs in and navigates, ShadowPrompt attaches to that same session.

use std::path::PathBuf;
use std::process::Command;

use super::PROFILE_MARKER;

pub const DEBUG_PORT: u16 = 9222;

pub fn launch_incognito() -> anyhow::Result<()> {
    let exe = locate_chrome()?;
    let profile = std::env::temp_dir().join(PROFILE_MARKER);
    let _ = std::fs::create_dir_all(&profile);

    Command::new(exe)
        .args([
            "--incognito",
            &format!("--remote-debugging-port={}", DEBUG_PORT),
            &format!("--user-data-dir={}", profile.display()),
            "--no-first-run",
            "--no-default-browser-check",
        ])
        .spawn()?;
    Ok(())
}

fn locate_chrome() -> anyhow::Result<PathBuf> {
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Ok(p);
        }
    }
    anyhow::bail!("Chrome/Edge not found in standard install locations")
}

/// Connects chromiumoxide to the debug port opened by `launch_incognito`.
pub async fn connect() -> anyhow::Result<chromiumoxide::Browser> {
    let (browser, mut handler) = chromiumoxide::Browser::connect(format!("http://localhost:{DEBUG_PORT}"))
        .await
        .map_err(|e| anyhow::anyhow!("Chrome debugger not reachable on :{DEBUG_PORT}: {e}"))?;
    // chromiumoxide requires the handler event-loop to be polled continuously — spawn it
    // detached, matching chromiumoxide's documented usage pattern.
    tokio::spawn(async move {
        while let Some(res) = futures::StreamExt::next(&mut handler).await {
            if let Err(e) = res {
                log::warn!("chromiumoxide handler: {e}");
            }
        }
    });
    Ok(browser)
}
