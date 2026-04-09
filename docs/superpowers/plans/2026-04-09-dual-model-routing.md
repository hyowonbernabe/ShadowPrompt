# Dual-Model Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow a different LLM model (and provider) to be used for Google Forms automation vs. OCR/clipboard queries, with both fully configurable from `config.toml` and the setup wizard.

**Architecture:** Add `browser_provider` to `ModelConfig` and `browser_model_id` to each provider config. `LlmClient::query()` and `LlmClient::query_with_image()` accept a new `ModelUseCase` enum (`General` | `Browser`) that selects the correct provider and model ID at call time. Empty `browser_*` fields transparently fall back to the general fields — no breaking change for existing configs.

**Tech Stack:** Rust 2021, `serde` 1.x (field defaults via `#[serde(default)]`), `egui` 0.29 / `eframe` 0.29 (ComboBox + TextEdit), `reqwest` 0.12 (LLM HTTP calls), `toml` 0.8.

---

## File Map

| File | Change |
|---|---|
| `shadow_prompt/src/config.rs` | Add `browser_provider` to `ModelConfig`; add `browser_model_id` to `GroqConfig`, `OpenRouterConfig`, `OllamaConfig` |
| `shadow_prompt/src/llm.rs` | Add `ModelUseCase` enum; update `query()`, `query_with_image()`, all internal helpers to accept + route on `use_case` |
| `shadow_prompt/src/browser/mod.rs` | Pass `ModelUseCase::Browser` to `LlmClient::query()` |
| `shadow_prompt/src/main.rs` | Pass `ModelUseCase::General` to all `LlmClient::query()` / `LlmClient::query_with_image()` call sites |
| `shadow_prompt/src/setup.rs` | Add `browser_model_id` fields per provider + `browser_provider` ComboBox to LLMProvider page; update `sync_provider_field` |
| `shadow_prompt/config/config.example.toml` | Document new `browser_provider` and `browser_model_id` fields |

---

## Task 1: Config schema — add browser model fields

**Context:** `config.rs` holds all serde structs. `ModelConfig` has `provider: String` and optional sub-configs for each provider. We're adding parallel `browser_*` fields that default to empty string (meaning "same as general"). Use `#[serde(default)]` throughout — zero breaking change for configs that don't have these fields.

**Files:**
- Modify: `shadow_prompt/src/config.rs`
- Modify: `shadow_prompt/config/config.example.toml`

- [ ] **Step 1: Write the failing test in `config.rs`**

Add this test at the bottom of `config.rs` inside the existing `#[cfg(test)] mod tests` block (after the existing `test_example_toml_parses` test):

```rust
#[test]
fn test_browser_model_defaults_to_empty() {
    let toml = r#"
[general]
mode = "stealth"
wake_key = "Ctrl+Shift+Space"
model_key = "Ctrl+Shift+V"
panic_key = "Ctrl+Shift+F12"

[visuals]
indicator_color = "#FF0000"
ready_color = "#00FF00"
cursor_change = false

[models]
provider = "auto"

[models.groq]
api_key = "gsk_test"
model_id = "llama-3.1-8b-instant"

[models.openrouter]
api_key = "sk-or-test"
model_id = "google/gemma-3-27b-it:free"

[models.ollama]
base_url = "http://localhost:11434"
model_id = "llama3"

[search]
enabled = true
max_results = 3

[rag]
enabled = true
knowledge_path = "knowledge"
index_path = "data/rag_index"
max_results = 3
min_score = 0.5

[safety]
daily_spend_limit_usd = 0.5
"#;
    let config: Config = toml::from_str(toml).expect("should parse without browser fields");
    // browser_provider absent → defaults to empty string (falls back to provider at runtime)
    assert_eq!(config.models.browser_provider, "");
    // browser_model_id absent → defaults to empty string (falls back to model_id at runtime)
    assert_eq!(config.models.groq.as_ref().unwrap().browser_model_id, "");
    assert_eq!(config.models.openrouter.as_ref().unwrap().browser_model_id, "");
    assert_eq!(config.models.ollama.as_ref().unwrap().browser_model_id, "");
}
```

- [ ] **Step 2: Run the test to confirm it fails**

```bash
cd shadow_prompt && cargo test test_browser_model_defaults_to_empty -- --nocapture 2>&1
```

Expected: compile error — `browser_provider` and `browser_model_id` fields don't exist yet.

- [ ] **Step 3: Add `browser_provider` to `ModelConfig`**

