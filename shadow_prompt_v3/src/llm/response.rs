// Parse OpenAI-compat response (OpenRouter normalized).
//
// Two wire shapes live here: the old non-streaming `Response` (kept as-is — it's also the
// *assembled* shape `mod.rs`'s SSE reader folds a whole stream down into, so nothing downstream
// of `LlmClient::call` had to change when streaming landed in M11) and the per-chunk streaming
// types below (`StreamChunk` etc.), one `data: {...}` SSE frame's worth of JSON each, per
// docs/OPENROUTER.md §5.

use super::messages::ToolCall;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    /// Which model actually served this request — ground truth, used by the `test_model`
    /// hotkey (design doc §3) instead of trusting the model's own possibly-wrong self-report.
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: AssistantMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub completion_tokens: u64,
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: u64,
}

// --- Streaming (SSE) chunk shapes — docs/OPENROUTER.md §5 ---
//
// Each `data: {...}` frame decodes to one `StreamChunk`. `mod.rs`'s `StreamAssembler` folds a
// whole sequence of these (plus the keep-alive comment lines and the terminal `[DONE]`, both
// handled below the JSON layer, before a frame's payload ever reaches `serde_json`) into one
// final `Response` above.

#[derive(Debug, Deserialize)]
pub struct StreamChunk {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Vec<StreamChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct StreamChoice {
    #[serde(default)]
    pub delta: Delta,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// One choice's incremental slice of an assistant turn. Both fields are additive across chunks:
/// `content` fragments concatenate in arrival order, `tool_calls` fragments accumulate per
/// `ToolCallDelta::index` (see `mod.rs::ToolCallAccum`) — neither is a complete value on its own.
#[derive(Debug, Default, Deserialize)]
pub struct Delta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallDelta>,
}

/// A fragment of one tool call, keyed by `index` (its position among however many tool calls
/// this turn makes, stable across the whole stream for that call). `id`/`type`/`function.name`
/// typically arrive once, on the chunk that opens the call; `function.arguments` arrives as a
/// raw JSON-string fragment on every chunk after that and must be concatenated in order — no
/// individual fragment is valid JSON by itself.
#[derive(Debug, Deserialize)]
pub struct ToolCallDelta {
    pub index: usize,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub function: Option<ToolCallFunctionDelta>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ToolCallFunctionDelta {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}
