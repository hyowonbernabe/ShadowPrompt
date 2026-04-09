use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[allow(dead_code)]
pub struct Config {
    pub general: GeneralConfig,
    pub visuals: VisualsConfig,
    pub models: ModelConfig,
    pub search: SearchConfig,
    pub rag: RagConfig,
    pub safety: SafetyConfig,
    #[serde(default)]
    pub http: HttpConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct SearchConfig {
    pub enabled: bool,
    pub max_results: usize,
    #[serde(default = "default_search_engine")]
    pub engine: String,
    #[serde(default)]
    pub serper_api_key: Option<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 3,
            engine: default_search_engine(),
            serper_api_key: None,
        }
    }
}

fn default_search_engine() -> String {
    "serper".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct GeneralConfig {
    pub mode: String,
    pub wake_key: String,
    pub model_key: String,
    pub panic_key: String,
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub tos_accepted: bool,
    #[serde(default)]
    pub tos_accepted_version: String,

    #[serde(default = "default_browser_pass")]
    pub key_browser_pass: String,
    #[serde(default = "default_browser_exec")]
    pub key_browser_exec: String,
    #[serde(default = "default_browser_exec_single")]
    pub key_browser_exec_single: String,
    #[serde(default = "default_browser_abort")]
    pub key_browser_abort: String,
    #[serde(default = "default_browser_incognito")]
    pub key_browser_incognito: String,

    #[serde(default = "default_key_cycle_model_general")]
    pub key_cycle_model_general: String,
    #[serde(default = "default_key_cycle_model_forms")]
    pub key_cycle_model_forms: String,
    #[serde(default = "default_key_cycle_provider")]
    pub key_cycle_provider: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mode: "stealth".to_string(),
            wake_key: "Ctrl+Shift+Space".to_string(),
            model_key: "Ctrl+Shift+V".to_string(),
            panic_key: "Ctrl+Shift+F12".to_string(),
            debug: false,
            tos_accepted: false,
            tos_accepted_version: String::new(),
            key_browser_pass: default_browser_pass(),
            key_browser_exec: default_browser_exec(),
            key_browser_exec_single: default_browser_exec_single(),
            key_browser_abort: default_browser_abort(),
            key_browser_incognito: default_browser_incognito(),
            key_cycle_model_general: default_key_cycle_model_general(),
            key_cycle_model_forms: default_key_cycle_model_forms(),
            key_cycle_provider: default_key_cycle_provider(),
        }
    }
}

fn default_browser_pass() -> String {
    "Ctrl+Shift+8".to_string()
}
fn default_browser_exec() -> String {
    "Ctrl+Shift+9".to_string()
}
fn default_browser_exec_single() -> String {
    "Ctrl+Shift+7".to_string()
}
fn default_browser_abort() -> String {
    "Ctrl+Shift+0".to_string()
}
fn default_browser_incognito() -> String {
    "Ctrl+Shift+I".to_string()
}

fn default_key_cycle_model_general() -> String {
    "Ctrl+Shift+LeftArrow".to_string()
}
fn default_key_cycle_model_forms() -> String {
    "Ctrl+Shift+RightArrow".to_string()
}
fn default_key_cycle_provider() -> String {
    "Ctrl+Shift+UpArrow".to_string()
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
    #[serde(default = "default_mcq_none")]
    pub color_mcq_none: String,

    #[serde(default = "default_processing")]
    pub color_processing: String,

    pub cursor_change: bool,

    #[serde(default = "default_color_true")]
    pub color_true: String,

    #[serde(default = "default_color_false")]
    pub color_false: String,

    #[serde(default = "default_true")]
    pub text_overlay_enabled: bool,

    #[serde(default = "default_text_overlay_position")]
    pub text_overlay_position: String,

    #[serde(default)]
    pub text_overlay_x_axis: i32,

    #[serde(default)]
    pub text_overlay_y_axis: i32,

    #[serde(default = "default_text_overlay_font_size")]
    pub text_overlay_font_size: i32,

    #[serde(default = "default_text_overlay_bg_opacity")]
    pub text_overlay_bg_opacity: u8,

    #[serde(default = "default_text_overlay_text_opacity")]
    pub text_overlay_text_opacity: u8,

    #[serde(default = "default_text_overlay_offset")]
    pub text_overlay_offset: i32,

    #[serde(default = "default_hide_key")]
    pub hide_key: String,

    // --- Form (Google Forms) Indicator ---
    #[serde(default = "default_form_indicator_position")]
    pub form_indicator_position: String,
    #[serde(default)]
    pub form_indicator_offset: i32,
    #[serde(default)]
    pub form_indicator_x_axis: i32,
    #[serde(default)]
    pub form_indicator_y_axis: i32,
    #[serde(default = "default_form_color_failed")]
    pub form_color_failed: String,
    #[serde(default = "default_form_color_aborted")]
    pub form_color_aborted: String,
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
            color_mcq_none: default_mcq_none(),
            color_processing: default_processing(),
            cursor_change: false,
            color_true: default_color_true(),
            color_false: default_color_false(),
            text_overlay_enabled: true,
            text_overlay_position: default_text_overlay_position(),
            text_overlay_font_size: default_text_overlay_font_size(),
            text_overlay_bg_opacity: default_text_overlay_bg_opacity(),
            text_overlay_text_opacity: default_text_overlay_text_opacity(),
            text_overlay_offset: default_text_overlay_offset(),
            text_overlay_x_axis: 0,
            text_overlay_y_axis: 0,
            hide_key: default_hide_key(),
            form_indicator_position: default_form_indicator_position(),
            form_indicator_offset: 0,
            form_indicator_x_axis: 0,
            form_indicator_y_axis: 0,
            form_color_failed: default_form_color_failed(),
            form_color_aborted: default_form_color_aborted(),
        }
    }
}

