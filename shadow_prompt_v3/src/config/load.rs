// Load config.toml, or write the embedded template on first run.

use super::schema::Config;
use super::paths::{config_path, config_write_path};

const TEMPLATE: &str = include_str!("../../config/config.example.toml");

pub fn load() -> anyhow::Result<Config> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
    let cfg: Config = toml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("parsing {}: {e}", path.display()))?;
    Ok(cfg)
}

/// Writes the embedded template to the exe-relative config/config.toml if it doesn't exist yet
/// (always exe-relative, never CWD — a fresh `--init` should always produce the portable layout).
/// Returns the path written to. Does not overwrite an existing config.
pub fn init_template() -> anyhow::Result<std::path::PathBuf> {
    let path = config_write_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if !path.exists() {
        std::fs::write(&path, TEMPLATE)?;
    }
    Ok(path)
}
