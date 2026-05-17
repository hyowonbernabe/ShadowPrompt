// Parse OpenAI-compat response (OpenRouter normalized).

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: AssistantMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub content: String,
}
