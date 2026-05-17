// TOML schema. Mirror config/config.example.toml.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub openrouter: OpenRouterConfig,
    pub hotkeys: HotkeysConfig,
    pub visuals: VisualsConfig,
    pub http: HttpConfig,
    pub forms: FormsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotkeysConfig {
    pub clipboard_query: String,
    pub ocr_query: String,
    pub forms_auto: String,
    pub forms_single: String,
    pub abort: String,
    pub launch_debugger: String,
    pub hide_toggle: String,
    pub restart_daemon: String,
    pub insta_delete: String,
    pub panic_kill: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisualsConfig {
    pub indicator_corner: String,
    pub indicator_size: u32,
    pub indicator_offset: [i32; 2],
    pub color_ready: String,
    pub color_processing: String,
    pub color_error: String,
    pub form_indicator_corner: String,
    pub form_indicator_offset: [i32; 2],
    pub form_color_running: String,
    pub form_color_failed: String,
    pub form_color_aborted: String,
    pub overlay_enabled: bool,
    pub overlay_corner: String,
    pub overlay_offset: [i32; 2],
    pub overlay_font_size: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    pub connect_timeout_secs: u64,
    pub read_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormsConfig {
    pub max_pages: u32,
    pub max_images_per_request: u32,
    pub max_image_long_edge_px: u32,
}
