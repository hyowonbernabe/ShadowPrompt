// Indicator pixel window (Win32). Topmost, no taskbar entry.

pub fn create() -> anyhow::Result<()> {
    // TODO: RegisterClassEx + CreateWindowEx with WS_EX_TOOLWINDOW | WS_EX_TOPMOST.
    Ok(())
}
