use serde::Deserialize;
use std::fs;
use anyhow::{Result, Context};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub general: GeneralConfig,
    pub visuals: VisualsConfig,
    pub models: ModelConfig,
    pub search: SearchConfig,
    pub rag: RagConfig,
    pub safety: SafetyConfig,
}



#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct SearchConfig {
    pub enabled: bool,
    pub max_results: usize,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct GeneralConfig {
    pub mode: String,
    pub wake_key: String,
    pub model_key: String,
    pub panic_key: String,
    pub use_rag: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct VisualsConfig {
    pub indicator_color: String, // Kept for backwards compatibility/parsing, but we might prefer color_processing
    pub ready_color: String,
    
    // New Fields
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default = "default_size")]
    pub size: i32,
    #[serde(default)]
    pub offset: i32,    
    #[serde(default)]
    pub x_axis: i32,
    #[serde(default)]
    pub y_axis: i32,

    // Colors
    #[serde(default = "default_mcq_a")]
    pub color_mcq_a: String,
    #[serde(default = "default_mcq_b")]
    pub color_mcq_b: String,
    #[serde(default = "default_mcq_c")]
    pub color_mcq_c: String,
    #[serde(default = "default_mcq_d")]
    pub color_mcq_d: String,
    
    #[serde(default = "default_processing")]
    pub color_processing: String,

    pub cursor_change: bool,
}

fn default_position() -> String { "top-right".to_string() }
fn default_size() -> i32 { 5 }
fn default_mcq_a() -> String { "#00FFFF".to_string() } // Cyan
fn default_mcq_b() -> String { "#FF00FF".to_string() } // Magenta
fn default_mcq_c() -> String { "#FFFF00".to_string() } // Yellow
fn default_mcq_d() -> String { "#000000".to_string() } // Black
fn default_processing() -> String { "#FF0000".to_string() } // Red

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ModelConfig {
    pub provider: String,
    pub openrouter: Option<OpenRouterConfig>,
    pub github_copilot: Option<HashMap<String, String>>, // Flexible for now
    pub ollama: Option<OllamaConfig>,
    pub groq: Option<GroqConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct GroqConfig {
    pub api_key: String,
    pub model_id: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model_id: String,
}



#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_id: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct RagConfig {
    pub chunk_size: usize,
    pub overlap: usize,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
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