In `config.rs`, find `ModelConfig`:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct ModelConfig {
    pub provider: String,
    pub openrouter: Option<OpenRouterConfig>,
    pub github_copilot: Option<HashMap<String, String>>,
    pub ollama: Option<OllamaConfig>,
    pub groq: Option<GroqConfig>,
}
```

Replace it with:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct ModelConfig {
    pub provider: String,
    /// Provider used for browser/Google Forms queries.
    /// Empty string = use same provider as `provider`.
    #[serde(default)]
    pub browser_provider: String,
    pub openrouter: Option<OpenRouterConfig>,
    pub github_copilot: Option<HashMap<String, String>>,
    pub ollama: Option<OllamaConfig>,
    pub groq: Option<GroqConfig>,
}
```

Update `ModelConfig::default()` accordingly:

```rust
impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "auto".to_string(),
            browser_provider: String::new(),
            openrouter: None,
            github_copilot: None,
            ollama: None,
            groq: Some(GroqConfig::default()),
        }
    }
}
```

- [ ] **Step 4: Add `browser_model_id` to `GroqConfig`**

Find `GroqConfig`:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct GroqConfig {
    pub api_key: String,
    pub model_id: String,
    #[serde(default)]
    pub supports_search: bool,
    #[serde(default)]
    pub supports_vision: bool,
}
```

Replace with:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct GroqConfig {
    pub api_key: String,
    pub model_id: String,
    /// Model ID for browser/Google Forms queries. Empty = use `model_id`.
    #[serde(default)]
    pub browser_model_id: String,
    #[serde(default)]
    pub supports_search: bool,
    #[serde(default)]
    pub supports_vision: bool,
}
```

Update `GroqConfig::default()`:

```rust
impl Default for GroqConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
            model_id: "llama-3.1-8b-instant".to_string(),
            browser_model_id: String::new(),
            supports_search: false,
            supports_vision: false,
        }
    }
}
```

- [ ] **Step 5: Add `browser_model_id` to `OpenRouterConfig`**

Find `OpenRouterConfig`:

```rust
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[allow(dead_code)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model_id: String,
    #[serde(default)]
    pub supports_search: bool,
    #[serde(default)]
    pub supports_vision: bool,
}
```

Replace with:

```rust
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[allow(dead_code)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model_id: String,
    /// Model ID for browser/Google Forms queries. Empty = use `model_id`.
    #[serde(default)]
    pub browser_model_id: String,
    #[serde(default)]
    pub supports_search: bool,
    #[serde(default)]
    pub supports_vision: bool,
}
```

`OpenRouterConfig` uses `#[derive(Default)]` so `browser_model_id` gets `String::new()` automatically — no manual `impl Default` change needed.

- [ ] **Step 6: Add `browser_model_id` to `OllamaConfig`**

Find `OllamaConfig`:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_id: String,
    #[serde(default)]
    pub supports_search: bool,
    #[serde(default)]
    pub supports_vision: bool,
}
```

Replace with:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_id: String,
    /// Model ID for browser/Google Forms queries. Empty = use `model_id`.
    #[serde(default)]
    pub browser_model_id: String,
    #[serde(default)]
    pub supports_search: bool,
    #[serde(default)]
    pub supports_vision: bool,
}
```

Update `OllamaConfig::default()`:

```rust
impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model_id: "llama3".to_string(),
            browser_model_id: String::new(),
            supports_search: false,
            supports_vision: false,
        }
    }
}
```

- [ ] **Step 7: Run the test to confirm it passes**

```bash
cd shadow_prompt && cargo test test_browser_model_defaults_to_empty -- --nocapture 2>&1
```

Expected: `test config::tests::test_browser_model_defaults_to_empty ... ok`

- [ ] **Step 8: Update `config.example.toml`**

Add `browser_provider` to the `[models]` section (after the existing `provider` line):

```toml
[models]
# Which provider to use for OCR / clipboard queries.
# Options: "groq", "openrouter", "ollama", "auto"
provider = "auto"

# Which provider to use for Google Forms automation.
# Leave empty (or omit) to use the same provider as above.
browser_provider = ""
```

Add `browser_model_id` to each sub-section. In `[models.groq]`:

```toml
[models.groq]
api_key  = "YOUR_GROQ_API_KEY_HERE"
model_id = "llama-3.1-8b-instant"
# Model used for Google Forms automation via this provider.
# Leave empty to reuse model_id above.
browser_model_id = ""
supports_search = false
supports_vision = false
```

