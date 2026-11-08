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
    write_atomic(&path, &with_account_section(&text, Some((code, api_key))))
}

/// Removes the `[account]` section (log out). The rest of the file is untouched.
pub fn clear_account() -> anyhow::Result<()> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path)?;
    write_atomic(&path, &with_account_section(&text, None))
}

/// Write to a temp file next to it, then swap it in: a USB stick pulled mid-write leaves the old
/// config intact instead of an empty or half-written one (which would stop the app starting).
pub(crate) fn write_atomic(path: &std::path::Path, text: &str) -> anyhow::Result<()> {
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?; // replaces the existing file on Windows
    Ok(())
}

/// The table name of a `[header]` / `[[header]]` / `["header"]` line (trailing comment ignored).
fn table_name(line: &str) -> Option<String> {
    let inner = line.trim().strip_prefix('[')?.trim_start_matches('[');
    let end = inner.find(']')?;
    Some(inner[..end].trim().trim_matches('"').trim_matches('\'').trim().to_string())
}

/// `text` with its `[account]` table(s) removed and, if given, a fresh one appended at the end.
/// Keeps the file's own line endings.
// ponytail: line-based, not a TOML editor — an `account.code = ...` dotted key written at the
// top level by hand isn't recognised; switch to toml_edit if anyone ever writes it that way.
fn with_account_section(text: &str, account: Option<(&str, &str)>) -> String {
    let nl = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = Vec::new();
    let mut in_account = false;
    for line in text.lines() {
        if let Some(name) = table_name(line) {
            in_account = name == "account" || name.starts_with("account.");
        }
        if !in_account {
            out.push(line);
        }
    }
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    let mut s = out.join(nl);
    s.push_str(nl);
    if let Some((code, key)) = account {
        let q = |v: &str| toml::Value::String(v.to_string()).to_string();
        let block = [
            "",
            "[account]",
            "# Saved login (written by the app; log out from the Account screen to remove).",
            &format!("code = {}", q(code)),
            &format!("api_key = {}", q(key)),
            "",
        ];
        s.push_str(&block.join(nl));
    }
    s
}

/// Applies a cloud config TOML over the local config.toml. Keeps the local `[account]` section
/// intact. Returns a list of human-readable changes (e.g. "3 hotkeys changed").
pub fn apply_cloud_config(cloud_toml: &str) -> anyhow::Result<Vec<String>> {
    use super::schema::Config;

    let cloud: Config = toml::from_str(cloud_toml)
        .map_err(|e| anyhow::anyhow!("cloud config parse error: {e}"))?;
    let local = load()?;
    let mut changes = Vec::new();

    // Count hotkey differences.
    let hotkey_diffs = count_hotkey_diffs(&local.hotkeys, &cloud.hotkeys);
    if hotkey_diffs > 0 {
        changes.push(format!("{hotkey_diffs} hotkey{}", if hotkey_diffs == 1 { "" } else { "s" }));
    }

    // Build the merged config: cloud values for everything, except what belongs to this PC:
    // the login ([account]) and this install's own OpenRouter key (never uploaded).
    let mut merged = cloud;
    merged.account = local.account.clone();
    merged.openrouter.api_key = local.openrouter.api_key.clone();

    let header = "# ShadowPrompt config — synced from cloud.\n\
                  # See config.example.toml for documentation on each field.\n\n";
    let body = toml::to_string_pretty(&merged)?;
    let path = config_path()?;
    write_atomic(&path, &format!("{header}{body}"))
        .map_err(|e| anyhow::anyhow!("writing config: {e}"))?;
    Ok(changes)
}

fn count_hotkey_diffs(a: &super::schema::HotkeysConfig, b: &super::schema::HotkeysConfig) -> usize {
    let mut n = 0;
    macro_rules! cmp { ($($f:ident),*) => { $( if a.$f != b.$f { n += 1; } )* } }
    cmp!(clipboard_query, screenshot_query, test_model, switch_model, add_context,
         reset_conversation, launch_debugger, forms_answer_page, forms_answer_all,
         forms_answer_page_axtree, forms_answer_all_axtree, abort, hide_toggle,
         help_toggle, restart_daemon, insta_delete, panic_kill, debug_open_tab,
         screen_answer, clear_dots, account, seb_unpatch, quit);
    n
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
    fn hand_written_headers_and_crlf_are_handled() {
        for header in ["[account]  # my login", "[\"account\"]", "[ account ]", "[account.extra]"] {
            let text = format!("[openrouter]\r\napi_key = \"k\"\r\n{header}\r\ncode = \"old\"\r\n");
            let out = with_account_section(&text, Some(("new", "sk-or-n")));
            let cfg: toml::Value = toml::from_str(&out).unwrap_or_else(|e| panic!("{header}: {e}\n{out}"));
            assert_eq!(cfg["account"]["code"].as_str(), Some("new"), "{header}");
            assert!(out.contains("\r\n[account]\r\n") && !out.contains("old"), "{header}: {out:?}");
        }
    }

    #[test]
    fn the_shipped_template_still_parses() {
        toml::from_str::<Config>(TEMPLATE).unwrap();
    }
}
