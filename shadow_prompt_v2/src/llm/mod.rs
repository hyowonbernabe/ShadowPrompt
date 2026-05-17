// LLM transport (OpenRouter) + request shaping + capability gating.

pub mod capabilities;
pub mod messages;
pub mod request;
pub mod response;
pub mod retry;
pub mod system_prompts;

use std::sync::Arc;

#[derive(Clone)]
pub struct LlmClient {
    pub http: Arc<reqwest::Client>,
    pub api_key: Arc<str>,
    pub model_id: Arc<str>,
}

impl LlmClient {
    pub fn new(api_key: String, model_id: String) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;
        Ok(Self {
            http: Arc::new(http),
            api_key: Arc::from(api_key),
            model_id: Arc::from(model_id),
        })
    }
}