In `[models.openrouter]`:

```toml
[models.openrouter]
api_key  = "YOUR_OPENROUTER_API_KEY_HERE"
model_id = "google/gemma-3-27b-it:free"
# Model used for Google Forms automation via this provider.
# Leave empty to reuse model_id above.
browser_model_id = ""
supports_search = false
supports_vision = false
```

In `[models.ollama]`:

```toml
[models.ollama]
base_url = "http://localhost:11434"
model_id = "llama3"
# Model used for Google Forms automation via this provider.
# Leave empty to reuse model_id above.
browser_model_id = ""
supports_search = false
supports_vision = false
```

- [ ] **Step 9: Update `test_example_toml_parses` to cover new fields**

In `config.rs`, inside the existing `test_example_toml_parses` test, add after the existing assertions:

```rust
// browser model fields present and defaulting correctly
assert_eq!(config.models.browser_provider, "");
assert_eq!(config.models.groq.as_ref().unwrap().browser_model_id, "");
assert_eq!(config.models.openrouter.as_ref().unwrap().browser_model_id, "");
assert_eq!(config.models.ollama.as_ref().unwrap().browser_model_id, "");
```

- [ ] **Step 10: Run all config tests**

```bash
cd shadow_prompt && cargo test config::tests -- --nocapture 2>&1
```

Expected: all config tests pass.

- [ ] **Step 11: Commit**

```bash
git add shadow_prompt/src/config.rs shadow_prompt/config/config.example.toml
git commit -m "feat: add browser_provider and browser_model_id fields to config schema"
```

---

## Task 2: LlmClient — add ModelUseCase routing

**Context:** `llm.rs` has `LlmClient::query()` and `LlmClient::query_with_image()`, each dispatching to a provider based on `config.models.provider`. We add `ModelUseCase` enum and thread it through every internal function. When `use_case == Browser`, the provider comes from `config.models.browser_provider` (falling back to `config.models.provider` if empty), and the model ID comes from the provider's `browser_model_id` (falling back to `model_id` if empty).

**Files:**
- Modify: `shadow_prompt/src/llm.rs`

- [ ] **Step 1: Write failing tests**

Add this test block at the bottom of `llm.rs` inside the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn test_resolve_model_id_general_uses_model_id() {
    assert_eq!(
        LlmClient::resolve_model_id("general-model", "browser-model", &ModelUseCase::General),
        "general-model"
    );
}

#[test]
fn test_resolve_model_id_browser_uses_browser_model_id_when_set() {
    assert_eq!(
        LlmClient::resolve_model_id("general-model", "browser-model", &ModelUseCase::Browser),
        "browser-model"
    );
}

#[test]
fn test_resolve_model_id_browser_falls_back_when_empty() {
    assert_eq!(
        LlmClient::resolve_model_id("general-model", "", &ModelUseCase::Browser),
        "general-model"
    );
}

#[test]
fn test_resolve_provider_browser_empty_falls_back() {
    assert_eq!(
        LlmClient::resolve_provider("auto", "", &ModelUseCase::Browser),
        "auto"
    );
}

#[test]
fn test_resolve_provider_browser_explicit() {
    assert_eq!(
        LlmClient::resolve_provider("auto", "groq", &ModelUseCase::Browser),
        "groq"
    );
}

#[test]
fn test_resolve_provider_general_ignores_browser_provider() {
    assert_eq!(
        LlmClient::resolve_provider("auto", "groq", &ModelUseCase::General),
        "auto"
    );
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cd shadow_prompt && cargo test test_resolve_model_id -- --nocapture 2>&1
```

Expected: compile error — `ModelUseCase`, `resolve_model_id`, `resolve_provider` don't exist yet.

- [ ] **Step 3: Add `ModelUseCase` enum and resolver helpers**

At the top of `llm.rs`, after the `use` imports, add:

```rust
/// Which feature context is calling the LLM.
/// Determines which provider + model ID from config to use.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelUseCase {
    /// OCR / clipboard queries — uses `config.models.provider` + `model_id`
    General,
    /// Google Forms automation — uses `config.models.browser_provider` + `browser_model_id`
    Browser,
}
```

Inside `impl LlmClient`, add these two private helpers **before** `query()`:

```rust
/// Returns the model ID to use for the given use case.
/// Falls back to `model_id` when `browser_model_id` is empty.
fn resolve_model_id<'a>(model_id: &'a str, browser_model_id: &'a str, use_case: &ModelUseCase) -> &'a str {
    match use_case {
        ModelUseCase::Browser if !browser_model_id.is_empty() => browser_model_id,
        _ => model_id,
    }
}

