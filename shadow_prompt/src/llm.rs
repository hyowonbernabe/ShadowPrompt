use anyhow::{Result, Context};
use reqwest::Client;
use serde_json::{json, Value};
use crate::config::Config;

pub struct LlmClient;

impl LlmClient {
    pub async fn query(prompt: &str, config: &Config) -> Result<String> {
        let client = Client::new();
        
        // Determine Provider
        match config.models.provider.as_str() {
            "openrouter" => Self::query_openrouter(&client, prompt, config).await,
            "ollama" => Self::query_ollama(&client, prompt, config).await,
            _ => anyhow::bail!("Unknown or unimplemented provider: {}", config.models.provider),
        }
    }

    fn load_system_prompt() -> String {
        std::fs::read_to_string("config/system_prompt.txt")
            .or_else(|_| std::fs::read_to_string("../config/system_prompt.txt"))
            .unwrap_or_else(|_| "You are a concise assistant.".to_string())
    }

    async fn query_openrouter(client: &Client, prompt: &str, config: &Config) -> Result<String> {
        let openrouter_config = config.models.openrouter.as_ref()
            .context("OpenRouter config missing")?;

        let system_prompt = Self::load_system_prompt();

        let body = json!({
            "model": openrouter_config.model_id,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        });

        let res = client.post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", openrouter_config.api_key))
            // .header("HTTP-Referer", "http://localhost:3000") // Optional
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
             let err_text = res.text().await?;
             anyhow::bail!("OpenRouter API Error: {}", err_text);
        }

        let json: Value = res.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .context("Failed to parse LLM response")?
            .to_string();

        Ok(content)
    }

    async fn query_ollama(client: &Client, prompt: &str, config: &Config) -> Result<String> {
         let ollama_config = config.models.ollama.as_ref().context("Ollama config missing")?;
         let system_prompt = Self::load_system_prompt();
         
         let body = json!({
             "model": ollama_config.model_id,
             "prompt": prompt,
             "system": system_prompt,
             "stream": false
         });

         let url = format!("{}/api/generate", ollama_config.base_url);
         let res = client.post(&url)
             .json(&body)
             .send()
             .await?;

         if !res.status().is_success() {
             anyhow::bail!("Ollama Error: {}", res.status());
         }

         let json: Value = res.json().await?;
         let response = json["response"].as_str().context("No response field")?.to_string();
         Ok(response)
    }

    async fn query_gemini(_client: &Client, _prompt: &str, _config: &Config) -> Result<String> {
        // Placeholder for direct Gemini API
        Ok("Gemini Direct API not yet implemented.".to_string())
    }
}
