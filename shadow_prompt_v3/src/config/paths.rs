// Path resolution — exe-relative first, falling back to the current working directory. This is
// the USB-portability mechanism (exe-relative: the whole install directory travels together,
// wherever it's run from) *and* the dev-workflow mechanism (CWD fallback: `cargo run` puts the
// built exe under `target/debug/`, nowhere near the crate root where `config/`/`knowledge/`
// actually live — without this fallback, every `cargo run` would need its own copy of those
// dirs shadowed into `target/debug/`, matching a real gap found by actually running `--probe`
// during development, not by any test).

use std::path::PathBuf;

pub fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("exe has no parent directory"))
}

/// Resolves a `dir_name` subdirectory (or the CWD itself, if `dir_name` is empty) preferring the
/// exe-relative location, falling back to CWD if the exe-relative one doesn't exist yet. For a
/// path we're about to *write* (like `config.toml` via `--init`), exe-relative is always used —
/// see `config_path`'s doc comment.
fn resolve_dir(dir_name: &str) -> PathBuf {
    if let Ok(dir) = exe_dir() {
        let candidate = dir.join(dir_name);
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from(dir_name)
}

/// The config file to *read*. Exe-relative if `config/` already exists there (the real
/// portable/installed case); otherwise the CWD's `config/` (a `cargo run` from the crate root).
/// `init_template` (in `load.rs`) always writes to the exe-relative path specifically — a fresh
/// `--init` should always produce the portable layout, never silently write into CWD instead.
pub fn config_path() -> anyhow::Result<PathBuf> {
    Ok(resolve_dir("config").join("config.toml"))
}

/// Always the exe-relative `config/config.toml` path, regardless of whether it exists yet —
/// used only by `--init`/first-run template writing, never by anything that just wants to read
/// an already-existing config.
pub fn config_write_path() -> anyhow::Result<PathBuf> {
    Ok(exe_dir()?.join("config").join("config.toml"))
}

pub fn knowledge_dir() -> anyhow::Result<PathBuf> {
    Ok(resolve_dir("knowledge"))
}

pub fn logs_dir() -> anyhow::Result<PathBuf> {
    Ok(resolve_dir("data").join("logs"))
}
