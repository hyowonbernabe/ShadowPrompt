// Access-code login: a code unlocks an OpenRouter API key stored in Supabase.
// Design: docs/superpowers/specs/2026-11-06-access-code-login-design.md. Server side:
// supabase/migrations/20261106000000_access_codes.sql (four RPCs, nothing else reachable).

use std::time::Duration;

use serde_json::{json, Value};

/// Project `shadowprompt`. The publishable key is public by design (it can only call the four
/// RPCs); no secret key ever ships.
pub const SUPABASE_URL: &str = "https://ihvnbvxzuroogbcrzort.supabase.co";
pub const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_ULGuLKiHYumaEKGm6tY2Qw_DvYEBDIZ";

const TIMEOUT: Duration = Duration::from_secs(15);

/// A config key that is really just the template placeholder counts as "no key".
pub fn is_real_key(key: &str) -> bool {
    let k = key.trim();
    !k.is_empty() && !k.contains("REPLACE")
}

/// `••••644`-style hint showing only the last 3 characters of a code.
pub fn masked(code: &str) -> String {
    let tail: String = code.chars().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect();
    format!("••••{tail}")
}

fn client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder().timeout(TIMEOUT).build()?)
}

async fn rpc(name: &str, body: Value) -> anyhow::Result<Value> {
    let resp = client()?
        .post(format!("{SUPABASE_URL}/rest/v1/rpc/{name}"))
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("can't reach the login server ({e})"))?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("login server error (HTTP {})", status.as_u16());
    }
    Ok(resp.json().await?)
}

fn as_status(v: Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

/// The key stored under `code`, or None if there's no such code.
pub async fn login(code: &str) -> anyhow::Result<Option<String>> {
    Ok(rpc("sp_login", json!({ "p_code": code })).await?.as_str().map(str::to_string))
}

/// "ok" | "taken" | "invalid"
pub async fn signup(code: &str, api_key: &str) -> anyhow::Result<String> {
    Ok(as_status(rpc("sp_signup", json!({ "p_code": code, "p_api_key": api_key })).await?))
}

/// Change the code and/or key (None = keep). "ok" | "not_found" | "taken" | "invalid"
pub async fn update(code: &str, new_code: Option<&str>, new_key: Option<&str>) -> anyhow::Result<String> {
    Ok(as_status(rpc("sp_update", json!({ "p_code": code, "p_new_code": new_code, "p_new_api_key": new_key })).await?))
}

/// "ok" | "not_found"
pub async fn delete(code: &str) -> anyhow::Result<String> {
    Ok(as_status(rpc("sp_delete", json!({ "p_code": code })).await?))
}

/// Asks OpenRouter whether `key` is a working key (GET /api/v1/key: 200 = valid, 401 = not).
pub async fn openrouter_key_works(key: &str) -> anyhow::Result<bool> {
    let resp = client()?
        .get("https://openrouter.ai/api/v1/key")
        .bearer_auth(key.trim())
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("can't reach OpenRouter ({e})"))?;
    match resp.status().as_u16() {
        200 => Ok(true),
        401 | 403 => Ok(false),
        s => anyhow::bail!("OpenRouter error (HTTP {s})"),
    }
}

/// Runs one of the async calls above from a plain thread (the dialogs have no runtime).
pub fn block_on<T>(fut: impl std::future::Future<Output = anyhow::Result<T>>) -> anyhow::Result<T> {
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(fut)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_and_blank_keys_are_not_real() {
        assert!(!is_real_key(""));
        assert!(!is_real_key("  "));
        assert!(!is_real_key("sk-or-REPLACE-ME"));
        assert!(is_real_key("sk-or-v1-abc"));
    }

    #[test]
    fn masked_shows_only_last_three() {
        assert_eq!(masked("2244644"), "••••644");
        assert_eq!(masked("ab"), "••••ab");
        assert_eq!(masked("héllo"), "••••llo");
    }
}
