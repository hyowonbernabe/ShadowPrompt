// Insta-delete: wipe clipboard, remove PATH entry, spawn detached hidden PowerShell that deletes
// install dir + exe after this process exits. Ported as-is from v2. No watchdog/dead-man's-switch
// involvement — design doc §11: that's a separate, deferred feature, this is the direct
// double-tap-triggered path only.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use crate::capture::clipboard;
use crate::config::paths::exe_dir;

use super::path_cleanup;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn execute() -> anyhow::Result<()> {
    let _ = clipboard::clear();
    // Chrome cleanup is a deliberate soft no-op, same reasoning v2 documented: we spawn Chrome
    // detached, so we don't hold a child-process handle to kill directly. Its CDP session
    // terminates when the WS connection closes (which happens when we exit); the temp profile
    // dir is removed by the rmdir /S below regardless.

    let install_dir = exe_dir()?;
    let exe = std::env::current_exe()?;
    let _ = path_cleanup::remove_from_user_path(&install_dir);

    let exe_s = normalize_path(&exe).replace('\'', "''");
    let dir_s = normalize_path(&install_dir).replace('\'', "''");

    let ps = format!(
        "Start-Sleep -Milliseconds 800; \
         Remove-Item -Force -ErrorAction SilentlyContinue '{exe_s}'; \
         Remove-Item -Recurse -Force -ErrorAction SilentlyContinue '{dir_s}'"
    );

    Command::new("powershell.exe")
        .args(["-WindowStyle", "Hidden", "-NoProfile", "-NonInteractive", "-Command", &ps])
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()?;

    std::process::exit(0);
}

fn normalize_path(p: &Path) -> String {
    let s = p.to_string_lossy().into_owned();
    s.strip_prefix(r"\\?\").map(String::from).unwrap_or(s)
}
