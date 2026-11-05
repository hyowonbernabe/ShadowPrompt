// Access-code login: a code unlocks an OpenRouter API key stored in Supabase.
// Design: docs/superpowers/specs/2026-11-06-access-code-login-design.md. Server side:
// supabase/migrations/20261106000000_access_codes.sql (four RPCs, nothing else reachable).

use std::time::Duration;

use serde_json::{json, Value};

/// Project `shadowprompt`. The publishable key is public by design (it can only call the four
/// RPCs); no secret key ever ships.
pub const SUPABASE_URL: &str = "https://ihvnbvxzuroogbcrzort.supabase.co";
pub const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_ULGuLKiHYumaEKGm6tY2Qw_DvYEBDIZ";

const TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
/// Network failures (no response at all) are retried after 1s, 2s, 4s — this runs on flaky
/// connections. A real answer from the server is never retried.
const ATTEMPTS: u32 = 4;

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
    Ok(reqwest::Client::builder().connect_timeout(CONNECT_TIMEOUT).timeout(TIMEOUT).build()?)
}

/// The RPC's result, and whether it took more than one try — after a retry the first attempt
/// may already have reached the server (only its reply got lost), which callers check for.
async fn rpc(name: &str, body: Value) -> anyhow::Result<(Value, bool)> {
    let http = client()?;
    let mut last = None;
    for attempt in 0..ATTEMPTS {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
        }
        let sent = http
            .post(format!("{SUPABASE_URL}/rest/v1/rpc/{name}"))
            .header("apikey", SUPABASE_PUBLISHABLE_KEY)
            .json(&body)
            .send()
            .await;
        let resp = match sent {
            Ok(r) => r,
            Err(e) => {
                last = Some(e);
                continue;
            }
        };
        let status = resp.status();
        if status.is_server_error() {
            last = None;
            continue; // 5xx: the server hiccuped; same as no answer
        }
        if !status.is_success() {
            anyhow::bail!("login server error (HTTP {})", status.as_u16());
        }
        match resp.json().await {
            Ok(v) => return Ok((v, attempt > 0)),
            Err(e) => last = Some(e), // reply cut off mid-way
        }
    }
    match last {
        Some(e) => anyhow::bail!("can't reach the login server; check the internet connection ({e})"),
        None => anyhow::bail!("the login server isn't answering right now; try again in a moment"),
    }
}

fn as_status(v: Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

/// The key stored under `code`, or None if there's no such code.
pub async fn login(code: &str) -> anyhow::Result<Option<String>> {
    Ok(rpc("sp_login", json!({ "p_code": code })).await?.0.as_str().map(str::to_string))
}

/// "ok" | "taken" | "invalid"
pub async fn signup(code: &str, api_key: &str) -> anyhow::Result<String> {
    let (v, retried) = rpc("sp_signup", json!({ "p_code": code, "p_api_key": api_key })).await?;
    let status = as_status(v);
    // A retry answered "taken": maybe our own first attempt created it. Same key = it did.
    if retried && status == "taken" && login(code).await?.as_deref() == Some(api_key) {
        return Ok("ok".into());
    }
    Ok(status)
}

/// Change the code and/or key (None = keep). "ok" | "not_found" | "taken" | "invalid"
pub async fn update(code: &str, new_code: Option<&str>, new_key: Option<&str>) -> anyhow::Result<String> {
    let (v, retried) = rpc("sp_update", json!({ "p_code": code, "p_new_code": new_code, "p_new_api_key": new_key })).await?;
    let status = as_status(v);
    // A retry answered "not_found": maybe our first attempt already moved the account to the new
    // code (or changed the key). Check the account where it should now be.
    if retried && status == "not_found" {
        let at = new_code.filter(|c| !c.is_empty()).unwrap_or(code);
        if let Some(key) = login(at).await? {
            if new_key.is_none_or(|k| k.is_empty() || k == key) {
                return Ok("ok".into());
            }
        }
    }
    Ok(status)
}

/// "ok" | "not_found"
pub async fn delete(code: &str) -> anyhow::Result<String> {
    let (v, retried) = rpc("sp_delete", json!({ "p_code": code })).await?;
    let status = as_status(v);
    // A retry answered "not_found": the first attempt already deleted it.
    Ok(if retried && status == "not_found" { "ok".into() } else { status })
}

/// Asks OpenRouter whether `key` is a working key (GET /api/v1/key: 200 = valid, 401 = not).
pub async fn openrouter_key_works(key: &str) -> anyhow::Result<bool> {
    let http = client()?;
    let mut last = String::new();
    for attempt in 0..ATTEMPTS {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
        }
        match http.get("https://openrouter.ai/api/v1/key").bearer_auth(key.trim()).send().await {
            Ok(resp) => match resp.status().as_u16() {
                200 => return Ok(true),
                401 | 403 => return Ok(false),
                s if s >= 500 => last = format!("HTTP {s}"),
                s => anyhow::bail!("OpenRouter error (HTTP {s})"),
            },
            Err(e) => last = e.to_string(),
        }
    }
    anyhow::bail!("can't reach OpenRouter to check the key; check the internet connection ({last})")
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
