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

use crate::knowledge::KnowledgeBundle;

const ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Clone)]
pub struct LlmClient {
    pub http: Arc<reqwest::Client>,
    pub api_key: Arc<str>,
    pub model_id: Arc<str>,
    pub knowledge: Arc<KnowledgeBundle>,
    pub cache_ttl: Arc<str>,
}

impl LlmClient {
    pub fn new(
        api_key: String,
        model_id: String,
        connect_secs: u64,
        read_secs: u64,
        knowledge: KnowledgeBundle,
        cache_ttl: String,
    ) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(connect_secs))
            .timeout(Duration::from_secs(read_secs))
            .build()?;
        Ok(Self {
            http: Arc::new(http),
            api_key: Arc::from(api_key),
            model_id: Arc::from(model_id),
            knowledge: Arc::new(knowledge),
            cache_ttl: Arc::from(cache_ttl),
        })
    }

    /// Build the system message: rules block + (optional) cached knowledge block.
    /// Pass the rules text (answer-mode-general or answer-mode-forms).
    pub fn system_message(&self, rules: &str) -> Message {
        let mut parts = vec![ContentPart::text(rules)];
        if !self.knowledge.is_empty() {
            let ttl: &'static str = match self.cache_ttl.as_ref() {
                "1h" | "1hour" | "60m" => "1h",
                _ => "5m",
            };
            parts.push(ContentPart::text_cached(
                format!("# Reference material\n{}", self.knowledge.text),
                ttl,
            ));
        }
        Message::System { content: parts }
    }

    pub async fn answer_text(&self, system_rules: &str, user: &str) -> anyhow::Result<String> {
        self.answer_text_inner(system_rules, user, false).await
    }

    pub async fn answer_text_online(&self, system_rules: &str, user: &str) -> anyhow::Result<String> {
        self.answer_text_inner(system_rules, user, true).await
    }

    async fn answer_text_inner(&self, system_rules: &str, user: &str, online: bool) -> anyhow::Result<String> {
        let msgs = vec![
            self.system_message(system_rules),
            Message::User { content: vec![ContentPart::text(user)] },
        ];
        self.call_internal(msgs, online).await
    }

    pub async fn call(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        self.call_internal(messages, false).await
    }

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
                if let Some(u) = &parsed.usage {
                    let write = u.cache_creation_input_tokens;
                    let read = u.cache_read();
                    if write > 0 || read > 0 {
                        log::info!(
                            "cache: write={write} read={read} prompt={} completion={}",
                            u.prompt_tokens, u.completion_tokens
                        );
                    }
                }
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
