// Load config.toml from exe-relative path. Validate model + key presence.

use super::schema::Config;
use super::paths::config_path;

pub fn load() -> anyhow::Result<Config> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path.display(), e))?;
    let cfg: Config = toml::from_str(&text)?;
    validate(&cfg)?;
    Ok(cfg)
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
