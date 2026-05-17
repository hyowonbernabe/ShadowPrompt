// Insta-delete: wipe clipboard, remove PATH entry, spawn detached hidden
// PowerShell that deletes install dir + exe, then exit. No console flash.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use crate::browser::PROFILE_MARKER;
use crate::capture::clipboard;
use crate::config::paths::exe_dir;

use super::chrome_cleanup;
use super::path_cleanup;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn execute() -> anyhow::Result<()> {
    let _ = clipboard::clear();
    let _ = chrome_cleanup::kill_spawned_browsers(PROFILE_MARKER);

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