fn default_position() -> String {
    "top-right".to_string()
}

fn default_text_overlay_position() -> String {
    "bottom-right".to_string()
}
fn default_size() -> i32 {
    3
}
fn default_mcq_a() -> String {
    "#00FFFF".to_string()
} // Cyan
fn default_mcq_b() -> String {
    "#FF00FF".to_string()
} // Magenta
fn default_mcq_c() -> String {
    "#FFFF00".to_string()
} // Yellow
fn default_mcq_d() -> String {
    "#000000".to_string()
}
fn default_mcq_none() -> String {
    "#FFFFFF".to_string()
}
fn default_processing() -> String {
    "#FF0000".to_string()
}

fn default_true() -> bool {
    true
}

fn default_color_true() -> String {
    "#00FF00".to_string()
}

fn default_color_false() -> String {
    "#800000".to_string()
}

#[allow(dead_code)]
fn default_text_size() -> i32 {
    12
}

fn default_text_overlay_font_size() -> i32 {
    8
}

fn default_text_overlay_offset() -> i32 {
    10
}

#[allow(dead_code)]
fn default_text_opacity() -> u8 {
    200
}

fn default_text_overlay_bg_opacity() -> u8 {
    0
}

fn default_text_overlay_text_opacity() -> u8 {
    255
}

fn default_hide_key() -> String {
    "Ctrl+Shift+H".to_string()
}

fn default_form_indicator_position() -> String {
    "bottom-right".to_string()
}

fn default_form_color_failed() -> String {
    "#FF00FF".to_string() // Magenta
}

fn default_form_color_aborted() -> String {
    "#FFA500".to_string() // Orange
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct ModelConfig {
    pub provider: String,
    /// Provider used for browser/Google Forms queries. Empty string = use same as `provider`.
    #[serde(default)]
    pub browser_provider: String,
    pub openrouter: Option<OpenRouterConfig>,
    pub github_copilot: Option<HashMap<String, String>>, // Flexible for now
    pub ollama: Option<OllamaConfig>,
    pub groq: Option<GroqConfig>,
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct GroqConfig {
    pub api_key: String,
    pub model_id: String,
    #[serde(default)]
    pub model_choices: Vec<String>,
    /// Model ID for browser/Google Forms queries. Empty = use `model_id`.
    #[serde(default)]
    pub browser_model_id: String,
    #[serde(default)]
    pub browser_model_choices: Vec<String>,
    #[serde(default = "default_true")]
    pub supports_search: bool,
    #[serde(default = "default_true")]
    pub supports_vision: bool,
}

impl Default for GroqConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
            model_id: "llama-3.1-8b-instant".to_string(),
            model_choices: Vec::new(),
            browser_model_id: String::new(),
            browser_model_choices: Vec::new(),
            supports_search: true,
            supports_vision: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[allow(dead_code)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model_id: String,
    #[serde(default)]
    pub model_choices: Vec<String>,
    /// Model ID for Google Forms automation. Leave empty to reuse model_id above.
    #[serde(default)]
    pub browser_model_id: String,
    #[serde(default)]
    pub browser_model_choices: Vec<String>,
    #[serde(default = "default_true")]
    pub supports_search: bool,
    #[serde(default = "default_true")]
    pub supports_vision: bool,
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct HttpConfig {
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_read_timeout")]
    pub read_timeout_secs: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            connect_timeout_secs: 10,
            read_timeout_secs: 30,
        }
    }
}

fn default_connect_timeout() -> u64 {
    10
}
fn default_read_timeout() -> u64 {
    30
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();

        let content = fs::read_to_string(&config_path)
            .context(format!("Failed to read config.toml at {:?}", config_path))?;

        let config: Config = toml::from_str(&content).context("Failed to parse config.toml")?;

        Ok(config)
    }

    #[allow(dead_code)]
    pub fn try_load() -> Option<Self> {
        let config_path = get_config_path();
        if !config_path.exists() {
            return None;
        }
        Self::load().ok()
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&config_path, content).context("Failed to write config.toml")?;

        Ok(())
    }

    pub fn mark_setup_complete() -> Result<()> {
        let marker_path = get_exe_dir().join("config").join(".setup_complete");
        if let Some(parent) = marker_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(marker_path, "done")?;
        Ok(())
    }

    pub fn is_setup_complete() -> bool {
        get_exe_dir()
            .join("config")
            .join(".setup_complete")
            .exists()
    }
}

