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

/// Saves a login into `config.toml`'s `[account]` section (replacing any earlier one). Only that
/// section is touched — comments, other sections and `openrouter.api_key` stay exactly as they
/// are.
pub fn save_account(code: &str, api_key: &str) -> anyhow::Result<()> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path)?;
    std::fs::write(&path, with_account_section(&text, Some((code, api_key))))?;
    Ok(())
}

/// Removes the `[account]` section (log out). The rest of the file is untouched.
pub fn clear_account() -> anyhow::Result<()> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path)?;
    std::fs::write(&path, with_account_section(&text, None))?;
    Ok(())
}

/// `text` with its `[account]` table removed and, if given, a fresh one appended at the end.
fn with_account_section(text: &str, account: Option<(&str, &str)>) -> String {
    let mut out = Vec::new();
    let mut in_account = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_account = t.trim_start_matches('[').trim_end_matches(']').trim() == "account" && !t.starts_with("[[");
        }
        if !in_account {
            out.push(line);
        }
    }
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    let mut s = out.join("\n");
    s.push('\n');
    if let Some((code, key)) = account {
        let q = |v: &str| toml::Value::String(v.to_string()).to_string();
        s.push_str(&format!(
            "\n[account]\n# Saved login (written by the app; log out from the Account screen to remove).\ncode = {}\napi_key = {}\n",
            q(code),
            q(key)
        ));
    }
    s
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_section_is_added_replaced_and_removed_without_touching_the_rest() {
        let base = "# my notes\n[openrouter]\napi_key = \"sk-mine\"\n\n[visuals]\nx = 1\n";
        let once = with_account_section(base, Some(("2244644", "sk-or-a")));
        assert!(once.starts_with(base));
        let cfg: toml::Value = toml::from_str(&once).unwrap();
        assert_eq!(cfg["account"]["code"].as_str(), Some("2244644"));
        assert_eq!(cfg["openrouter"]["api_key"].as_str(), Some("sk-mine"));

        let twice = with_account_section(&once, Some(("new\"code", "sk-or-b")));
        assert_eq!(twice.matches("[account]").count(), 1);
        let cfg: toml::Value = toml::from_str(&twice).unwrap();
        assert_eq!(cfg["account"]["code"].as_str(), Some("new\"code"));
        assert_eq!(cfg["account"]["api_key"].as_str(), Some("sk-or-b"));

        assert_eq!(with_account_section(&twice, None), base);
    }

    #[test]
    fn account_section_in_the_middle_is_removed_cleanly() {
        let text = "[openrouter]\napi_key = \"k\"\n[account]\ncode = \"c\"\napi_key = \"x\"\n[visuals]\ny = 2\n";
        assert_eq!(with_account_section(text, None), "[openrouter]\napi_key = \"k\"\n[visuals]\ny = 2\n");
    }

    #[test]
    fn the_shipped_template_still_parses() {
        toml::from_str::<Config>(TEMPLATE).unwrap();
    }
}
