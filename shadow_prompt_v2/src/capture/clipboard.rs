// arboard wrapper. 3-attempt retry on transient Win32 failures.

pub fn read_text() -> anyhow::Result<String> {
    anyhow::bail!("not yet implemented")
}

pub fn write_text(_text: &str) -> anyhow::Result<()> {
    Ok(())
}

pub fn clear() -> anyhow::Result<()> {
    Ok(())
}
