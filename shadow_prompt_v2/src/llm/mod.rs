// LLM transport (OpenRouter) + request shaping + capability gating.

pub mod capabilities;
pub mod messages;
pub mod request;
pub mod response;
pub mod retry;
pub mod system_prompts;

use std::sync::Arc;
use std::time::Duration;

use messages::{ContentPart, Message};
use request::build;
use response::Response;

const ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Clone)]
pub struct LlmClient {
    pub http: Arc<reqwest::Client>,
    pub api_key: Arc<str>,
    pub model_id: Arc<str>,
}

impl LlmClient {
    pub fn new(api_key: String, model_id: String, connect_secs: u64, read_secs: u64) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(connect_secs))
            .timeout(Duration::from_secs(read_secs))
            .build()?;
        Ok(Self {
            http: Arc::new(http),
            api_key: Arc::from(api_key),
            model_id: Arc::from(model_id),
        })
    }

    /// Single round-trip: system + user text → assistant text. No history.
    pub async fn answer_text(&self, system: &str, user: &str) -> anyhow::Result<String> {
        self.answer_text_inner(system, user, false).await
    }

    /// Same as answer_text but forces live web search via OpenRouter's :online suffix.
    pub async fn answer_text_online(&self, system: &str, user: &str) -> anyhow::Result<String> {
        self.answer_text_inner(system, user, true).await
    }

    async fn answer_text_inner(&self, system: &str, user: &str, online: bool) -> anyhow::Result<String> {
        let msgs = vec![
            Message::System { content: system.to_string() },
            Message::User { content: vec![ContentPart::Text { text: user.to_string() }] },
        ];
        self.call_internal(msgs, online).await
    }

    /// Multi-message exchange (used by Forms with images + history).
    pub async fn call(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        self.call_internal(messages, false).await
    }

    /// Same as call but forces live web search via OpenRouter's :online suffix.
    pub async fn call_online(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        self.call_internal(messages, true).await
    }

    async fn call_internal(&self, messages: Vec<Message>, online: bool) -> anyhow::Result<String> {
        let effective_model: String = if online && !self.model_id.ends_with(":online") {
            format!("{}:online", self.model_id)
        } else {
            self.model_id.to_string()
        };
        let caps = capabilities::for_model(&effective_model);
        let body = build(&effective_model, &messages, caps);
        let body = serde_json::to_string(&body)?;
        let http = self.http.clone();
        let api_key = self.api_key.clone();

        retry::with_retry(|| {
            let http = http.clone();
            let api_key = api_key.clone();
            let body = body.clone();
            async move {
                let resp = http
                    .post(ENDPOINT)
                    .bearer_auth(&*api_key)
                    .header("Content-Type", "application/json")
                    .header("HTTP-Referer", "https://github.com/hyowonbernabe/ShadowPrompt")
                    .header("X-Title", "ShadowPrompt")
                    .body(body)
                    .send()
                    .await?;
                let status = resp.status();
                let text = resp.text().await?;
                if !status.is_success() {
                    anyhow::bail!("openrouter {status}: {text}");
                }
                let parsed: Response = serde_json::from_str(&text)
                    .map_err(|e| anyhow::anyhow!("parse response: {e}; body={text}"))?;
                parsed
                    .choices
                    .into_iter()
                    .next()
                    .map(|c| c.message.content)
                    .ok_or_else(|| anyhow::anyhow!("no choices in response"))
            }
        })
        .await
    }
}
