// Build OpenRouter requests with capability-conditional fields.

use super::capabilities::Capabilities;
use super::messages::Message;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Request<'a> {
    pub model: &'a str,
    pub messages: &'a [Message],
    pub max_tokens: u32,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
}

#[derive(Debug, Serialize)]
pub struct Reasoning {
    pub effort: &'static str, // "high"
    pub exclude: bool,         // true — don't return reasoning tokens
}

pub fn build<'a>(model_id: &'a str, messages: &'a [Message], caps: Capabilities) -> Request<'a> {
    Request {
        model: model_id,
        messages,
        max_tokens: 4096,
        stream: false,
        reasoning: caps.reasoning.then_some(Reasoning { effort: "high", exclude: true }),
    }
}