/// Returns the provider string to use for the given use case.
/// Falls back to `provider` when `browser_provider` is empty.
fn resolve_provider<'a>(provider: &'a str, browser_provider: &'a str, use_case: &ModelUseCase) -> &'a str {
    match use_case {
        ModelUseCase::Browser if !browser_provider.is_empty() => browser_provider,
        _ => provider,
    }
}
```

- [ ] **Step 4: Run the resolver tests**

```bash
cd shadow_prompt && cargo test test_resolve -- --nocapture 2>&1
```

Expected: all 6 resolver tests pass.

- [ ] **Step 5: Update `query()` signature and dispatch**

Replace the existing `query()` function:

```rust
pub async fn query(prompt: &str, config: &Config) -> Result<String> {
```

with:

```rust
pub async fn query(prompt: &str, config: &Config, use_case: ModelUseCase) -> Result<String> {
    let connect_timeout = Duration::from_secs(config.http.connect_timeout_secs);
    let read_timeout = Duration::from_secs(config.http.read_timeout_secs);

    let client = reqwest::Client::builder()
        .connect_timeout(connect_timeout)
        .timeout(read_timeout)
        .build()?;

    let provider = Self::resolve_provider(
        &config.models.provider,
        &config.models.browser_provider,
        &use_case,
    );

    match provider {
        "groq" => Self::query_with_retry_groq(&client, prompt, config, &use_case).await,
        "openrouter" => Self::query_with_retry_openrouter(&client, prompt, config, &use_case).await,
        "ollama" => Self::query_with_retry_ollama(&client, prompt, config, &use_case).await,
        "auto" => Self::query_with_fallback(&client, prompt, config, &use_case).await,
        "github_copilot" => anyhow::bail!("GitHub Copilot provider not fully implemented yet"),
        _ => anyhow::bail!("Unknown provider: {}", provider),
    }
}
```

- [ ] **Step 6: Update all retry wrappers to accept `use_case`**

Replace the three retry wrapper signatures and their inner call:

**`query_with_retry_groq`:**
```rust
async fn query_with_retry_groq(client: &Client, prompt: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    let max_retries = 3;
    let base_delay = Duration::from_secs(1);
    let mut last_error = None;
    for attempt in 0..max_retries {
        match Self::query_groq(client, prompt, config, use_case).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                let error_str = last_error.as_ref().unwrap().to_string().to_lowercase();
                if Self::is_retryable_error(&error_str) && attempt < max_retries - 1 {
                    let delay = base_delay * 2u32.pow(attempt as u32);
                    log::warn!("Groq attempt {} failed, retrying in {:?}...", attempt + 1, delay);
                    sleep(delay).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries failed")))
}
```

**`query_with_retry_openrouter`:**
```rust
async fn query_with_retry_openrouter(client: &Client, prompt: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    let max_retries = 3;
    let base_delay = Duration::from_secs(1);
    let mut last_error = None;
    for attempt in 0..max_retries {
        match Self::query_openrouter(client, prompt, config, use_case).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                let error_str = last_error.as_ref().unwrap().to_string().to_lowercase();
                if Self::is_retryable_error(&error_str) && attempt < max_retries - 1 {
                    let delay = base_delay * 2u32.pow(attempt as u32);
                    log::warn!("OpenRouter attempt {} failed, retrying in {:?}...", attempt + 1, delay);
                    sleep(delay).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries failed")))
}
```

**`query_with_retry_ollama`:**
```rust
async fn query_with_retry_ollama(client: &Client, prompt: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    let max_retries = 3;
    let base_delay = Duration::from_secs(1);
    let mut last_error = None;
    for attempt in 0..max_retries {
        match Self::query_ollama(client, prompt, config, use_case).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                let error_str = last_error.as_ref().unwrap().to_string().to_lowercase();
                if Self::is_retryable_error(&error_str) && attempt < max_retries - 1 {
                    let delay = base_delay * 2u32.pow(attempt as u32);
                    log::warn!("Ollama attempt {} failed, retrying in {:?}...", attempt + 1, delay);
                    sleep(delay).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries failed")))
}
```

- [ ] **Step 7: Update `query_with_fallback` to accept `use_case`**

Replace:

```rust
async fn query_with_fallback(client: &Client, prompt: &str, config: &Config) -> Result<String> {
```

with:

```rust
async fn query_with_fallback(client: &Client, prompt: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    // Priority 1: Groq
    if let Some(groq) = &config.models.groq {
        if !groq.api_key.is_empty() && groq.api_key != "your_groq_api_key_here" {
            match Self::query_with_retry_groq(client, prompt, config, use_case).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    let error_str = e.to_string().to_lowercase();
                    if Self::is_retryable_error(&error_str) {
                        log::warn!("Groq failed (retryable): {}. Falling back...", e);
                    } else {
                        log::error!("Groq failed: {}. Trying next provider...", e);
                    }
                }
            }
        }
    }

    // Priority 2: OpenRouter
    if let Some(or) = &config.models.openrouter {
        if !or.api_key.is_empty() && or.api_key != "your_openrouter_api_key_here" {
            match Self::query_with_retry_openrouter(client, prompt, config, use_case).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    let error_str = e.to_string().to_lowercase();
                    if Self::is_retryable_error(&error_str) {
                        log::warn!("OpenRouter failed (retryable): {}. Falling back...", e);
                    } else {
                        log::error!("OpenRouter failed: {}. Trying next provider...", e);
                    }
                }
            }
        }
    }

    // Priority 3: Ollama
    if config.models.ollama.is_some() {
        match Self::query_with_retry_ollama(client, prompt, config, use_case).await {
            Ok(res) => return Ok(res),
            Err(e) => {
                log::error!("Ollama failed: {}", e);
            }
        }
    }

    anyhow::bail!("All providers failed. Please check your API keys and network connection.")
}
```

- [ ] **Step 8: Update `query_groq` to use `resolve_model_id`**

Replace:

```rust
async fn query_groq(client: &Client, prompt: &str, config: &Config) -> Result<String> {
    let groq_config = config.models.groq.as_ref()
        .context("Groq config missing")?;
    let system_prompt = Self::load_system_prompt();
    let body = json!({
        "model": groq_config.model_id,
```

with:

```rust
async fn query_groq(client: &Client, prompt: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    let groq_config = config.models.groq.as_ref()
        .context("Groq config missing")?;
    let model_id = Self::resolve_model_id(&groq_config.model_id, &groq_config.browser_model_id, use_case);
    let system_prompt = Self::load_system_prompt();
    let body = json!({
        "model": model_id,
```

(Leave the rest of the function body unchanged.)

- [ ] **Step 9: Update `query_openrouter` to use `resolve_model_id`**

Replace:

```rust
async fn query_openrouter(client: &Client, prompt: &str, config: &Config) -> Result<String> {
    let openrouter_config = config.models.openrouter.as_ref()
        .context("OpenRouter config missing")?;
    let system_prompt = Self::load_system_prompt();
    let body = json!({
        "model": openrouter_config.model_id,
```

with:

```rust
async fn query_openrouter(client: &Client, prompt: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    let openrouter_config = config.models.openrouter.as_ref()
        .context("OpenRouter config missing")?;
    let model_id = Self::resolve_model_id(&openrouter_config.model_id, &openrouter_config.browser_model_id, use_case);
    let system_prompt = Self::load_system_prompt();
    let body = json!({
        "model": model_id,
```

- [ ] **Step 10: Update `query_ollama` to use `resolve_model_id`**

Replace:

```rust
async fn query_ollama(client: &Client, prompt: &str, config: &Config) -> Result<String> {
    let ollama_config = config.models.ollama.as_ref().context("Ollama config missing")?;
    let system_prompt = Self::load_system_prompt();
    let body = json!({
        "model": ollama_config.model_id,
```

with:

```rust
async fn query_ollama(client: &Client, prompt: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    let ollama_config = config.models.ollama.as_ref().context("Ollama config missing")?;
    let model_id = Self::resolve_model_id(&ollama_config.model_id, &ollama_config.browser_model_id, use_case);
    let system_prompt = Self::load_system_prompt();
    let body = json!({
        "model": model_id,
```

- [ ] **Step 11: Update `query_with_image` signature**

Replace:

```rust
pub async fn query_with_image(prompt: &str, image_base64: &str, config: &Config) -> Result<String> {
```

with:

```rust
pub async fn query_with_image(prompt: &str, image_base64: &str, config: &Config, use_case: ModelUseCase) -> Result<String> {
    let connect_timeout = Duration::from_secs(config.http.connect_timeout_secs);
    let read_timeout = Duration::from_secs(config.http.read_timeout_secs);

    let client = reqwest::Client::builder()
        .connect_timeout(connect_timeout)
        .timeout(read_timeout)
        .build()?;

    let provider = Self::resolve_provider(
        &config.models.provider,
        &config.models.browser_provider,
        &use_case,
    );

    match provider {
        "groq" => Self::query_groq_with_image(&client, prompt, image_base64, config, &use_case).await,
        "openrouter" => Self::query_openrouter_with_image(&client, prompt, image_base64, config, &use_case).await,
        "ollama" => Self::query_ollama_with_image(&client, prompt, image_base64, config, &use_case).await,
        "auto" => {
            if let Some(groq) = &config.models.groq {
                if !groq.api_key.is_empty() && groq.api_key != "your_groq_api_key_here" {
                    if let Ok(res) = Self::query_groq_with_image(&client, prompt, image_base64, config, &use_case).await {
                        return Ok(res);
                    }
                }
            }
            if let Some(or) = &config.models.openrouter {
                if !or.api_key.is_empty() && or.api_key != "your_openrouter_api_key_here" {
                    if let Ok(res) = Self::query_openrouter_with_image(&client, prompt, image_base64, config, &use_case).await {
                        return Ok(res);
                    }
                }
            }
            anyhow::bail!("No vision-capable provider available")
        }
        _ => anyhow::bail!("Provider does not support vision: {}", provider),
    }
}
```

- [ ] **Step 12: Update `query_groq_with_image`, `query_openrouter_with_image`, `query_ollama_with_image`**

For each of the three vision functions, add `use_case: &ModelUseCase` parameter and call `resolve_model_id`. The pattern is identical for all three — shown here for Groq:

```rust
async fn query_groq_with_image(client: &Client, prompt: &str, image_base64: &str, config: &Config, use_case: &ModelUseCase) -> Result<String> {
    let groq_config = config.models.groq.as_ref()
        .context("Groq config missing")?;
    let model_id = Self::resolve_model_id(&groq_config.model_id, &groq_config.browser_model_id, use_case);
    let system_prompt = Self::load_system_prompt();
    let body = json!({
        "model": model_id,
        // ... rest unchanged
    });
    // rest of function unchanged
}
```

Apply the same pattern to `query_openrouter_with_image` (uses `openrouter_config.model_id` / `openrouter_config.browser_model_id`) and `query_ollama_with_image` (uses `ollama_config.model_id` / `ollama_config.browser_model_id`).

- [ ] **Step 13: Update `test_provider` to always use `General`**

`test_provider` is used in the setup wizard to check connectivity — it always tests the general provider. The function doesn't need a `use_case` param; hardcode `General` internally:

```rust
pub async fn test_provider(provider: &str, config: &Config) -> Result<String> {
    let connect_timeout = Duration::from_secs(5);
    let read_timeout = Duration::from_secs(15);
    let client = reqwest::Client::builder()
        .connect_timeout(connect_timeout)
        .timeout(read_timeout)
        .build()?;
    let test_prompt = "Reply with only the word 'OK' if you can read this.";
    match provider {
        "groq" => Self::query_groq(&client, test_prompt, config, &ModelUseCase::General).await,
        "openrouter" => Self::query_openrouter(&client, test_prompt, config, &ModelUseCase::General).await,
        "ollama" => Self::query_ollama(&client, test_prompt, config, &ModelUseCase::General).await,
        _ => anyhow::bail!("Unknown provider: {}", provider),
    }
}
```

- [ ] **Step 14: Run all llm tests**

```bash
cd shadow_prompt && cargo test llm::tests -- --nocapture 2>&1
```

Expected: all tests pass (extract_content tests unchanged + new resolver tests).

- [ ] **Step 15: Commit**

```bash
git add shadow_prompt/src/llm.rs
git commit -m "feat: add ModelUseCase enum and dual-model routing to LlmClient"
```

---

## Task 3: Wire `browser/mod.rs` to use `ModelUseCase::Browser`

**Context:** `browser/mod.rs` line 207 calls `crate::llm::LlmClient::query(&prompt, &config).await?`. This is the only LLM call in the browser flow — it needs to pass `ModelUseCase::Browser`. The `use` block at the top of `browser/mod.rs` doesn't currently import from `llm`. Add the import and update the call.

**Files:**
- Modify: `shadow_prompt/src/browser/mod.rs`

- [ ] **Step 1: Add `use crate::llm::ModelUseCase;` import**

At the top of `browser/mod.rs`, after the existing `use crate::config::Config;` line:

```rust
use crate::llm::ModelUseCase;
```

- [ ] **Step 2: Update the LLM call**

Find (approximately line 207):

```rust
let llm_res = crate::llm::LlmClient::query(&prompt, &config).await?;
```

Replace with:

```rust
let llm_res = crate::llm::LlmClient::query(&prompt, &config, ModelUseCase::Browser).await?;
```

- [ ] **Step 3: Verify compilation**

```bash
cd shadow_prompt && cargo check 2>&1
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add shadow_prompt/src/browser/mod.rs
git commit -m "feat: use ModelUseCase::Browser for Google Forms LLM calls"
```

---

## Task 4: Wire `main.rs` — pass `ModelUseCase::General`

**Context:** `main.rs` has four `LlmClient` call sites: two `query()` in the `InputEvent::Model` handler, two `query()` in the `InputEvent::OCRRect` handler, and two `query_with_image()` in the OCR handler. All are general queries. Find each and add `ModelUseCase::General`.

**Files:**
- Modify: `shadow_prompt/src/main.rs`

- [ ] **Step 1: Add import**

In `main.rs`, find:

```rust
use crate::llm::LlmClient;
```

Replace with:

```rust
use crate::llm::{LlmClient, ModelUseCase};
```

- [ ] **Step 2: Update OCR vision query call**

Find (in the `InputEvent::OCRRect` handler, vision path):

```rust
match LlmClient::query_with_image(prompt, &image_b64, &config_clone).await {
```

Replace with:

```rust
match LlmClient::query_with_image(prompt, &image_b64, &config_clone, ModelUseCase::General).await {
```

- [ ] **Step 3: Update OCR fallback text query call**

Find (in the same handler, OCR fallback path inside the `Err(e)` arm of query_with_image):

```rust
match LlmClient::query(&text, &config_clone).await {
```

Replace with:

```rust
match LlmClient::query(&text, &config_clone, ModelUseCase::General).await {
```

- [ ] **Step 4: Update OCR text-only path query call**

Find (in the `InputEvent::OCRRect` handler, non-vision text path):

```rust
match LlmClient::query(&text, &config_clone).await {
```

There are two occurrences in the OCR handler — the first is the vision fallback (already updated in Step 3), the second is the pure-text path. Update the second one:

```rust
match LlmClient::query(&text, &config_clone, ModelUseCase::General).await {
```

- [ ] **Step 5: Update clipboard (Model key) query call**

Find (in the `InputEvent::Model` handler):

```rust
match LlmClient::query(&augmented_prompt, &config_clone).await {
```

Replace with:

```rust
match LlmClient::query(&augmented_prompt, &config_clone, ModelUseCase::General).await {
```

- [ ] **Step 6: Verify compilation and all tests**

```bash
cd shadow_prompt && cargo test 2>&1
```

Expected: all tests pass, no compile errors.

- [ ] **Step 7: Commit**

```bash
git add shadow_prompt/src/main.rs
git commit -m "feat: pass ModelUseCase::General to all OCR/clipboard LLM calls in main"
```

---

## Task 5: Update setup wizard — browser model fields

**Context:** The wizard's `SetupWizard` struct in `setup.rs` holds mutable config state. The LLMProvider page (page 3 of 8) shows fields for each provider. We need to add:
1. A `browser_provider` ComboBox at the top of the LLMProvider page — lets users pick the provider for form automation independently (options: "auto", "groq", "openrouter", "ollama", "" = same as general).
2. For each provider section that's enabled, a `browser_model_id` text field labeled "Form Model ID" below the existing `model_id` field.
3. Update `sync_provider_field()` to also write `config.models.browser_provider` if it's empty (default it to `config.models.provider`).

The egui pattern used throughout this file is `ui.horizontal(|ui| { ui.label("..."); ui.text_edit_singleline(&mut self.config.models.groq.as_mut().unwrap().field); })`. Follow the same pattern.

**Files:**
- Modify: `shadow_prompt/src/setup.rs`

- [ ] **Step 1: Read the LLMProvider page render function**

The LLMProvider page UI is rendered in a `fn render_llm_provider(&mut self, ui: &mut egui::Ui)` method (or inlined in the main `update()` call under `SetupPage::LLMProvider =>`). Read the section of `setup.rs` starting at `SetupPage::LLMProvider` to confirm the exact rendering location and structure before editing. Use:

```bash
grep -n "LLMProvider\|llm_provider\|render_llm\|browser_model_id" shadow_prompt/src/setup.rs 2>&1
```

- [ ] **Step 2: Add `browser_provider` ComboBox to the LLMProvider page**

Find the section of `setup.rs` that renders the LLMProvider page. Locate the line where it shows the provider description text or the first provider section header. **Above** the first provider section (Groq), add a horizontal row with the ComboBox:

```rust
ui.add_space(6.0);
ui.label(egui::RichText::new("Form Automation Provider").strong());
ui.horizontal(|ui| {
    ui.label("Provider for Google Forms:");
    egui::ComboBox::from_id_salt("browser_provider_combo")
        .selected_text(if self.config.models.browser_provider.is_empty() {
            "Same as above"
        } else {
            &self.config.models.browser_provider
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.config.models.browser_provider, String::new(), "Same as above");
            ui.selectable_value(&mut self.config.models.browser_provider, "auto".to_string(), "auto");
            ui.selectable_value(&mut self.config.models.browser_provider, "groq".to_string(), "groq");
            ui.selectable_value(&mut self.config.models.browser_provider, "openrouter".to_string(), "openrouter");
            ui.selectable_value(&mut self.config.models.browser_provider, "ollama".to_string(), "ollama");
        });
});
ui.add_space(6.0);
```

- [ ] **Step 3: Add `browser_model_id` field to the Groq section**

In the Groq provider section, find the `model_id` field rendering. It will look like:

```rust
ui.horizontal(|ui| {
    ui.label("Model ID:");
    ui.text_edit_singleline(&mut self.config.models.groq.as_mut().unwrap().model_id);
});
```

Immediately **after** that block, add:

```rust
ui.horizontal(|ui| {
    ui.label("Form Model ID:");
    ui.add(
        egui::TextEdit::singleline(
            &mut self.config.models.groq.as_mut().unwrap().browser_model_id
        ).hint_text("Leave blank to reuse Model ID above")
    );
});
```

- [ ] **Step 4: Add `browser_model_id` field to the OpenRouter section**

Find the OpenRouter `model_id` field. After it, add:

```rust
ui.horizontal(|ui| {
    ui.label("Form Model ID:");
    ui.add(
        egui::TextEdit::singleline(
            &mut self.config.models.openrouter.as_mut().unwrap().browser_model_id
        ).hint_text("Leave blank to reuse Model ID above")
    );
});
```

- [ ] **Step 5: Add `browser_model_id` field to the Ollama section**

Find the Ollama `model_id` field. After it, add:

```rust
ui.horizontal(|ui| {
    ui.label("Form Model ID:");
    ui.add(
        egui::TextEdit::singleline(
            &mut self.config.models.ollama.as_mut().unwrap().browser_model_id
        ).hint_text("Leave blank to reuse Model ID above")
    );
});
```

- [ ] **Step 6: Update `sync_provider_field` to handle `browser_provider`**

Find `fn sync_provider_field(&mut self)` in `setup.rs`. The current function sets `self.config.models.provider`. After the line that sets `self.config.models.provider`, add:

```rust
// If browser_provider was left as empty string (meaning "same as general"),
// leave it empty — the runtime resolver handles the fallback.
// Only override if the user explicitly picked a browser_provider in the ComboBox.
// Nothing to do here — browser_provider is already written directly by the ComboBox.
```

No code change is actually required here because `browser_provider` is written live by the ComboBox binding (`&mut self.config.models.browser_provider`). The comment is just for clarity. Add it as a code comment, not actual code.

- [ ] **Step 7: Ensure Ollama section initializes `browser_model_id`**

The Ollama section in the wizard conditionally creates `config.models.ollama` when enabled. Find where Ollama is initialized in the wizard (when the checkbox is checked). It likely does something like:

```rust
if self.provider_state.ollama_enabled && self.config.models.ollama.is_none() {
    self.config.models.ollama = Some(OllamaConfig::default());
}
```

Since `OllamaConfig::default()` now sets `browser_model_id: String::new()`, no change is needed here — but verify this is the case by checking the initialization path.

- [ ] **Step 8: Verify the wizard compiles**

```bash
cd shadow_prompt && cargo check 2>&1
```

Expected: no compile errors.

- [ ] **Step 9: Run all tests**

```bash
cd shadow_prompt && cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 10: Commit**

```bash
git add shadow_prompt/src/setup.rs
git commit -m "feat: add browser provider and form model ID fields to setup wizard"
```

---

## Final: Push

After all 5 tasks are complete and all tests pass:

```bash
git push origin main
```
