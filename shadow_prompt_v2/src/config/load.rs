// Load config.toml from exe-relative path. Validate model + key presence.

use super::paths::config_path;
use super::schema::Config;
use std::path::{Path, PathBuf};

const TEMPLATE: &str = include_str!("../../config/config.example.toml");

/// Optional trial key baked in at compile time via the SHADOWPROMPT_TRIAL_KEY env
/// variable. Release CI sets it; local dev builds leave it empty so the user
/// must paste their own key.
const TRIAL_KEY: Option<&str> = option_env!("SHADOWPROMPT_TRIAL_KEY");

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
/// Substitutes the bundled trial key (if any) into the empty `api_key = ""` slot.
/// Idempotent.
pub fn init_template() -> anyhow::Result<PathBuf> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        let text = with_bundled_key(TEMPLATE);
        std::fs::write(&path, text)?;
    }
    Ok(path)
}

fn with_bundled_key(template: &str) -> String {
    match TRIAL_KEY {
        Some(k) if !k.trim().is_empty() => template.replace(
            r#"api_key = """#,
            &format!(r#"api_key = "{}""#, k.trim()),
        ),
        _ => template.to_string(),
    }
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
