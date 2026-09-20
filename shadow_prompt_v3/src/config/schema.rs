// TOML schema. Mirrors config/config.example.toml. See docs/SHADOWPROMPT_V3_DESIGN.md for the
// design decisions behind every field here — this file should never need its own prose
// documentation beyond field-level comments, per CLAUDE.md's "don't hand-maintain what code
// already answers" rule.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub openrouter: OpenRouterConfig,
    pub hotkeys: HotkeysConfig,
    pub visuals: VisualsConfig,
    pub http: HttpConfig,
    pub forms: FormsConfig,
    #[serde(default)]
    pub knowledge: KnowledgeConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenRouterConfig {
    pub api_key: String,

    /// Fallback chain, tried in order by OpenRouter's own `models` array — design doc §8.
    /// Default order: gemini-3.5-flash-lite -> claude-sonnet-5 -> the three free models
    /// ranked by actual uptime (nex last). `switch_model` (design doc §8/§10) swaps which of
    /// the first two entries is tried first at runtime; it does not remove either from the
    /// chain, both stay present, only their order changes.
    #[serde(default = "default_model_chain")]
    pub models: Vec<String>,
}

fn default_model_chain() -> Vec<String> {
    vec![
        "google/gemini-3.5-flash-lite".to_string(),
        "anthropic/claude-sonnet-5".to_string(),
        "inclusionai/ling-3.0-flash-sante:free".to_string(),
        "inclusionai/ling-3.0-flash-fin:free".to_string(),
        "nex-agi/nex-n2.5-mini:free".to_string(),
    ]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotkeysConfig {
    // Answer
    pub clipboard_query: String,
    pub screenshot_query: String,
    pub test_model: String,
    pub switch_model: String,
    // Google Forms
    pub launch_debugger: String,
    /// v3 legacy (design doc §7.4) — default/primary as of 2026-09-20.
    pub forms_answer_page: String,
    pub forms_answer_all: String,
    /// v3 new (design doc §7.3) — AX-tree/`fill_page` engine, parked but kept reachable.
    /// `#[serde(default)]` so a config file written before this pair existed still parses.
    #[serde(default = "default_forms_answer_page_axtree")]
    pub forms_answer_page_axtree: String,
    #[serde(default = "default_forms_answer_all_axtree")]
    pub forms_answer_all_axtree: String,
    // Utility
    pub abort: String,
    pub hide_toggle: String,
    pub help_toggle: String,
    // Lifecycle
    pub restart_daemon: String,
    pub insta_delete: String,
    pub panic_kill: String,
    /// Dev-only test hotkey: opens a new tab in the debug Chrome window via the exact same code
    /// path Forms itself uses, without the LLM loop around it — lets "does this open a new tab
    /// or a new window" get tested instantly. Only does anything under the `debug` Cargo feature
    /// (see `actions::debug_open_tab`); harmless to parse in a production build otherwise.
    /// `#[serde(default)]` so this doesn't break deserializing a config written before this
    /// field existed.
    #[serde(default = "default_debug_open_tab")]
    pub debug_open_tab: String,
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
    pub form_color_done: String,
    pub form_color_failed: String,
    pub form_color_aborted: String,

    pub overlay_enabled: bool,
    pub overlay_corner: String,
    pub overlay_offset: [i32; 2],
    pub overlay_font_size: u32,

    /// Gradient-crawl display — design doc §6. Only engages when the answer doesn't fit in
    /// `crawl_trigger_lines` lines at the overlay's wrap width; below that it's static, same
    /// as v2's plain overlay.
    #[serde(default = "default_crawl_trigger_lines")]
    pub crawl_trigger_lines: u32,
    /// Milliseconds per line advance in the crawl.
    #[serde(default = "default_crawl_ms_per_line")]
    pub crawl_ms_per_line: u64,
}

fn default_crawl_trigger_lines() -> u32 {
    2
}
fn default_crawl_ms_per_line() -> u64 {
    2000
}
fn default_debug_open_tab() -> String {
    "ctrl+shift+alt+z".to_string()
}
fn default_forms_answer_page_axtree() -> String {
    "ctrl+shift+alt+j".to_string()
}
fn default_forms_answer_all_axtree() -> String {
    "ctrl+shift+alt+u".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    pub connect_timeout_secs: u64,
    pub read_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormsConfig {
    /// Bug-safety backstop only, not a UX constraint — design doc §7.3. Should never fire on a
    /// normal form; guards against a broken Next-button loop running forever unattended.
    #[serde(default = "default_max_pages")]
    pub max_pages: u32,
    pub max_images_per_request: u32,
    pub max_image_long_edge_px: u32,
}

fn default_max_pages() -> u32 {
    20
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KnowledgeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Per-`read_doc` output cap in bytes — design doc §4 / OPENCODE.md §3.4's output-bounding
    /// steal. No `active_subjects` field: v3 drops that concept entirely, `list_docs` just
    /// enumerates the whole sandboxed `knowledge/` tree on demand.
    #[serde(default = "default_max_doc_bytes")]
    pub max_doc_bytes: usize,
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_doc_bytes: default_max_doc_bytes(),
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_max_doc_bytes() -> usize {
    100_000
}
