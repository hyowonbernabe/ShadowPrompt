// Exe-relative path resolution. All on-disk state lives next to the executable.

use std::path::PathBuf;

pub fn exe_dir() -> anyhow::Result<PathBuf> {
    let path = std::env::current_exe()?;
    path.parent()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("executable has no parent directory"))
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    Ok(exe_dir()?.join("config").join("config.toml"))
}

pub fn log_path() -> anyhow::Result<PathBuf> {
    Ok(exe_dir()?.join("data").join("logs").join("app.log"))
}
