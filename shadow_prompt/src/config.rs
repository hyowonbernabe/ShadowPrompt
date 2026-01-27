use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, Context};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[allow(dead_code)]
pub struct Config {
    pub general: GeneralConfig,
    pub visuals: VisualsConfig,
    pub models: ModelConfig,
    pub search: SearchConfig,
    pub rag: RagConfig,
    pub safety: SafetyConfig,
}



#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct SearchConfig {
    pub enabled: bool,
    pub max_results: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 3,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct GeneralConfig {
    pub mode: String,
    pub wake_key: String,
    pub model_key: String,
    pub panic_key: String,
    pub use_rag: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mode: "stealth".to_string(),
            wake_key: "Ctrl+Shift+Space".to_string(),
            model_key: "Ctrl+Shift+V".to_string(),
            panic_key: "Ctrl+Shift+F12".to_string(),
            use_rag: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl Default for VisualsConfig {
    fn default() -> Self {
        Self {
            indicator_color: "#FF0000".to_string(),
            ready_color: "#00FF00".to_string(),
            position: default_position(),
            size: default_size(),
            offset: 0,
            x_axis: 0,
            y_axis: 0,
            color_mcq_a: default_mcq_a(),
            color_mcq_b: default_mcq_b(),
            color_mcq_c: default_mcq_c(),
            color_mcq_d: default_mcq_d(),
            color_processing: default_processing(),
            cursor_change: false,
        }
    }
}

fn default_position() -> String { "top-right".to_string() }
fn default_size() -> i32 { 5 }
fn default_mcq_a() -> String { "#00FFFF".to_string() } // Cyan
fn default_mcq_b() -> String { "#FF00FF".to_string() } // Magenta
fn default_mcq_c() -> String { "#FFFF00".to_string() } // Yellow
fn default_mcq_d() -> String { "#000000".to_string() } // Black
fn default_processing() -> String { "#FF0000".to_string() } // Red

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct ModelConfig {
    pub provider: String,
    pub openrouter: Option<OpenRouterConfig>,
    pub github_copilot: Option<HashMap<String, String>>, // Flexible for now
    pub ollama: Option<OllamaConfig>,
    pub groq: Option<GroqConfig>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "groq".to_string(),
            openrouter: None,
            github_copilot: None,
            ollama: None,
            groq: Some(GroqConfig::default()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct GroqConfig {
    pub api_key: String,
    pub model_id: String,
}

impl Default for GroqConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
            model_id: "llama-3.1-8b-instant".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[allow(dead_code)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model_id: String,
}



#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model_id: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model_id: "llama3".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct RagConfig {
    pub enabled: bool,
    pub knowledge_path: String,
    pub index_path: String,
    pub max_results: usize,
    pub min_score: f32,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            knowledge_path: "knowledge".to_string(),
            index_path: "data/rag_index".to_string(),
            max_results: 3,
            min_score: 0.5,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct SafetyConfig {
    pub daily_spend_limit_usd: f64,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            daily_spend_limit_usd: 0.50,
        }
    }
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

    pub fn save(&self) -> Result<()> {
        let config_path = "config/config.toml";
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        if let Some(parent) = std::path::Path::new(config_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(config_path, content)
            .context("Failed to write config.toml")?;
        
        Ok(())
    }

    pub fn mark_setup_complete() -> Result<()> {
        fs::write("config/.setup_complete", "done")?;
        Ok(())
    }

    pub fn is_setup_complete() -> bool {
        std::path::Path::new("config/.setup_complete").exists()
    }
}
