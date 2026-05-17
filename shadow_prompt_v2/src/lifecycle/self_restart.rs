// Self-restart: spawn detached hidden PowerShell that relaunches the exe after
// a brief delay, then exit cleanly. No console flash.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn execute() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let exe_str = normalize_path(&exe);

    let ps = format!(
        "Start-Sleep -Milliseconds 800; Start-Process -FilePath '{}'",
        exe_str.replace('\'', "''")
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
