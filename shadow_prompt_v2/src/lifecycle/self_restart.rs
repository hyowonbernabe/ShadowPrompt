// Self-restart: spawn detached cmd.exe to relaunch the exe after a short delay,
// then exit cleanly.

pub fn execute() -> anyhow::Result<()> {
    // TODO: build cmd line, CreateProcess with DETACHED_PROCESS | CREATE_NO_WINDOW,
    // then std::process::exit(0).
    anyhow::bail!("not yet implemented")
}
