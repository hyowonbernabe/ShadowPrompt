// Load config.toml from exe-relative path. Validate model + key presence.

use super::schema::Config;
use super::paths::config_path;
use std::path::{Path, PathBuf};

const TEMPLATE: &str = include_str!("../../config/config.example.toml");

pub fn load() -> anyhow::Result<Config> {
    load_from(&config_path()?)
}

pub fn load_from(path: &Path) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path.display(), e))?;
    let cfg: Config = toml::from_str(&text)?;
    validate(&cfg)?;
    Ok(cfg)
}

/// Write the embedded template to the canonical config path if it does not exist.
/// Returns the path. Idempotent.
pub fn init_template() -> anyhow::Result<PathBuf> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        std::fs::write(&path, TEMPLATE)?;
    }
    Ok(path)
}

fn validate(cfg: &Config) -> anyhow::Result<()> {
    if cfg.openrouter.api_key.trim().is_empty() {
        anyhow::bail!("openrouter.api_key is empty — edit config.toml");
    }
    if cfg.openrouter.model_id.trim().is_empty() {
        anyhow::bail!("openrouter.model_id is empty — edit config.toml");
    }
    Ok(())
}