pub fn get_exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Resolves a file inside the config/ folder.
/// Tries exe-relative first (production / USB), then CWD (cargo run / dev).
pub fn get_config_file_path(filename: &str) -> std::path::PathBuf {
    let exe_path = get_exe_dir().join("config").join(filename);
    if exe_path.exists() {
        return exe_path;
    }
    let cwd_path = std::path::PathBuf::from("config").join(filename);
    if cwd_path.exists() {
        return cwd_path;
    }
    exe_path
}

pub fn get_config_path() -> std::path::PathBuf {
    let exe_dir = get_exe_dir();
    let config_path = exe_dir.join("config").join("config.toml");

    if config_path.exists() {
        return config_path;
    }

    let cwd_config = std::path::PathBuf::from("config/config.toml");
    if cwd_config.exists() {
        return cwd_config;
    }

    config_path
}

pub fn ensure_directories() -> Result<()> {
    let exe_dir = get_exe_dir();

    let knowledge_dir = exe_dir.join("knowledge");
    if !knowledge_dir.exists() {
        fs::create_dir_all(&knowledge_dir).context("Failed to create knowledge directory")?;
        println!("[*] Created knowledge directory: {:?}", knowledge_dir);
    }

    let config_dir = exe_dir.join("config");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    let data_dir = exe_dir.join("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).context("Failed to create data directory")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_toml_parses() {
        let example = include_str!("../config/config.example.toml");
        // Replace placeholder API keys so parsing doesn't fail on validation
        let content = example
            .replace("YOUR_GROQ_API_KEY_HERE", "")
            .replace("YOUR_OPENROUTER_API_KEY_HERE", "");
        let config: Config =
            toml::from_str(&content).expect("config.example.toml should parse without errors");
        assert_eq!(config.general.wake_key, "Ctrl+Shift+Space");
        assert_eq!(config.general.panic_key, "Ctrl+Shift+F12");
        assert_eq!(config.general.key_browser_pass, "Ctrl+Shift+8");
        assert_eq!(config.visuals.hide_key, "Ctrl+Shift+H");
        assert_eq!(config.visuals.form_indicator_position, "bottom-right");
        assert_eq!(config.visuals.form_color_failed, "#FF00FF");
        assert_eq!(config.visuals.form_color_aborted, "#FFA500");
        assert_eq!(config.visuals.form_indicator_offset, 0);
        assert_eq!(config.visuals.form_indicator_x_axis, 0);
        assert_eq!(config.visuals.form_indicator_y_axis, 0);
        assert_eq!(config.http.connect_timeout_secs, 10);
        assert!(config.search.serper_api_key.is_none());
        assert_eq!(config.models.browser_provider, "");
        assert_eq!(config.models.groq.as_ref().unwrap().browser_model_id, "");
        assert_eq!(
            config.models.openrouter.as_ref().unwrap().browser_model_id,
            ""
        );
        assert_eq!(config.models.ollama.as_ref().unwrap().browser_model_id, "");
    }

    #[test]
    fn test_form_indicator_config_defaults() {
        // Old config without form indicator fields — must parse and default correctly.
        let toml = r##"
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
"##;
        let config: Config =
            toml::from_str(toml).expect("should parse without form indicator fields");
        assert_eq!(config.visuals.form_indicator_position, "bottom-right");
        assert_eq!(config.visuals.form_indicator_offset, 0);
        assert_eq!(config.visuals.form_indicator_x_axis, 0);
        assert_eq!(config.visuals.form_indicator_y_axis, 0);
        assert_eq!(config.visuals.form_color_failed, "#FF00FF");
        assert_eq!(config.visuals.form_color_aborted, "#FFA500");
    }

    #[test]
    fn test_browser_model_defaults_to_empty() {
        let toml = r##"
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
"##;
        let config: Config = toml::from_str(toml).expect("should parse without browser fields");
        assert_eq!(config.models.browser_provider, "");
        assert_eq!(config.models.groq.as_ref().unwrap().browser_model_id, "");
        assert_eq!(
            config.models.openrouter.as_ref().unwrap().browser_model_id,
            ""
        );
        assert_eq!(config.models.ollama.as_ref().unwrap().browser_model_id, "");
    }
}
