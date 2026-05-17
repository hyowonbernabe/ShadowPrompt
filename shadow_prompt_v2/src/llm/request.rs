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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPrefs>,
}

#[derive(Debug, Serialize)]
pub struct Reasoning {
    pub effort: &'static str,
    pub exclude: bool,
}

/// OpenRouter routing preferences. We pin Anthropic models to the Anthropic
/// provider because Bedrock/Vertex routes for newer Claude models often lag
/// behind on vision/feature support, returning "Could not process image".
#[derive(Debug, Serialize)]
pub struct ProviderPrefs {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<&'static str>,
    pub allow_fallbacks: bool,
}

pub fn build<'a>(model_id: &'a str, messages: &'a [Message], caps: Capabilities) -> Request<'a> {
    Request {
        model: model_id,
        messages,
        max_tokens: 4096,
        stream: false,
        reasoning: caps.reasoning.then_some(Reasoning { effort: "high", exclude: true }),
        provider: provider_prefs_for(model_id),
    }
}

fn provider_prefs_for(model_id: &str) -> Option<ProviderPrefs> {
    if model_id.starts_with("anthropic/") {
        Some(ProviderPrefs {
            order: vec!["anthropic"],
            allow_fallbacks: true,
        })
    } else {
        None
    }
}
