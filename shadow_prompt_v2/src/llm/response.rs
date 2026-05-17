// Parse OpenAI-compat response (OpenRouter normalized).

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
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
    pub content: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub completion_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: u64,
}

impl Usage {
    /// Best-effort cache-read accounting across Anthropic-style and OpenAI-style usage fields.
    pub fn cache_read(&self) -> u64 {
        self.cache_read_input_tokens
            .max(self.prompt_tokens_details.as_ref().map(|d| d.cached_tokens).unwrap_or(0))
    }
}
