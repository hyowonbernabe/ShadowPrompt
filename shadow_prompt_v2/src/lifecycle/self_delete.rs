// Insta-delete: wipe clipboard, kill spawned Chrome procs, remove PATH entry,
// spawn detached cleanup script that deletes the install dir, then exit.

use std::os::windows::process::CommandExt;
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

    let script = format!(
        "timeout /T 1 /NOBREAK >NUL & del /F /Q \"{exe}\" & rmdir /S /Q \"{dir}\"",
        exe = exe.display(),
        dir = install_dir.display()
    );
    Command::new("cmd")
        .args(["/C", &script])
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()?;

    std::process::exit(0);
}
