// Message + content-part types matching OpenAI-compat chat completions schema,
// extended with Anthropic-style cache_control for prompt caching.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    /// System message. Content is an array of parts so individual blocks can
    /// carry cache_control markers (Anthropic supports this; OpenAI-compat
    /// accepts it via OpenRouter passthrough).
    System { content: Vec<ContentPart> },
    User { content: Vec<ContentPart> },
    Assistant { content: String },
}

impl Message {
    /// Convenience: build a plain-text system message with a single block.
    pub fn system_text(text: impl Into<String>) -> Self {
        Message::System {
            content: vec![ContentPart::text(text)],
        }
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
        ContentPart::Text {
            text: s.into(),
            cache_control: Some(CacheControl::ephemeral(ttl)),
        }
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
            "5m" | "5min" | "5minutes" => None, // default; omit field
            "1h" | "1hour" | "60m" => Some("1h"),
            other => {
                log::warn!("knowledge: unknown cache_ttl '{other}', defaulting to 5m");
                None
            }
        };
        Self { kind: "ephemeral", ttl: ttl_normalized }
    }
}
