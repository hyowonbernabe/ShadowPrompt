// Message + content-part + tool-call types for OpenRouter's chat/completions wire format.
// Structured output goes through tool calls only (design doc §2) — no response_format anywhere
// in this module by design, not by omission.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    System { content: Vec<ContentPart> },
    User { content: Vec<ContentPart> },
    Assistant {
        content: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<ToolCall>,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}

impl Message {
    pub fn system_text(text: impl Into<String>) -> Self {
        Message::System { content: vec![ContentPart::text(text)] }
    }

    pub fn assistant_text(text: impl Into<String>) -> Self {
        Message::Assistant { content: Some(text.into()), tool_calls: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    #[serde(rename = "image_url")]
    Image { image_url: ImageUrl },
}

impl ContentPart {
    pub fn text(s: impl Into<String>) -> Self {
        ContentPart::Text { text: s.into(), cache_control: None }
    }

    pub fn text_cached(s: impl Into<String>, ttl: &'static str) -> Self {
        ContentPart::Text { text: s.into(), cache_control: Some(CacheControl::ephemeral(ttl)) }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<&'static str>,
}

impl CacheControl {
    pub fn ephemeral(ttl: &'static str) -> Self {
        let ttl_normalized = match ttl {
            "5m" | "5min" | "5minutes" => None,
            "1h" | "1hour" | "60m" => Some("1h"),
            other => {
                log::warn!("cache_control: unknown ttl '{other}', defaulting to 5m");
                None
            }
        };
        Self { kind: "ephemeral", ttl: ttl_normalized }
    }
}

/// A tool call the model made, as returned in an assistant message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String, // always "function"
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    /// Raw JSON string, per the OpenAI-compat wire format — decode against the tool's own
    /// schema at the call site, not here (design doc's OPENCODE.md steal: invalid args become a
    /// typed, model-visible error, not a crash).
    pub arguments: String,
}

/// A tool definition advertised to the model in a request's `tools` array.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub kind: &'static str, // "function"
    pub function: ToolFunctionDef,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolFunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema object
}
