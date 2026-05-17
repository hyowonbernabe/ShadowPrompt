// Self-restart: spawn detached cmd that relaunches the exe after a brief delay,
// then exit cleanly.

use std::os::windows::process::CommandExt;
use std::process::Command;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn execute() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let exe_str = exe.display().to_string();

    Command::new("cmd")
        .args(["/C", &format!("timeout /T 1 /NOBREAK >NUL & start \"\" \"{exe_str}\"")])
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()?;

    std::process::exit(0);
}
