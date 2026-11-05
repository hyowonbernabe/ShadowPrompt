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
    #[serde(default)]
    pub screen_answer: ScreenAnswerConfig,
    /// Access-code login (docs/superpowers/specs/2026-11-06-access-code-login-design.md). Written
    /// by the app on log in / sign up, cleared on log out; never touches `openrouter.api_key`.
    #[serde(default)]
    pub account: AccountConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AccountConfig {
    #[serde(default)]
    pub code: String,
    /// The key fetched with `code`, saved so the app works offline.
    #[serde(default)]
    pub api_key: String,
}

impl AccountConfig {
    pub fn logged_in(&self) -> bool {
        !self.code.is_empty() && crate::account::is_real_key(&self.api_key)
    }
}

/// Screen Answer (all-monitor capture -> Claude -> tracked dots). OpenRouter + Claude only.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScreenAnswerConfig {
    /// Own fallback chain, separate from `openrouter.models`; `switch_model` doesn't touch it.
    #[serde(default = "default_screen_answer_models")]
    pub models: Vec<String>,
    /// Patch-match similarity (0..1) a dot needs to stay visible. Lower = follows more loosely,
    /// higher = hides sooner.
    #[serde(default = "default_match_threshold")]
    pub match_threshold: f32,
}

impl Default for ScreenAnswerConfig {
    fn default() -> Self {
        Self { models: default_screen_answer_models(), match_threshold: default_match_threshold() }
    }
}

fn default_screen_answer_models() -> Vec<String> {
    vec![
        "anthropic/claude-sonnet-5.5".to_string(),
        "anthropic/claude-opus-5.5".to_string(),
        "anthropic/claude-sonnet-5".to_string(),
    ]
}
fn default_match_threshold() -> f32 {
    0.9
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenRouterConfig {
    pub api_key: String,

    /// Fallback chain, tried in order by OpenRouter's own `models` array — design doc §8.
    /// Default order: claude-sonnet-5.5 -> gemini-3.8-flash -> the three free models
    /// (picked and live-tested 2026-11-05: two vision-capable, then the strongest text-only one;
    /// free models get pulled from OpenRouter often, so re-check these when backups fail). `switch_model` (design doc §8/§10) swaps which of
    /// the first two entries is tried first at runtime; it does not remove either from the
    /// chain, both stay present, only their order changes.
    #[serde(default = "default_model_chain")]
    pub models: Vec<String>,
}

pub(crate) fn default_model_chain() -> Vec<String> {
    vec![
        "anthropic/claude-sonnet-5.5".to_string(),
        "google/gemini-3.8-flash".to_string(),
        "nvidia/nemotron-3.5-super-vl:free".to_string(),
        "dots-studio/dots-3-note-preview:free".to_string(),
        "nvidia/nemotron-3-ultra-550b-a55b:free".to_string(),
    ]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotkeysConfig {
    // Answer
    pub clipboard_query: String,
    pub screenshot_query: String,
    pub test_model: String,
    pub switch_model: String,
    // Conversation memory. `#[serde(default)]` so a config written before these existed parses.
    #[serde(default = "default_add_context")]
    pub add_context: String,
    #[serde(default = "default_reset_conversation")]
    pub reset_conversation: String,
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
    /// Screen Answer: capture all monitors, answer, draw dots. Clear-dots removes only the dots.
    #[serde(default = "default_screen_answer")]
    pub screen_answer: String,
    #[serde(default = "default_clear_dots")]
    pub clear_dots: String,
    /// Account screen: change key / code, log out, delete.
    #[serde(default = "default_account")]
    pub account: String,
}

fn default_account() -> String {
    "ctrl+shift+alt+k".to_string()
}

fn default_screen_answer() -> String {
    "ctrl+shift+alt+w".to_string()
}
fn default_clear_dots() -> String {
    "ctrl+shift+alt+d".to_string()
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

    /// Screen Answer dots: colour and edge length in px. Pulses in alpha, never fully solid.
    #[serde(default = "default_dot_color")]
    pub dot_color: String,
    #[serde(default = "default_dot_size")]
    pub dot_size: u32,
}

fn default_dot_color() -> String {
    "#FF0000".to_string()
}
fn default_dot_size() -> u32 {
    3
}

fn default_crawl_trigger_lines() -> u32 {
    2
}
fn default_crawl_ms_per_line() -> u64 {
    2000
}
fn default_add_context() -> String {
    "ctrl+shift+alt+a".to_string()
}
fn default_reset_conversation() -> String {
    "ctrl+shift+alt+n".to_string()
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
    /// Owner's cloud doc bundle (cloud_docs/). Built-in default so existing configs get it
    /// without editing; `""` turns cloud off.
    #[serde(default = "default_cloud_url")]
    pub cloud_url: String,
}

/// Public, read-only URL of the owner's GitHub Pages bundle (cloud_docs/) — not a secret.
pub const DEFAULT_CLOUD_URL: &str = "https://hyowonbernabe.github.io/ShadowPrompt/bundle.json";

fn default_cloud_url() -> String {
    DEFAULT_CLOUD_URL.to_string()
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_doc_bytes: default_max_doc_bytes(),
            cloud_url: default_cloud_url(),
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_max_doc_bytes() -> usize {
    100_000
}
