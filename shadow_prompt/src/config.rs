use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, Context};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub visuals: VisualsConfig,
    pub models: ModelConfig,
    pub rag: RagConfig,
    pub safety: SafetyConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    pub mode: String,
    pub wake_key: String,
    pub model_key: String,
    pub panic_key: String,
    pub use_rag: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VisualsConfig {
    pub indicator_color: String,
    pub ready_color: String,
    pub cursor_change: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelConfig {
    pub provider: String,
    pub openrouter: Option<OpenRouterConfig>,
    pub github_copilot: Option<HashMap<String, String>>, // Flexible for now
    pub google_antigravity: Option<HashMap<String, String>>,
    pub google_gemini_api: Option<GeminiConfig>,
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RagConfig {
    pub chunk_size: usize,
    pub overlap: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SafetyConfig {
    pub daily_spend_limit_usd: f64,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try to read config.toml from the /config directory relative to the executable
        // Or for dev, relative to the crate root.
        let config_path = "config/config.toml";
        
        let content = fs::read_to_string(config_path)
            .or_else(|_| fs::read_to_string("../config/config.toml")) // Try parent if in bin
            .context("Failed to read config.toml. Ensure 'config/config.toml' exists.")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse config.toml")?;

        Ok(config)
    }
}
